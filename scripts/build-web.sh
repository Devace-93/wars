#!/usr/bin/env bash
# Build the browser client into apps/web/pkg (served by the server at /play/).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown -p wars-web "$@"
wasm-bindgen --target web --no-typescript --out-dir apps/web/pkg target/wasm32-unknown-unknown/release/wars_web.wasm
ls -la apps/web/pkg/
