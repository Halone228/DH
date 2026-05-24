# ── Stage 1: Build Rust server ──────────────────────────────────────────────
FROM rust:1.78-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libsqlite3-dev libwayland-dev libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

COPY crates/domain/Cargo.toml crates/domain/Cargo.toml
COPY crates/ports/Cargo.toml crates/ports/Cargo.toml
COPY crates/application/Cargo.toml crates/application/Cargo.toml
COPY crates/adapter-sqlite/Cargo.toml crates/adapter-sqlite/Cargo.toml
COPY crates/adapter-telegram/Cargo.toml crates/adapter-telegram/Cargo.toml
COPY crates/adapter-system/Cargo.toml crates/adapter-system/Cargo.toml
COPY crates/scheduler/Cargo.toml crates/scheduler/Cargo.toml
COPY crates/bot/Cargo.toml crates/bot/Cargo.toml
COPY crates/tma/Cargo.toml crates/tma/Cargo.toml
COPY crates/server-desktop-api/Cargo.toml crates/server-desktop-api/Cargo.toml
COPY crates/protocol/Cargo.toml crates/protocol/Cargo.toml
COPY crates/app/Cargo.toml crates/app/Cargo.toml

# Desktop crates (needed for workspace resolution)
COPY crates/desktop-domain/Cargo.toml crates/desktop-domain/Cargo.toml
COPY crates/desktop-ports/Cargo.toml crates/desktop-ports/Cargo.toml
COPY crates/desktop-application/Cargo.toml crates/desktop-application/Cargo.toml
COPY crates/desktop-adapter-dbus/Cargo.toml crates/desktop-adapter-dbus/Cargo.toml
COPY crates/desktop-adapter-http/Cargo.toml crates/desktop-adapter-http/Cargo.toml
COPY crates/desktop-adapter-sqlite/Cargo.toml crates/desktop-adapter-sqlite/Cargo.toml
COPY crates/desktop-adapter-wayland/Cargo.toml crates/desktop-adapter-wayland/Cargo.toml
COPY crates/desktop-app/Cargo.toml crates/desktop-app/Cargo.toml

# Create dummy source files to cache dependency compilation
RUN mkdir -p crates/domain/src && echo "" > crates/domain/src/lib.rs && \
    mkdir -p crates/ports/src && echo "" > crates/ports/src/lib.rs && \
    mkdir -p crates/application/src && echo "" > crates/application/src/lib.rs && \
    mkdir -p crates/adapter-sqlite/src && echo "" > crates/adapter-sqlite/src/lib.rs && \
    mkdir -p crates/adapter-telegram/src && echo "" > crates/adapter-telegram/src/lib.rs && \
    mkdir -p crates/adapter-system/src && echo "" > crates/adapter-system/src/lib.rs && \
    mkdir -p crates/scheduler/src && echo "" > crates/scheduler/src/lib.rs && \
    mkdir -p crates/bot/src && echo "" > crates/bot/src/lib.rs && \
    mkdir -p crates/tma/src && echo "" > crates/tma/src/lib.rs && \
    mkdir -p crates/server-desktop-api/src && echo "" > crates/server-desktop-api/src/lib.rs && \
    mkdir -p crates/protocol/src && echo "" > crates/protocol/src/lib.rs && \
    mkdir -p crates/app/src && echo "fn main(){}" > crates/app/src/main.rs && \
    mkdir -p crates/desktop-domain/src && echo "" > crates/desktop-domain/src/lib.rs && \
    mkdir -p crates/desktop-ports/src && echo "" > crates/desktop-ports/src/lib.rs && \
    mkdir -p crates/desktop-application/src && echo "" > crates/desktop-application/src/lib.rs && \
    mkdir -p crates/desktop-adapter-dbus/src && echo "" > crates/desktop-adapter-dbus/src/lib.rs && \
    mkdir -p crates/desktop-adapter-http/src && echo "" > crates/desktop-adapter-http/src/lib.rs && \
    mkdir -p crates/desktop-adapter-sqlite/src && echo "" > crates/desktop-adapter-sqlite/src/lib.rs && \
    mkdir -p crates/desktop-adapter-wayland/src && echo "" > crates/desktop-adapter-wayland/src/lib.rs && \
    mkdir -p crates/desktop-app/src && echo "fn main(){}" > crates/desktop-app/src/main.rs

# Build dependencies only (cache this layer)
ENV DATABASE_URL=sqlite::memory:
RUN cargo build -p dayhelper-app --release 2>/dev/null || true

# Now copy actual source and build for real
COPY . .
RUN touch crates/*/src/lib.rs crates/*/src/main.rs
RUN cargo build -p dayhelper-app --release

# ── Stage 2: Build frontend ─────────────────────────────────────────────────
FROM node:20-slim AS frontend
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# ── Stage 3: Runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/dayhelper-app /app/dayhelper-app
COPY --from=frontend /app/dist /app/frontend/dist

EXPOSE 8080
ENV RUST_LOG=info
CMD ["/app/dayhelper-app"]
