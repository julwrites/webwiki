use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;
use crate::Route;
use crate::hooks::use_create_file;
use crate::components::icons::IconPlus;
use common::FileNode;

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

    let create_file = use_create_file(current_volume);
    let on_new_file = Callback::from(move |_| create_file.emit(()));

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
                    <button class="drawer-close-btn" onclick={on_new_file} title="New File" style="margin-right: 8px">
                        <IconPlus />
                    </button>
                    <button class="drawer-close-btn" onclick={on_close}>{"✕"}</button>
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

    if volumes.len() > 1 {
        html! {
            <div class="volume-switcher">
                <select onchange={on_change} value={current_volume}>
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
