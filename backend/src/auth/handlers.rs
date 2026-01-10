use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::auth::verify_password;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    message: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Find user
    let user_opt = state.users.iter().find(|u| u.username == payload.username);

    if let Some(user) = user_opt {
        if verify_password(&payload.password, &user.password_hash, &user.salt) {
            session
                .insert("user", &user.username)
                .await
                .expect("Failed to insert session");
            return (
                StatusCode::OK,
                Json(LoginResponse {
                    success: true,
                    message: "Logged in".into(),
                }),
            );
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(LoginResponse {
            success: false,
            message: "Invalid credentials".into(),
        }),
    )
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("Failed to delete session");
    (
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            message: "Logged out".into(),
        }),
    )
}

pub async fn check_auth(session: Session) -> impl IntoResponse {
    if let Some(user) = session.get::<String>("user").await.unwrap_or(None) {
        return (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                message: user,
            }),
        );
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(LoginResponse {
            success: false,
            message: "Not logged in".into(),
        }),
    )
}
