# =============================================================================
# Stage 1: Chef — prepare dependency recipe for caching
# =============================================================================
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# =============================================================================
# Stage 2: Planner — generate the recipe.json from lockfile + manifests
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Builder — cache dependencies, install dx, bundle the app
# =============================================================================
FROM chef AS builder

# System dependencies for building Dioxus (webkit/gtk needed by dioxus crate)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    && rm -rf /var/lib/apt/lists/*

# Install dioxus-cli
RUN cargo install dioxus-cli

# Add wasm target for client-side build
RUN rustup target add wasm32-unknown-unknown

# Cook dependencies (cached layer — only rebuilds when deps change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full source and bundle (offline mode uses committed .sqlx/ cache)
COPY . .
ENV SQLX_OFFLINE=true
RUN dx bundle --package app --platform web --release

# =============================================================================
# Stage 4: Runtime — minimal image with just the bundled output
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash appuser

WORKDIR /app

# Copy the bundled fullstack output from the builder
# dx bundle --platform web puts output in target/dx/<app-name>/release/web/
COPY --from=builder /app/target/dx/app/release/web/ ./

# Ensure the server binary is executable
RUN chmod +x ./app

# Switch to non-root user
USER appuser

# Fly.io requires listening on 0.0.0.0
ENV IP=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["./app"]
