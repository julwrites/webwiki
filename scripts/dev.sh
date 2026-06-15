#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"

# Ensure we're running from the project root
cd "$(dirname "$0")/.."

# Function to check if a command exists
command_exists() {
    command -v "$1" > /dev/null 2>&1
}

echo "Checking dependencies..."

# Check and install wasm-pack
if ! command_exists wasm-pack; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
else
    echo "wasm-pack is already installed."
fi

# Check and install cargo-watch
if ! command_exists cargo-watch; then
    echo "cargo-watch not found. Installing..."
    cargo install cargo-watch
else
    echo "cargo-watch is already installed."
fi

echo "Starting local development servers..."
echo "Press Ctrl+C to stop."

# frontend/static/ is the single source of truth for all static assets.
# backend/static -> ../frontend/static (symlink), so the backend always
# serves the same files Docker does.

# Build the frontend once before starting watches.
echo "Building initial frontend static files -> frontend/static/ ..."
(cd frontend && wasm-pack build --target web --out-name wasm --out-dir ./static)

# Watch frontend: rebuild into frontend/static/ on any source change.
cargo watch \
    -w frontend/src \
    -w common/src \
    -s 'cd frontend && wasm-pack build --target web --out-name wasm --out-dir ./static' &
FRONTEND_PID=$!

# Watch backend: recompile and restart on source changes.
# Run from the project root so "static" resolves to backend/static -> ../frontend/static.
# wiki_data/ lives in the repo root — same volume Docker mounts.
export WIKI_PATH="$(pwd)/wiki_data"
export DEV_BYPASS_AUTH=true
mkdir -p "$WIKI_PATH"

cd backend
cargo watch -w src -w ../common/src -x 'run' &
BACKEND_PID=$!
cd ..

# Trap Ctrl+C (SIGINT) to kill both background jobs cleanly
trap "echo 'Stopping development servers...'; kill $FRONTEND_PID $BACKEND_PID 2>/dev/null" EXIT

echo ""
echo "Dev server running at http://localhost:3000"
echo "Static assets served from: frontend/static/"
echo "Wiki data: $WIKI_PATH"
echo ""

# Wait for both processes
wait
