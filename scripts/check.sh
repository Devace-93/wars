#!/usr/bin/env bash
# Strict pre-commit check: format, lint and test the whole workspace.
set -euo pipefail
cd "$(dirname "$0")"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features wars-client/net -- -D warnings
cargo test --workspace --features wars-client/net
