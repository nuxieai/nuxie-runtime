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
video_bindgen="$repo_dir/target/browser-tools/bin/wasm-bindgen"
if [[ ! -x "$video_bindgen" ]]; then
  "$video_cargo" install wasm-bindgen-cli --version 0.2.126 --locked --root "$repo_dir/target/browser-tools"
fi
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 "$video_cargo" build -p video-qualification --lib --target wasm32-unknown-unknown --profile "$video_profile"
video_output="$repo_dir/target/video-browser-proof"
mkdir -p "$video_output"
"$video_bindgen" "$repo_dir/target/wasm32-unknown-unknown/$video_profile_dir/video_qualification.wasm" --out-dir "$video_output" --target web
cp "$proof_dir/index.html" "$video_output/index.html"
cp "$proof_dir/sync.html" "$video_output/sync.html"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-sync.mp4" "$video_output/"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-audio.mp4" "$video_output/"
printf 'Serve this generated directory on an available localhost port: %s\n' "$video_output"

cp "$proof_dir/benchmark.html" "$video_output/benchmark.html"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-720p.mp4" "$video_output/"
