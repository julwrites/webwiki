use common::HistoryResponse;
use gloo_net::http::Request;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HistoryModalProps {
    pub is_open: bool,
    pub volume: String,
    pub path: String,
    pub on_close: Callback<()>,
}

#[function_component(HistoryModal)]
pub fn history_modal(props: &HistoryModalProps) -> Html {
    let history_entries = use_state(Vec::new);
    let loading = use_state(|| false);
    let error = use_state(|| Option::<String>::None);

    {
        let is_open = props.is_open;
        let volume = props.volume.clone();
        let path = props.path.clone();
        let history_entries = history_entries.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((is_open, volume, path), move |(open, v, p)| {
            if *open {
                loading.set(true);
                error.set(None);
                let v = v.clone();
                let p = p.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/git/{}/history/{}", v, p);
                    match Request::get(&url).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<HistoryResponse>().await {
                                    Ok(data) => {
                                        history_entries.set(data.entries);
                                    }
                                    Err(e) => {
                                        error.set(Some(format!("Failed to parse history: {}", e)))
                                    }
                                }
                            } else {
                                error.set(Some(format!("Server returned {}", resp.status())));
                            }
                        }
                        Err(e) => error.set(Some(format!("Network error: {}", e))),
                    }
                    loading.set(false);
                });
            }
            || ()
        });
    }

    if !props.is_open {
        return html! {};
    }

    let on_close = props.on_close.clone();

    html! {
        <div class="modal-overlay" onclick={move |_| on_close.emit(())}>
            <div class="modal-content history-modal" onclick={|e: MouseEvent| e.stop_propagation()}>
                <div class="modal-header">
                    <h2>{ format!("History for {}", if props.path.is_empty() { props.volume.clone() } else { props.path.clone() }) }</h2>
                    <button class="icon-btn" onclick={let on_close = props.on_close.clone(); move |_| on_close.emit(())}>{"×"}</button>
                </div>

                <div class="modal-body" style="max-height: 60vh; overflow-y: auto;">
                    if *loading {
                        <p>{"Loading history..."}</p>
                    } else if let Some(err) = &*error {
                        <div class="error-msg">{ err }</div>
                    } else if history_entries.is_empty() {
                        <p>{"No history found."}</p>
                    } else {
                        <div class="history-list" style="display: flex; flex-direction: column; gap: 1rem;">
                            {for history_entries.iter().map(|entry| {
                                let date = js_sys::Date::new_0();
                                date.set_time((entry.timestamp as f64) * 1000.0);
                                let date_str = String::from(date.to_locale_string("en-US", &js_sys::Object::new()));

                                html! {
                                    <div class="history-item" style="border: 1px solid var(--border-color); padding: 0.5rem; border-radius: 4px;">
                                        <div style="font-weight: bold; margin-bottom: 0.25rem;">{ &entry.message }</div>
                                        <div style="font-size: 0.85em; color: var(--text-color); opacity: 0.8;">
                                            { format!("{} <{}>", entry.author_name, entry.author_email) }
                                        </div>
                                        <div style="font-size: 0.8em; color: var(--text-color); opacity: 0.6; margin-top: 0.25rem;">
                                            { format!("{} • {}", date_str, &entry.commit_hash[0..7]) }
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}
