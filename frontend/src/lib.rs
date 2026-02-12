mod commit_modal;
mod components;
mod hooks;
mod parsers;
mod search_bar;

use commit_modal::CommitModal;
use common::WikiPage;
use components::bottom_bar::BottomBar;
use components::command_palette::CommandPalette;
use components::drawer::Drawer;
use gloo_net::http::Request;
use gloo_storage::Storage;
use hooks::{use_create_file, use_key_handler, KeyHandlerProps};
use parsers::WikiLinkParser;
use pulldown_cmark::{html, Options, Parser};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setupEditor(
        element_id: &str,
        initial_content: &str,
        callback: &Closure<dyn FnMut(String)>,
        vim_mode: bool,
    );
    fn wrapSelection(element_id: &str, prefix: &str, suffix: &str);
    fn insertTextAtCursor(element_id: &str, text: &str);
    fn toggleHeader(element_id: &str, level: i32);
    fn renderMermaid();
    fn renderGraphviz(element_id: &str, content: &str);
    fn renderDrawio(element_id: &str, xml: &str);
}

#[derive(Clone, Routable, PartialEq)]
pub(crate) enum Route {
    #[at("/wiki/:volume/*path")]
    Wiki { volume: String, path: String },
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout />
        </BrowserRouter>
    }
}

use serde::Deserialize;

#[derive(Clone, Deserialize, Default)]
struct GitStatus {
    commits_ahead: usize,
    commits_behind: usize,
}

#[function_component(Layout)]
fn layout() -> Html {
    let route = use_route::<Route>();
    let navigator = use_navigator().unwrap();
    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    let show_commit_modal = use_state(|| false);
    let is_drawer_open = use_state(|| false);
    let is_search_open = use_state(|| false);
    let is_editing = use_state(|| false);

    let commits_ahead = use_state(|| 0);
    let commits_behind = use_state(|| 0);

    // Reset editing state on navigation
    {
        let is_editing = is_editing.clone();
        use_effect_with(current_volume.clone(), move |_| {
            is_editing.set(false);
            || ()
        });
    }

    let vim_mode = use_state(|| {
        if let Ok(stored) = gloo_storage::LocalStorage::get("vim_mode") {
            return stored;
        }
        // Check User Agent
        let window = gloo_utils::window();
        let navigator = window.navigator();
        if let Ok(ua) = navigator.user_agent() {
            if ua.to_lowercase().contains("mobile") {
                return false;
            }
        }
        true
    });

    // Theme state
    let theme = use_state(|| {
        let storage = gloo_storage::LocalStorage::get("theme");
        storage.unwrap_or_else(|_| "dark".to_string())
    });

    {
        let theme = theme.clone();
        use_effect_with((*theme).clone(), move |theme_val| {
            let _ = gloo_utils::document()
                .document_element()
                .map(|el| el.set_attribute("data-theme", theme_val));
            || ()
        });
    }

    // Fetch Git Status Effect
    {
        let volume = current_volume.clone();
        let commits_ahead = commits_ahead.clone();
        let commits_behind = commits_behind.clone();
        use_effect_with(volume, move |volume| {
            let volume = volume.clone();
            let commits_ahead = commits_ahead.clone();
            let commits_behind = commits_behind.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/git/{}/fetch", volume);
                let resp = Request::post(&url).send().await;
                if let Ok(r) = resp {
                    if let Ok(status) = r.json::<GitStatus>().await {
                        commits_ahead.set(status.commits_ahead);
                        commits_behind.set(status.commits_behind);
                    }
                }
            });
            || ()
        });
    }

    let refresh_git_status = {
        let volume = current_volume.clone();
        let commits_ahead = commits_ahead.clone();
        let commits_behind = commits_behind.clone();
        Callback::from(move |_| {
            let volume = volume.clone();
            let commits_ahead = commits_ahead.clone();
            let commits_behind = commits_behind.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/git/{}/fetch", volume);
                let resp = Request::post(&url).send().await;
                if let Ok(r) = resp {
                    if let Ok(status) = r.json::<GitStatus>().await {
                        commits_ahead.set(status.commits_ahead);
                        commits_behind.set(status.commits_behind);
                    }
                }
            });
        })
    };

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new_theme = if *theme == "dark" { "light" } else { "dark" };
            let _ = gloo_storage::LocalStorage::set("theme", new_theme);
            theme.set(new_theme.to_string());
        })
    };

    // Actions
    let on_toggle_drawer = {
        let is_drawer_open = is_drawer_open.clone();
        Callback::from(move |_| is_drawer_open.set(!*is_drawer_open))
    };

    let on_search_trigger = {
        let is_search_open = is_search_open.clone();
        Callback::from(move |_| is_search_open.set(true))
    };

    let on_edit_trigger = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| is_editing.set(true))
    };

    let on_edit_toggle = {
        let is_editing = is_editing.clone();
        Callback::from(move |val: bool| is_editing.set(val))
    };

    let on_pull_click = {
        let volume = current_volume.clone();
        let refresh = refresh_git_status.clone();
        Callback::from(move |_| {
            let volume = volume.clone();
            let refresh = refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/git/{}/pull", volume);
                let resp = Request::post(&url).send().await;
                match resp {
                    Ok(r) if r.ok() => {
                        gloo_dialogs::alert("Successfully pulled from remote!");
                        refresh.emit(());
                    }
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        gloo_dialogs::alert(&format!("Failed to pull: {}", text));
                    }
                    Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                }
            });
        })
    };

    let on_push_click = {
        let volume = current_volume.clone();
        let refresh = refresh_git_status.clone();
        Callback::from(move |_| {
            let volume = volume.clone();
            let refresh = refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/git/{}/push", volume);
                let resp = Request::post(&url).send().await;
                match resp {
                    Ok(r) if r.ok() => {
                        gloo_dialogs::alert("Successfully pushed to remote!");
                        refresh.emit(());
                    }
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        gloo_dialogs::alert(&format!("Failed to push: {}", text));
                    }
                    Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                }
            });
        })
    };

    let on_commit_click = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(true))
    };

    let on_close_commit_modal = {
        let show_commit_modal = show_commit_modal.clone();
        let refresh = refresh_git_status.clone();
        Callback::from(move |_| {
            show_commit_modal.set(false);
            refresh.emit(());
        })
    };

    let on_home_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Home))
    };

    let on_new_file_click = use_create_file(current_volume.clone());

    // Key Handler
    use_key_handler(KeyHandlerProps {
        on_search: {
            let is_search_open = is_search_open.clone();
            Callback::from(move |_| is_search_open.set(true))
        },
        on_pull: on_pull_click.clone(),
        on_push: on_push_click.clone(),
        on_commit: on_commit_click.clone(),
        on_edit: on_edit_trigger.clone(),
    });

    let is_dark = *theme == "dark";

    html! {
        <div class="container">
            <Drawer
                is_open={*is_drawer_open}
                on_close={let is_drawer_open = is_drawer_open.clone(); move |_| is_drawer_open.set(false)}
            />

            <main class="content">
                <Switch<Route> render={
                    let vim_mode = *vim_mode;
                    let is_editing_val = *is_editing;
                    let on_edit_toggle = on_edit_toggle.clone();
                    move |routes| switch(routes, vim_mode, is_editing_val, on_edit_toggle.clone())
                } />
            </main>

            <BottomBar
                on_toggle_drawer={on_toggle_drawer}
                on_search={on_search_trigger}
                on_pull={on_pull_click}
                on_push={on_push_click}
                on_commit={on_commit_click}
                on_edit={on_edit_trigger}
                on_home={on_home_click}
                on_new_file={on_new_file_click}
                on_theme_toggle={toggle_theme.clone()}
                is_dark={is_dark}
                commits_ahead={*commits_ahead}
                commits_behind={*commits_behind}
                is_drawer_open={*is_drawer_open}
            />

            <CommandPalette
                is_open={*is_search_open}
                on_close={let is_search_open = is_search_open.clone(); move |_| is_search_open.set(false)}
                on_theme_toggle={toggle_theme.clone()}
                current_volume={current_volume.clone()}
            />

            if *show_commit_modal {
                <CommitModal on_close={on_close_commit_modal} volume={current_volume} />
            }
        </div>
    }
}

// Switch function
fn switch(routes: Route, vim_mode: bool, is_editing: bool, on_edit_toggle: Callback<bool>) -> Html {
    match routes {
        Route::Wiki { volume, path } => {
            html! { <WikiViewer
                volume={volume}
                path={path}
                vim_mode={vim_mode}
                is_editing={is_editing}
                on_edit_toggle={on_edit_toggle}
            /> }
        }
        Route::Home => {
            html! { <WikiViewer
                volume={"default".to_string()}
                path={"index.md".to_string()}
                vim_mode={vim_mode}
                is_editing={is_editing}
                on_edit_toggle={on_edit_toggle}
            /> }
        }
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}

#[derive(Properties, PartialEq, Clone)]
struct WikiViewerProps {
    volume: String,
    path: String,
    vim_mode: bool,
    is_editing: bool,
    on_edit_toggle: Callback<bool>,
}

#[derive(PartialEq)]
enum ViewMode {
    Loading,
    Page(WikiPage),
    Image(String),
    Pdf(String),
    Error(String),
}

#[function_component(WikiViewer)]
fn wiki_viewer(props: &WikiViewerProps) -> Html {
    let view_mode = use_state(|| ViewMode::Loading);

    // We use the prop to control editing state
    let is_editing = props.is_editing;
    let on_edit_toggle = props.on_edit_toggle.clone();

    let path = props.path.clone();
    let volume = props.volume.clone();
    let vim_mode = props.vim_mode;

    {
        let view_mode = view_mode.clone();
        let path = path.clone();
        let volume = volume.clone();
        use_effect_with((volume.clone(), path.clone()), move |_| {
            let view_mode = view_mode.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/wiki/{}/{}", volume, path);
                let resp = Request::get(&url).send().await;

                match resp {
                    Ok(r) if r.ok() => {
                        let content_type = r.headers().get("Content-Type").unwrap_or_default();
                        if content_type.contains("application/json") {
                            let page: WikiPage = r.json().await.unwrap_or_else(|_| WikiPage {
                                path: path.clone(),
                                content: "Error parsing JSON".to_string(),
                            });
                            view_mode.set(ViewMode::Page(page));
                        } else if content_type.starts_with("image/") {
                            view_mode.set(ViewMode::Image(url));
                        } else if content_type == "application/pdf" {
                            view_mode.set(ViewMode::Pdf(url));
                        } else {
                            // Handle known binary types or unknown types that fall through
                            // If it's not JSON (WikiPage), Image, or PDF, it's something we can't display.
                            // The backend returns raw bytes for these.
                            view_mode.set(ViewMode::Error(format!(
                                "Unsupported file type: {}",
                                content_type
                            )));
                        }
                    }
                    _ => view_mode.set(ViewMode::Page(WikiPage {
                        path: path.clone(),
                        content: "# Page Not Found\n\nClick edit to create it.".to_string(),
                    })),
                }
            });
            || ()
        });
    }

    // Effect to trigger diagram rendering when content changes or editing ends
    {
        let view_mode = view_mode.clone();
        use_effect_with(
            (view_mode.clone(), is_editing),
            move |(view_mode, is_editing)| {
                // Only render if we are NOT editing and have a page
                if !*is_editing {
                    if let ViewMode::Page(page) = &**view_mode {
                        let ext = std::path::Path::new(&page.path)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        match ext.as_str() {
                            "mermaid" | "mmd" => renderMermaid(),
                            "dot" => renderGraphviz("graphviz-container", &page.content),
                            "drawio" | "dio" => renderDrawio("drawio-container", &page.content),
                            _ => {}
                        }
                    }
                }
                || ()
            },
        );
    }

    let on_edit_click = {
        let on_edit_toggle = on_edit_toggle.clone();
        Callback::from(move |_| on_edit_toggle.emit(true))
    };

    let on_delete_click = {
        let path = path.clone();
        let volume = volume.clone();
        Callback::from(move |_| {
            if gloo_dialogs::confirm(&format!("Are you sure you want to delete {}?", path)) {
                let path = path.clone();
                let volume = volume.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/wiki/{}/{}", volume, path);
                    let resp = Request::delete(&url).send().await;
                    match resp {
                        Ok(r) if r.ok() => {
                            gloo_dialogs::alert("File deleted.");
                            // Force reload to update file tree
                            let _ = gloo_utils::window().location().set_href("/");
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
    };

    let on_save = {
        let path = path.clone();
        let volume = volume.clone();
        let view_mode = view_mode.clone();
        let on_edit_toggle = on_edit_toggle.clone();

        Callback::from(move |new_content: String| {
            let path = path.clone();
            let volume = volume.clone();
            let view_mode = view_mode.clone();
            let on_edit_toggle = on_edit_toggle.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let page = WikiPage {
                    path: path.clone(),
                    content: new_content.clone(),
                };
                let req = Request::put(&format!("/api/wiki/{}/{}", volume, path))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&page).unwrap());

                if let Ok(req) = req {
                    let resp = req.send().await;
                    if let Ok(r) = resp {
                        if r.ok() {
                            view_mode.set(ViewMode::Page(page));
                            on_edit_toggle.emit(false);
                        } else {
                            gloo_dialogs::alert(&format!("Failed to save: {}", r.status()));
                        }
                    }
                }
            });
        })
    };

    if is_editing {
        let current_content = match &*view_mode {
            ViewMode::Page(p) => p.content.clone(),
            _ => String::new(),
        };

        html! {
             <div class="wiki-editor">
                <div class="toolbar">
                    <span class="path">{ &path }</span>
                    <button onclick={let on_edit_toggle = on_edit_toggle.clone(); move |_| on_edit_toggle.emit(false)}>{ "Cancel" }</button>
                </div>
                <Editor key={path.clone()} content={current_content} on_save={on_save} vim_mode={vim_mode} />
             </div>
        }
    } else {
        match &*view_mode {
            ViewMode::Loading => html! {
                <div class="wiki-viewer">
                    <div class="toolbar">
                        <span class="path">{ &path }</span>
                    </div>
                    <div class="loading">{ "Loading..." }</div>
                </div>
            },
            ViewMode::Page(page) => {
                let ext = std::path::Path::new(&page.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let render_content = match ext.as_str() {
                    "md" | "markdown" => {
                        let html_output = {
                            let mut options = Options::empty();
                            options.insert(Options::ENABLE_TABLES);
                            options.insert(Options::ENABLE_FOOTNOTES);
                            options.insert(Options::ENABLE_STRIKETHROUGH);
                            options.insert(Options::ENABLE_TASKLISTS);

                            let parser = Parser::new_ext(&page.content, options);
                            let wiki_parser = WikiLinkParser::new(parser, volume.clone());

                            let mut html_output = String::new();
                            html::push_html(&mut html_output, wiki_parser);
                            html_output
                        };
                        let div = gloo_utils::document().create_element("div").unwrap();
                        div.set_inner_html(&html_output);
                        Html::VRef(div.into())
                    }
                    "json" | "toml" | "yaml" | "yml" | "opml" => html! {
                        <pre><code class={format!("language-{}", ext)}>{ &page.content }</code></pre>
                    },
                    "mermaid" | "mmd" => html! {
                        <div class="mermaid">{ &page.content }</div>
                    },
                    "dot" => html! {
                        <div id="graphviz-container"></div>
                    },
                    "drawio" | "dio" => html! {
                        <div id="drawio-container"></div>
                    },
                    _ => html! {
                        <pre><code>{ &page.content }</code></pre>
                    },
                };

                html! {
                    <div class="wiki-viewer">
                        <div class="toolbar">
                            <span class="path">{ &path }</span>
                            <div class="toolbar-controls">
                                <button onclick={on_edit_click}>{ "Edit" }</button>
                                <button onclick={on_delete_click.clone()} class="delete-btn">{ "Delete" }</button>
                            </div>
                        </div>
                        <div class="markdown-body">
                            { render_content }
                        </div>
                    </div>
                }
            }
            ViewMode::Image(url) => html! {
                <div class="wiki-viewer">
                    <div class="toolbar">
                        <span class="path">{ &path }</span>
                        <div class="toolbar-controls">
                            <button onclick={on_delete_click.clone()} class="delete-btn">{ "Delete" }</button>
                        </div>
                    </div>
                    <div class="image-viewer">
                        <img src={url.clone()} alt={path.clone()} />
                    </div>
                </div>
            },
            ViewMode::Pdf(url) => html! {
                <div class="wiki-viewer">
                     <div class="toolbar">
                        <span class="path">{ &path }</span>
                        <div class="toolbar-controls">
                            <button onclick={on_delete_click.clone()} class="delete-btn">{ "Delete" }</button>
                        </div>
                    </div>
                    <div class="pdf-viewer">
                        <embed src={url.clone()} type="application/pdf" width="100%" height="800px" />
                    </div>
                </div>
            },
            ViewMode::Error(msg) => html! {
                <div class="error-viewer">
                    <h3>{ "Error displaying file" }</h3>
                    <p>{ msg }</p>
                </div>
            },
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct EditorProps {
    content: String,
    on_save: Callback<String>,
    vim_mode: bool,
}

#[function_component(Editor)]
fn editor(props: &EditorProps) -> Html {
    let content = props.content.clone();
    let on_save = props.on_save.clone();
    let vim_mode = props.vim_mode;

    // Store the closure in a ref to keep it alive
    let closure_ref = use_mut_ref(|| Option::<Closure<dyn FnMut(String)>>::None);

    use_effect_with(vim_mode, move |&vim_mode| {
        let on_save = on_save.clone();

        let closure = Closure::wrap(Box::new(move |text: String| {
            on_save.emit(text);
        }) as Box<dyn FnMut(String)>);

        setupEditor("code-editor", &content, &closure, vim_mode);

        // Store closure in the ref instead of forgetting it
        *closure_ref.borrow_mut() = Some(closure);

        move || {
            // Drop the closure when component unmounts
            *closure_ref.borrow_mut() = None;
        }
    });

    let on_bold_click = Callback::from(|_| {
        wrapSelection("code-editor", "**", "**");
    });

    let on_italic_click = Callback::from(|_| {
        wrapSelection("code-editor", "_", "_");
    });

    let on_link_click = Callback::from(|_| {
        wrapSelection("code-editor", "[[", "]]");
    });

    let on_h1_click = Callback::from(|_| {
        toggleHeader("code-editor", 1);
    });

    let on_h2_click = Callback::from(|_| {
        toggleHeader("code-editor", 2);
    });

    let on_h3_click = Callback::from(|_| {
        toggleHeader("code-editor", 3);
    });

    html! {
        <div class="editor-container">
            <div class="editor-toolbar-actions">
                <div class="btn-group">
                    <button class="toolbar-btn" onclick={on_bold_click} title="Bold"><strong>{"B"}</strong></button>
                    <button class="toolbar-btn" onclick={on_italic_click} title="Italic"><em>{"I"}</em></button>
                    <button class="toolbar-btn" onclick={on_link_click} title="Link">{"Link"}</button>
                </div>
                <div class="btn-group">
                    <button class="toolbar-btn" onclick={on_h1_click} title="Heading 1">{"H1"}</button>
                    <button class="toolbar-btn" onclick={on_h2_click} title="Heading 2">{"H2"}</button>
                    <button class="toolbar-btn" onclick={on_h3_click} title="Heading 3">{"H3"}</button>
                </div>
            </div>
            <textarea id="code-editor" />
            <p class="editor-help">
                { if vim_mode { "Vim Mode: :w to save, or Ctrl+S" } else { "Ctrl+S to save" } }
            </p>
        </div>
    }
}
