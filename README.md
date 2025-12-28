# Vimwiki Web

Vimwiki Web is a modern, self-hosted web interface for [Vimwiki](https://github.com/vimwiki/vimwiki), built with Rust. It provides a seamless editing experience with Vim keybindings, Markdown support, and Git integration, all within a browser.

## Features

-   **Web-Based Editing**: Edit your wiki from anywhere using a browser.
-   **Vim Mode**: Integrated Vim keybindings (powered by CodeMirror) for a familiar editing experience.
-   **Markdown Support**: Full support for CommonMark/GFM and WikiLinks (`[[Link]]`).
-   **Git Integration**:
    -   **Draft Workflow**: Save changes to disk instantly.
    -   **Commit Control**: Manually commit changes with custom messages and authorship.
-   **Fast & Efficient**: Built with Rust (Axum for backend, Yew for frontend) and WebAssembly.
-   **Dockerized**: Easy deployment using Docker and Docker Compose.

## Tech Stack

-   **Backend**: [Axum](https://github.com/tokio-rs/axum) (Rust)
-   **Frontend**: [Yew](https://yew.rs/) (Rust/Wasm)
-   **Build System**: Docker (Multi-stage build)
-   **Editor**: CodeMirror 5 (with Vim mode)

## Prerequisites

-   [Docker](https://docs.docker.com/get-docker/)
-   [Docker Compose](https://docs.docker.com/compose/install/)
-   [Rust](https://www.rust-lang.org/tools/install) (only for local development)

## Getting Started

### Using Docker (Recommended)

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/vimwiki-web.git
    cd vimwiki-web
    ```

2.  **Start the application:**
    ```bash
    docker-compose up --build
    ```
    This will build the backend and frontend, then start the server on port 3000. It mounts a local `wiki_data` directory to persist your wiki files.

3.  **Access the Wiki:**
    Open your browser and navigate to `http://localhost:3000`.

### Local Development

1.  **Install `wasm-pack`:**
    ```bash
    cargo install wasm-pack
    ```

2.  **Build the Frontend:**
    ```bash
    cd frontend
    wasm-pack build --target web --out-name wasm --out-dir ./static
    ```

3.  **Run the Backend:**
    ```bash
    cd ../backend
    # Ensure you have a 'wiki_data' directory in the project root or set WIKI_PATH
    export WIKI_PATH=../wiki_data
    cargo run
    ```

## Project Structure

-   `backend/`: Axum server code. Handles file operations, Git interactions, and serving static files.
-   `frontend/`: Yew application code. Handles the UI, editor logic, and client-side routing.
-   `common/`: Shared Rust types and logic between backend and frontend.
-   `docs/`: Project documentation (Architecture, Features, Tasks).
-   `scripts/`: Helper scripts for tasks and maintenance.
-   `docker-compose.yml`: Deployment configuration.
-   `Dockerfile`: Multi-stage build definition.

## Documentation

For more detailed information, please refer to the `docs/` directory:
-   [System Overview](docs/architecture/001_system_overview.md)
-   [Initial Scope & Features](docs/features/001_initial_scope.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
