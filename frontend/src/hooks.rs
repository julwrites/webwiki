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
    let navigator = use_navigator().expect("use_create_file must be used inside a BrowserRouter");

    Callback::from(move |_| {
        if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None) {
            if !path.trim().is_empty() {
                navigator.push(&Route::Wiki {
                    volume: current_volume.clone(),
                    path: path.trim().to_string(),
                });
            }
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

    let last_ctrl_f = use_mut_ref(|| 0.0);
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
            let last_ctrl_f = last_ctrl_f.clone();
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

                // SEARCH CHORD: <Ctrl+f> <Ctrl+f>
                // Note: browser treats Ctrl+f as a single key event with ctrl_key=true
                if (e.ctrl_key() || e.meta_key()) && key.to_lowercase() == "f" {
                    // Prevent browser "Find"
                    e.prevent_default();

                    let mut last = last_ctrl_f.borrow_mut();
                    let diff = timestamp - *last;

                    // 50ms < diff < 500ms (debounce slight bounces but catch double taps)
                    if diff < 500.0 && diff > 50.0 {
                        // Double press detected
                        props.on_search.emit(());
                        *last = 0.0; // Reset
                    } else {
                        *last = timestamp;
                    }
                    // Reset leader buffer on Ctrl+F
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

            window
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
                .unwrap();

            move || {
                window
                    .remove_event_listener_with_callback(
                        "keydown",
                        on_keydown.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });
    }
}
