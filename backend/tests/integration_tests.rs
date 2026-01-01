use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::git::{self, GitState, GitStatusResponse};
use backend::search::SearchResult;
use backend::{app, AppState};
use common::{FileNode, WikiPage};
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
    let results: Vec<SearchResult> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "a.md");
    assert!(results[0].matches[0].contains("hello"));
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
    let status_response: GitStatusResponse = serde_json::from_slice(&body_bytes).unwrap();
    let statuses = status_response.files;

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
    let status_response: GitStatusResponse = serde_json::from_slice(&body_bytes).unwrap();
    let statuses = status_response.files;
    assert!(statuses.is_empty());
}

#[tokio::test]
async fn test_git_restore() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    let repo = git2::Repository::init(&repo_path).unwrap();

    // Configure user
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
    }

    // Create and commit a file
    let file_path = repo_path.join("test.md");
    fs::write(&file_path, "initial content").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("test.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Modify the file
    fs::write(&file_path, "modified content").unwrap();

    let git_state = Arc::new(GitState {
        repo_path: repo_path.clone(),
    });
    let state = Arc::new(AppState {
        wiki_path: repo_path.clone(),
        git_state,
    });
    let app = app(state);

    // Verify it is modified
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/git/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = axum::body::to_bytes(status_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: GitStatusResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(status.files.len(), 1);
    assert_eq!(status.files[0].status, "Modified");

    // Restore the file
    let restore_req = git::RestoreRequest {
        files: vec!["test.md".to_string()],
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/git/restore")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&restore_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify content is back to initial
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "initial content");

    // Verify status is clean
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/git/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = axum::body::to_bytes(status_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: GitStatusResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert!(status.files.is_empty());
}

#[tokio::test]
async fn test_git_commits_ahead() {
    let temp_dir_origin = TempDir::new().unwrap();
    let temp_dir_local = TempDir::new().unwrap();
    let repo_path_origin = temp_dir_origin.path().to_path_buf();
    let repo_path_local = temp_dir_local.path().to_path_buf();

    // Init origin
    let repo_origin = git2::Repository::init_bare(&repo_path_origin).unwrap();

    // Clone to local (simulating by init and adding remote)
    let repo_local = git2::Repository::init(&repo_path_local).unwrap();
    {
        let mut config = repo_local.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
    }

    // Create initial commit in local
    let file_path = repo_path_local.join("README.md");
    fs::write(&file_path, "README").unwrap();
    let mut index = repo_local.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo_local.find_tree(tree_id).unwrap();
    let sig = repo_local.signature().unwrap();
    repo_local
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Add remote and push
    repo_local
        .remote("origin", repo_path_origin.to_str().unwrap())
        .unwrap();
    let mut remote = repo_local.find_remote("origin").unwrap();
    // We can't easily push in test without setting up more callbacks or using file protocol which might need bare repo setup correctly.
    // Instead, let's just use two local repos and set up upstream manually if possible.

    // Easier approach:
    // 1. Create a commit.
    // 2. Branch off.
    // 3. Commit more on one branch.
    // 4. Set upstream tracking.
    // 5. Check counts.

    // Create branch 'main' (already there usually)
    let head = repo_local.head().unwrap();
    let head_commit = repo_local.find_commit(head.target().unwrap()).unwrap();

    // Create a new commit on HEAD
    fs::write(&file_path, "Updated README").unwrap();
    let mut index = repo_local.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo_local.find_tree(tree_id).unwrap();
    repo_local
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Second commit",
            &tree,
            &[&head_commit],
        )
        .unwrap();

    // Now simulate that origin is at the first commit.
    // We need to set a remote tracking branch.
    // Let's create a ref refs/remotes/origin/master pointing to the first commit.
    repo_local
        .reference(
            "refs/remotes/origin/master",
            head_commit.id(),
            true,
            "Setting up remote",
        )
        .unwrap();

    // Set up tracking for master
    let mut branch = repo_local
        .find_branch("master", git2::BranchType::Local)
        .unwrap();
    branch.set_upstream(Some("origin/master")).unwrap();

    let git_state = Arc::new(GitState {
        repo_path: repo_path_local.clone(),
    });
    let state = Arc::new(AppState {
        wiki_path: repo_path_local.clone(),
        git_state,
    });
    let app = app(state);

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
    let status: GitStatusResponse = serde_json::from_slice(&body_bytes).unwrap();

    // We are 1 commit ahead of origin/master
    assert_eq!(status.commits_ahead, 1);
}
