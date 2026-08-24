#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
runtime_revision="4ac7b32798da0482e441ef09304dc3b480ed3ee5"
dawn_revision="211333b2e3e429c3508f25c81c547f602adf448c"
emsdk_revision="948c31acd3f369a5da276e33ab2ed57108c165e5"
emsdk_dir="$repo_root/target/backend-port/emsdk-5.0.6"
port="$repo_root/target/backend-port/dawn-emdawn-5.0.6/emdawnwebgpu_pkg/emdawnwebgpu.port.py"
renderer_out="$runtime_dir/tests/out/webgpu_5_0_6"
target_dir="$repo_root/target/renderer-webgpu-live-reference"
rust_toolchain_dir="/Users/levi/.rustup/toolchains/1.91.1-aarch64-apple-darwin"

test "$(git -C "$runtime_dir" rev-parse HEAD)" = "$runtime_revision"
test -z "$(git -C "$runtime_dir" status --short --untracked-files=no)"
test "$(git -C "$runtime_dir/renderer/dependencies/dawn" rev-parse HEAD)" = "$dawn_revision"
test -z "$(git -C "$runtime_dir/renderer/dependencies/dawn" status --short --untracked-files=no)"
test "$(git -C "$emsdk_dir" rev-parse HEAD)" = "$emsdk_revision"
test -x "$emsdk_dir/upstream/emscripten/em++"
test -f "$port"
test -f "$renderer_out/librive_pls_renderer.a"
test -f "$renderer_out/librive.a"
test "$("$rust_toolchain_dir/bin/rustc" --version | awk '{print $2}')" = "1.91.1"

source "$emsdk_dir/emsdk_env.sh" >/dev/null
export EMSDK="$emsdk_dir"
export EMDAWNWEBGPU_PORT="$port"
export RIVE_RUNTIME_DIR="$runtime_dir"
export RIVE_RENDERER_OUT_DIR="$renderer_out"
export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER="$emsdk_dir/upstream/emscripten/emcc"
export RUSTC="$rust_toolchain_dir/bin/rustc"
export RUSTDOC="$rust_toolchain_dir/bin/rustdoc"
export RUSTFLAGS="-C link-arg=--use-port=$port \
-C link-arg=-sDEFAULT_TO_CXX \
-C link-arg=-sASYNCIFY=1 \
-C link-arg=-sALLOW_MEMORY_GROWTH=1 \
-C link-arg=-sINVOKE_RUN=0 \
-C link-arg=-sEXIT_RUNTIME=0 \
-C link-arg=-sEXPORTED_RUNTIME_METHODS=FS,callMain,Asyncify"

cd "$repo_root"
"$rust_toolchain_dir/bin/cargo" build \
  --release \
  --offline \
  --target wasm32-unknown-emscripten \
  --target-dir "$target_dir" \
  --package renderer-replay \
  --no-default-features \
  --features perf-dawn

test -f "$target_dir/wasm32-unknown-emscripten/release/renderer-replay.js"
test -f "$target_dir/wasm32-unknown-emscripten/release/deps/renderer_replay.wasm"
echo "built pinned C++ WebGPU browser source oracle"
