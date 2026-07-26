#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
rive_runtime=${RIVE_RUNTIME_DIR:?set RIVE_RUNTIME_DIR to the pinned rive-runtime checkout}
expected_rive=d788e8ec6e8b598526607d6a1e8818e8b637b60c
fixture="$repo_root/tools/editor-next-runtime-defects/fixtures/loc014-box-shadow.rive-stream"
output="$repo_root/target/editor-next-runtime-defects/loc014"

test "$(git -C "$rive_runtime" rev-parse HEAD)" = "$expected_rive"
git -C "$rive_runtime" diff --quiet
git -C "$rive_runtime" diff --cached --quiet
test "$(shasum -a 256 "$fixture" | awk '{print $1}')" = b66d076a01e4e1587024508f88957d045d18ef4c78db92b0a6e6f189359b946f

make -C "$repo_root" renderer-rust-replay-release renderer-dawn-live-reference-replay renderer-dawn-live-reference-check \
  RIVE_RUNTIME_DIR="$rive_runtime" RENDERER_JOBS="${RENDERER_JOBS:-4}"
test "$(shasum -a 256 "$repo_root/target/renderer-dawn-live-reference/renderer-replay" | awk '{print $1}')" = 867ae8ca2fe96cb6321d3d0f4bf5487aa9b7d2a5eb3d36c4d55f4d1d42860765
test "$(shasum -a 256 "$repo_root/target/renderer-golden/release/renderer-replay" | awk '{print $1}')" = 1259afaa779a2efee71823a9906544613c58a1dd03af19db1a98bd8d4822ef25
mkdir -p "$output"
"$repo_root/target/renderer-dawn-live-reference/renderer-replay" \
  --stream "$fixture" --output "$output/cpp.png" --backend ffi-dawn --mode msaa
"$repo_root/target/renderer-golden/release/renderer-replay" \
  --stream "$fixture" --output "$output/rust.png" --backend rust-wgpu --mode msaa
cargo run --quiet --manifest-path "$repo_root/tools/pixel-compare/Cargo.toml" \
  --bin pixel-compare -- "$output/cpp.png" "$output/rust.png" \
  --max-channel-delta 0 --max-different-pixels 0 --artifact "$output/diff.png"
test "$(shasum -a 256 "$output/cpp.png" | awk '{print $1}')" = f5c5e920cefde88fea5f4f7e1477151f9614de9730bebe0774a4a96ef55b3fa1
