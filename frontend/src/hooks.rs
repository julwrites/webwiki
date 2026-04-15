use crate::Route;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_router::prelude::*;

#[hook]
pub fn use_create_file(current_volume: String) -> Callback<()> {
    let navigator_opt = use_navigator();

    Callback::from(move |_| {
        if let Some(navigator) = &navigator_opt {
            if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None)
            {
                if !path.trim().is_empty() {
                    navigator.push(&Route::Wiki {
                        volume: current_volume.clone(),
                        path: path.trim().to_string(),
                    });
                }
            }
        } else {
            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(
                "use_create_file must be used inside a BrowserRouter",
            ));
        }
    })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyBindings {
    pub leader: String,
    pub pull: String,
    pub push: String,
    pub commit: String,
    pub edit: Vec<String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            leader: " ".to_string(),
            pull: "gl".to_string(),
            push: "gp".to_string(),
            commit: "gc".to_string(),
            edit: vec!["e".to_string(), "i".to_string(), "a".to_string()],
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct KeyHandlerProps {
    pub on_search: Callback<()>,
    pub on_pull: Callback<()>,
    pub on_push: Callback<()>,
    pub on_commit: Callback<()>,
    pub on_edit: Callback<()>,
}

#[hook]
pub fn use_key_handler(props: KeyHandlerProps) {
    let bindings =
        use_state(|| LocalStorage::get("key_bindings").unwrap_or_else(|_| KeyBindings::default()));

    let initial_props = props.clone();
    let props_ref = use_mut_ref(move || initial_props);
    *props_ref.borrow_mut() = props;

    // Buffer for Leader key sequences
    let key_buffer = use_mut_ref(String::new);
    let buffer_timeout = use_mut_ref(|| 0.0);

    {
        let bindings = bindings.clone();
        let props_ref = props_ref.clone();

        use_effect_with((), move |_| {
            let window = gloo_utils::window();
            let bindings = bindings.clone();
            let props_ref = props_ref.clone();
            let key_buffer = key_buffer.clone();
            let buffer_timeout = buffer_timeout.clone();

            let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                // Ignore if focus is on an input or textarea
                if let Some(target) = e.target() {
                    if target.dyn_ref::<web_sys::HtmlInputElement>().is_some() {
                        // Allow normal typing in inputs
                        return;
                    }
                    if target.dyn_ref::<web_sys::HtmlTextAreaElement>().is_some() {
                        // Allow normal typing in textareas
                        return;
                    }
                    if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                        if el.is_content_editable() {
                            // Allow normal typing in contenteditable
                            return;
                        }
                    }
                }

                let props = props_ref.borrow();
                let key = e.key();
                let timestamp = e.time_stamp();

                // SEARCH SHORTCUT: <Ctrl+o>
                if (e.ctrl_key() || e.meta_key()) && key.to_lowercase() == "o" {
                    e.prevent_default();
                    props.on_search.emit(());
                    // Reset leader buffer
                    *key_buffer.borrow_mut() = String::new();
                    return;
                }

                // Normal keys (only if no modifiers)
                if !e.ctrl_key() && !e.meta_key() && !e.alt_key() {
                    // If buffer is empty and key is leader, start sequence
                    let mut buffer = key_buffer.borrow_mut();
                    let mut last_timeout = buffer_timeout.borrow_mut();

                    // Check timeout (1 second for sequence)
                    if timestamp - *last_timeout > 1000.0 {
                        *buffer = String::new();
                    }
                    *last_timeout = timestamp;

                    if buffer.is_empty() {
                        if key == bindings.leader {
                            e.prevent_default();
                            buffer.push_str(&key);
                        } else if bindings.edit.contains(&key) {
                            e.prevent_default();
                            props.on_edit.emit(());
                        }
                    } else {
                        // Buffer not empty, append key
                        e.prevent_default();
                        buffer.push_str(&key);

                        // Check for matches
                        // Remove leader from start for comparison if leader is space
                        // Actually, let's construct the full sequence we expect: leader + sequence
                        // bindings.pull e.g. "gl". So we expect " gl" (space then g then l).

                        let expected_pull = format!("{}{}", bindings.leader, bindings.pull);
                        let expected_push = format!("{}{}", bindings.leader, bindings.push);
                        let expected_commit = format!("{}{}", bindings.leader, bindings.commit);

                        if *buffer == expected_pull {
                            props.on_pull.emit(());
                            *buffer = String::new();
                        } else if *buffer == expected_push {
                            props.on_push.emit(());
                            *buffer = String::new();
                        } else if *buffer == expected_commit {
                            props.on_commit.emit(());
                            *buffer = String::new();
                        } else {
                            // Check if buffer is still a valid prefix of any command
                            let is_prefix = expected_pull.starts_with(&*buffer)
                                || expected_push.starts_with(&*buffer)
                                || expected_commit.starts_with(&*buffer);

                            if !is_prefix {
                                // Invalid sequence, reset
                                *buffer = String::new();
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);

            let _ = window
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());

            move || {
                let _ = window.remove_event_listener_with_callback(
                    "keydown",
                    on_keydown.as_ref().unchecked_ref(),
                );
            }
        });
    }
}

use common::RenameRequest;
use gloo_net::http::Request;

#[hook]
pub fn use_rename_file(current_volume: String, current_path: String) -> Callback<()> {
    let navigator = use_navigator();

    Callback::from(move |_| {
        if let Some(new_path) = gloo_dialogs::prompt("Enter new file path:", Some(&current_path)) {
            if new_path != current_path && !new_path.is_empty() {
                let path = current_path.clone();
                let volume = current_volume.clone();
                let navigator = navigator.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/rename/{}/{}", volume, path);
                    let payload = RenameRequest {
                        new_path: new_path.clone(),
                    };
                    let resp = Request::post(&url).json(&payload).unwrap().send().await;
                    match resp {
                        Ok(r) if r.status() == 401 => {
                            let current_path = gloo_utils::window()
                                .location()
                                .pathname()
                                .unwrap_or_default();
                            if current_path != "/login" {
                                let _ = gloo_utils::window().location().set_href("/login");
                            }
                        }
                        Ok(r) if r.ok() => {
                            if let Some(nav) = navigator {
                                nav.push(&Route::Wiki {
                                    volume,
                                    path: new_path,
                                });
                            }
                        }
                        Ok(r) => {
                            let text = r.text().await.unwrap_or_default();
                            gloo_dialogs::alert(&format!("Failed to rename: {}", text));
                        }
                        Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                    }
                });
            }
        }
    })
}

#[hook]
pub fn use_delete_file(current_volume: String, current_path: String) -> Callback<()> {
    Callback::from(move |_| {
        if gloo_dialogs::confirm(&format!(
            "Are you sure you want to delete {}?",
            current_path
        )) {
            let path = current_path.clone();
            let volume = current_volume.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/wiki/{}/{}", volume, path);
                let resp = Request::delete(&url).send().await;
                match resp {
                    Ok(r) if r.status() == 401 => {
                        let current_path = gloo_utils::window()
                            .location()
                            .pathname()
                            .unwrap_or_default();
                        if current_path != "/login" {
                            let _ = gloo_utils::window().location().set_href("/login");
                        }
                    }
                    Ok(r) if r.ok() => {
                        let _ = gloo_utils::window()
                            .location()
                            .set_href(&format!("/wiki/{}", volume));
                    }
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        gloo_dialogs::alert(&format!("Failed to delete: {}", text));
                    }
                    Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                }
            });
        }
    })
}
