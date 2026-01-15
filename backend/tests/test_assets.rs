use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use backend::git::GitState;
use backend::{app, AppState};
use common::auth::{hash_password, User};
use std::collections::HashMap;
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

    let mut volumes = HashMap::new();
    volumes.insert("default".to_string(), temp_dir.path().to_path_buf());

    let mut git_states = HashMap::new();
    git_states.insert(
        "default".to_string(),
        Arc::new(GitState::new(temp_dir.path().to_path_buf())),
    );

    // We need a user with permissions for "default" volume
    let mut users = Vec::new();
    let mut permissions = HashMap::new();
    permissions.insert("default".to_string(), "r".to_string());
    let (hash, salt) = hash_password("password");
    users.push(User {
        username: "testuser".to_string(),
        password_hash: hash,
        salt,
        permissions,
    });

    let state = Arc::new(AppState {
        volumes,
        git_states,
        users,
    });
    let app = app(state);

    // Login
    let login_payload = r#"{"username":"testuser", "password":"password"}"#;
    let request = Request::builder()
        .method("POST")
        .uri("/api/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Request the image
    let request = Request::builder()
        .uri("/api/wiki/default/assets/images/test.png")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body_bytes, "fake png content".as_bytes());
}
