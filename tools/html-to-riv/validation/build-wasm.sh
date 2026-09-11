#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
# Use one matching rustup compiler/std pair, even if Homebrew cargo leads PATH.
if command -v rustup >/dev/null 2>&1; then
  export RUSTC="$(rustup which rustc)"
fi
cargo build -p nuxie-html-to-riv --locked --lib --target wasm32-unknown-unknown --release
mkdir -p tools/html-to-riv/dist
cp "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/nuxie_html_to_riv.wasm" tools/html-to-riv/dist/html-to-riv.wasm
