use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::git::GitState;
use backend::{app, AppState};
use std::{fs, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_serve_image() {
    let temp_dir = TempDir::new().unwrap();
    // Create assets/images structure
    let assets_dir = temp_dir.path().join("assets");
    fs::create_dir(&assets_dir).unwrap();
    let images_dir = assets_dir.join("images");
    fs::create_dir(&images_dir).unwrap();

    // Write a dummy image file
    let image_path = images_dir.join("test.png");
    fs::write(&image_path, "fake png content").unwrap();

    let git_state = Arc::new(GitState {
        repo_path: temp_dir.path().to_path_buf(),
    });
    let state = Arc::new(AppState {
        wiki_path: temp_dir.path().to_path_buf(),
        git_state,
        users: Vec::new(),
    });
    let app = app(state);

    // Request the image
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/images/test.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body_bytes, "fake png content".as_bytes());
}
