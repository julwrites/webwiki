#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"

# Ensure we're running from the project root
cd "$(dirname "$0")/.."

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
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

# Build the frontend once before starting the backend watch, to ensure static files exist initially
echo "Building initial frontend static files..."
(cd frontend && wasm-pack build --target web --out-name wasm --out-dir ../static)

# Watch frontend
# Rebuild the frontend on any changes to `frontend/src` or `common/src`
cargo watch -w frontend/src -w common/src -s 'cd frontend && wasm-pack build --target web --out-name wasm --out-dir ../static' &
FRONTEND_PID=$!

# Watch backend
# Recompile and restart the backend on any changes to `backend/src` or `common/src`
# WIKI_PATH is set to local wiki_data
export WIKI_PATH="$HOME/julwrites/wiki"
export DEV_BYPASS_AUTH=true
# Create wiki_data if it doesn't exist to prevent crash
mkdir -p "$WIKI_PATH"

cargo watch -w backend/src -w common/src -x 'run -p backend' &
BACKEND_PID=$!

# Trap Ctrl+C (SIGINT) to kill both background jobs cleanly
trap "echo 'Stopping development servers...'; kill $FRONTEND_PID $BACKEND_PID" EXIT

# Wait for both processes
wait
