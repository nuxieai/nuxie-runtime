#!/usr/bin/env bash
set -euo pipefail
proof_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$proof_dir/../../.." && pwd)
cd "$repo_dir"
video_profile=${VIDEO_PROOF_PROFILE:-dev}
case "$video_profile" in
  dev) video_profile_dir=debug ;;
  release) video_profile_dir=release ;;
  *) printf 'Unsupported VIDEO_PROOF_PROFILE: %s\n' "$video_profile" >&2; exit 1 ;;
esac
video_cargo=$(rustup which --toolchain stable cargo)
export RUSTC=$(rustup which --toolchain stable rustc)
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=--export-table"
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 "$video_cargo" build -p video-qualification --features webgpu --lib --target wasm32-unknown-unknown --profile "$video_profile"
video_output="$repo_dir/target/video-browser-proof/webgpu"
mkdir -p "$video_output"
"$repo_dir/target/browser-tools/bin/wasm-bindgen" "$repo_dir/target/wasm32-unknown-unknown/$video_profile_dir/video_qualification.wasm" --out-dir "$video_output" --target web --keep-lld-exports
cp "$repo_dir/tools/webgpu-renderer-replay/webgpu-host.js" "$video_output/"
python3 "$repo_dir/tools/webgpu-renderer-replay/inject_webgpu_imports.py" "$video_output/video_qualification.js" ./webgpu-host.js
cp "$proof_dir/webgpu.html" "$video_output/index.html"
cp "$proof_dir/sync.html" "$video_output/sync.html"
cp "$repo_dir/fixtures/video/red-blue-sync.mp4" "$video_output/"
cp "$repo_dir/fixtures/video/red-blue-audio.mp4" "$video_output/"

cp "$proof_dir/benchmark.html" "$video_output/benchmark.html"
cp "$repo_dir/fixtures/video/red-blue-720p.mp4" "$video_output/"
