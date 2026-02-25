use crate::AppState;
use axum::Json;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use common::User;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

pub const USER_SESSION_KEY: &str = "user";

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> Result<StatusCode, StatusCode> {
    let expected_username = std::env::var("WIKI_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let expected_password = std::env::var("WIKI_PASSWORD").map_err(|_| {
        eprintln!("WIKI_PASSWORD environment variable is not set!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if payload.username == expected_username && payload.password == expected_password {
        // For single user mode, we grant 'rw' to all configured volumes
        let mut permissions = std::collections::HashMap::new();

        for volume_name in state.volumes.keys() {
            permissions.insert(volume_name.clone(), "rw".to_string());
        }

        let user = User {
            username: payload.username,
            permissions,
        };

        session
            .insert(USER_SESSION_KEY, user)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn require_auth(
    session: Session,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user: Option<User> = session
        .get(USER_SESSION_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if user.is_some() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn require_write_access(
    Path(volume): Path<String>,
    session: Session,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user: User = session
        .get(USER_SESSION_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if let Some(perm) = user.permissions.get(&volume) {
        if perm.contains('w') {
            return Ok(next.run(req).await);
        }
    }

    Err(StatusCode::FORBIDDEN)
}
