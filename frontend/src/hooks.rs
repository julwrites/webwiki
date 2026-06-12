use crate::Route;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Shortcuts {
    pub edit: String,
    pub save: String,
    pub cancel: String,
    pub search: String,
    pub toggle_drawer: String,
    pub copy_link: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            edit: "Ctrl+E".to_string(),
            save: "Ctrl+S".to_string(),
            cancel: "Escape".to_string(),
            search: "Ctrl+K, Ctrl+P".to_string(),
            toggle_drawer: "Ctrl+B".to_string(),
            copy_link: "Ctrl+Shift+C".to_string(),
        }
    }
}

pub fn matches_shortcut(e: &KeyboardEvent, shortcut_def: &str) -> bool {
    let key = e.key().to_lowercase();
    for part in shortcut_def.split(',') {
        let part = part.trim();
        let mut req_ctrl = false;
        let mut req_shift = false;
        let mut req_alt = false;
        let mut req_key = String::new();

        for token in part.split('+') {
            let token = token.trim().to_lowercase();
            match token.as_str() {
                "ctrl" | "cmd" | "meta" => req_ctrl = true,
                "shift" => req_shift = true,
                "alt" => req_alt = true,
                _ => req_key = token,
            }
        }

        let is_ctrl = e.ctrl_key() || e.meta_key();
        let is_shift = e.shift_key();
        let is_alt = e.alt_key();

        if is_ctrl == req_ctrl && is_shift == req_shift && is_alt == req_alt {
            if key == req_key {
                return true;
            }
        }
    }
    false
}

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

#[derive(Clone, PartialEq)]
pub struct KeyHandlerProps {
    pub on_search: Callback<()>,
    pub on_edit: Callback<()>,
    pub on_save: Callback<()>,
    pub on_cancel: Callback<()>,
    pub on_toggle_drawer: Callback<()>,
    pub on_copy_link: Callback<()>,
}

#[hook]
pub fn use_key_handler(props: KeyHandlerProps) {
    let initial_props = props.clone();
    let props_ref = use_mut_ref(move || initial_props);
    *props_ref.borrow_mut() = props;

    // Load shortcuts
    let shortcuts = LocalStorage::get::<Shortcuts>("shortcuts").unwrap_or_else(|_| Shortcuts::default());

    {
        let props_ref = props_ref.clone();

        use_effect_with((), move |_| {
            let window = gloo_utils::window();
            let props_ref = props_ref.clone();

            let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                let props = props_ref.borrow();
                
                // Allow ESC/cancel to pass through everywhere so modals/editors can handle it globally if needed
                if matches_shortcut(&e, &shortcuts.cancel) {
                    e.prevent_default();
                    props.on_cancel.emit(());
                    return;
                }

                // Global Shortcuts (Even inside inputs if they don't override)
                if matches_shortcut(&e, &shortcuts.save) {
                    e.prevent_default();
                    props.on_save.emit(());
                    return;
                }

                if matches_shortcut(&e, &shortcuts.search) {
                    e.prevent_default();
                    props.on_search.emit(());
                    return;
                }

                if matches_shortcut(&e, &shortcuts.toggle_drawer) {
                    e.prevent_default();
                    props.on_toggle_drawer.emit(());
                    return;
                }

                if matches_shortcut(&e, &shortcuts.edit) {
                    e.prevent_default();
                    props.on_edit.emit(());
                    return;
                }

                if matches_shortcut(&e, &shortcuts.copy_link) {
                    e.prevent_default();
                    props.on_copy_link.emit(());
                    return;
                }

                // If focus is on an input or textarea, don't hijack normal typing
                if let Some(target) = e.target() {
                    if target.dyn_ref::<web_sys::HtmlInputElement>().is_some() { return; }
                    if target.dyn_ref::<web_sys::HtmlTextAreaElement>().is_some() { return; }
                    if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                        if el.is_content_editable() { return; }
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
