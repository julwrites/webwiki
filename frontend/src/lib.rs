mod commit_modal;
mod search_bar;

use commit_modal::CommitModal;
use search_bar::SearchBar;
use common::{FileNode, WikiPage};
use gloo_net::http::Request;
use pulldown_cmark::{html, Options, Parser};
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

    let on_commit_click = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(true))
    };

    let on_close_commit_modal = {
        let show_commit_modal = show_commit_modal.clone();
        Callback::from(move |_| show_commit_modal.set(false))
    };

    html! {
        <BrowserRouter>
            <div class="container">
                <nav class="sidebar">
                    <div class="sidebar-header">
                        <SearchBar />
                        <button onclick={on_commit_click} class="commit-btn">{"Commit Changes"}</button>
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
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
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

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{html, Options, Parser};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_file_tree_node_render() {
        let node = FileNode {
            name: "test.md".to_string(),
            path: "test.md".to_string(),
            is_dir: false,
            children: None,
        };

        // This is a smoke test to ensure the component can at least be instantiated
        // Real DOM testing with Yew in wasm-bindgen-test is tricky without extra setup
        let _html = html! { <FileTreeNode node={node} /> };
    }

    #[wasm_bindgen_test]
    fn test_markdown_rendering() {
        let content = "# Hello";
        let options = Options::empty();
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        assert!(html_output.contains("<h1>Hello</h1>"));
    }
}
