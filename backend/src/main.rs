use backend::git::GitState;
use backend::AppState;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let wiki_path = std::env::var("WIKI_PATH").unwrap_or_else(|_| "wiki_data".to_string());
    let wiki_path_buf = PathBuf::from(wiki_path);
    let git_state = Arc::new(GitState {
        repo_path: wiki_path_buf.clone(),
    });

    let state = Arc::new(AppState {
        wiki_path: wiki_path_buf,
        git_state,
    });

    let app = backend::app(state);

    // run it
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
