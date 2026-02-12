use crate::components::icons::{
    IconDownload, IconEdit, IconGitCommit, IconMenu, IconSearch, IconUpload,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BottomBarProps {
    pub on_toggle_drawer: Callback<()>,
    pub on_search: Callback<()>,
    pub on_pull: Callback<()>,
    pub on_push: Callback<()>,
    pub on_commit: Callback<()>,
    pub on_edit: Callback<()>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
    pub is_drawer_open: bool,
}

#[function_component(BottomBar)]
pub fn bottom_bar(props: &BottomBarProps) -> Html {
    let on_toggle = props.on_toggle_drawer.clone();
    let on_search = props.on_search.clone();
    let on_pull = props.on_pull.clone();
    let on_push = props.on_push.clone();
    let on_commit = props.on_commit.clone();
    let on_edit = props.on_edit.clone();

    html! {
        <div class="bottom-bar">
            // Drawer Toggle
            <button
                class={classes!("bottom-bar-btn", if props.is_drawer_open { "active" } else { "" })}
                onclick={move |_| on_toggle.emit(())}
                title="Toggle File Tree"
            >
                <IconMenu />
            </button>

            // Search Trigger (Dominant)
            <div class="bottom-bar-search-trigger" onclick={move |_| on_search.emit(())}>
                <IconSearch />
                <span>{"Search files..."}</span>
            </div>

            // Git Controls Group
            <div class="bottom-bar-group">
                <button class="bottom-bar-btn" onclick={move |_| on_pull.emit(())} title="Pull">
                    <IconDownload />
                    if props.commits_behind > 0 {
                        <span class="badge">{ props.commits_behind }</span>
                    }
                </button>
                <button class="bottom-bar-btn" onclick={move |_| on_commit.emit(())} title="Commit">
                    <IconGitCommit />
                </button>
                <button class="bottom-bar-btn" onclick={move |_| on_push.emit(())} title="Push">
                    <IconUpload />
                    if props.commits_ahead > 0 {
                        <span class="badge">{ props.commits_ahead }</span>
                    }
                </button>
            </div>

            // Edit Action
             <button class="bottom-bar-btn" onclick={move |_| on_edit.emit(())} title="Edit Page">
                <IconEdit />
            </button>
        </div>
    }
}
