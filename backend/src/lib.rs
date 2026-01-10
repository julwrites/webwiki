pub mod auth;
pub mod git;

use crate::auth::handlers::{check_auth, login, logout};
use crate::auth::middleware::auth_middleware;
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use common::{auth::User, FileNode, WikiPage};
use git::{git_routes, GitState};
use std::{fs, path::PathBuf, sync::Arc};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub mod search;
use search::search_wiki;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    q: String,
}

pub struct AppState {
    pub wiki_path: PathBuf,
    pub git_state: Arc<GitState>,
    pub users: Vec<User>,
}

pub fn app(state: Arc<AppState>) -> Router {
    let assets_path = state.wiki_path.join("assets");

    // Session layer (Memory Store for now)
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax); // Allow cross-site navigation

    // API Router
    let api_router = Router::new()
        .route("/wiki/{*path}", get(read_page))
        .route("/wiki/{*path}", put(write_page))
        .route("/tree", get(get_tree))
        .route("/search", get(search_handler))
        .nest("/git", git_routes().with_state(state.git_state.clone()))
        // Auth endpoints
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/check-auth", get(check_auth));

    Router::new()
        .nest("/api", api_router)
        // Serve "assets" from the wiki directory
        .nest_service("/assets", ServeDir::new(assets_path))
        // Serve all other static files from "static" dir, falling back to index.html for SPA routing
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(session_layer)
        .with_state(state)
}

async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let wiki_path = state.wiki_path.clone();
    let query = params.q.clone();

    let results = tokio::task::spawn_blocking(move || search_wiki(&wiki_path, &query))
        .await
        .unwrap_or_default();

    Json(results)
}

async fn read_page(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let mut file_path = state.wiki_path.join(&path);

    // If it's a directory or likely a wikilink without extension, try adding .md
    if !file_path.exists() || file_path.is_dir() {
        let md_path = file_path.with_extension("md");
        if md_path.exists() {
            file_path = md_path;
        }
    }

    // Safety check: prevent directory traversal
    if !file_path.starts_with(&state.wiki_path) {
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
    Path(path): Path<String>,
    Json(payload): Json<WikiPage>,
) -> impl IntoResponse {
    if path.contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = state.wiki_path.join(&path);

    // Safety check
    if !file_path.starts_with(&state.wiki_path) {
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

async fn get_tree(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let root = &state.wiki_path;
    let tree = build_file_tree(root, root);
    Json(tree).into_response()
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
