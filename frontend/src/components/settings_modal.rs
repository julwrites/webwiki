use crate::hooks::KeyBindings;
use gloo_storage::{LocalStorage, Storage};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_close: Callback<()>,
}

#[function_component(SettingsModal)]
pub fn settings_modal(props: &Props) -> Html {
    let keybindings = use_state(|| {
        LocalStorage::get::<KeyBindings>("key_bindings").unwrap_or_else(|_| KeyBindings::default())
    });

    let author_name = use_state(|| {
        LocalStorage::get::<String>("author_name").unwrap_or_else(|_| "Wiki User".to_string())
    });

    let author_email = use_state(|| {
        LocalStorage::get::<String>("author_email")
            .unwrap_or_else(|_| "user@example.com".to_string())
    });

    let on_leader_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.leader = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_pull_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.pull = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_push_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.push = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_commit_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.commit = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_search_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.search = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_new_file_input = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.new_file = input.value();
            keybindings.set(new_kb);
        })
    };

    let on_edit_change = {
        let keybindings = keybindings.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_kb = (*keybindings).clone();
            new_kb.edit = input
                .value()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            keybindings.set(new_kb);
        })
    };

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

    let error_msg = use_state(String::new);

    let on_save = {
        let keybindings = keybindings.clone();
        let author_name = author_name.clone();
        let author_email = author_email.clone();
        let on_close = props.on_close.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let kb = (*keybindings).clone();

            // Validation to prevent conflicts
            let actions = [
                (&kb.pull, "Pull"),
                (&kb.push, "Push"),
                (&kb.commit, "Commit"),
                (&kb.search, "Search"),
                (&kb.new_file, "New File"),
            ];

            // Check for duplicates in main actions
            let mut has_conflict = false;
            for i in 0..actions.len() {
                for j in (i + 1)..actions.len() {
                    if actions[i].0 == actions[j].0 {
                        error_msg.set(format!(
                            "Conflict: {} and {} have the same keybinding",
                            actions[i].1, actions[j].1
                        ));
                        has_conflict = true;
                        break;
                    }
                }
                if has_conflict {
                    break;
                }
            }

            if has_conflict {
                return;
            }

            error_msg.set(String::new());

            let _ = LocalStorage::set("key_bindings", kb);
            let _ = LocalStorage::set("author_name", (*author_name).clone());
            let _ = LocalStorage::set("author_email", (*author_email).clone());

            // It triggers immediately after local storage is set.
            // When user tries to type the new commands, use_key_handler needs to pick it up.
            // Right now use_key_handler only picks up on mount. We will fix that later or force reload.
            // For now, prompt the user that a reload is required for changes to take effect,
            // or we'll solve it in the hook logic directly.
            gloo_dialogs::alert(
                "Keybindings saved! A page reload might be required for changes to fully apply.",
            );
            on_close.emit(());
        })
    };

    let on_cancel = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let edit_keys_str = keybindings.edit.join(", ");

    html! {
        <div class="modal-overlay" onclick={on_cancel.clone()}>
            <div class="modal" onclick={|e: MouseEvent| e.stop_propagation()}>
                <h2>{"Settings"}</h2>
                if !(*error_msg).is_empty() {
                    <div style="color: var(--color-danger-fg); margin-bottom: 15px;">
                        { (*error_msg).clone() }
                    </div>
                }
                <div style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 20px;">
                    <div>
                        <label for="author-name">{"Git Author Name: "}</label>
                        <input id="author-name" type="text" value={(*author_name).clone()} oninput={on_author_name_input} />
                    </div>
                    <div>
                        <label for="author-email">{"Git Author Email: "}</label>
                        <input id="author-email" type="text" value={(*author_email).clone()} oninput={on_author_email_input} />
                    </div>
                    <hr style="margin: 10px 0; border: 0; border-top: 1px solid var(--border-color);" />
                    <div>
                        <label for="leader-key">{"Leader Key: "}</label>
                        <input id="leader-key" type="text" value={keybindings.leader.clone()} oninput={on_leader_input} />
                    </div>
                    <div>
                        <label for="pull-action">{"Pull Action: "}</label>
                        <input id="pull-action" type="text" value={keybindings.pull.clone()} oninput={on_pull_input} />
                    </div>
                    <div>
                        <label for="push-action">{"Push Action: "}</label>
                        <input id="push-action" type="text" value={keybindings.push.clone()} oninput={on_push_input} />
                    </div>
                    <div>
                        <label for="commit-action">{"Commit Action: "}</label>
                        <input id="commit-action" type="text" value={keybindings.commit.clone()} oninput={on_commit_input} />
                    </div>
                    <div>
                        <label for="search-action">{"Search Action: "}</label>
                        <input id="search-action" type="text" value={keybindings.search.clone()} oninput={on_search_input} />
                    </div>
                    <div>
                        <label for="new-file-action">{"New File Action: "}</label>
                        <input id="new-file-action" type="text" value={keybindings.new_file.clone()} oninput={on_new_file_input} />
                    </div>
                    <div>
                        <label for="edit-action">{"Edit Action (comma separated): "}</label>
                        <input id="edit-action" type="text" value={edit_keys_str} onchange={on_edit_change} />
                    </div>
                </div>
                <div style="display: flex; justify-content: flex-end; gap: 10px;">
                    <button onclick={on_cancel}>{"Cancel"}</button>
                    <button onclick={on_save} style="background-color: var(--color-accent-fg); color: #ffffff; border-color: transparent;">{"Save"}</button>
                </div>
            </div>
        </div>
    }
}
