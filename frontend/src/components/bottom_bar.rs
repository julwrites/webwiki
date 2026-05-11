use crate::components::icons::{
    IconCopy, IconDownload, IconEdit, IconGitCommit, IconHistory, IconHome, IconMenu, IconMoon,
    IconPlus, IconSearch, IconSettings, IconSun, IconUpload,
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
    pub on_settings: Callback<()>,
    pub on_history: Callback<()>,
    pub on_copy_link: Callback<()>,
    pub is_dark: bool,
    pub commits_ahead: usize,
    pub commits_behind: usize,
    pub uncommitted_files: usize,
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
                title="Toggle File Tree" aria-label="Toggle File Tree"
                aria-expanded={props.is_drawer_open.to_string()}
            >
                <IconMenu />
            </button>

            // Home
            <button class="bottom-bar-btn" onclick={move |_| on_home.emit(())} title="Home" aria-label="Home">
                <IconHome />
            </button>

            // Search Trigger (Dominant)
            <button class="bottom-bar-search-trigger" onclick={move |_| on_search.emit(())} aria-label="Search files" title="Search files (Ctrl+K)">
                <IconSearch />
                <span>{"Search files... (Ctrl+K)"}</span>
            </button>

            // New File
            <button class="bottom-bar-btn" onclick={move |_| on_new_file.emit(())} title="New File" aria-label="New File">
                <IconPlus />
            </button>

            // Git Controls Group (Responsive)
            <div class="git-menu-container">
                // Desktop: Show individual buttons
                <div class="desktop-git-controls">
                    <button class="bottom-bar-btn" onclick={move |_| on_pull.emit(())} title="Pull" aria-label="Pull">
                        <IconDownload />
                        if props.commits_behind > 0 {
                            <span class="badge">{ props.commits_behind }</span>
                        }
                    </button>
                    <button class="bottom-bar-btn" onclick={move |_| on_commit.emit(())} title="Commit" aria-label="Commit">
                        <IconGitCommit />
                        if props.uncommitted_files > 0 {
                            <span class="badge" style="background-color: var(--accent-color);">{ props.uncommitted_files }</span>
                        }
                    </button>
                    <button class="bottom-bar-btn" onclick={move |_| on_push.emit(())} title="Push" aria-label="Push">
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
                    title="Git Actions" aria-label="Git Actions"
                    aria-expanded={(*is_git_menu_open).to_string()}
                >
                    <IconGitCommit />
                    if props.commits_ahead > 0 || props.commits_behind > 0 || props.uncommitted_files > 0 {
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
                            } title="Pull" aria-label="Pull">
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
                            } title="Commit" aria-label="Commit">
                                <IconGitCommit />
                                <span>{"Commit"}</span>
                                if props.uncommitted_files > 0 {
                                    <span class="badge" style="margin-left:auto">{ props.uncommitted_files }</span>
                                }
                            </button>
                            <button onclick={
                                let on_push = props.on_push.clone();
                                let close = close_git_menu.clone();
                                move |_| { on_push.emit(()); close.emit(()); }
                            } title="Push" aria-label="Push">
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
             <button class="bottom-bar-btn" onclick={move |_| on_edit.emit(())} title="Edit Page" aria-label="Edit Page">
                <IconEdit />
            </button>

            // History Action
            <button class="bottom-bar-btn" onclick={let on_history = props.on_history.clone(); move |_| on_history.emit(())} title="Page History" aria-label="Page History">
                <IconHistory />
            </button>

            // Copy Link
            <button class="bottom-bar-btn" onclick={let on_copy_link = props.on_copy_link.clone(); move |_| on_copy_link.emit(())} title="Copy Link" aria-label="Copy Link">
                <IconCopy />
            </button>

            // Settings
            <button class="bottom-bar-btn" onclick={let on_settings = props.on_settings.clone(); move |_| on_settings.emit(())} title="Settings" aria-label="Settings">
                <IconSettings />
            </button>

            // Theme Toggle
             <button class="bottom-bar-btn" onclick={move |_| on_theme_toggle.emit(())} title="Toggle Theme" aria-label="Toggle Theme">
                if props.is_dark {
                    <IconSun />
                } else {
                    <IconMoon />
                }
            </button>
        </div>
    }
}
