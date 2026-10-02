#!/usr/bin/env bash
# Builds the Rust client for the browser (WebGL2) into web/pkg.
# Needs: rustup target add wasm32-unknown-unknown; cargo install wasm-bindgen-cli
# at the version in Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p aoa-client --lib --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/aoa_client.wasm
