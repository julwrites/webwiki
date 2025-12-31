use backend::git::{self, GitState};
use backend::{app, AppState};
use common::{FileNode, WikiPage};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::{fs, sync::Arc};
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
async fn test_search() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("a.md"), "hello world").unwrap();
    fs::write(temp_dir.path().join("b.md"), "goodbye world").unwrap();
    fs::write(temp_dir.path().join("c.txt"), "hello world").unwrap(); // Should be ignored

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
                .uri("/api/search?q=hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();

    // Note: SearchResult struct is private in backend::search, so we use serde_json::Value
    // or we should export SearchResult.
    // Let's assume we check fields loosely or export it.
    // For now I'll check existence.

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["path"], "a.md");
    // assert!(results[0]["matches"][0].as_str().unwrap().contains("hello"));
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
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/git/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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

    let response = app
        .clone()
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
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let statuses: Vec<git::FileStatus> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(statuses.is_empty());
}
