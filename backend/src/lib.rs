pub mod auth;
pub mod git;
pub mod session_store;

use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use common::{FileNode, RenameRequest, WikiPage};
use git::{git_routes, GitState};
use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::SessionManagerLayer;

pub fn is_forbidden_path(path: &str) -> bool {
    let sanitized = path.trim_start_matches('/');
    sanitized.is_empty()
        || sanitized == "."
        || sanitized.contains("..")
        || sanitized.contains('\\')
        || sanitized.starts_with('.')
        || sanitized.split('/').any(|segment| segment.starts_with('.'))
}

pub mod search;
use search::search_wiki;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    q: String,
    volume: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TreeParams {
    volume: Option<String>,
}

pub struct AppState {
    pub volumes: HashMap<String, PathBuf>,
    pub git_states: HashMap<String, Arc<GitState>>,
}

pub fn app(state: Arc<AppState>) -> Router {
    let session_dir = std::env::var("SESSION_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let base_path = state
                .volumes
                .get("default")
                .cloned()
                .or_else(|| state.volumes.values().next().cloned())
                .unwrap_or_else(|| PathBuf::from("wiki_data"));
            base_path.join(".webwiki").join("sessions")
        });

    let session_store = session_store::FileSessionStore::new(&session_dir)
        .expect("Failed to initialize session store");

    session_store::spawn_cleanup_task(
        session_store.clone(),
        std::time::Duration::from_secs(3600),
    );

    let session_secure = std::env::var("SESSION_SECURE_COOKIE")
        .map(|v| v == "true")
        .unwrap_or(false);

    app_with_session_store(state, session_store, session_secure)
}

pub fn app_with_session_store<S: tower_sessions::SessionStore + Clone>(
    state: Arc<AppState>,
    session_store: S,
    secure: bool,
) -> Router {
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(secure)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd)
        .with_always_save(true);

    // API Router
    let protected_router = Router::new()
        .route("/logout", post(logout_handler))
        .route("/wiki/{volume}/{*path}", get(read_page))
        .route("/wiki/{volume}/{*path}", put(write_page))
        .route("/wiki/{volume}/{*path}", delete(delete_page))
        .route("/rename/{volume}/{*path}", post(rename_page))
        .route("/upload/{volume}/{*path}", post(upload_file))
        .route("/tree", get(get_tree))
        .route("/search", get(search_handler))
        .nest(
            "/git/{volume}",
            git_routes()
                .with_state(state.clone())
                .layer(middleware::from_fn(auth::require_write_access)),
        )
        .layer(middleware::from_fn(auth::require_auth));

    let api_router = Router::new()
        .route("/login", post(auth::login))
        .merge(protected_router)
        .layer(session_layer);

    Router::new()
        .route("/wiki/{volume}/{*path}", get(serve_wiki_asset))
        .nest("/api", api_router)
        // Serve all other static files from "static" dir, falling back to index.html for SPA routing
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .with_state(state)
}

async fn resolve_wiki_path(wiki_path: &std::path::Path, path: &str) -> PathBuf {
    let sanitized_path = path.trim_start_matches('/');
    let file_path = wiki_path.join(sanitized_path);

    if !file_path.starts_with(wiki_path) {
        return file_path;
    }

    let meta = tokio::fs::metadata(&file_path).await.ok();

    // If it's a directory, always target its index.md
    if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
        return file_path.join("index.md");
    }

    // If it's a file that exactly exists at this path, use it directly
    if meta.is_some() {
        return file_path;
    }

    // Otherwise, if it has no extension, we default to markdown (.md)
    if file_path.extension().is_none() {
        return file_path.with_extension("md");
    }

    file_path
}

async fn serve_wiki_asset(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    // Prevent deleting root, navigating up, or accessing hidden/internal files
    if is_forbidden_path(&path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = wiki_path.join(&path);
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if let Ok(meta) = tokio::fs::metadata(&file_path).await {
        if meta.is_file() {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();

            let text_extensions = [
                "", "md", "markdown", "json", "toml", "yaml", "yml", "opml", "dot", "mermaid",
                "mmd", "drawio", "dio",
            ];
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !text_extensions.contains(&ext.as_str()) {
                if let Ok(bytes) = tokio::fs::read(&file_path).await {
                    return ([(header::CONTENT_TYPE, mime.to_string())], bytes).into_response();
                }
            }
        }
    }

    // Fallback to index.html
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => ([(header::CONTENT_TYPE, "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let mut results = Vec::new();

    if let Some(volume_name) = params.volume {
        // Search in specific volume
        if let Some(path) = state.volumes.get(&volume_name) {
            {
                let mut vol_results = tokio::task::spawn_blocking({
                    let path = path.clone();
                    let q = params.q.clone();
                    move || search_wiki(&path, &q)
                })
                .await
                .unwrap_or_default();

                for res in &mut vol_results {
                    res.volume = Some(volume_name.clone());
                }
                results.extend(vol_results);
            }
        }
    } else {
        // Search in all allowed volumes
        for (volume_name, path) in &state.volumes {
            {
                let mut vol_results = tokio::task::spawn_blocking({
                    let path = path.clone();
                    let q = params.q.clone();
                    move || search_wiki(&path, &q)
                })
                .await
                .unwrap_or_default();

                for res in &mut vol_results {
                    res.volume = Some(volume_name.clone());
                }
                results.extend(vol_results);
            }
        }
    }

    Json(results).into_response()
}

async fn read_page(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    if is_forbidden_path(&path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = resolve_wiki_path(wiki_path, &path).await;

    // Safety check: prevent directory traversal
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let final_meta = match tokio::fs::metadata(&file_path).await {
        Ok(meta) => meta,
        Err(_) => return (StatusCode::NOT_FOUND, "Page not found").into_response(),
    };

    let mime = mime_guess::from_path(&file_path).first_or_text_plain();

    // Explicit text extensions that should be served as WikiPage (text content)
    let text_extensions = [
        "", "md", "markdown", "json", "toml", "yaml", "yml", "opml", "dot", "mermaid", "mmd",
        "drawio", "dio",
    ];

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_explicit_text = text_extensions.contains(&ext.as_str());

    // Check if file is small (< 2MB)
    let is_small = final_meta.len() < 2 * 1024 * 1024;

    // Determine if we should attempt to serve as WikiPage (text content)
    // 1. Explicit text extension
    // 2. Small file AND not a known binary type (Image/PDF)
    let is_image_or_pdf =
        mime.type_().as_str() == "image" || mime.essence_str() == "application/pdf";

    let should_try_text = is_explicit_text || (is_small && !is_image_or_pdf);

    if should_try_text {
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => {
                // Try to convert to UTF-8 string
                match String::from_utf8(bytes.clone()) {
                    Ok(content) => Json(WikiPage { path, content }).into_response(),
                    Err(_) => {
                        // Not valid UTF-8, fallback to raw bytes
                        ([(header::CONTENT_TYPE, mime.to_string())], bytes).into_response()
                    }
                }
            }
            Err(_) => (StatusCode::NOT_FOUND, "Page not found").into_response(),
        }
    } else {
        // Binary / Image / PDF / Large Unknown
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => ([(header::CONTENT_TYPE, mime.to_string())], bytes).into_response(),
            Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
        }
    }
}

async fn write_page(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
    Json(payload): Json<WikiPage>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    if is_forbidden_path(&path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = resolve_wiki_path(wiki_path, &path).await;

    // Safety check
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if tokio::fs::create_dir_all(parent).await.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create directory",
            )
                .into_response();
        }
    }

    match tokio::fs::write(&file_path, payload.content).await {
        Ok(_) => (StatusCode::OK, "Saved").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn rename_page(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
    Json(payload): Json<RenameRequest>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    if is_forbidden_path(&path) || is_forbidden_path(&payload.new_path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let old_file_path = resolve_wiki_path(wiki_path, &path).await;
    let new_file_path = resolve_wiki_path(wiki_path, &payload.new_path).await;

    // Safety check
    if !old_file_path.starts_with(wiki_path) || !new_file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Ensure parent directory exists for new path
    if let Some(parent) = new_file_path.parent() {
        if tokio::fs::create_dir_all(parent).await.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create directory",
            )
                .into_response();
        }
    }

    match tokio::fs::rename(&old_file_path, &new_file_path).await {
        Ok(_) => {
            // Auto-update links across all markdown files in the volume using spawn_blocking
            // to avoid blocking the async executor with synchronous I/O.
            let new_path = payload.new_path.clone();
            let wiki_path_clone = wiki_path.clone();
            let _ = tokio::task::spawn_blocking(move || {
                if let Ok(re) = regex::Regex::new(&format!(
                    r"\[\[((?:[^:|\]]+:)?)({})(?:\|([^\]]+))?\]\]",
                    regex::escape(&path)
                )) {
                    for entry in walkdir::WalkDir::new(wiki_path_clone)
                        .into_iter()
                        .filter_entry(|e| {
                            if e.depth() == 0 {
                                return true;
                            }
                            !e.file_name().to_string_lossy().starts_with('.')
                        })
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file()
                            && entry.path().extension().is_some_and(|ext| ext == "md")
                        {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                let result = re.replace_all(&content, |caps: &regex::Captures| {
                                    let vol = caps.get(1).map_or("", |m| m.as_str());
                                    let desc = caps.get(3).map_or("", |m| m.as_str());

                                    if desc.is_empty() {
                                        format!("[[{}{}]]", vol, new_path)
                                    } else {
                                        format!("[[{}{}|{}]]", vol, new_path, desc)
                                    }
                                });

                                if result != content {
                                    let _ = std::fs::write(entry.path(), result.as_bytes());
                                }
                            }
                        }
                    }
                }
            })
            .await;

            (StatusCode::OK, "Renamed").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_page(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    if is_forbidden_path(&path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let raw_path = wiki_path.join(path.trim_start_matches('/'));
    let file_path = if tokio::fs::metadata(&raw_path)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        raw_path
    } else {
        resolve_wiki_path(wiki_path, &path).await
    };

    // Safety check
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let meta = match tokio::fs::metadata(&file_path).await {
        Ok(m) => m,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    if meta.is_dir() {
        match tokio::fs::remove_dir_all(&file_path).await {
            Ok(_) => (StatusCode::OK, "Deleted").into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        match tokio::fs::remove_file(&file_path).await {
            Ok(_) => (StatusCode::OK, "Deleted").into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

async fn get_tree(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TreeParams>,
) -> impl IntoResponse {
    if let Some(volume) = params.volume {
        // Return tree for specific volume
        let wiki_path = match state.volumes.get(&volume) {
            Some(p) => p,
            None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
        };

        let wiki_path_clone = wiki_path.clone();
        let tree = tokio::task::spawn_blocking(move || {
            build_file_tree(&wiki_path_clone, &wiki_path_clone)
        })
        .await
        .unwrap_or_default();
        Json(tree).into_response()
    } else {
        // Return list of volumes as directories
        let mut nodes = Vec::new();
        for volume_name in state.volumes.keys() {
            nodes.push(FileNode {
                name: volume_name.clone(),
                path: volume_name.clone(), // Path is just the volume name
                is_dir: true,
                children: None, // Frontend can fetch children when expanded
            });
        }
        // Sort volumes alphabetically
        nodes.sort_by(|a, b| a.name.cmp(&b.name));
        Json(nodes).into_response()
    }
}

fn build_file_tree(root: &PathBuf, current: &PathBuf) -> Vec<FileNode> {
    let mut nodes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden files/dirs (like .git)
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let is_dir = path.is_dir();

            let children = if is_dir {
                Some(build_file_tree(root, &path))
            } else {
                None
            };

            nodes.push(FileNode {
                name,
                path: relative_path,
                is_dir,
                children,
            });
        }
    }

    // Sort directories first, then files
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    nodes
}

async fn logout_handler(session: tower_sessions::Session) -> impl IntoResponse {
    let _ = session.flush().await;
    StatusCode::OK
}

async fn upload_file(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    if is_forbidden_path(&path) {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = wiki_path.join(&path);

    // Safety check
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if tokio::fs::create_dir_all(parent).await.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create directory",
            )
                .into_response();
        }
    }

    match tokio::fs::write(&file_path, body).await {
        Ok(_) => (StatusCode::OK, "Saved").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_resolve_wiki_path_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        tokio::fs::write(&file_path, "hello").await.unwrap();

        let resolved = resolve_wiki_path(dir.path(), "doc.md").await;
        assert_eq!(resolved, file_path);
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_directory_target() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("folder");
        tokio::fs::create_dir(&sub_dir).await.unwrap();

        let resolved = resolve_wiki_path(dir.path(), "folder").await;
        assert_eq!(resolved, sub_dir.join("index.md"));
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_extensionless_with_md_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("notes.md");
        tokio::fs::write(&file_path, "notes").await.unwrap();

        let resolved = resolve_wiki_path(dir.path(), "notes").await;
        assert_eq!(resolved, file_path);
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_extensionless_with_dir_index() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("topic");
        tokio::fs::create_dir(&sub_dir).await.unwrap();
        let index_path = sub_dir.join("index.md");
        tokio::fs::write(&index_path, "index").await.unwrap();

        let resolved = resolve_wiki_path(dir.path(), "topic").await;
        assert_eq!(resolved, index_path);
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_extensionless_new_file() {
        let dir = tempdir().unwrap();
        let resolved = resolve_wiki_path(dir.path(), "new_page").await;
        assert_eq!(resolved, dir.path().join("new_page.md"));
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_new_file_with_extension() {
        let dir = tempdir().unwrap();
        let resolved = resolve_wiki_path(dir.path(), "diagram.svg").await;
        assert_eq!(resolved, dir.path().join("diagram.svg"));
    }

    #[tokio::test]
    async fn test_resolve_wiki_path_leading_slash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        tokio::fs::write(&file_path, "hello").await.unwrap();

        let resolved = resolve_wiki_path(dir.path(), "/doc.md").await;
        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_is_forbidden_path() {
        assert!(is_forbidden_path(""));
        assert!(is_forbidden_path("/"));
        assert!(is_forbidden_path("."));
        assert!(is_forbidden_path(".."));
        assert!(is_forbidden_path("../foo"));
        assert!(is_forbidden_path("foo/../bar"));
        assert!(is_forbidden_path(".webwiki"));
        assert!(is_forbidden_path(".webwiki/sessions/123.json"));
        assert!(is_forbidden_path("/.webwiki/sessions"));
        assert!(is_forbidden_path("foo/.secret"));
        assert!(is_forbidden_path(".git/config"));

        // Allowed paths
        assert!(!is_forbidden_path("doc"));
        assert!(!is_forbidden_path("doc.md"));
        assert!(!is_forbidden_path("/doc.md"));
        assert!(!is_forbidden_path("folder/subfolder/page.md"));
        assert!(!is_forbidden_path("image.png"));
    }

    static TEST_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[tokio::test]
    async fn test_session_persists_across_app_restart() {
        use tower::ServiceExt;
        use axum::body::Body;
        use axum::http::Request;

        let _guard = TEST_ENV_MUTEX.lock().unwrap();

        std::env::set_var("WIKI_USERNAME", "alice");
        std::env::set_var("WIKI_PASSWORD", "secret123");
        std::env::set_var("DEV_BYPASS_AUTH", "false");

        let wiki_dir = tempdir().unwrap();
        let sessions_dir = wiki_dir.path().join(".webwiki").join("sessions");
        let store1 = session_store::FileSessionStore::new(&sessions_dir).unwrap();

        let mut volumes = HashMap::new();
        volumes.insert("default".to_string(), wiki_dir.path().to_path_buf());
        let state1 = Arc::new(AppState {
            volumes: volumes.clone(),
            git_states: HashMap::new(),
        });

        let app1 = app_with_session_store(state1, store1, false);

        // 1. Attempt protected endpoint without session -> 401
        let req = Request::builder()
            .uri("/api/tree")
            .body(Body::empty())
            .unwrap();
        let res = app1.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // 2. Login
        let login_payload = serde_json::json!({
            "username": "alice",
            "password": "secret123",
            "stay_signed_in": false
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
            .unwrap();
        let res = app1.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Extract cookie header
        let cookie_header = res
            .headers()
            .get("set-cookie")
            .expect("set-cookie header should be present")
            .to_str()
            .unwrap()
            .to_string();
        let cookie_val = cookie_header.split(';').next().unwrap().to_string();

        // Verify session file was written to disk
        let session_files = std::fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();
        assert_eq!(session_files.len(), 1);

        // 3. Make protected request using cookie on app1 -> 200 OK
        let req = Request::builder()
            .uri("/api/tree")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app1.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 4. Simulate RE-DEPLOY: app1 is dropped, new app2 and new store2 instance are initialized
        let store2 = session_store::FileSessionStore::new(&sessions_dir).unwrap();
        let state2 = Arc::new(AppState {
            volumes: volumes.clone(),
            git_states: HashMap::new(),
        });
        let app2 = app_with_session_store(state2, store2, false);

        // 5. Make protected request to app2 using the same cookie -> 200 OK (NOT logged out!)
        let req = Request::builder()
            .uri("/api/tree")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app2.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 6. Test logout
        let req = Request::builder()
            .method("POST")
            .uri("/api/logout")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app2.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 7. Verify session file on disk is deleted after logout
        let session_files_after = std::fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();
        assert_eq!(session_files_after.len(), 0);

        // 8. Subsequent protected request -> 401 UNAUTHORIZED
        let req = Request::builder()
            .uri("/api/tree")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app2.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_stay_signed_in_session_persists_90_days() {
        use tower::ServiceExt;
        use axum::body::Body;
        use axum::http::Request;

        let _guard = TEST_ENV_MUTEX.lock().unwrap();

        std::env::set_var("WIKI_USERNAME", "bob");
        std::env::set_var("WIKI_PASSWORD", "secret456");
        std::env::set_var("DEV_BYPASS_AUTH", "false");

        let wiki_dir = tempdir().unwrap();
        let sessions_dir = wiki_dir.path().join(".webwiki").join("sessions");
        let store1 = session_store::FileSessionStore::new(&sessions_dir).unwrap();

        let mut volumes = HashMap::new();
        volumes.insert("default".to_string(), wiki_dir.path().to_path_buf());
        let state1 = Arc::new(AppState {
            volumes: volumes.clone(),
            git_states: HashMap::new(),
        });

        let app1 = app_with_session_store(state1, store1, false);

        // 1. Login with stay_signed_in = true
        let login_payload = serde_json::json!({
            "username": "bob",
            "password": "secret456",
            "stay_signed_in": true
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
            .unwrap();
        let res = app1.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let cookie_header = res
            .headers()
            .get("set-cookie")
            .expect("set-cookie header should be present")
            .to_str()
            .unwrap()
            .to_string();
        assert!(cookie_header.to_lowercase().contains("max-age"));
        let cookie_val = cookie_header.split(';').next().unwrap().to_string();

        // 2. Perform protected request
        let req = Request::builder()
            .uri("/api/tree")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app1.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Response cookie should STILL have Max-Age and not be downgraded
        if let Some(subsequent_cookie) = res.headers().get("set-cookie") {
            let cookie_str = subsequent_cookie.to_str().unwrap().to_lowercase();
            assert!(
                cookie_str.contains("max-age"),
                "Cookie must retain Max-Age on subsequent requests when stay_signed_in is true"
            );
        }

        // Verify session file on disk has ~90 days expiry
        let session_file = std::fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .expect("Session file should exist");
        let content = std::fs::read(session_file.path()).unwrap();
        let record: tower_sessions::session::Record = serde_json::from_slice(&content).unwrap();
        let min_expected = time::OffsetDateTime::now_utc() + time::Duration::days(88);
        assert!(
            record.expiry_date >= min_expected,
            "Stored expiry date should remain ~90 days in future"
        );

        // 3. Simulate RE-DEPLOY: app1 dropped, brand new app2 instance
        let store2 = session_store::FileSessionStore::new(&sessions_dir).unwrap();
        let state2 = Arc::new(AppState {
            volumes,
            git_states: HashMap::new(),
        });
        let app2 = app_with_session_store(state2, store2, false);

        let req = Request::builder()
            .uri("/api/tree")
            .header("cookie", &cookie_val)
            .body(Body::empty())
            .unwrap();
        let res = app2.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
