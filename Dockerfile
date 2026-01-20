# Stage 1: Build Frontend
FROM rust:1.85 AS frontend-builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app
COPY . .

# Build frontend
WORKDIR /app/frontend
RUN wasm-pack build --target web --out-name wasm --out-dir ./static

# Stage 2: Build Backend
FROM rust:1.85 AS backend-builder

WORKDIR /app
COPY . .

# Build backend
WORKDIR /app/backend
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/backend /app/backend_bin

# Copy frontend static files
COPY --from=frontend-builder /app/frontend/static /app/static
COPY --from=frontend-builder /app/frontend/index.html /app/static/index.html

# Expose port
EXPOSE 3000

# Run
CMD ["/app/backend_bin"]
