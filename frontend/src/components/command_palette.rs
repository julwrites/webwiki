use common::FileNode;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::hooks::use_create_file;
use crate::search_bar::SearchResult;
use crate::Route;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub is_open: bool,
    pub on_close: Callback<()>,
    pub on_theme_toggle: Callback<()>,
    pub current_volume: String,
}

#[derive(Clone, PartialEq)]
enum CommandType {
    Navigation(Route), // Route
    Action(Callback<()>),
    Search(String, Option<String>, Option<String>), // Path, Volume, Matches
    CreateFile,
}

#[derive(Clone, PartialEq)]
struct CommandItem {
    title: String,
    description: String,
    command_type: CommandType,
}

fn flatten_tree(node: &FileNode, acc: &mut Vec<String>) {
    if !node.is_dir {
        acc.push(node.path.clone());
    }
    if let Some(children) = &node.children {
        for child in children {
            flatten_tree(child, acc);
        }
    }
}

#[function_component(CommandPalette)]
pub fn command_palette(props: &Props) -> Html {
    let query = use_state(String::new);
    let selected_index = use_state(|| 0);
    let search_results = use_state(Vec::<SearchResult>::new);
    let file_list = use_state(Vec::<String>::new); // Client-side file list
    let navigator = use_navigator().unwrap();
    let input_ref = use_node_ref();
    let debounce_timer = use_state(|| None::<gloo_timers::callback::Timeout>);
    let last_request_timestamp = use_state(|| 0.0);

    // Fetch file tree when volume changes or palette opens
    {
        let file_list = file_list.clone();
        let current_volume = props.current_volume.clone();
        let is_open = props.is_open;

        use_effect_with((current_volume.clone(), is_open), move |(volume, open)| {
            if *open {
                let volume = volume.clone();
                spawn_local(async move {
                    let url = format!("/api/tree?volume={}", volume);
                    if let Ok(resp) = Request::get(&url).send().await {
                        if resp.ok() {
                            if let Ok(nodes) = resp.json::<Vec<FileNode>>().await {
                                let mut paths = Vec::new();
                                for node in nodes {
                                    flatten_tree(&node, &mut paths);
                                }
                                file_list.set(paths);
                            }
                        }
                    }
                });
            }
            || ()
        });
    }

    let static_commands = {
        let on_theme_toggle = props.on_theme_toggle.clone();
        use_memo((), move |_| {
            let on_theme_toggle = on_theme_toggle.clone();
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
                        on_theme_toggle.emit(());
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

    // Filtered commands + file matches + server search results
    let filtered_items = {
        let query = query.clone();
        let static_commands = static_commands.clone();
        let search_results = search_results.clone();
        let file_list = file_list.clone();
        let current_volume = props.current_volume.clone();

        use_memo(
            (
                (*query).clone(),
                (*search_results).clone(),
                (*file_list).clone(),
                current_volume,
            ),
            move |(q, results, files, volume)| {
                let mut items = Vec::new();
                let q_lower = q.to_lowercase();

                // 1. Static commands
                for cmd in static_commands.iter() {
                    if cmd.title.to_lowercase().contains(&q_lower)
                        || cmd.description.to_lowercase().contains(&q_lower)
                    {
                        items.push(cmd.clone());
                    }
                }

                // 2. Client-side file matches (if query length > 1)
                if q.len() > 1 {
                    let mut file_matches = 0;
                    for path in files {
                        if file_matches >= 10 {
                            break;
                        } // Limit file results
                        if path.to_lowercase().contains(&q_lower) {
                            items.push(CommandItem {
                                title: path.clone(),
                                description: format!("File in {}", volume),
                                command_type: CommandType::Navigation(Route::Wiki {
                                    volume: volume.clone(),
                                    path: path.clone(),
                                }),
                            });
                            file_matches += 1;
                        }
                    }
                }

                // 3. Server search results
                for result in results.iter() {
                    // Avoid duplicates if client-side found it (simple check)
                    // (Optional optimization: check if path is already in items)

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
                            result.matches.first().cloned(),
                        ),
                    });
                }

                items
            },
        )
    };

    // Reset state when opening
    {
        let is_open = props.is_open;
        let query = query.clone();
        let selected_index = selected_index.clone();
        let search_results = search_results.clone();
        let input_ref = input_ref.clone();

        use_effect_with(is_open, move |open| {
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

    let create_file = use_create_file(props.current_volume.clone());

    let execute_command = {
        let on_close = props.on_close.clone();
        let navigator = navigator.clone();
        let create_file = create_file.clone();

        Callback::from(move |item: CommandItem| {
            on_close.emit(());
            match item.command_type {
                CommandType::Navigation(route) => navigator.push(&route),
                CommandType::Search(path, volume, _) => navigator.push(&Route::Wiki {
                    volume: volume.unwrap_or_else(|| "default".to_string()),
                    path,
                }),
                CommandType::Action(cb) => cb.emit(()),
                CommandType::CreateFile => {
                    create_file.emit(());
                }
            }
        })
    };

    let on_keydown = {
        let selected_index = selected_index.clone();
        let filtered_items = filtered_items.clone();
        let execute_command = execute_command.clone();
        let on_close = props.on_close.clone();

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
                    on_close.emit(());
                }
                _ => {}
            }
        })
    };

    if !props.is_open {
        return html! {};
    }

    let on_close = props.on_close.clone();

    html! {
        <div class="command-palette-overlay" onclick={move |_| on_close.emit(())}>
            <div class={classes!("command-palette-modal", "bottom-sheet")} onclick={|e: MouseEvent| e.stop_propagation()}>
                <div class="command-palette-handle"></div>
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
