# ── Stage 1: Build ──────────────────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /app

# The workspace: server (root), shared simulation and the wgpu client.
RUN rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli --version 0.2.129 --locked
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY crates/ crates/
COPY scripts/ scripts/
COPY web/index.html web/index.html
RUN cargo build --release --locked -p age-of-agents && ./scripts/build_web.sh

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (SSL certs + Python for Modal compatibility)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/age-of-agents /app/age-of-agents

# Copy the current client assets (sprites, textures, UI)
COPY assets/ /app/assets/
COPY --from=builder /app/web/ /app/web/

EXPOSE 8000

ENTRYPOINT ["/app/age-of-agents"]