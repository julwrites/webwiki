#!/bin/bash
# scripts/docker-dev.sh
#
# Builds and runs WebWiki in a local Docker container on http://localhost:3000.
# This gives a faithful reproduction of the production environment (same Dockerfile,
# same frontend/static/ tree, same WASM build) so bugs caught here will be caught
# before deploying to wiki.tehj.io.
#
# Usage:
#   ./scripts/docker-dev.sh              # full rebuild + run
#   ./scripts/docker-dev.sh --no-rebuild # skip docker build, just restart container
#
# Requirements: docker

set -e
cd "$(dirname "$0")/.."

IMAGE="webwiki:local"
CONTAINER="webwiki-dev"
PORT=3000
WIKI_DATA="$(pwd)/wiki_data"

NO_REBUILD=false
for arg in "$@"; do
    if [ "$arg" = "--no-rebuild" ]; then
        NO_REBUILD=true
    fi
done

# Stop any existing dev container
if docker ps -q --filter "name=$CONTAINER" | grep -q .; then
    echo "Stopping existing container '$CONTAINER'..."
    docker stop "$CONTAINER" > /dev/null
fi
if docker ps -aq --filter "name=$CONTAINER" | grep -q .; then
    docker rm "$CONTAINER" > /dev/null
fi

# Build Docker image
if [ "$NO_REBUILD" = false ]; then
    echo ""
    echo "Building Docker image '$IMAGE'..."
    echo "(This compiles Rust + WASM from scratch — takes ~2-5 min on first run)"
    echo ""
    docker build -t "$IMAGE" .
    echo ""
    echo "Build complete."
else
    echo "Skipping Docker build (--no-rebuild)."
fi

# Ensure wiki_data exists
mkdir -p "$WIKI_DATA"

# Run the container
echo ""
echo "Starting container '$CONTAINER' on http://localhost:$PORT ..."
echo "Wiki data:  $WIKI_DATA"
echo "Auth:       DEV_BYPASS_AUTH=true (no login required)"
echo ""

docker run \
    --name "$CONTAINER" \
    --rm \
    -p "${PORT}:3000" \
    -v "${WIKI_DATA}:/wiki_data" \
    -e "WIKI_PATH=/wiki_data" \
    -e "DEV_BYPASS_AUTH=true" \
    "$IMAGE" &

DOCKER_PID=$!

# Wait for server to be ready
echo "Waiting for server to be ready..."
for i in $(seq 1 30); do
    if curl -sf "http://localhost:$PORT" > /dev/null 2>&1; then
        echo "Server is up at http://localhost:$PORT"
        break
    fi
    if [ "$i" = "30" ]; then
        echo "Server did not start after 30 seconds."
        docker logs "$CONTAINER" 2>/dev/null || true
        exit 1
    fi
    sleep 1
done

echo ""
echo "----------------------------------------------------------"
echo "  WebWiki running at http://localhost:$PORT"
echo "  Press Ctrl+C to stop and remove the container."
echo "----------------------------------------------------------"
echo ""

# Stream container logs
docker logs -f "$CONTAINER" 2>&1 &
LOGS_PID=$!

# Cleanup on exit
cleanup() {
    echo ""
    echo "Stopping container '$CONTAINER'..."
    kill "$LOGS_PID" 2>/dev/null || true
    docker stop "$CONTAINER" 2>/dev/null || true
    wait "$DOCKER_PID" 2>/dev/null || true
    echo "Done."
}
trap cleanup EXIT INT TERM

# Block until Ctrl+C
wait "$DOCKER_PID"
