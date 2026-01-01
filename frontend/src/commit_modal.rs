use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub message: String,
    pub files: Vec<String>,
    pub author_name: String,
    pub author_email: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub files: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Clone, Deserialize)]
pub struct GitStatusResponse {
    pub files: Vec<FileStatus>,
    pub commits_ahead: usize,
}

#[derive(Properties, PartialEq)]
pub struct CommitModalProps {
    pub on_close: Callback<()>,
}

#[function_component(CommitModal)]
pub fn commit_modal(props: &CommitModalProps) -> Html {
    let message = use_state(String::new);
    let author_name =
        use_state(|| LocalStorage::get("author_name").unwrap_or_else(|_| "Wiki User".to_string()));
    let author_email = use_state(|| {
        LocalStorage::get("author_email").unwrap_or_else(|_| "user@example.com".to_string())
    });
    let files = use_state(Vec::new);
    let commits_ahead = use_state(|| 0);
    let selected_files = use_state(std::collections::HashSet::new);
    let is_loading = use_state(|| true);
    let error = use_state(String::new);
    // Refresh trigger to reload status
    let refresh_trigger = use_state(|| 0);

    {
        let files = files.clone();
        let commits_ahead = commits_ahead.clone();
        let is_loading = is_loading.clone();
        let refresh = *refresh_trigger;
        use_effect_with(refresh, move |_| {
            spawn_local(async move {
                let resp = Request::get("/api/git/status").send().await;
                match resp {
                    Ok(r) if r.ok() => {
                        if let Ok(status_resp) = r.json::<GitStatusResponse>().await {
                            files.set(status_resp.files);
                            commits_ahead.set(status_resp.commits_ahead);
                        } else {
                            // Fallback for backward compatibility if backend returns Vec<FileStatus>
                            // This logic might be needed if backend update isn't deployed yet or during dev
                            // But since we are updating both, we expect GitStatusResponse.
                            // However, let's just assume GitStatusResponse for now as per plan.
                        }
                    }
                    _ => {
                        // Handle error
                    }
                }
                is_loading.set(false);
            });
            || ()
        });
    }

    let on_commit = {
        let message = message.clone();
        let author_name = author_name.clone();
        let author_email = author_email.clone();
        let selected_files = selected_files.clone();
        let on_close = props.on_close.clone();
        let error = error.clone();

        Callback::from(move |_| {
            let message = message.clone();
            let author_name = author_name.clone();
            let author_email = author_email.clone();
            let selected_files = selected_files.clone();
            let on_close = on_close.clone();
            let error = error.clone();

            spawn_local(async move {
                let req = CommitRequest {
                    message: (*message).clone(),
                    files: selected_files.iter().cloned().collect(),
                    author_name: (*author_name).clone(),
                    author_email: (*author_email).clone(),
                };

                let resp = Request::post("/api/git/commit")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap())
                    .unwrap()
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.ok() => {
                        on_close.emit(());
                    }
                    _ => {
                        error.set("Failed to commit changes".to_string());
                    }
                }
            });
        })
    };

    let on_discard = {
        let selected_files = selected_files.clone();
        let error = error.clone();
        let refresh_trigger = refresh_trigger.clone();

        Callback::from(move |_| {
            let selected_files = selected_files.clone();
            let error = error.clone();
            let refresh_trigger = refresh_trigger.clone();

            spawn_local(async move {
                if selected_files.is_empty() {
                    return;
                }

                let req = RestoreRequest {
                    files: selected_files.iter().cloned().collect(),
                };

                let resp = Request::post("/api/git/restore")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap())
                    .unwrap()
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.ok() => {
                        // Clear selected files and refresh status
                        let empty: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        selected_files.set(empty);
                        refresh_trigger.set(*refresh_trigger + 1);
                    }
                    _ => {
                        error.set("Failed to discard changes".to_string());
                    }
                }
            });
        })
    };

    let toggle_file = {
        let selected_files = selected_files.clone();
        Callback::from(move |path: String| {
            let mut new_set = (*selected_files).clone();
            if new_set.contains(&path) {
                new_set.remove(&path);
            } else {
                new_set.insert(path);
            }
            selected_files.set(new_set);
        })
    };

    if *is_loading {
        return html! { <div class="modal">{"Loading status..."}</div> };
    }

    html! {
        <div class="modal-overlay">
            <div class="modal">
                <h2>{"Commit Changes"}</h2>

                <div class="sync-status">
                    <span>{"Pending Sync: "} {*commits_ahead}</span>
                </div>

                <div class="field">
                    <label>{"Name"}</label>
                    <input
                        value={(*author_name).clone()}
                        oninput={
                            let n = author_name.clone();
                            move |e: InputEvent| {
                                let val = e.target_unchecked_into::<HtmlInputElement>().value();
                                let _ = LocalStorage::set("author_name", &val);
                                n.set(val);
                            }
                        }
                    />
                </div>
                <div class="field">
                    <label>{"Email"}</label>
                    <input
                        value={(*author_email).clone()}
                        oninput={
                            let e = author_email.clone();
                            move |ev: InputEvent| {
                                let val = ev.target_unchecked_into::<HtmlInputElement>().value();
                                let _ = LocalStorage::set("author_email", &val);
                                e.set(val);
                            }
                        }
                    />
                </div>

                <div class="file-list">
                    <h3>{"Changes"}</h3>
                    {for files.iter().map(|f| {
                        let path = f.path.clone();
                        let is_checked = selected_files.contains(&path);
                        let on_change = {
                            let path = path.clone();
                            let toggle = toggle_file.clone();
                            move |_| toggle.emit(path.clone())
                        };
                        html! {
                            <div class="file-item">
                                <input type="checkbox" checked={is_checked} onclick={on_change} />
                                <span class="status">{&f.status}</span>
                                <span class="path">{&f.path}</span>
                            </div>
                        }
                    })}
                </div>

                <div class="field">
                    <label>{"Message"}</label>
                    <textarea
                        value={(*message).clone()}
                        oninput={let m = message.clone(); move |e: InputEvent| m.set(e.target_unchecked_into::<HtmlTextAreaElement>().value())}
                    />
                </div>

                if !error.is_empty() {
                    <div class="error">{&*error}</div>
                }

                <div class="actions">
                    <button onclick={on_commit}>{"Commit"}</button>
                    <button onclick={on_discard} disabled={selected_files.is_empty()} class="secondary">{"Discard Changes"}</button>
                    <button onclick={let on_close = props.on_close.clone(); move |_| on_close.emit(())}>{"Cancel"}</button>
                </div>
            </div>
        </div>
    }
}
