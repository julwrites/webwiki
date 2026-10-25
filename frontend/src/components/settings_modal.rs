use crate::hooks::{validate_all_shortcuts, Shortcuts};
use gloo_storage::{LocalStorage, Storage};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_close: Callback<()>,
    pub on_save_shortcuts: Callback<Shortcuts>,
}

#[function_component(SettingsModal)]
pub fn settings_modal(props: &Props) -> Html {
    let shortcuts = use_state(Shortcuts::load);

    let author_name = use_state(|| {
        LocalStorage::get::<String>("author_name").unwrap_or_else(|_| "Wiki User".to_string())
    });

    let author_email = use_state(|| {
        LocalStorage::get::<String>("author_email")
            .unwrap_or_else(|_| "user@example.com".to_string())
    });

    let on_author_name_input = {
        let author_name = author_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            author_name.set(input.value());
        })
    };

    let on_author_email_input = {
        let author_email = author_email.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            author_email.set(input.value());
        })
    };

    let on_shortcut_change = |field: &'static str| {
        let shortcuts = shortcuts.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut s = (*shortcuts).clone();
            match field {
                "edit" => s.edit = input.value(),
                "save" => s.save = input.value(),
                "cancel" => s.cancel = input.value(),
                "search" => s.search = input.value(),
                "toggle_drawer" => s.toggle_drawer = input.value(),
                "copy_link" => s.copy_link = input.value(),
                "undo" => s.undo = input.value(),
                "redo" => s.redo = input.value(),
                "scroll_down" => s.scroll_down = input.value(),
                "scroll_up" => s.scroll_up = input.value(),
                "insert_datetime" => s.insert_datetime = input.value(),
                _ => {}
            }
            shortcuts.set(s);
        })
    };

    let error_msg = use_state(String::new);

    let on_reset_defaults = {
        let shortcuts = shortcuts.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            error_msg.set(String::new());
            shortcuts.set(Shortcuts::default());
        })
    };

    let on_save = {
        let shortcuts = shortcuts.clone();
        let author_name = author_name.clone();
        let author_email = author_email.clone();
        let on_close = props.on_close.clone();
        let on_save_shortcuts = props.on_save_shortcuts.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let current_shortcuts = (*shortcuts).clone();
            if let Err(err) = validate_all_shortcuts(&current_shortcuts) {
                error_msg.set(err);
                return;
            }
            error_msg.set(String::new());

            let _ = LocalStorage::set("shortcuts", current_shortcuts.clone());
            let _ = LocalStorage::set("author_name", (*author_name).clone());
            let _ = LocalStorage::set("author_email", (*author_email).clone());

            on_save_shortcuts.emit(current_shortcuts);
            on_close.emit(());
        })
    };

    let on_cancel = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <div class="modal-overlay" onclick={on_cancel.clone()}>
            <div class="modal" role="dialog" aria-modal="true" aria-labelledby="settings-modal-title" onclick={|e: MouseEvent| e.stop_propagation()}>
                <h2 id="settings-modal-title">{"Settings"}</h2>
                if !(*error_msg).is_empty() {
                    <div role="alert" class="text-danger mb-3">
                        { (*error_msg).clone() }
                    </div>
                }
                <div class="modal-body">
                    <div class="shortcuts-grid">
                        <div class="form-group">
                            <label for="author-name">{"Git Author Name:"}</label>
                            <input id="author-name" type="text" value={(*author_name).clone()} oninput={on_author_name_input} placeholder="e.g. John Doe" />
                        </div>
                        <div class="form-group">
                            <label for="author-email">{"Git Author Email:"}</label>
                            <input id="author-email" type="text" value={(*author_email).clone()} oninput={on_author_email_input} placeholder="e.g. john@example.com" />
                        </div>
                    </div>
                    <hr class="border-0 border-t mt-3 mb-3" />
                    <h3 class="mt-0 mb-1">{"Keyboard Shortcuts"}</h3>
                    <p class="text-sm text-muted mb-3">
                        {"Supports single keys ('u'), modifiers ('Ctrl+E'), and chords ('<Ctrl+p><Ctrl+f>'). Separate alternatives with commas."}
                    </p>
                    <h4 class="text-sm font-semibold mb-2">{"Global & Navigation"}</h4>
                    <div class="shortcuts-grid mb-3">
                        <div class="form-group">
                            <label for="shortcut-search">{"Open Document / Search:"}</label>
                            <input id="shortcut-search" type="text" value={shortcuts.search.clone()} oninput={on_shortcut_change("search")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-edit">{"Edit Page:"}</label>
                            <input id="shortcut-edit" type="text" value={shortcuts.edit.clone()} oninput={on_shortcut_change("edit")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-save">{"Save Page:"}</label>
                            <input id="shortcut-save" type="text" value={shortcuts.save.clone()} oninput={on_shortcut_change("save")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-cancel">{"Cancel / Exit Edit:"}</label>
                            <input id="shortcut-cancel" type="text" value={shortcuts.cancel.clone()} oninput={on_shortcut_change("cancel")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-drawer">{"Toggle Side Panel:"}</label>
                            <input id="shortcut-drawer" type="text" value={shortcuts.toggle_drawer.clone()} oninput={on_shortcut_change("toggle_drawer")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-link">{"Copy Link:"}</label>
                            <input id="shortcut-link" type="text" value={shortcuts.copy_link.clone()} oninput={on_shortcut_change("copy_link")} />
                        </div>
                    </div>
                    <h4 class="text-sm font-semibold mb-2">{"Editor & Vim Mode"}</h4>
                    <div class="shortcuts-grid">
                        <div class="form-group">
                            <label for="shortcut-undo">{"Undo (Vim Normal):"}</label>
                            <input id="shortcut-undo" type="text" value={shortcuts.undo.clone()} oninput={on_shortcut_change("undo")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-redo">{"Redo (Vim Normal):"}</label>
                            <input id="shortcut-redo" type="text" value={shortcuts.redo.clone()} oninput={on_shortcut_change("redo")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-scroll-down">{"Scroll Half-Page Down:"}</label>
                            <input id="shortcut-scroll-down" type="text" value={shortcuts.scroll_down.clone()} oninput={on_shortcut_change("scroll_down")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-scroll-up">{"Scroll Half-Page Up:"}</label>
                            <input id="shortcut-scroll-up" type="text" value={shortcuts.scroll_up.clone()} oninput={on_shortcut_change("scroll_up")} />
                        </div>
                        <div class="form-group">
                            <label for="shortcut-insert-datetime">{"Insert Date/Time:"}</label>
                            <input id="shortcut-insert-datetime" type="text" value={shortcuts.insert_datetime.clone()} oninput={on_shortcut_change("insert_datetime")} />
                        </div>
                    </div>
                </div>
                <div class="actions mt-3">
                    <button class="btn mr-auto" onclick={on_reset_defaults} aria-label="Reset shortcuts to defaults">{"Reset to Defaults"}</button>
                    <button class="btn" onclick={on_cancel} aria-label="Cancel settings changes">{"Cancel"}</button>
                    <button class="btn btn-primary" onclick={on_save} aria-label="Save settings">{"Save"}</button>
                </div>
            </div>
        </div>
    }
}
