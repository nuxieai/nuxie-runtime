#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RIVE_RUNTIME_DIR="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
UPSTREAM_REF="4ac7b32798da0482e441ef09304dc3b480ed3ee5"
EMSDK_REF="e5bd3d0874e302a18f13c5b41f5bacf9a40c8e59"
EMSDK_DIR="$RIVE_RUNTIME_DIR/build/dependencies/emsdk_3.1.61"
EMSCRIPTEN_DIR="$EMSDK_DIR/upstream/emscripten"
RUST_TOOLCHAIN="1.91.1-aarch64-apple-darwin"
RUST_TOOLCHAIN_DIR="/Users/levi/.rustup/toolchains/$RUST_TOOLCHAIN"
TARGET_DIR="$REPO_ROOT/target/renderer-webgl2-live-reference-1.91"
RENDERER_OUT="$RIVE_RUNTIME_DIR/renderer/out/cpp-webgl2-oracle"

test "$(git -C "$RIVE_RUNTIME_DIR" rev-parse HEAD)" = "$UPSTREAM_REF"
test -z "$(git -C "$RIVE_RUNTIME_DIR" status --short --untracked-files=no)"
test "$(git -C "$EMSDK_DIR" rev-parse HEAD)" = "$EMSDK_REF"
$EMSCRIPTEN_DIR/emcc --version | head -1 | grep -q ') 3\.1\.61 ('
test "$($RUST_TOOLCHAIN_DIR/bin/rustc --version | awk '{print $2}')" = "1.91.1"

if [[ ! -f "$RENDERER_OUT/librive_pls_renderer.a" ]]; then
  (
    cd "$RIVE_RUNTIME_DIR/renderer"
    RIVE_OUT=out/cpp-webgl2-oracle RIVE_EMSDK_VERSION=3.1.61 \
      ../build/build_rive.sh ninja release wasm --no-lto -- \
      rive rive_pls_renderer rive_decoders libpng zlib libjpeg libwebp \
      rive_harfbuzz rive_sheenbidi rive_yoga
  )
fi

export RUSTC="$RUST_TOOLCHAIN_DIR/bin/rustc"
export RUSTDOC="$RUST_TOOLCHAIN_DIR/bin/rustdoc"
export EMSDK="$EMSDK_DIR"
export EM_CONFIG="$EMSDK_DIR/.emscripten"
export PATH="$EMSDK_DIR:$EMSCRIPTEN_DIR:$PATH"
export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER="$EMSCRIPTEN_DIR/emcc"
# Keep Rust/C++ release optimization, but skip Emscripten's post-link Binaryen
# optimization: 3.1.61 predates a target-feature spelling emitted by Rust 1.91.
export RUSTFLAGS="-C link-arg=-sMIN_WEBGL_VERSION=2 \
-C link-arg=-sMAX_WEBGL_VERSION=2 \
-C link-arg=-sALLOW_MEMORY_GROWTH=1 \
-C link-arg=-sEXIT_RUNTIME=0 \
-C link-arg=-sINVOKE_RUN=0 \
-C link-arg=-sEXPORTED_RUNTIME_METHODS=FS,callMain \
-C link-arg=-O0"

cd "$REPO_ROOT"
rustup run 1.91.1 cargo build \
  --release \
  --target wasm32-unknown-emscripten \
  --target-dir "$TARGET_DIR" \
  -p renderer-replay \
  --no-default-features \
  --features ffi-webgl2

test -f "$TARGET_DIR/wasm32-unknown-emscripten/release/renderer-replay.js"
test -f "$TARGET_DIR/wasm32-unknown-emscripten/release/renderer_replay.wasm"
