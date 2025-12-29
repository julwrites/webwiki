use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use git2::{Repository, StatusOptions, Status, IndexAddOption};
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
}

async fn get_status(State(state): State<Arc<GitState>>) -> Result<Json<Vec<FileStatus>>, StatusCode> {
    let repo = Repository::open(&state.repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let mut index = repo.index().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for file in payload.files {
        let path = std::path::Path::new(&file);
        index.add_path(path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    index.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tree_id = index.write_tree().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tree = repo.find_tree(tree_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signature = git2::Signature::now(&payload.author_name, &payload.author_email)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let parent_commit = match repo.head() {
        Ok(head) => {
            let target = head.target().unwrap();
            Some(repo.find_commit(target).unwrap())
        },
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
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
