use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub path: String,
    pub matches: Vec<String>,
    pub volume: Option<String>,
}

#[function_component(SearchBar)]
pub fn search_bar() -> Html {
    let query = use_state(String::new);
    let results = use_state(Vec::<SearchResult>::new);
    let is_searching = use_state(|| false);
    let navigator = use_navigator().unwrap();

    let on_input = {
        let query = query.clone();
        let results = results.clone();
        let is_searching = is_searching.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            query.set(value.clone());

            if value.len() < 2 {
                results.set(Vec::new());
                return;
            }

            let results = results.clone();
            let is_searching = is_searching.clone();

            // In a real app we might want to debounce this
            spawn_local(async move {
                is_searching.set(true);
                let encoded_value = js_sys::encode_uri_component(&value);
                let url = format!("/api/search?q={}", encoded_value);
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(data) = resp.json::<Vec<SearchResult>>().await {
                                results.set(data);
                            }
                        }
                    }
                    Err(_) => {
                        results.set(Vec::new());
                    }
                }
                is_searching.set(false);
            });
        })
    };

    let on_select = {
        let query = query.clone();
        let results = results.clone();
        let navigator = navigator.clone();

        Callback::from(move |(volume, path): (Option<String>, String)| {
            query.set(String::new());
            results.set(Vec::new());
            let vol = volume.unwrap_or_else(|| "default".to_string());
            navigator.push(&Route::Wiki { volume: vol, path });
        })
    };

    html! {
        <div class="search-bar">
            <input
                type="text"
                placeholder="Search..."
                value={(*query).clone()}
                oninput={on_input}
            />
            if !results.is_empty() {
                <div class="search-results">
                    {for results.iter().map(|r| {
                        let path = r.path.clone();
                        let volume = r.volume.clone();
                        let on_click = {
                            let on_select = on_select.clone();
                            let path = path.clone();
                            let volume = volume.clone();
                            move |_| on_select.emit((volume.clone(), path.clone()))
                        };

                        let display_path = if let Some(ref v) = r.volume {
                            format!("{}: {}", v, r.path)
                        } else {
                            r.path.clone()
                        };

                        html! {
                            <div class="search-result-item" onclick={on_click}>
                                <div class="path">{display_path}</div>
                                <div class="snippet">{r.matches.first().unwrap_or(&String::new())}</div>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}
