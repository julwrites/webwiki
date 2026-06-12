# Vimwiki Web

Vimwiki Web is a modern, self-hosted web interface for [Vimwiki](https://github.com/vimwiki/vimwiki), built with Rust. It provides a seamless editing experience with Vim keybindings, Markdown support, and Git integration, all within a browser. It is fully mobile responsive, converting into an optimized experience for smaller screens.

## Features (Verified & Working)

-   **Web-Based Editing**: Edit your wiki from anywhere using a browser. (Fully verified & stable)
    -   **Vim Mode**: Integrated Vim keybindings (powered by CodeMirror) for a familiar editing experience on Desktop.
    -   **Standard Mode**: A clean, responsive fallback editor that fills the screen dynamically, optimized specifically for Mobile devices.
    -   **Cross-Device Saving**: Flawless save functionality that safely unmounts and instantly displays the parsed markdown preview.
-   **Markdown Support**: Full support for CommonMark/GFM formatting, WikiLinks (`[[Link]]`), and a side-by-side preview mode.
-   **File Explorer (Drawer)**: A clean side-drawer that displays the full hierarchy of your local wiki files.
-   **Command Palette (Search)**: Accessible via `Ctrl+K` or the UI, enabling fast, global, fuzzy file search.
-   **Git Integration**:
    -   **Draft Workflow**: Save changes to disk instantly.
    -   **Git Menu**: A unified interface to Fetch, Pull, Commit, and Push changes to external remote repositories directly from the UI.
    -   **Commit Control**: Manually review uncommitted files and commit them.
-   **Multi-Volume Support**: Mount multiple independent wiki directories (volumes) with granular access control.
-   **Dark/Light Themes**: A built-in toggle allowing you to switch between a default Dark Mode or a Light Mode.

## Tech Stack

-   **Backend**: [Axum](https://github.com/tokio-rs/axum) (Rust)
-   **Frontend**: [Yew](https://yew.rs/) (Rust/Wasm)
-   **Editor**: CodeMirror 5 (with Vim mode) & Standard Textarea
-   **Build System**: Docker (Multi-stage build)

## Prerequisites

-   [Docker](https://docs.docker.com/get-docker/) & [Docker Compose](https://docs.docker.com/compose/install/) (for production)
-   [Rust](https://www.rust-lang.org/tools/install) (for local development)

## Local Development (Docker-Free)

For rapid iteration, you can run the application directly on your host machine without Docker.

1.  **Start the Local Development Environment:**
    ```bash
    ./scripts/dev.sh
    ```
    This script will:
    - Automatically check for and install `wasm-pack` and `cargo-watch` if they are not present.
    - Run both the frontend WebAssembly build and the backend server concurrently in watch mode.
    - Automatically recompile when files in `backend/src`, `frontend/src`, or `common/src` change.
    - Store the wiki data locally in a `wiki_data` directory at the project root.

2.  **Authentication Bypass (Local Dev)**: 
    The `dev.sh` script automatically exports `DEV_BYPASS_AUTH=true`. This entirely bypasses the application's login screen and backend write access validations, allowing you to test the editor and layout instantly without mocking an encrypted `users.json` file.

3.  **Access the Wiki:**
    Open your browser and navigate to `http://localhost:3000`.

## Production & Configuration

### Environment Variables

*   `VOLUMES`: (Optional) A JSON string defining the volumes. Example: `{"personal": "/data/personal", "work": "/data/work"}`. If not set, `WIKI_PATH` is used to create a "default" volume.
*   `WIKI_PATH`: (Optional) Path to the default wiki directory (used if `VOLUMES` is not set). Defaults to `wiki_data`.
*   `AUTH_SECRET`: Secret key for encrypting `users.json`.
*   `USERS_FILE`: Path to the users file (default: `users.json`).
*   `GIT_TOKEN` / `GIT_USERNAME`: Credentials for Git remote operations.

### Multi-Volume Support

The application supports mounting multiple independent directories as "Volumes".
Configure them using the `VOLUMES` environment variable. Each volume is independent and supports its own Git repository.

### Authentication and Permissions

In a production environment, users and permissions are strictly enforced and stored in an encrypted `users.json` file.
Use the `wiki-auth` tool to manage users.

1.  **Add/Update User**:
    ```bash
    cargo run --bin wiki-auth -- add-user users.json
    ```
    It will prompt for username, password, and permissions.
    Permissions format: `volume:mode`, where mode is `r` (read) or `rw` (read-write).
    Example: `personal:rw,work:r`

### Using Docker (Recommended for Prod)

1.  **Start the application:**
    ```bash
    docker-compose up --build
    ```
    This will build the backend and frontend, then start the server on port 3000. It mounts a local `wiki_data` directory to persist your wiki files.

## Project Structure

-   `backend/`: Axum server code. Handles file operations, Git interactions, and serving static files.
-   `frontend/`: Yew application code. Handles the UI, editor logic, and client-side routing.
-   `common/`: Shared Rust types and logic between backend and frontend.
-   `docs/`: Project documentation (Architecture, Features).
-   `scripts/`: Helper scripts for maintenance.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
