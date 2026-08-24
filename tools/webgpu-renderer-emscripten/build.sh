#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR must point at the pinned rive-runtime checkout}"
dawn_dir="$runtime_dir/renderer/dependencies/dawn"
expected_runtime_revision="4ac7b32798da0482e441ef09304dc3b480ed3ee5"
expected_dawn_revision="211333b2e3e429c3508f25c81c547f602adf448c"
expected_emsdk_revision="948c31acd3f369a5da276e33ab2ed57108c165e5"
emsdk_version="5.0.6"
emsdk_dir="${EMSDK_DIR:-$root/target/backend-port/emsdk-$emsdk_version}"
dawn_build_dir="${EMDAWNWEBGPU_BUILD_DIR:-$root/target/backend-port/dawn-emdawn-$emsdk_version}"
package_dir="$dawn_build_dir/emdawnwebgpu_pkg"
jobs="${EMDAWNWEBGPU_JOBS:-4}"

if [[ ! -d "$dawn_dir/.git" ]]; then
    mkdir -p "$(dirname "$dawn_dir")"
    git clone https://dawn.googlesource.com/dawn "$dawn_dir"
    git -C "$dawn_dir" checkout --detach "$expected_dawn_revision"
    python3 "$dawn_dir/tools/fetch_dawn_dependencies.py" --directory "$dawn_dir"
fi

for command in cmake git ninja rustup; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing exact WebGPU browser build tool: $command" >&2
        exit 2
    fi
done
if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then
    echo "EMDAWNWEBGPU_JOBS must be a positive integer, got '$jobs'" >&2
    exit 2
fi

actual_runtime_revision="$(git -C "$runtime_dir" rev-parse HEAD)"
if [[ "$actual_runtime_revision" != "$expected_runtime_revision" ]]; then
    echo "rive-runtime drifted: expected $expected_runtime_revision, got $actual_runtime_revision" >&2
    exit 2
fi
if ! git -C "$runtime_dir" diff --quiet || ! git -C "$runtime_dir" diff --cached --quiet; then
    echo "rive-runtime has tracked changes; refusing a non-pinned browser build" >&2
    exit 2
fi
actual_dawn_revision="$(git -C "$dawn_dir" rev-parse HEAD)"
if [[ "$actual_dawn_revision" != "$expected_dawn_revision" ]]; then
    echo "Dawn drifted: expected $expected_dawn_revision, got $actual_dawn_revision" >&2
    exit 2
fi
if ! git -C "$dawn_dir" diff --quiet || ! git -C "$dawn_dir" diff --cached --quiet; then
    echo "Dawn has tracked changes; refusing a non-pinned browser build" >&2
    exit 2
fi
if ! grep -Fq "emsdk.git@$expected_emsdk_revision" "$dawn_dir/DEPS"; then
    echo "Dawn's emsdk pin no longer matches $expected_emsdk_revision" >&2
    exit 2
fi

if [[ ! -d "$emsdk_dir/.git" ]]; then
    git clone --branch "$emsdk_version" --depth 1 \
        https://github.com/emscripten-core/emsdk.git "$emsdk_dir"
fi
actual_emsdk_revision="$(git -C "$emsdk_dir" rev-parse HEAD)"
if [[ "$actual_emsdk_revision" != "$expected_emsdk_revision" ]]; then
    echo "emsdk drifted: expected $expected_emsdk_revision, got $actual_emsdk_revision" >&2
    exit 2
fi
if [[ ! -x "$emsdk_dir/upstream/emscripten/emcc" ]]; then
    "$emsdk_dir/emsdk" install "$emsdk_version"
    "$emsdk_dir/emsdk" activate "$emsdk_version"
fi
source "$emsdk_dir/emsdk_env.sh" >/dev/null
if ! emcc --version | head -1 | grep -q ") $emsdk_version ("; then
    echo "activated emcc is not exact version $emsdk_version" >&2
    exit 2
fi

emcmake cmake -S "$dawn_dir" -B "$dawn_build_dir" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DDAWN_BUILD_SAMPLES=OFF \
    -DDAWN_BUILD_TESTS=OFF \
    -DDAWN_BUILD_BENCHMARKS=OFF \
    -DDAWN_BUILD_FUZZERS=OFF \
    -DDAWN_FETCH_DEPENDENCIES=OFF \
    -DDAWN_USE_GLFW=OFF \
    -DDAWN_WERROR=OFF
cmake --build "$dawn_build_dir" --target emdawnwebgpu_pkg --parallel "$jobs"

stable_cargo="${CARGO:-$(rustup which --toolchain stable cargo)}"
export RUSTC="${RUSTC:-$(rustup which --toolchain stable rustc)}"
port="$package_dir/emdawnwebgpu.port.py"
common_rustflags="-C link-arg=--use-port=$port \
-C link-arg=-sDEFAULT_TO_CXX \
-C link-arg=-sASYNCIFY=1 \
-C link-arg=-sALLOW_MEMORY_GROWTH=1"

RUSTFLAGS="$common_rustflags" \
"$stable_cargo" build \
    --release \
    --offline \
    --package webgpu-renderer-emscripten \
    --target wasm32-unknown-emscripten

RUSTFLAGS="$common_rustflags \
-C link-arg=-sINVOKE_RUN=0 \
-C link-arg=-sEXIT_RUNTIME=0 \
-C link-arg=-sEXPORTED_RUNTIME_METHODS=FS,callMain,Asyncify" \
"$stable_cargo" build \
    --release \
    --offline \
    --package renderer-replay \
    --no-default-features \
    --features browser-webgpu-exact \
    --target wasm32-unknown-emscripten

mkdir -p "$root/tools/webgpu-renderer-emscripten/pkg"
cp "$root/target/wasm32-unknown-emscripten/release/webgpu-renderer-emscripten.js" \
    "$root/tools/webgpu-renderer-emscripten/pkg/webgpu-renderer-emscripten.js"
cp "$root/target/wasm32-unknown-emscripten/release/deps/webgpu_renderer_emscripten.wasm" \
    "$root/tools/webgpu-renderer-emscripten/pkg/webgpu_renderer_emscripten.wasm"
cp "$root/target/wasm32-unknown-emscripten/release/renderer-replay.js" \
    "$root/tools/webgpu-renderer-emscripten/pkg/renderer-replay.js"
cp "$root/target/wasm32-unknown-emscripten/release/deps/renderer_replay.wasm" \
    "$root/tools/webgpu-renderer-emscripten/pkg/renderer_replay.wasm"

echo "built exact WebGPU browser root in tools/webgpu-renderer-emscripten/pkg"
