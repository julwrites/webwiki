#!/bin/sh
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking Formatting..."
cargo fmt --all -- --check

# Backend
echo "Checking Backend..."
cd backend
cargo clippy -- -D warnings
cargo test
cd ..

# Frontend
echo "Checking Frontend..."
cd frontend
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cd ..

echo "Pre-commit checks passed!"
