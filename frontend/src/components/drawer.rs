use crate::components::icons::{IconPlus, IconUpload, IconCopy};
use crate::hooks::use_create_file;
use crate::Route;
use common::FileNode;
use gloo_net::http::Request;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DrawerProps {
    pub is_open: bool,
    pub on_close: Callback<MouseEvent>,
}

#[function_component(Drawer)]
pub fn drawer(props: &DrawerProps) -> Html {
    let on_close = props.on_close.clone();
    let route = use_route::<Route>();
    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    let create_file = use_create_file(current_volume.clone());
    let on_new_file = Callback::from(move |_| create_file.emit(()));

    let file_input_ref = use_node_ref();

    let on_upload_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let current_volume = current_volume.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.item(0) {
                    let file_name = file.name();
                    let volume = current_volume.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        let path = format!("assets/images/{}", file_name);
                        let url = format!("/api/upload/{}/{}", volume, path);

                        let request = match Request::post(&url).body(file) {
                            Ok(req) => req,
                            Err(e) => {
                                gloo_dialogs::alert(&format!("Failed to construct request: {}", e));
                                return;
                            }
                        };

                        let resp = request.send().await;

                        match resp {
                            Ok(r) if r.ok() => {
                                gloo_dialogs::alert(&format!("Successfully uploaded to {}", path));
                            }
                            Ok(r) => {
                                let err = r.text().await.unwrap_or_default();
                                gloo_dialogs::alert(&format!("Failed to upload: {}", err));
                            }
                            Err(e) => {
                                gloo_dialogs::alert(&format!("Error uploading file: {}", e));
                            }
                        }
                    });
                }
            }
            // Clear input value so same file can be uploaded again if needed
            input.set_value("");
        })
    };

    html! {
        <>
            <div
                class={classes!("drawer-overlay", if props.is_open { "open" } else { "" })}
                onclick={on_close.clone()}
            ></div>
            <div class={classes!("drawer", if props.is_open { "open" } else { "" })}>
                <div class="drawer-header">
                    <VolumeSwitcher />
                    <div style="flex: 1"></div>
                    <button class="drawer-close-btn" onclick={on_upload_click} title="Upload Image" aria-label="Upload Image" style="margin-right: 8px">
                        <IconUpload />
                    </button>
                    <input
                        type="file"
                        ref={file_input_ref}
                        style="display: none;"
                        onchange={on_file_change}
                        accept="image/*"
                    />
                    <button class="drawer-close-btn" onclick={on_new_file} title="New File" aria-label="New File" style="margin-right: 8px">
                        <IconPlus />
                    </button>
                    <button class="drawer-close-btn" onclick={on_close} title="Close" aria-label="Close Drawer">{"✕"}</button>
                </div>
                <div class="drawer-content">
                    <FileTree />
                </div>
            </div>
        </>
    }
}

#[function_component(VolumeSwitcher)]
fn volume_switcher() -> Html {
    let volumes = use_state(Vec::<FileNode>::new);
    let navigator = use_navigator();
    let route = use_route::<Route>();

    let current_volume = match route {
        Some(Route::Wiki { volume, .. }) => volume,
        _ => "default".to_string(),
    };

    {
        let volumes = volumes.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/tree").send().await {
                    let fetched_volumes: Vec<FileNode> = response.json().await.unwrap_or_default();
                    volumes.set(fetched_volumes);
                }
            });
            || ()
        });
    }

    let on_change = {
        let navigator = navigator.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            if let Some(nav) = &navigator {
                nav.push(&Route::Wiki {
                    volume: value,
                    path: "index.md".to_string(),
                });
            }
        })
    };

    if volumes.len() > 1 {
        html! {
            <div class="volume-switcher" style="display: flex; align-items: center; gap: 8px;">
                <label for="volume-select" style="color: var(--text-color); font-size: 0.9em;">{"Volume:"}</label>
                <select id="volume-select" onchange={on_change} value={current_volume}>
                    { for volumes.iter().map(|v| html! { <option value={v.name.clone()}>{ &v.name }</option> }) }
                </select>
            </div>
        }
    } else {
        html! {}
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
                if let Ok(response) = Request::get(&url).send().await {
                    let fetched_tree: Vec<FileNode> = response.json().await.unwrap_or_default();
                    tree.set(fetched_tree);
                }
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
        let on_copy_link = {
            let volume = volume.clone();
            let path = node.path.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let link = if volume == "default" {
                    format!("[[{}]]", path)
                } else {
                    format!("[[{}:{}]]", volume, path)
                };
                if let Some(window) = web_sys::window() {
                    let _ = window.navigator().clipboard().write_text(&link);
                    // Provide a small visual feedback if needed, but clipboard is enough
                }
            })
        };

        // Link to /wiki/path/to/file
        html! {
            <li class="file-tree-item">
                <Link<Route> to={Route::Wiki { volume: volume.clone(), path: node.path.clone() }}>{ &node.name }</Link<Route>>
                <button class="tree-copy-btn" onclick={on_copy_link} title="Copy Link" aria-label="Copy Link">
                    <IconCopy />
                </button>
            </li>
        }
    }

}
