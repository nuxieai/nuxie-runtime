#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
clang -fobjc-arc -fblocks \
  "$root/crates/nuxie-video-host/tests/apple-paused-seek.m" \
  "$root/crates/nuxie-video-host/src/apple/player.m" \
  -framework Foundation -framework AVFoundation -framework CoreMedia \
  -framework CoreVideo -framework QuartzCore -o "$scratch/paused-seek"
"$scratch/paused-seek" "$root/fixtures/video/red-blue-audio.mp4"
