use backend::git::GitState;
use backend::AppState;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use common::auth::decrypt_users;

#[tokio::main]
async fn main() {
    let wiki_path = std::env::var("WIKI_PATH").unwrap_or_else(|_| "wiki_data".to_string());
    let wiki_path_buf = PathBuf::from(wiki_path);
    let git_state = Arc::new(GitState {
        repo_path: wiki_path_buf.clone(),
    });

    let users_file = std::env::var("USERS_FILE").unwrap_or_else(|_| "users.json".to_string());
    let auth_secret = std::env::var("AUTH_SECRET").unwrap_or_else(|_| "secret".to_string());

    let users = if Path::new(&users_file).exists() {
        let content = std::fs::read_to_string(&users_file).expect("Failed to read users file");
        decrypt_users(&content, &auth_secret).expect("Failed to decrypt users file")
    } else {
        println!("Warning: No users file found at {}. Authentication will fail for all users.", users_file);
        Vec::new()
    };

    let state = Arc::new(AppState {
        wiki_path: wiki_path_buf,
        git_state,
        users,
    });

    let app = backend::app(state);

    // run it
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
