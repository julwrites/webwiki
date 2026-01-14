use backend::git::GitState;
use backend::AppState;
use common::auth::decrypt_users;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Volume Configuration
    let volumes_env = std::env::var("VOLUMES").ok();
    let volumes: HashMap<String, PathBuf> = if let Some(v_str) = volumes_env {
        serde_json::from_str(&v_str).expect("Invalid JSON in VOLUMES env var")
    } else {
        let wiki_path = std::env::var("WIKI_PATH").unwrap_or_else(|_| "wiki_data".to_string());
        let mut map = HashMap::new();
        map.insert("default".to_string(), PathBuf::from(wiki_path));
        map
    };

    // Git Configuration (Initialize GitState for all volumes)
    let mut git_states = HashMap::new();
    for (name, path) in &volumes {
        git_states.insert(name.clone(), Arc::new(GitState::new(path.clone())));
    }

    let users_file = std::env::var("USERS_FILE").unwrap_or_else(|_| "users.json".to_string());
    let auth_secret = std::env::var("AUTH_SECRET").unwrap_or_else(|_| "secret".to_string());

    let users = if Path::new(&users_file).exists() {
        let content = std::fs::read_to_string(&users_file).expect("Failed to read users file");
        decrypt_users(&content, &auth_secret).expect("Failed to decrypt users file")
    } else {
        println!(
            "Warning: No users file found at {}. Authentication will fail for all users.",
            users_file
        );
        Vec::new()
    };

    let state = Arc::new(AppState {
        volumes,
        git_states,
        users,
    });

    let app = backend::app(state);

    // run it
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
