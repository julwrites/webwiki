use axum::{
    extract::FromRequestParts,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Redirect, Response, IntoResponse},
    RequestPartsExt,
};
use tower_sessions::Session;

pub struct AuthUser(pub String);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = parts
            .extract::<Session>()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Session missing"))?;

        if let Some(user) = session.get::<String>("user").await.unwrap_or(None) {
            Ok(AuthUser(user))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
        }
    }
}

pub async fn auth_middleware(
    session: Session,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();

    // Allow login endpoints and static assets
    if path == "/login" || path.starts_with("/api/login") || path.starts_with("/assets/") || path == "/favicon.ico" {
        return Ok(next.run(request).await);
    }

    // Check if authenticated
    if let Some(_user) = session.get::<String>("user").await.unwrap_or(None) {
        return Ok(next.run(request).await);
    }

    // If API request, return 401
    if path.starts_with("/api/") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Otherwise redirect to login
    Ok(Redirect::to("/login").into_response())
}
