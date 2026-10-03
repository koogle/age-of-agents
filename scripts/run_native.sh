#!/usr/bin/env bash
# Rebuilds and launches the native app (macOS, Linux, or Windows with Bash).
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run -p aoa-client --release --locked
