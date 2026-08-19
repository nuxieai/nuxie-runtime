#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: native Metal tracer binary check requires macOS" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
artifact="$repo_root/target/release-size/examples/native_metal_tracer_root"

cargo build \
  --manifest-path "$repo_root/Cargo.toml" \
  --profile release-size \
  -p nuxie-renderer \
  --example native_metal_tracer_root \
  --features native-metal-experimental
"$artifact"

if ! file "$artifact" | grep -q 'Mach-O'; then
  echo "error: native Metal tracer root is not a Mach-O" >&2
  exit 1
fi

symbols="$(mktemp)"
strings_file="$(mktemp)"
trap 'rm -f "$symbols" "$strings_file"' EXIT
nm -j "$artifact" >"$symbols"
strings "$artifact" >"$strings_file"

forbidden='wgpu|naga|wgsl'
if grep -Eiq "$forbidden" "$symbols" "$strings_file"; then
  echo "error: rooted native Metal tracer Mach-O retains a forbidden renderer dependency" >&2
  grep -Ei "$forbidden" "$symbols" "$strings_file" | head -40 >&2
  exit 1
fi

bytes="$(stat -f '%z' "$artifact")"
digest="$(shasum -a 256 "$artifact" | awk '{print $1}')"
echo "native Metal tracer Mach-O passed: bytes=$bytes sha256=$digest"
