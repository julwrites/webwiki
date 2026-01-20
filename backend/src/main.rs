use backend::git::GitState;
use backend::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
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

    let state = Arc::new(AppState {
        volumes,
        git_states,
    });

    let app = backend::app(state);

    // run it
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
