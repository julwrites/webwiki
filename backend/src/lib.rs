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
use tower_http::services::{ServeDir, ServeFile};

pub mod search;
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
    let assets_path = state.wiki_path.join("assets");

    Router::new()
        .route("/api/wiki/{*path}", get(read_page))
        .route("/api/wiki/{*path}", put(write_page))
        .route("/api/tree", get(get_tree))
        .route("/api/search", get(search_handler))
        .nest("/api/git", git_routes().with_state(state.git_state.clone()))
        // Serve "assets" from the wiki directory
        .nest_service("/assets", ServeDir::new(assets_path))
        // Serve all other static files from "static" dir, falling back to index.html for SPA routing
        .fallback_service(
            ServeDir::new("static").fallback(ServeFile::new("static/index.html")),
        )
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
