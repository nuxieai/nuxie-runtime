#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
STABLE_CARGO="${CARGO:-$(rustup which --toolchain stable cargo)}"
export RUSTC="${RUSTC:-$(rustup which --toolchain stable rustc)}"

WASM_BINDGEN_VERSION=0.2.126
TOOLS_ROOT="$ROOT/target/browser-tools"
WASM_BINDGEN="$TOOLS_ROOT/bin/wasm-bindgen"
if [[ ! -x "$WASM_BINDGEN" ]] ||
   [[ "$($WASM_BINDGEN --version 2>/dev/null || true)" != "wasm-bindgen $WASM_BINDGEN_VERSION" ]]; then
  "$STABLE_CARGO" install wasm-bindgen-cli \
    --version "$WASM_BINDGEN_VERSION" \
    --locked \
    --root "$TOOLS_ROOT"
fi

"$STABLE_CARGO" build \
  --release \
  --package browser-renderer-smoke \
  --target wasm32-unknown-unknown

"$WASM_BINDGEN" \
  "$ROOT/target/wasm32-unknown-unknown/release/browser_renderer_smoke.wasm" \
  --out-dir "$ROOT/tools/browser-renderer-smoke/pkg" \
  --target web
