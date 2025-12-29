pub mod git;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use common::{FileNode, WikiPage};
use git::{git_routes, GitState};
use std::{fs, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;

struct AppState {
    wiki_path: PathBuf,
    git_state: Arc<GitState>,
}

#[tokio::main]
async fn main() {
    let wiki_path = std::env::var("WIKI_PATH").unwrap_or_else(|_| "wiki_data".to_string());
    let wiki_path_buf = PathBuf::from(wiki_path);
    let git_state = Arc::new(GitState {
        repo_path: wiki_path_buf.clone(),
    });

    let state = Arc::new(AppState {
        wiki_path: wiki_path_buf,
        git_state,
    });

    let app = app(state);

    // run it
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/wiki/{*path}", get(read_page))
        .route("/api/wiki/{*path}", put(write_page))
        .route("/api/tree", get(get_tree))
        .nest("/api/git", git_routes().with_state(state.git_state.clone()))
        .fallback_service(ServeDir::new("static"))
        .with_state(state)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tempfile::TempDir;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_read_page() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "# Hello World").unwrap();

        let git_state = Arc::new(GitState {
            repo_path: temp_dir.path().to_path_buf(),
        });
        let state = Arc::new(AppState {
            wiki_path: temp_dir.path().to_path_buf(),
            git_state,
        });
        let app = app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/wiki/test.md")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: WikiPage = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body.content, "# Hello World");
        assert_eq!(body.path, "test.md");
    }

    #[tokio::test]
    async fn test_read_page_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let git_state = Arc::new(GitState {
            repo_path: temp_dir.path().to_path_buf(),
        });
        let state = Arc::new(AppState {
            wiki_path: temp_dir.path().to_path_buf(),
            git_state,
        });
        let app = app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/wiki/missing.md")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_write_page() {
        let temp_dir = TempDir::new().unwrap();
        let git_state = Arc::new(GitState {
            repo_path: temp_dir.path().to_path_buf(),
        });
        let state = Arc::new(AppState {
            wiki_path: temp_dir.path().to_path_buf(),
            git_state,
        });
        let app = app(state);

        let page = WikiPage {
            path: "new.md".to_string(),
            content: "# New Content".to_string(),
        };
        let json_body = serde_json::to_string(&page).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/wiki/new.md")
                    .header("content-type", "application/json")
                    .body(Body::from(json_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let file_path = temp_dir.path().join("new.md");
        assert!(file_path.exists());
        let content = fs::read_to_string(file_path).unwrap();
        assert_eq!(content, "# New Content");
    }

    #[tokio::test]
    async fn test_get_tree() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("folder")).unwrap();
        fs::write(temp_dir.path().join("root.md"), "").unwrap();
        fs::write(temp_dir.path().join("folder/child.md"), "").unwrap();

        let git_state = Arc::new(GitState {
            repo_path: temp_dir.path().to_path_buf(),
        });
        let state = Arc::new(AppState {
            wiki_path: temp_dir.path().to_path_buf(),
            git_state,
        });
        let app = app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tree")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let nodes: Vec<FileNode> = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(nodes.len(), 2);
        // "folder" should come before "root.md" because directories are sorted first
        assert_eq!(nodes[0].name, "folder");
        assert!(nodes[0].is_dir);
        assert_eq!(nodes[1].name, "root.md");
    }

    #[tokio::test]
    async fn test_path_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let git_state = Arc::new(GitState {
            repo_path: temp_dir.path().to_path_buf(),
        });
        let state = Arc::new(AppState {
            wiki_path: temp_dir.path().to_path_buf(),
            git_state,
        });
        let app = app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/wiki/../secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_git_status_and_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        let repo = git2::Repository::init(&repo_path).unwrap();

        // Configure user for the repo (required for commits)
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();
        }

        // Create a file
        let file_path = repo_path.join("test.md");
        fs::write(&file_path, "initial content").unwrap();

        let git_state = Arc::new(GitState {
            repo_path: repo_path.clone(),
        });
        let state = Arc::new(AppState {
            wiki_path: repo_path.clone(),
            git_state,
        });
        let app = app(state);

        // Check status - should be new/untracked
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/git/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let statuses: Vec<git::FileStatus> = serde_json::from_slice(&body_bytes).unwrap();

        assert!(!statuses.is_empty());
        assert_eq!(statuses[0].path, "test.md");
        assert_eq!(statuses[0].status, "New");

        // Commit changes
        let commit_req = git::CommitRequest {
            message: "First commit".to_string(),
            files: vec!["test.md".to_string()],
            author_name: "Tester".to_string(),
            author_email: "tester@example.com".to_string(),
        };

        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/git/commit")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&commit_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Check status again - should be empty or clean
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/git/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let statuses: Vec<git::FileStatus> = serde_json::from_slice(&body_bytes).unwrap();
        assert!(statuses.is_empty());
    }
}
