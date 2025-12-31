pub mod git;

use axum::extract::Query;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use common::{FileNode, WikiPage};
use git::{git_routes, GitState};
use std::{fs, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use tower_http::services::ServeDir;

mod search;
use search::search_wiki;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    q: String,
}

pub struct AppState {
    pub wiki_path: PathBuf,
    pub git_state: Arc<GitState>,
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/wiki/{*path}", get(read_page))
        .route("/api/wiki/{*path}", put(write_page))
        .route("/api/tree", get(get_tree))
        .route("/api/search", get(search_handler))
        .nest("/api/git", git_routes().with_state(state.git_state.clone()))
        .nest_service("/assets", ServeDir::new(state.wiki_path.join("assets")))
        .fallback(index_handler)
        .with_state(state)
}

async fn index_handler(uri: axum::http::Uri) -> impl IntoResponse {
    // Try to serve static file first
    let path = uri.path().trim_start_matches('/');
    let static_path = PathBuf::from("static").join(path);

    if !path.is_empty() && static_path.exists() && static_path.is_file() {
        match ServeDir::new("static")
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
        {
            Ok(res) => return res.into_response(),
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Static file error: {}", err),
                )
                    .into_response()
            }
        }
    }

    // Fallback to index.html
    match fs::read_to_string("static/index.html") {
        Ok(content) => (axum::http::StatusCode::OK, axum::response::Html(content)).into_response(),
        Err(_) => (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
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

    match fs::read_to_string(&file_path) {
        Ok(content) => Json(WikiPage { path, content }).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Page not found").into_response(),
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
