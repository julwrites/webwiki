#!/bin/sh
set -e

echo "Running pre-commit checks..."

# Backend
echo "Checking Backend..."
cd backend
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cd ..

# Frontend
echo "Checking Frontend..."
cd frontend
cargo fmt -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cd ..

echo "Pre-commit checks passed!"
