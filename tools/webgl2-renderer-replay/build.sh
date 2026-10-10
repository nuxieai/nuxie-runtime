#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUST_TOOLCHAIN=1.94.1
PINNED_CARGO="${CARGO:-$(rustup which --toolchain "$RUST_TOOLCHAIN" cargo)}"
export RUSTC="${RUSTC:-$(rustup which --toolchain "$RUST_TOOLCHAIN" rustc)}"

WASM_BINDGEN_VERSION=0.2.126
TOOLS_ROOT="$ROOT/target/browser-tools"
WASM_BINDGEN="$TOOLS_ROOT/bin/wasm-bindgen"
if [[ ! -x "$WASM_BINDGEN" ]] ||
   [[ "$($WASM_BINDGEN --version 2>/dev/null || true)" != "wasm-bindgen $WASM_BINDGEN_VERSION" ]]; then
  "$PINNED_CARGO" install wasm-bindgen-cli \
    --version "$WASM_BINDGEN_VERSION" \
    --locked \
    --root "$TOOLS_ROOT"
fi

python3 "$ROOT/tools/bazel/runtime.py" build \
  --release \
  --package webgl2-renderer-replay \
  --target wasm32-unknown-unknown

"$WASM_BINDGEN" \
  "$ROOT/target/wasm32-unknown-unknown/release/webgl2_renderer_replay.wasm" \
  --out-dir "$ROOT/tools/webgl2-renderer-replay/pkg" \
  --target web
