#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if [[ "$(uname -s)" != Darwin ]]; then
    echo "error: the no-WGSL passthrough probe requires macOS and a real Metal device" >&2
    exit 2
fi

feature_tree="$(mktemp "${TMPDIR:-/tmp}/apple-msl-no-wgsl-features.XXXXXX")"
trap 'rm -f "$feature_tree"' EXIT

cargo tree --locked -p apple-msl-no-wgsl-probe -e features >"$feature_tree"

if grep -Eq 'wgpu feature "wgsl"' "$feature_tree"; then
    echo "error: apple-msl-no-wgsl-probe resolved wgpu's WGSL feature" >&2
    grep -E 'wgpu feature "wgsl"' "$feature_tree" >&2
    exit 1
fi
if grep -Eq 'naga feature "wgsl-in"' "$feature_tree"; then
    echo "error: apple-msl-no-wgsl-probe resolved Naga's WGSL parser" >&2
    grep -E 'naga feature "wgsl-in"' "$feature_tree" >&2
    exit 1
fi

cargo run --locked -p apple-msl-no-wgsl-probe
