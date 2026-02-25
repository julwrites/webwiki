use crate::components::icons::{
    IconDownload, IconEdit, IconGitCommit, IconHome, IconMenu, IconMoon, IconPlus, IconSearch,
    IconSun, IconUpload,
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
    pub on_home: Callback<()>,
    pub on_new_file: Callback<()>,
    pub on_theme_toggle: Callback<()>,
    pub is_dark: bool,
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
    let on_home = props.on_home.clone();
    let on_new_file = props.on_new_file.clone();
    let on_theme_toggle = props.on_theme_toggle.clone();

    let is_git_menu_open = use_state(|| false);

    let toggle_git_menu = {
        let is_git_menu_open = is_git_menu_open.clone();
        Callback::from(move |_| is_git_menu_open.set(!*is_git_menu_open))
    };

    let close_git_menu = {
        let is_git_menu_open = is_git_menu_open.clone();
        Callback::from(move |_: ()| is_git_menu_open.set(false))
    };

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

            // Home
            <button class="bottom-bar-btn" onclick={move |_| on_home.emit(())} title="Home">
                <IconHome />
            </button>

            // Search Trigger (Dominant)
            <div class="bottom-bar-search-trigger" onclick={move |_| on_search.emit(())}>
                <IconSearch />
                <span>{"Search files..."}</span>
            </div>

            // New File
            <button class="bottom-bar-btn" onclick={move |_| on_new_file.emit(())} title="New File">
                <IconPlus />
            </button>

            // Git Controls Group (Responsive)
            <div class="git-menu-container">
                // Desktop: Show individual buttons
                <div class="desktop-git-controls">
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

                // Mobile: Show Menu Toggle
                <button
                    class={classes!("bottom-bar-btn", "mobile-git-toggle", if *is_git_menu_open { "active" } else { "" })}
                    onclick={toggle_git_menu}
                    title="Git Actions"
                >
                    <IconGitCommit />
                    if props.commits_ahead > 0 || props.commits_behind > 0 {
                        <span class="badge">{"!"}</span>
                    }
                </button>

                // Mobile Menu Popup
                if *is_git_menu_open {
                    <>
                        <div
                            class="menu-backdrop"
                            onclick={let close = close_git_menu.clone(); move |_| close.emit(())}
                            style="position:fixed;top:0;left:0;right:0;bottom:0;z-index:1000;cursor:default;"
                        />
                        <div class="git-menu-popup">
                            <button onclick={
                                let on_pull = props.on_pull.clone();
                                let close = close_git_menu.clone();
                                move |_| { on_pull.emit(()); close.emit(()); }
                            }>
                                <IconDownload />
                                <span>{"Pull"}</span>
                                if props.commits_behind > 0 {
                                    <span class="badge" style="margin-left:auto">{ props.commits_behind }</span>
                                }
                            </button>
                            <button onclick={
                                let on_commit = props.on_commit.clone();
                                let close = close_git_menu.clone();
                                move |_| { on_commit.emit(()); close.emit(()); }
                            }>
                                <IconGitCommit />
                                <span>{"Commit"}</span>
                            </button>
                            <button onclick={
                                let on_push = props.on_push.clone();
                                let close = close_git_menu.clone();
                                move |_| { on_push.emit(()); close.emit(()); }
                            }>
                                <IconUpload />
                                <span>{"Push"}</span>
                                if props.commits_ahead > 0 {
                                    <span class="badge" style="margin-left:auto">{ props.commits_ahead }</span>
                                }
                            </button>
                        </div>
                    </>
                }
            </div>

            // Edit Action
             <button class="bottom-bar-btn" onclick={move |_| on_edit.emit(())} title="Edit Page">
                <IconEdit />
            </button>

            // Theme Toggle
             <button class="bottom-bar-btn" onclick={move |_| on_theme_toggle.emit(())} title="Toggle Theme">
                if props.is_dark {
                    <IconSun />
                } else {
                    <IconMoon />
                }
            </button>
        </div>
    }
}
