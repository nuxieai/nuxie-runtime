#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WASM_BINDGEN=$(python3 "$ROOT/tools/bazel/browser-tools.py")

python3 "$ROOT/tools/bazel/runtime.py" build \
  --release \
  --package webgl2-renderer-replay \
  --target wasm32-unknown-unknown --target-dir "$ROOT/target"

"$WASM_BINDGEN" \
  "$ROOT/target/wasm32-unknown-unknown/release/webgl2_renderer_replay.wasm" \
  --out-dir "$ROOT/tools/webgl2-renderer-replay/pkg" \
  --target web
