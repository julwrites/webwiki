use crate::components::login::{LoginRequest, LoginResponse};
use common::auth::User;
use gloo_net::http::Request;

pub async fn login(username: String, password: String) -> Result<LoginResponse, gloo_net::Error> {
    let payload = LoginRequest { username, password };

    let resp = Request::post("/api/login").json(&payload)?.send().await?;

    if resp.ok() {
        resp.json().await
    } else {
        Err(gloo_net::Error::GlooError("Login failed".into()))
    }
}
