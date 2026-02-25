pub mod auth;
pub mod git;

use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use common::{FileNode, WikiPage};
use git::{git_routes, GitState};
use std::collections::HashMap;
use std::{fs, path::PathBuf, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{MemoryStore, SessionManagerLayer};

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
    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS (Cloudflare handles this)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd);

    // API Router
    let protected_router = Router::new()
        .route("/logout", post(logout_handler))
        .route("/wiki/{volume}/{*path}", get(read_page))
        .route("/wiki/{volume}/{*path}", put(write_page))
        .route("/wiki/{volume}/{*path}", delete(delete_page))
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
        .merge(protected_router);

    Router::new()
        .route("/wiki/{volume}/{*path}", get(serve_wiki_asset))
        .nest("/api", api_router)
        // Serve all other static files from "static" dir, falling back to index.html for SPA routing
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(session_layer)
        .with_state(state)
}

async fn serve_wiki_asset(
    State(state): State<Arc<AppState>>,
    Path((volume, path)): Path<(String, String)>,
) -> impl IntoResponse {
    let wiki_path = match state.volumes.get(&volume) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Volume not found").into_response(),
    };

    // Prevent deleting root or navigating up
    if path.is_empty() || path == "/" || path == "." || path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = wiki_path.join(&path);
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if file_path.exists() && file_path.is_file() {
        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();

        let text_extensions = [
            "md", "markdown", "json", "toml", "yaml", "yml", "opml", "dot", "mermaid", "mmd",
            "drawio", "dio",
        ];
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !text_extensions.contains(&ext.as_str()) {
            if let Ok(bytes) = fs::read(&file_path) {
                return ([(header::CONTENT_TYPE, mime.to_string())], bytes).into_response();
            }
        }
    }

    // Fallback to index.html
    match fs::read_to_string("static/index.html") {
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

    if path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let mut file_path = wiki_path.join(&path);

    // If it's a directory or likely a wikilink without extension, try adding .md
    if !file_path.exists() || file_path.is_dir() {
        let md_path = file_path.with_extension("md");
        if md_path.exists() {
            file_path = md_path;
        }
    }

    // Safety check: prevent directory traversal
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "Page not found").into_response();
    }

    let mime = mime_guess::from_path(&file_path).first_or_text_plain();

    // Explicit text extensions that should be served as WikiPage (text content)
    let text_extensions = [
        "md", "markdown", "json", "toml", "yaml", "yml", "opml", "dot", "mermaid", "mmd", "drawio",
        "dio",
    ];

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_explicit_text = text_extensions.contains(&ext.as_str());

    // Check if file is small (< 2MB)
    let is_small = match fs::metadata(&file_path) {
        Ok(meta) => meta.len() < 2 * 1024 * 1024,
        Err(_) => false,
    };

    // Determine if we should attempt to serve as WikiPage (text content)
    // 1. Explicit text extension
    // 2. Small file AND not a known binary type (Image/PDF)
    let is_image_or_pdf =
        mime.type_().as_str() == "image" || mime.essence_str() == "application/pdf";

    let should_try_text = is_explicit_text || (is_small && !is_image_or_pdf);

    if should_try_text {
        match fs::read(&file_path) {
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
        match fs::read(&file_path) {
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

    if path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = wiki_path.join(&path);

    // Safety check
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create directory",
            )
                .into_response();
        }
    }

    match fs::write(&file_path, payload.content) {
        Ok(_) => (StatusCode::OK, "Saved").into_response(),
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

    if path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = wiki_path.join(&path);

    // Safety check
    if !file_path.starts_with(wiki_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    if file_path.is_dir() {
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

        let tree = build_file_tree(wiki_path, wiki_path);
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

    if let Ok(entries) = fs::read_dir(current) {
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
    let _ = session.delete().await;
    StatusCode::OK
}
