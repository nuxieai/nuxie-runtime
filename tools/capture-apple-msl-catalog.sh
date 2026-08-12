#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
capture_dir="${NUXIE_APPLE_MSL_CAPTURE_DIR:-$root/target/apple-msl-capture}"

NUXIE_APPLE_MSL_CAPTURE_DIR="$capture_dir" cargo run --locked \
  --manifest-path "$root/Cargo.toml" \
  -p apple-msl-capture -- "$root" "$capture_dir"
"$root/tools/generate-apple-msl-catalog.sh"
