mod api;
mod commit_modal;
mod components;
mod parsers;
mod search_bar;

use commit_modal::CommitModal;
use common::{FileNode, WikiPage};
use components::command_palette::CommandPalette;
use components::login::Login;
use gloo_net::http::Request;
use gloo_storage::Storage;
use parsers::WikiLinkParser;
use pulldown_cmark::{html, Options, Parser};
use search_bar::SearchBar;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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
enum Route {
    #[at("/wiki/:volume/*path")]
    Wiki { volume: String, path: String },
    #[at("/login")]
    Login,
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
            <AuthWrapper>
                <Layout />
            </AuthWrapper>
        </BrowserRouter>
    }
}

#[function_component(Layout)]
fn layout() -> Html {
    let route = use_route::<Route>();
    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    let show_commit_modal = use_state(|| false);
    let is_authenticated = use_state(|| false);
    let is_sidebar_open = use_state(|| false);
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

    let navigator = use_navigator().unwrap();

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new_theme = if *theme == "dark" { "light" } else { "dark" };
            let _ = gloo_storage::LocalStorage::set("theme", new_theme);
            theme.set(new_theme.to_string());
        })
    };

    let on_new_file_click = {
        let navigator = navigator.clone();
        let current_volume = current_volume.clone();
        Callback::from(move |_| {
            if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None)
            {
                if !path.trim().is_empty() {
                    navigator.push(&Route::Wiki {
                        volume: current_volume.clone(),
                        path: path.trim().to_string(),
                    });
                }
            }
        })
    };

    let toggle_sidebar = {
        let is_sidebar_open = is_sidebar_open.clone();
        Callback::from(move |_| is_sidebar_open.set(!*is_sidebar_open))
    };

    let close_sidebar = {
        let is_sidebar_open = is_sidebar_open.clone();
        Callback::from(move |_| is_sidebar_open.set(false))
    };

    let toggle_vim_mode = {
        let vim_mode = vim_mode.clone();
        Callback::from(move |e: yew::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let checked = input.checked();
            let _ = gloo_storage::LocalStorage::set("vim_mode", checked);
            vim_mode.set(checked);
        })
    };

    // Sidebar resizing state
    let sidebar_width = use_state(|| 250);
    let is_resizing = use_state(|| false);

    let start_resizing = {
        let is_resizing = is_resizing.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            is_resizing.set(true);
        })
    };

    {
        let sidebar_width = sidebar_width.clone();
        let is_resizing = is_resizing.clone();
        use_effect_with(*is_resizing, move |resizing| {
            if *resizing {
                let window = gloo_utils::window();

                let on_move = {
                    let sidebar_width = sidebar_width.clone();
                    Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                        let new_width = e.client_x();
                        let new_width = new_width.clamp(200, 600);
                        sidebar_width.set(new_width);
                    }) as Box<dyn FnMut(_)>)
                };

                let on_up = {
                    let is_resizing = is_resizing.clone();
                    Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                        is_resizing.set(false);
                    }) as Box<dyn FnMut(_)>)
                };

                let _ = window.add_event_listener_with_callback(
                    "mousemove",
                    on_move.as_ref().unchecked_ref(),
                );
                let _ = window
                    .add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref());

                Box::new(move || {
                    let _ = window.remove_event_listener_with_callback(
                        "mousemove",
                        on_move.as_ref().unchecked_ref(),
                    );
                    let _ = window.remove_event_listener_with_callback(
                        "mouseup",
                        on_up.as_ref().unchecked_ref(),
                    );
                }) as Box<dyn FnOnce()>
            } else {
                Box::new(|| {}) as Box<dyn FnOnce()>
            }
        });
    }

    let on_commit_click = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(true))
    };

    let on_close_commit_modal = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(false))
    };

    let on_sync_click = {
        let volume = current_volume.clone();
        Callback::from(move |_| {
            let volume = volume.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/git/{}/push", volume);
                let resp = Request::post(&url).send().await;
                match resp {
                    Ok(r) if r.ok() => gloo_dialogs::alert("Successfully pushed to remote!"),
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        gloo_dialogs::alert(&format!("Failed to push: {}", text));
                    }
                    Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                }
            });
        })
    };

    let on_logout_click = {
        let is_authenticated = is_authenticated.clone();
        Callback::from(move |_| {
            let is_authenticated = is_authenticated.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = Request::post("/api/logout").send().await;
                is_authenticated.set(false);
                gloo_utils::window().location().reload().unwrap();
            });
        })
    };

    html! {
        <div class="container">
            <button class="sidebar-toggle-btn" onclick={toggle_sidebar}>
                {"☰"}
            </button>
            <div
                class={classes!("sidebar-overlay", if *is_sidebar_open { "visible" } else { "" })}
                onclick={close_sidebar.clone()}
            ></div>
            <nav
                class={classes!("sidebar", if *is_sidebar_open { "open" } else { "" })}
                style={format!("width: {}px", *sidebar_width)}
            >
                <div class="sidebar-header">
                    <VolumeSwitcher />
                    <SearchBar />
                </div>
                <div class="sidebar-content" onclick={close_sidebar}>
                    <FileTree />
                </div>
                <div class="sidebar-footer">
                    <div class="sidebar-controls">
                        <div class="action-buttons">
                            <button onclick={on_new_file_click} class="new-file-btn">{"New File"}</button>
                            <button onclick={on_commit_click} class="commit-btn">{"Commit"}</button>
                            <button onclick={on_sync_click} class="sync-btn">{"Sync"}</button>
                        </div>
                        <div class="action-buttons">
                            <button onclick={on_logout_click} class="logout-btn">{"Logout"}</button>
                        </div>
                        <button onclick={toggle_theme.clone()} class="theme-btn">
                            { if *theme == "dark" { "Light Mode" } else { "Dark Mode" } }
                        </button>
                        <div class="toggle-switch">
                            <label>
                                <input type="checkbox" checked={*vim_mode} onchange={toggle_vim_mode} />
                                {" Vim Mode"}
                            </label>
                        </div>
                    </div>
                </div>
                <div
                    class={classes!("sidebar-resizer", if *is_resizing { "resizing" } else { "" })}
                    onmousedown={start_resizing}
                ></div>
            </nav>
            <main class="content">
                <Switch<Route> render={move |routes| switch(routes, *vim_mode)} />
            </main>
            <CommandPalette on_theme_toggle={toggle_theme.clone()} current_volume={current_volume.clone()} />
            if *show_commit_modal {
                <CommitModal on_close={on_close_commit_modal} volume={current_volume} />
            }
        </div>
    }
}

// Wrapper to handle initial auth check and redirect to Login
#[function_component(AuthWrapper)]
fn auth_wrapper(props: &HtmlProperties) -> Html {
    let navigator = use_navigator().expect("AuthWrapper must be inside BrowserRouter");
    let is_loading = use_state(|| true);

    // Check if we are on the login page to avoid infinite loop
    let location = use_location().expect("AuthWrapper must be inside BrowserRouter");
    let is_login_page = location.path() == "/login";

    {
        let navigator = navigator.clone();
        let is_loading = is_loading.clone();
        use_effect_with((), move |_| {
            if is_login_page {
                is_loading.set(false);
                // Return dummy cleanup
                return;
            }

            wasm_bindgen_futures::spawn_local(async move {
                let resp = Request::get("/api/check-auth").send().await;
                match resp {
                    Ok(r) if r.ok() => {
                        is_loading.set(false);
                    }
                    _ => {
                        navigator.push(&Route::Login);
                        is_loading.set(false);
                    }
                }
            });
        });
    }

    if *is_loading {
        return html! { <div class="loading-screen">{"Loading..."}</div> };
    }

    if is_login_page {
        // Login page doesn't really care about vim mode, but we need to satisfy the signature
        // or just pass false.
        return html! { <Switch<Route> render={|routes| switch(routes, false)} /> };
    }

    html! {
        <>
            { for props.children.iter() }
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct HtmlProperties {
    pub children: Children,
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}

fn switch(routes: Route, vim_mode: bool) -> Html {
    match routes {
        Route::Login => html! { <Login /> },
        Route::Wiki { volume, path } => {
            html! { <WikiViewer volume={volume} path={path} vim_mode={vim_mode} /> }
        }
        Route::Home => {
            html! { <WikiViewer volume={"default".to_string()} path={"index.md".to_string()} vim_mode={vim_mode} /> }
        }
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}

#[function_component(VolumeSwitcher)]
fn volume_switcher() -> Html {
    let volumes = use_state(Vec::<FileNode>::new);
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>();

    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    {
        let volumes = volumes.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_volumes: Vec<FileNode> = Request::get("/api/tree")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap_or_default();
                volumes.set(fetched_volumes);
            });
            || ()
        });
    }

    let on_change = {
        let navigator = navigator.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            navigator.push(&Route::Wiki {
                volume: value,
                path: "index.md".to_string(),
            });
        })
    };

    html! {
        <div class="volume-switcher">
            <select onchange={on_change} value={current_volume}>
                { for volumes.iter().map(|v| html! { <option value={v.name.clone()}>{ &v.name }</option> }) }
            </select>
        </div>
    }
}

#[function_component(FileTree)]
fn file_tree() -> Html {
    let tree = use_state(Vec::<FileNode>::new);
    let route = use_route::<Route>();

    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    {
        let tree = tree.clone();
        let volume = current_volume.clone();
        use_effect_with(volume, move |volume| {
            let tree = tree.clone();
            let volume = volume.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/tree?volume={}", volume);
                let fetched_tree: Vec<FileNode> = Request::get(&url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap_or_default();
                tree.set(fetched_tree);
            });
            || ()
        });
    }

    html! {
        <div class="file-tree">
            <h3>{ "Files" }</h3>
            <ul>
                { for tree.iter().map(|node| html! { <FileTreeNode node={node.clone()} volume={current_volume.clone()} /> }) }
            </ul>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct FileTreeNodeProps {
    node: FileNode,
    volume: String,
}

#[function_component(FileTreeNode)]
fn file_tree_node(props: &FileTreeNodeProps) -> Html {
    let node = &props.node;
    let volume = &props.volume;
    // Default to collapsed for directories
    let is_expanded = use_state(|| false);

    let toggle_expanded = {
        let is_expanded = is_expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            is_expanded.set(!*is_expanded);
        })
    };

    if node.is_dir {
        let icon = if *is_expanded { "▼" } else { "▶" };
        html! {
            <li>
                <div onclick={toggle_expanded}>
                    <span class="tree-toggle">{ icon }</span>
                    <span class="folder-label folder">{ &node.name }</span>
                </div>
                if *is_expanded {
                    if let Some(children) = &node.children {
                        <ul>
                            { for children.iter().map(|child| html! { <FileTreeNode node={child.clone()} volume={volume.clone()} /> }) }
                        </ul>
                    }
                }
            </li>
        }
    } else {
        // Link to /wiki/path/to/file
        html! {
            <li>
                <Link<Route> to={Route::Wiki { volume: volume.clone(), path: node.path.clone() }}>{ &node.name }</Link<Route>>
            </li>
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct WikiViewerProps {
    volume: String,
    path: String,
    vim_mode: bool,
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
    let is_editing = use_state(|| false);
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
        let is_editing = is_editing.clone();
        use_effect_with(
            (view_mode.clone(), *is_editing),
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
        let is_editing = is_editing.clone();
        Callback::from(move |_| is_editing.set(true))
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
        let is_editing = is_editing.clone();
        Callback::from(move |new_content: String| {
            let path = path.clone();
            let volume = volume.clone();
            let view_mode = view_mode.clone();
            let is_editing = is_editing.clone();
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
                            is_editing.set(false);
                        } else {
                            gloo_dialogs::alert(&format!("Failed to save: {}", r.status()));
                        }
                    }
                }
            });
        })
    };

    if *is_editing {
        let current_content = match &*view_mode {
            ViewMode::Page(p) => p.content.clone(),
            _ => String::new(),
        };

        html! {
             <div class="wiki-editor">
                <div class="toolbar">
                    <span class="path">{ &path }</span>
                    <button onclick={let is_editing = is_editing.clone(); move |_| is_editing.set(false)}>{ "Cancel" }</button>
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
