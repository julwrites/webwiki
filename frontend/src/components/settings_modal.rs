use gloo_storage::{LocalStorage, Storage};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use crate::hooks::Shortcuts;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_close: Callback<()>,
}

#[function_component(SettingsModal)]
pub fn settings_modal(props: &Props) -> Html {
    let shortcuts = use_state(|| LocalStorage::get::<Shortcuts>("shortcuts").unwrap_or_else(|_| Shortcuts::default()));
    
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
                _ => {}
            }
            shortcuts.set(s);
        })
    };

    let error_msg = use_state(String::new);

    let on_save = {
        let shortcuts = shortcuts.clone();
        let author_name = author_name.clone();
        let author_email = author_email.clone();
        let on_close = props.on_close.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            error_msg.set(String::new());

            let _ = LocalStorage::set("shortcuts", (*shortcuts).clone());
            let _ = LocalStorage::set("author_name", (*author_name).clone());
            let _ = LocalStorage::set("author_email", (*author_email).clone());

            gloo_dialogs::alert("Settings saved! Reload the page to apply new shortcuts.");
            on_close.emit(());
        })
    };

    let on_cancel = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <div class="modal-overlay" onclick={on_cancel.clone()}>
            <div class="modal" onclick={|e: MouseEvent| e.stop_propagation()}>
                <h2>{"Settings"}</h2>
                if !(*error_msg).is_empty() {
                    <div role="alert" style="color: var(--color-danger-fg); margin-bottom: 15px;">
                        { (*error_msg).clone() }
                    </div>
                }
                <div class="modal-body" style="max-height: 70vh; overflow-y: auto;">
                    <div class="form-group">
                        <label for="author-name">{"Git Author Name:"}</label>
                        <input id="author-name" type="text" value={(*author_name).clone()} oninput={on_author_name_input} placeholder="e.g. John Doe" />
                    </div>
                    <div class="form-group">
                        <label for="author-email">{"Git Author Email:"}</label>
                        <input id="author-email" type="text" value={(*author_email).clone()} oninput={on_author_email_input} placeholder="e.g. john@example.com" />
                    </div>
                    <hr style="margin: 10px 0; border: 0; border-top: 1px solid var(--border-color);" />
                    <h3>{"Keyboard Shortcuts"}</h3>
                    <p style="font-size: 0.9em; color: var(--color-muted-fg); margin-bottom: 15px;">
                        {"Use modifiers like Ctrl, Cmd, Shift, Alt. Combine with '+'. Example: 'Ctrl+Shift+C'. Separate alternatives with commas."}
                    </p>
                    <div class="form-group">
                        <label for="shortcut-edit">{"Edit Page:"}</label>
                        <input id="shortcut-edit" type="text" value={shortcuts.edit.clone()} oninput={on_shortcut_change("edit")} />
                    </div>
                    <div class="form-group">
                        <label for="shortcut-save">{"Save Page:"}</label>
                        <input id="shortcut-save" type="text" value={shortcuts.save.clone()} oninput={on_shortcut_change("save")} />
                    </div>
                    <div class="form-group">
                        <label for="shortcut-cancel">{"Cancel/Exit Edit:"}</label>
                        <input id="shortcut-cancel" type="text" value={shortcuts.cancel.clone()} oninput={on_shortcut_change("cancel")} />
                    </div>
                    <div class="form-group">
                        <label for="shortcut-search">{"Quick Search:"}</label>
                        <input id="shortcut-search" type="text" value={shortcuts.search.clone()} oninput={on_shortcut_change("search")} />
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
                <div class="actions">
                    <button class="btn" onclick={on_cancel} aria-label="Cancel settings changes">{"Cancel"}</button>
                    <button class="btn btn-primary" onclick={on_save} aria-label="Save settings">{"Save"}</button>
                </div>
            </div>
        </div>
    }
}
