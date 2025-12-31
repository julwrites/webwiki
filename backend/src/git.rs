use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use git2::{Repository, Status, StatusOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct GitState {
    pub repo_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitRequest {
    pub message: String,
    pub files: Vec<String>,
    pub author_name: String,
    pub author_email: String,
}

pub fn git_routes() -> Router<Arc<GitState>> {
    Router::new()
        .route("/status", get(get_status))
        .route("/commit", post(commit_changes))
        .route("/push", post(push_changes))
}

async fn get_status(
    State(state): State<Arc<GitState>>,
) -> Result<Json<Vec<FileStatus>>, StatusCode> {
    let repo = Repository::open(&state.repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut file_statuses = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.contains(Status::INDEX_NEW) || status.contains(Status::WT_NEW) {
            "New"
        } else if status.contains(Status::INDEX_MODIFIED) || status.contains(Status::WT_MODIFIED) {
            "Modified"
        } else if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED) {
            "Deleted"
        } else if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED) {
            "Renamed"
        } else {
            "Unknown"
        };

        file_statuses.push(FileStatus {
            path,
            status: status_str.to_string(),
        });
    }

    Ok(Json(file_statuses))
}

async fn commit_changes(
    State(state): State<Arc<GitState>>,
    Json(payload): Json<CommitRequest>,
) -> Result<StatusCode, StatusCode> {
    let repo = Repository::open(&state.repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut index = repo
        .index()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for file in payload.files {
        let path = std::path::Path::new(&file);
        index
            .add_path(path)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    index
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tree_id = index
        .write_tree()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tree = repo
        .find_tree(tree_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signature = git2::Signature::now(&payload.author_name, &payload.author_email)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let parent_commit = match repo.head() {
        Ok(head) => {
            let target = head.target().unwrap();
            Some(repo.find_commit(target).unwrap())
        }
        Err(_) => None,
    };

    let parents = if let Some(ref parent) = parent_commit {
        vec![parent]
    } else {
        vec![]
    };

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &payload.message,
        &tree,
        &parents,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

async fn push_changes(State(state): State<Arc<GitState>>) -> Result<StatusCode, String> {
    let repo = Repository::open(&state.repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;
    
    let mut callbacks = git2::RemoteCallbacks::new();

    let env_token = std::env::var("GIT_TOKEN").ok();
    let env_username = std::env::var("GIT_USERNAME").ok();

    // Set up authentication callback
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        let username = env_username
            .as_deref()
            .or(username_from_url)
            .unwrap_or("git");
        
        if let Some(token) = &env_token {
            git2::Cred::userpass_plaintext(username, token.trim())
        } else {
            Err(git2::Error::from_str("No GIT_TOKEN provided in environment"))
        }
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Get the current branch name to push
    let head = repo
        .head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let branch_name = head
        .shorthand()
        .ok_or("Failed to get branch name")?;
    
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| format!("Git push failed: {}", e))?;

    Ok(StatusCode::OK)
}
