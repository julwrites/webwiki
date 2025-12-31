mod commit_modal;
mod search_bar;

use commit_modal::CommitModal;
use common::{FileNode, WikiPage};
use gloo_net::http::Request;
use gloo_storage::Storage;
use pulldown_cmark::{html, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};
use search_bar::SearchBar;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setupEditor(element_id: &str, initial_content: &str, callback: &Closure<dyn FnMut(String)>);
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/wiki/*path")]
    Wiki { path: String },
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
pub fn app() -> Html {
    let show_commit_modal = use_state(|| false);

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

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new_theme = if *theme == "dark" { "light" } else { "dark" };
            let _ = gloo_storage::LocalStorage::set("theme", new_theme);
            theme.set(new_theme.to_string());
        })
    };

    let on_commit_click = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(true))
    };

    let on_close_commit_modal = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(false))
    };

    let on_sync_click = Callback::from(|_| {
        wasm_bindgen_futures::spawn_local(async move {
            let resp = Request::post("/api/git/push").send().await;
            match resp {
                Ok(r) if r.ok() => gloo_dialogs::alert("Successfully pushed to remote!"),
                Ok(r) => {
                    let text = r.text().await.unwrap_or_default();
                    gloo_dialogs::alert(&format!("Failed to push: {}", text));
                }
                Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
            }
        });
    });

    html! {
        <BrowserRouter>
            <div class="container">
                <nav class="sidebar">
                    <div class="sidebar-header">
                        <div class="sidebar-controls">
                            <SearchBar />
                            <div class="action-buttons">
                                <button onclick={on_commit_click} class="commit-btn">{"Commit"}</button>
                                <button onclick={on_sync_click} class="sync-btn">{"Sync"}</button>
                            </div>
                            <button onclick={toggle_theme} class="theme-btn">
                                { if *theme == "dark" { "Light Mode" } else { "Dark Mode" } }
                            </button>
                        </div>
                    </div>
                    <FileTree />
                </nav>
                <main class="content">
                    <Switch<Route> render={switch} />
                </main>
                if *show_commit_modal {
                    <CommitModal on_close={on_close_commit_modal} />
                }
            </div>
        </BrowserRouter>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Wiki { path } => html! { <WikiViewer path={path} /> },
        Route::Home => html! { <WikiViewer path={"index.md".to_string()} /> },
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}

#[function_component(FileTree)]
fn file_tree() -> Html {
    let tree = use_state(Vec::<FileNode>::new);
    {
        let tree = tree.clone();
        use_effect_with((), move |_| {
            let tree = tree.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_tree: Vec<FileNode> = Request::get("/api/tree")
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
                { for tree.iter().map(|node| html! { <FileTreeNode node={node.clone()} /> }) }
            </ul>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct FileTreeNodeProps {
    node: FileNode,
}

#[function_component(FileTreeNode)]
fn file_tree_node(props: &FileTreeNodeProps) -> Html {
    let node = &props.node;
    if node.is_dir {
        html! {
            <li>
                <span class="folder">{ &node.name }</span>
                if let Some(children) = &node.children {
                    <ul>
                        { for children.iter().map(|child| html! { <FileTreeNode node={child.clone()} /> }) }
                    </ul>
                }
            </li>
        }
    } else {
        // Link to /wiki/path/to/file
        let _link_path = format!("/wiki/{}", node.path);
        html! {
            <li>
                <Link<Route> to={Route::Wiki { path: node.path.clone() }}>{ &node.name }</Link<Route>>
            </li>
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct WikiViewerProps {
    path: String,
}

/// A wrapper around pulldown_cmark::Parser to handle WikiLinks.
struct WikiLinkParser<'a> {
    parser: Parser<'a>,
    events: std::collections::VecDeque<Event<'a>>,
}

impl<'a> WikiLinkParser<'a> {
    fn new(parser: Parser<'a>) -> Self {
        Self {
            parser,
            events: std::collections::VecDeque::new(),
        }
    }
}

impl<'a> Iterator for WikiLinkParser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.events.pop_front() {
            return Some(event);
        }

        let event = self.parser.next()?;

        // If it's text, try to merge with subsequent text events
        if let Event::Text(text) = event {
            let mut buffer = String::from(text.as_ref());
            let mut next_non_text: Option<Event<'a>> = None;

            for next_event in self.parser.by_ref() {
                match next_event {
                    Event::Text(t) => {
                        buffer.push_str(t.as_ref());
                    }
                    other => {
                        next_non_text = Some(other);
                        break;
                    }
                }
            }

            // Now `buffer` contains all merged text.
            // Process `buffer` for wikilinks.

            let mut start_idx = 0;
            let text_str = buffer.as_str();
            let mut found_wikilink = false;

            while let Some(open_idx) = text_str[start_idx..].find("[[") {
                let absolute_open_idx = start_idx + open_idx;
                if let Some(close_idx) = text_str[absolute_open_idx..].find("]]") {
                    let absolute_close_idx = absolute_open_idx + close_idx;

                    found_wikilink = true;

                    if absolute_open_idx > start_idx {
                        self.events.push_back(Event::Text(CowStr::from(
                            text_str[start_idx..absolute_open_idx].to_string(),
                        )));
                    }

                    let content = &text_str[absolute_open_idx + 2..absolute_close_idx];
                    let (link, label) = if let Some(pipe_idx) = content.find('|') {
                        (&content[..pipe_idx], &content[pipe_idx + 1..])
                    } else {
                        (content, content)
                    };

                    let link_url = format!("/wiki/{}", link.trim());
                    let label_text = label.trim().to_string();

                    self.events.push_back(Event::Start(Tag::Link {
                        link_type: LinkType::Inline,
                        dest_url: CowStr::from(link_url),
                        title: CowStr::from(""),
                        id: "".into(),
                    }));
                    self.events.push_back(Event::Text(CowStr::from(label_text)));
                    self.events.push_back(Event::End(TagEnd::Link));

                    start_idx = absolute_close_idx + 2;
                } else {
                    break;
                }
            }

            if found_wikilink {
                if start_idx < text_str.len() {
                    self.events
                        .push_back(Event::Text(CowStr::from(text_str[start_idx..].to_string())));
                }
            } else {
                // No wikilinks found, emit the whole merged text
                self.events.push_back(Event::Text(CowStr::from(buffer)));
            }

            // Finally, append the non-text event if we found one
            if let Some(e) = next_non_text {
                self.events.push_back(e);
            }

            return self.events.pop_front();
        }

        Some(event)
    }
}

#[function_component(WikiViewer)]
fn wiki_viewer(props: &WikiViewerProps) -> Html {
    let content = use_state(String::new);
    let is_editing = use_state(|| false);
    let path = props.path.clone();

    {
        let content = content.clone();
        let path = path.clone();
        use_effect_with(path.clone(), move |_| {
            let content = content.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/wiki/{}", path);
                let resp = Request::get(&url).send().await;

                match resp {
                    Ok(r) if r.ok() => {
                        let page: WikiPage = r.json().await.unwrap_or_else(|_| WikiPage {
                            path: path.clone(),
                            content: "Error parsing JSON".to_string(),
                        });
                        content.set(page.content);
                    }
                    _ => content.set("# Page Not Found\n\nClick edit to create it.".to_string()),
                }
            });
            || ()
        });
    }

    let on_edit_click = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| is_editing.set(true))
    };

    let on_save = {
        let path = path.clone();
        let content_state = content.clone();
        let is_editing = is_editing.clone();
        Callback::from(move |new_content: String| {
            let path = path.clone();
            let content_state = content_state.clone();
            let is_editing = is_editing.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let page = WikiPage {
                    path: path.clone(),
                    content: new_content.clone(),
                };
                let req = Request::put(&format!("/api/wiki/{}", path))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&page).unwrap());

                if let Ok(req) = req {
                    let resp = req.send().await;
                    if let Ok(r) = resp {
                        if r.ok() {
                            content_state.set(new_content);
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
        html! {
             <div class="wiki-editor">
                <div class="toolbar">
                    <span class="path">{ &path }</span>
                    <button onclick={let is_editing = is_editing.clone(); move |_| is_editing.set(false)}>{ "Cancel" }</button>
                </div>
                <Editor content={(*content).clone()} on_save={on_save} />
             </div>
        }
    } else {
        let html_output = {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);

            let parser = Parser::new_ext(&content, options);
            let wiki_parser = WikiLinkParser::new(parser);

            let mut html_output = String::new();
            html::push_html(&mut html_output, wiki_parser);
            html_output
        };

        // Use a wrapper to inject HTML safely
        let div = gloo_utils::document().create_element("div").unwrap();
        div.set_inner_html(&html_output);
        let node = Html::VRef(div.into());

        html! {
            <div class="wiki-viewer">
                <div class="toolbar">
                    <span class="path">{ &path }</span>
                    <button onclick={on_edit_click}>{ "Edit" }</button>
                </div>
                <div class="markdown-body">
                    { node }
                </div>
            </div>
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct EditorProps {
    content: String,
    on_save: Callback<String>,
}

#[function_component(Editor)]
fn editor(props: &EditorProps) -> Html {
    let content = props.content.clone();
    let on_save = props.on_save.clone();

    use_effect_with((), move |_| {
        let on_save = on_save.clone();
        let closure = Closure::wrap(Box::new(move |text: String| {
            on_save.emit(text);
        }) as Box<dyn FnMut(String)>);

        setupEditor("code-editor", &content, &closure);

        // Keep closure alive?
        // In a real app we need to manage the closure lifetime or leak it carefully.
        // For simplicity, we forget it here, but this leaks memory every time editor is opened.
        // Better: store in a ref or return a cleanup function.
        closure.forget();

        || ()
    });

    html! {
        <div>
            <textarea id="code-editor" />
            <p class="editor-help">{ "Vim Mode: :w to save, or Ctrl+S" }</p>
        </div>
    }
}
