#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

prohibited_pattern='webgl2|femtovg|BrowserBackendPreference|BrowserBackend|WebGl2Factory|WebGl2Frame|WebGl2GpuCanvasRenderer|fallback_reason'
prohibited_paths=(
  "$repo_dir/crates/nuxie-renderer"
  "$repo_dir/crates/nuxie/src/lib.rs"
  "$repo_dir/tools/browser-renderer-smoke"
  "$repo_dir/Cargo.toml"
  "$repo_dir/Cargo.lock"
  "$repo_dir/.github/workflows/ci.yml"
)

if rg -n -i "$prohibited_pattern" "${prohibited_paths[@]}"; then
  echo "browser-webgpu-only check found a retired implementation, API, dependency, or test surface" >&2
  exit 1
fi

test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2.rs"
test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2_limits.rs"

echo "browser-webgpu-only summary: browser-smoke=pass gpu-smoke=pass prohibited-surface=0"
