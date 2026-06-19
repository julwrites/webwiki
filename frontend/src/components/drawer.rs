use crate::components::icons::{IconCopy, IconEdit, IconPlus, IconSearch, IconTrash, IconUpload};
use crate::hooks::{use_create_file, use_delete_file, use_rename_file};
use crate::Route;
use common::FileNode;
use gloo_net::http::Request;
use web_sys::{Event, HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DrawerProps {
    pub is_open: bool,
    pub on_close: Callback<MouseEvent>,
    pub on_search: Callback<()>,
}

#[function_component(Drawer)]
pub fn drawer(props: &DrawerProps) -> Html {
    let on_close = props.on_close.clone();
    let on_search = props.on_search.clone();
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
                    <div class="flex-1"></div>
                    <button class="btn-icon mr-2" onclick={move |_| on_search.emit(())} title="Search" aria-label="Search">
                        <IconSearch />
                    </button>
                    <button class="btn-icon mr-2" onclick={on_upload_click} title="Upload Image" aria-label="Upload Image">
                        <IconUpload />
                    </button>
                    <input
                        type="file"
                        ref={file_input_ref}
                        class="d-none"
                        onchange={on_file_change}
                        accept="image/*"
                    />
                    <button class="btn-icon mr-2" onclick={on_new_file} title="New File" aria-label="New File">
                        <IconPlus />
                    </button>
                    <button class="btn-icon" onclick={on_close} title="Close" aria-label="Close Drawer">{"✕"}</button>
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
            <div class="volume-switcher flex items-center gap-2">
                <label for="volume-select" class="text-sm text-muted">{"Volume:"}</label>
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
            if tree.is_empty() {
                <div class="p-4 text-muted text-sm">
                    {"No files found. Use the + button above to create one."}
                </div>
            } else {
                <ul>
                    { for tree.iter().map(|node| html! { <FileTreeNode node={node.clone()} volume={current_volume.clone()} /> }) }
                </ul>
            }
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

    // Hooks must be at the top level of function components
    let on_rename_hook = use_rename_file(volume.clone(), node.path.clone());
    let on_delete_hook = use_delete_file(volume.clone(), node.path.clone());

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
        let on_keydown = {
            let is_expanded = is_expanded.clone();
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Enter" || e.key() == " " {
                    e.prevent_default();
                    e.stop_propagation();
                    is_expanded.set(!*is_expanded);
                }
            })
        };
        let dir_name = node.name.clone();

        html! {
            <li>
                <div
                    onclick={toggle_expanded}
                    onkeydown={on_keydown}
                    role="button"
                    tabindex="0"
                    aria-expanded={if *is_expanded { "true" } else { "false" }}
                    aria-label={format!("Toggle directory {}", dir_name)}
                    class="flex items-center rounded-md cursor-pointer"
                >
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
        let on_rename_click = {
            let on_rename_hook = on_rename_hook.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                on_rename_hook.emit(());
            })
        };

        let on_delete_click = {
            let on_delete_hook = on_delete_hook.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                on_delete_hook.emit(());
            })
        };

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

        let file_name = node.name.clone();

        // Link to /wiki/path/to/file
        html! {
            <li class="file-tree-item">
                <Link<Route> to={Route::Wiki { volume: volume.clone(), path: node.path.clone() }}>{ &node.name }</Link<Route>>
                <div class="file-tree-actions flex gap-1">
                    <button class="btn-icon" onclick={on_rename_click} title={format!("Rename {}", file_name)} aria-label={format!("Rename {}", file_name)}>
                        <IconEdit />
                    </button>
                    <button class="btn-icon" onclick={on_delete_click} title={format!("Delete {}", file_name)} aria-label={format!("Delete {}", file_name)}>
                        <IconTrash />
                    </button>
                    <button class="btn-icon" onclick={on_copy_link} title={format!("Copy Link to {}", file_name)} aria-label={format!("Copy Link to {}", file_name)}>
                        <IconCopy />
                    </button>
                </div>
            </li>
        }
    }
}
