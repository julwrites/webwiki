# System Architecture: Vimwiki-Web

## Overview
Vimwiki-Web is a self-hosted web interface for Vimwiki, designed to replicate the functionality of Gollum but with a focus on Vimwiki compatibility and a modern Rust stack. It allows users to edit Markdown-based wikis with Vim-style keybindings, manage files, and version control changes using Git.

## Tech Stack

### Core
*   **Language:** Rust (2021 edition)
*   **Backend Framework:** Axum
*   **Frontend Framework:** Yew (WebAssembly)
*   **Git Integration:** `git2` crate (libgit2 bindings) or direct CLI calls (TBD based on performance/complexity trade-offs, likely `git2` for robustness).

### Infrastructure
*   **Containerization:** Docker
*   **Orchestration:** Docker Compose
*   **Storage:** Local filesystem mount (Bind mount of the user's vimwiki directory).
*   **Network:** Exposed via Cloudflare Tunnel (assumed external configuration, app listens on internal port).

## Components

### 1. Backend Service (Axum)
The backend serves as the bridge between the browser and the filesystem/git repository.
*   **API Layer:** RESTful endpoints for:
    *   File Operations: Read, Write, Create, Delete, Rename, Move.
    *   Git Operations: Status, Stage, Commit, Log, Diff.
    *   Search: Full-text search across the wiki.
    *   Assets: Upload and retrieval of images/attachments.
*   **Wiki Engine:**
    *   Path resolution: Mapping URL slugs to filesystem paths (handling `[[WikiLinks]]`).
    *   Markdown Parsing (Server-side for search indexing/metadata, Client-side for preview).
*   **Auth Middleware:**
    *   Simple session/token-based auth to identify the commit author.

### 2. Frontend Application (Yew)
A Single Page Application (SPA) compiled to WebAssembly.
*   **Router:** Handles navigation based on wiki slugs.
*   **Editor Component:**
    *   Integration with a code editor library (e.g., CodeMirror 6 or Monaco) configured for Markdown syntax and Vim keybindings.
*   **Preview Component:**
    *   Live rendering of Markdown to HTML.
    *   Intercepts `[[WikiLink]]` clicks to route internally.
*   **Sidebar Navigation:**
    *   File tree view.
*   **Git Interface:**
    *   Draft status indicator.
    *   Commit modal (Message input + Author verification).

## Data Flow

1.  **Read:** Browser requests `/wiki/MyPage`. Backend resolves `MyPage` to `MyPage.md` (or index), reads content, returns JSON/Markdown.
2.  **Edit:** User modifies text in Browser (Vim mode). Frontend maintains local state (Draft).
3.  **Save (Draft):** User explicitly saves or auto-save triggers. Frontend PUTs content to Backend. Backend writes to filesystem (Working Directory). *Git status is now "Modified".*
4.  **Commit:** User clicks "Commit". Frontend POSTs commit metadata (message, author). Backend uses `git2` to stage and commit the file.
5.  **Search:** User types in search bar. Backend scans files (using `ripgrep` or `tantivy` index) and returns matches.

## Security Considerations
*   **Authentication:** Application assumes it is behind a trusted proxy (Cloudflare Zero Trust) for access control.
*   **Internal Auth:** A minimal login screen or environment config to set the "Committer Name/Email" to ensure Git history is attributed correctly.
*   **FileSystem Access:** Strictly scoped to the mounted volume to prevent directory traversal attacks.
