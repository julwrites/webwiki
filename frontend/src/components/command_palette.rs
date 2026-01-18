use gloo_net::http::Request;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::search_bar::SearchResult;
use crate::Route;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_theme_toggle: Callback<MouseEvent>,
    pub current_volume: String,
}

#[derive(Clone, PartialEq)]
enum CommandType {
    Navigation(Route), // Route
    Action(Callback<()>),
    Search(String, Option<String>), // Path, Volume
    CreateFile,
}

#[derive(Clone, PartialEq)]
struct CommandItem {
    title: String,
    description: String,
    command_type: CommandType,
}

#[function_component(CommandPalette)]
pub fn command_palette(props: &Props) -> Html {
    let is_open = use_state(|| false);
    let query = use_state(String::new);
    let selected_index = use_state(|| 0);
    let search_results = use_state(Vec::<SearchResult>::new);
    let navigator = use_navigator().unwrap();
    let input_ref = use_node_ref();
    let debounce_timer = use_state(|| None::<gloo_timers::callback::Timeout>);
    let last_request_timestamp = use_state(|| 0.0);

    let static_commands = {
        let on_theme_toggle = props.on_theme_toggle.clone();
        use_memo((), move |_| {
            vec![
                CommandItem {
                    title: "Go to Home".to_string(),
                    description: "Navigate to the home page".to_string(),
                    command_type: CommandType::Navigation(Route::Home),
                },
                CommandItem {
                    title: "Toggle Theme".to_string(),
                    description: "Switch between light and dark mode".to_string(),
                    command_type: CommandType::Action(Callback::from(move |_| {
                        on_theme_toggle.emit(MouseEvent::new("click").unwrap());
                    })),
                },
                CommandItem {
                    title: "Create New File".to_string(),
                    description: "Create a new file in the current volume".to_string(),
                    command_type: CommandType::CreateFile,
                },
            ]
        })
    };

    // Filtered commands + search results
    let filtered_items = {
        let query = query.clone();
        let static_commands = static_commands.clone();
        let search_results = search_results.clone();

        use_memo(
            ((*query).clone(), (*search_results).clone()),
            move |(q, results)| {
                let mut items = Vec::new();
                let q_lower = q.to_lowercase();

                // Filter static commands
                for cmd in static_commands.iter() {
                    if cmd.title.to_lowercase().contains(&q_lower)
                        || cmd.description.to_lowercase().contains(&q_lower)
                    {
                        items.push(cmd.clone());
                    }
                }

                // Add search results
                for result in results.iter() {
                    let title = if let Some(ref v) = result.volume {
                        format!("{}: {}", v, result.path)
                    } else {
                        result.path.clone()
                    };
                    items.push(CommandItem {
                        title,
                        description: result.matches.first().cloned().unwrap_or_default(),
                        command_type: CommandType::Search(
                            result.path.clone(),
                            result.volume.clone(),
                        ),
                    });
                }

                items
            },
        )
    };

    // Keyboard listener for opening/closing
    {
        let is_open = is_open.clone();
        use_effect(move || {
            let window = gloo_utils::window();

            let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
                    e.prevent_default();
                    is_open.set(!*is_open);
                } else if e.key() == "Escape" && *is_open {
                    is_open.set(false);
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

    // Reset state when opening
    {
        let is_open = is_open.clone();
        let query = query.clone();
        let selected_index = selected_index.clone();
        let search_results = search_results.clone();
        let input_ref = input_ref.clone();

        use_effect_with(*is_open, move |open| {
            if *open {
                query.set(String::new());
                selected_index.set(0);
                search_results.set(Vec::new());

                // Focus input
                let input_ref = input_ref.clone();
                let timeout = gloo_timers::callback::Timeout::new(50, move || {
                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        let _ = input.focus();
                    }
                });
                timeout.forget();
            }
            || ()
        });
    }

    let on_input = {
        let query = query.clone();
        let search_results = search_results.clone();
        let selected_index = selected_index.clone();
        let debounce_timer = debounce_timer.clone();
        let last_request_timestamp = last_request_timestamp.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            query.set(value.clone());
            selected_index.set(0);

            // Clear previous timer
            debounce_timer.set(None);

            if value.len() < 2 {
                search_results.set(Vec::new());
                return;
            }

            let search_results = search_results.clone();
            let last_request_timestamp = last_request_timestamp.clone();

            // Debounce
            let timer = gloo_timers::callback::Timeout::new(300, move || {
                let current_timestamp = js_sys::Date::now();
                last_request_timestamp.set(current_timestamp);

                spawn_local(async move {
                    let encoded_value = js_sys::encode_uri_component(&value);
                    let url = format!("/api/search?q={}", encoded_value);
                    match Request::get(&url).send().await {
                        Ok(resp) if resp.ok() => {
                            // Race condition check:
                            // Ideally we'd compare request IDs, but timestamp check on receive
                            // isn't perfect if requests return out of order.
                            // However, with debouncing, we only send one request every 300ms.
                            // A simple check is to verify if the query still matches the current input.
                            // But we can't easily access the live input value here without another ref.
                            // Instead, we just trust the latest response for now as debouncing mitigates mostly.
                            // For robust race handling, we'd need to capture the timestamp in the closure
                            // and compare it against last_request_timestamp, but that needs to be updated.

                            // Let's implement a simple "latest wins" by checking if this task's timestamp
                            // matches the latest one.

                            if *last_request_timestamp == current_timestamp {
                                if let Ok(data) = resp.json::<Vec<SearchResult>>().await {
                                    search_results.set(data);
                                }
                            }
                        }
                        _ => {
                            if *last_request_timestamp == current_timestamp {
                                search_results.set(Vec::new());
                            }
                        }
                    }
                });
            });

            debounce_timer.set(Some(timer));
        })
    };

    let execute_command = {
        let is_open = is_open.clone();
        let navigator = navigator.clone();
        let current_volume = props.current_volume.clone();

        Callback::from(move |item: CommandItem| {
            is_open.set(false);
            match item.command_type {
                CommandType::Navigation(route) => navigator.push(&route),
                CommandType::Search(path, volume) => navigator.push(&Route::Wiki {
                    volume: volume.unwrap_or_else(|| "default".to_string()),
                    path,
                }),
                CommandType::Action(cb) => cb.emit(()),
                CommandType::CreateFile => {
                    if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None) {
                        if !path.trim().is_empty() {
                            navigator.push(&Route::Wiki {
                                volume: current_volume.clone(),
                                path: path.trim().to_string(),
                            });
                        }
                    }
                }
            }
        })
    };

    let on_keydown = {
        let selected_index = selected_index.clone();
        let filtered_items = filtered_items.clone();
        let execute_command = execute_command.clone();
        let is_open = is_open.clone();

        Callback::from(move |e: KeyboardEvent| {
            let max_index = if filtered_items.is_empty() {
                0
            } else {
                filtered_items.len() - 1
            };

            match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    if *selected_index < max_index {
                        selected_index.set(*selected_index + 1);
                    }
                }
                "ArrowUp" => {
                    e.prevent_default();
                    if *selected_index > 0 {
                        selected_index.set(*selected_index - 1);
                    }
                }
                "Enter" => {
                    e.prevent_default();
                    if !filtered_items.is_empty() {
                        if let Some(item) = filtered_items.get(*selected_index) {
                            execute_command.emit(item.clone());
                        }
                    }
                }
                "Escape" => {
                    e.prevent_default();
                    is_open.set(false);
                }
                _ => {}
            }
        })
    };

    if !*is_open {
        return html! {};
    }

    html! {
        <div class="command-palette-overlay" onclick={let is_open = is_open.clone(); move |_| is_open.set(false)}>
            <div class="command-palette-modal" onclick={|e: MouseEvent| e.stop_propagation()}>
                <input
                    ref={input_ref}
                    id="command-palette-input"
                    type="text"
                    class="command-palette-input"
                    placeholder="Type a command or search..."
                    value={(*query).clone()}
                    oninput={on_input}
                    onkeydown={on_keydown}
                    autocomplete="off"
                />
                <div class="command-palette-results">
                    {for filtered_items.iter().enumerate().map(|(index, item)| {
                        let is_selected = index == *selected_index;
                        let item_clone = item.clone();
                        let execute_command = execute_command.clone();
                        let onclick = Callback::from(move |_| {
                            execute_command.emit(item_clone.clone());
                        });

                        html! {
                            <div
                                class={classes!("command-palette-item", if is_selected { "selected" } else { "" })}
                                onclick={onclick}
                            >
                                <div class="command-palette-item-title">{ &item.title }</div>
                                <div class="command-palette-item-desc">{ &item.description }</div>
                            </div>
                        }
                    })}
                    if filtered_items.is_empty() {
                        <div class="command-palette-empty">{"No results found"}</div>
                    }
                </div>
            </div>
        </div>
    }
}
