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

    let error_msg = use_state(String::new);

    let on_save = {
        let keybindings = keybindings.clone();
        let on_close = props.on_close.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let kb = (*keybindings).clone();

            // Validation to prevent conflicts
            let actions = [
                (&kb.pull, "Pull"),
                (&kb.push, "Push"),
                (&kb.commit, "Commit"),
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
                <h2>{"Configure Keybindings"}</h2>
                if !(*error_msg).is_empty() {
                    <div style="color: var(--color-danger-fg); margin-bottom: 15px;">
                        { (*error_msg).clone() }
                    </div>
                }
                <div style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 20px;">
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
