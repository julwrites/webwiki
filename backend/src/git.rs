use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use git2::{Repository, Status, StatusOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct GitState {
    pub repo_path: PathBuf,
    pub write_lock: Arc<Mutex<()>>,
}

impl GitState {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            write_lock: Arc::new(Mutex::new(())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitStatusResponse {
    pub files: Vec<FileStatus>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitRequest {
    pub message: String,
    pub files: Vec<String>,
    pub author_name: String,
    pub author_email: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RestoreRequest {
    pub files: Vec<String>,
}

pub fn git_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/status", get(get_status))
        .route("/fetch", post(fetch_changes))
        .route("/commit", post(commit_changes))
        .route("/push", post(push_changes))
        .route("/pull", post(pull_changes))
        .route("/restore", post(restore_changes))
}

async fn get_status(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
) -> Result<Json<GitStatusResponse>, StatusCode> {
    let git_state = state.git_states.get(&volume).ok_or(StatusCode::NOT_FOUND)?;
    // Acquire lock to ensure we don't read status while a commit/restore is happening
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut file_statuses = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            let status_str = if status.contains(Status::INDEX_NEW)
                || status.contains(Status::WT_NEW)
            {
                "New"
            } else if status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_MODIFIED)
            {
                "Modified"
            } else if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED)
            {
                "Deleted"
            } else if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED)
            {
                "Renamed"
            } else {
                "Unknown"
            };

            file_statuses.push(FileStatus {
                path,
                status: status_str.to_string(),
            });
        }

        // Calculate commits ahead and behind
        let commits_ahead = calculate_commits_ahead(&repo).unwrap_or_default();
        let commits_behind = calculate_commits_behind(&repo).unwrap_or_default();

        Ok(Json(GitStatusResponse {
            files: file_statuses,
            commits_ahead,
            commits_behind,
        }))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result
}

fn calculate_commits_ahead(repo: &Repository) -> Result<usize, git2::Error> {
    let head = repo.head()?;
    let head_oid = head
        .target()
        .ok_or_else(|| git2::Error::from_str("HEAD not a ref"))?;

    let head_name = head
        .name()
        .ok_or_else(|| git2::Error::from_str("HEAD has no name"))?;

    let upstream_name_buf = match repo.branch_upstream_name(head_name) {
        Ok(name) => name,
        Err(_) => return Ok(0),
    };

    let upstream_name = upstream_name_buf
        .as_str()
        .ok_or_else(|| git2::Error::from_str("Upstream name not valid UTF-8"))?;

    let upstream = repo.find_reference(upstream_name)?;

    let upstream_oid = upstream
        .target()
        .ok_or_else(|| git2::Error::from_str("Upstream not a ref"))?;

    let (ahead, _) = repo.graph_ahead_behind(head_oid, upstream_oid)?;
    Ok(ahead)
}

fn calculate_commits_behind(repo: &Repository) -> Result<usize, git2::Error> {
    let head = repo.head()?;
    let head_oid = head
        .target()
        .ok_or_else(|| git2::Error::from_str("HEAD not a ref"))?;

    let head_name = head
        .name()
        .ok_or_else(|| git2::Error::from_str("HEAD has no name"))?;

    let upstream_name_buf = match repo.branch_upstream_name(head_name) {
        Ok(name) => name,
        Err(_) => return Ok(0),
    };

    let upstream_name = upstream_name_buf
        .as_str()
        .ok_or_else(|| git2::Error::from_str("Upstream name not valid UTF-8"))?;

    let upstream = repo.find_reference(upstream_name)?;

    let upstream_oid = upstream
        .target()
        .ok_or_else(|| git2::Error::from_str("Upstream not a ref"))?;

    let (_, behind) = repo.graph_ahead_behind(head_oid, upstream_oid)?;
    Ok(behind)
}

async fn fetch_changes(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
) -> Result<Json<GitStatusResponse>, String> {
    let git_state = state
        .git_states
        .get(&volume)
        .ok_or("Volume not found".to_string())?;
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<Json<GitStatusResponse>, String> {
        let repo = Repository::open(&repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        let mut callbacks = git2::RemoteCallbacks::new();
        let env_token = std::env::var("GIT_TOKEN").ok();
        let env_username = std::env::var("GIT_USERNAME").ok();

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            let username = env_username
                .as_deref()
                .or(username_from_url)
                .unwrap_or("git");

            if let Some(token) = &env_token {
                git2::Cred::userpass_plaintext(username, token.trim())
            } else {
                Err(git2::Error::from_str(
                    "No GIT_TOKEN provided in environment",
                ))
            }
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote
            .fetch(&[] as &[&str], Some(&mut fetch_options), None)
            .map_err(|e| format!("Git fetch failed: {}", e))?;

        // Re-calculate counts
        let commits_ahead = calculate_commits_ahead(&repo).unwrap_or_default();
        let commits_behind = calculate_commits_behind(&repo).unwrap_or_default();

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| format!("Status error: {}", e))?;
        let mut file_statuses = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            let status_str = if status.contains(Status::INDEX_NEW)
                || status.contains(Status::WT_NEW)
            {
                "New"
            } else if status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_MODIFIED)
            {
                "Modified"
            } else if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED)
            {
                "Deleted"
            } else if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED)
            {
                "Renamed"
            } else {
                "Unknown"
            };
            file_statuses.push(FileStatus {
                path,
                status: status_str.to_string(),
            });
        }

        Ok(Json(GitStatusResponse {
            files: file_statuses,
            commits_ahead,
            commits_behind,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

async fn pull_changes(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
) -> Result<StatusCode, String> {
    let git_state = state
        .git_states
        .get(&volume)
        .ok_or("Volume not found".to_string())?;
    // Lock to prevent concurrent git operations
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        // 1. Fetch
        let mut callbacks = git2::RemoteCallbacks::new();
        let env_token = std::env::var("GIT_TOKEN").ok();
        let env_username = std::env::var("GIT_USERNAME").ok();

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            let username = env_username
                .as_deref()
                .or(username_from_url)
                .unwrap_or("git");

            if let Some(token) = &env_token {
                git2::Cred::userpass_plaintext(username, token.trim())
            } else {
                Err(git2::Error::from_str(
                    "No GIT_TOKEN provided in environment",
                ))
            }
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote
            .fetch(&[] as &[&str], Some(&mut fetch_options), None)
            .map_err(|e| format!("Git fetch failed: {}", e))?;

        // 2. Merge Analysis
        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {}", e))?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|e| format!("Failed to get annotated commit: {}", e))?;
        let analysis = repo
            .merge_analysis(&[&fetch_commit])
            .map_err(|e| format!("Merge analysis failed: {}", e))?;

        if analysis.0.is_up_to_date() {
            return Ok(StatusCode::OK);
        } else if analysis.0.is_fast_forward() {
            let mut reference = repo
                .find_reference("HEAD")
                .map_err(|e| format!("Failed to find HEAD: {}", e))?;
            let name = reference.name().unwrap_or("HEAD").to_string();
            let msg = format!(
                "Fast-Forward: Setting {} to id: {}",
                name,
                fetch_commit.id()
            );
            reference
                .set_target(fetch_commit.id(), &msg)
                .map_err(|e| format!("Failed to set HEAD target: {}", e))?;
            repo.set_head(&name)
                .map_err(|e| format!("Failed to set HEAD: {}", e))?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| format!("Failed to checkout HEAD: {}", e))?;
        } else if analysis.0.is_normal() {
            let head_commit = repo
                .head()
                .and_then(|h| h.peel_to_commit())
                .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

            repo.merge(&[&fetch_commit], None, None)
                .map_err(|e| format!("Merge failed: {}", e))?;

            if repo.index().unwrap().has_conflicts() {
                return Err("Merge resulted in conflicts. Please resolve manually.".to_string());
            }

            // Create Merge Commit
            let signature = repo
                .signature()
                .map_err(|e| format!("Failed to get signature: {}", e))?;

            let mut index = repo
                .index()
                .map_err(|e| format!("Failed to get index: {}", e))?;
            let tree_id = index
                .write_tree()
                .map_err(|e| format!("Failed to write tree: {}", e))?;
            let tree = repo
                .find_tree(tree_id)
                .map_err(|e| format!("Failed to find tree: {}", e))?;

            let fetch_commit_obj = repo
                .find_commit(fetch_commit.id())
                .map_err(|e| format!("Failed to find fetch commit: {}", e))?;
            let parents = vec![&head_commit, &fetch_commit_obj];

            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Merge remote-tracking branch 'origin/HEAD'",
                &tree,
                &parents,
            )
            .map_err(|e| format!("Failed to create merge commit: {}", e))?;

            // Cleanup
            repo.cleanup_state()
                .map_err(|e| format!("Failed to cleanup state: {}", e))?;
        } else {
            return Err("Merge analysis returned unsupported result".to_string());
        }

        Ok(StatusCode::OK)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result
}

async fn commit_changes(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
    Json(payload): Json<CommitRequest>,
) -> Result<StatusCode, StatusCode> {
    let git_state = state.git_states.get(&volume).ok_or(StatusCode::NOT_FOUND)?;
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result
}

async fn restore_changes(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
    Json(payload): Json<RestoreRequest>,
) -> Result<StatusCode, StatusCode> {
    let git_state = state.git_states.get(&volume).ok_or(StatusCode::NOT_FOUND)?;
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force(); // Overwrite working directory changes

        for file in &payload.files {
            checkout_builder.path(file);
        }

        // Checkout HEAD to restore files
        repo.checkout_head(Some(&mut checkout_builder))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::OK)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result
}

async fn push_changes(
    State(state): State<Arc<AppState>>,
    Path(volume): Path<String>,
) -> Result<StatusCode, String> {
    let git_state = state
        .git_states
        .get(&volume)
        .ok_or("Volume not found".to_string())?;
    let _lock = git_state.write_lock.lock().await;
    let repo_path = git_state.repo_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&repo_path)
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
                Err(git2::Error::from_str(
                    "No GIT_TOKEN provided in environment",
                ))
            }
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Get the current branch name to push
        let head = repo
            .head()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let branch_name = head.shorthand().ok_or("Failed to get branch name")?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| format!("Git push failed: {}", e))?;

        Ok(StatusCode::OK)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result
}
