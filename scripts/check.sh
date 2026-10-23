#!/usr/bin/env bash
# Strict pre-commit check: format, lint and test the whole workspace.
set -euo pipefail
cd "$(dirname "$0")"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features wars-client/net -- -D warnings
cargo test --workspace --features wars-client/net

# Browser client (only when the wasm target is installed; CI installs it).
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
  cargo clippy --target wasm32-unknown-unknown -p wars-web -- -D warnings
fi
