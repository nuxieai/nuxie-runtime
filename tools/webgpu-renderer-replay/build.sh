#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WASM_BINDGEN=$(python3 "$ROOT/tools/bazel/browser-tools.py")
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=--export-table"

python3 "$ROOT/tools/bazel/runtime.py" build \
  --release \
  --package webgpu-renderer-replay \
  --target wasm32-unknown-unknown --target-dir "$ROOT/target"

"$WASM_BINDGEN" \
  "$ROOT/target/wasm32-unknown-unknown/release/webgpu_renderer_replay.wasm" \
  --out-dir "$ROOT/tools/webgpu-renderer-replay/pkg" \
  --target web \
  --keep-lld-exports

python3 "$ROOT/tools/webgpu-renderer-replay/inject_webgpu_imports.py" \
  "$ROOT/tools/webgpu-renderer-replay/pkg/webgpu_renderer_replay.js"
