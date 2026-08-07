#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

test ! -e "$repo_dir/crates/nuxie-browser-adapter"

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

smoke_source="$repo_dir/tools/browser-renderer-smoke/src/lib.rs"
if rg -n 'BrowserFactory|BrowserFrame|BrowserResizeError|browser_surface_lifecycle|finish_with_readback' \
  "$repo_dir/Cargo.toml" \
  "$repo_dir/Cargo.lock" \
  "$repo_dir/tools/browser-renderer-smoke"; then
  echo "browser-webgpu-only check found browser product lifecycle policy in the runtime workspace" >&2
  exit 1
fi

cpu_presentation_pattern='CanvasRenderingContext2d|ImageData|put_image_data|present_pixels'
if rg -n "$cpu_presentation_pattern" \
  "$repo_dir/crates/nuxie-renderer/src/presentation.rs" \
  "$repo_dir/crates/nuxie-renderer/Cargo.toml" \
  "$smoke_source"; then
  echo "browser-webgpu-only check found a CPU canvas presentation path" >&2
  exit 1
fi

presentation_source="$repo_dir/crates/nuxie-renderer/src/presentation.rs"
rg -q 'get_current_texture\(\)' "$presentation_source"
rg -q 'queue\.present\(self\.texture\)' "$presentation_source"
rg -q 'WgpuPresentationAlpha::Premultiplied => wgpu::CompositeAlphaMode::PreMultiplied' \
  "$presentation_source"
rg -q 'pub async fn present\(self, frame: WgpuFrame\)' "$presentation_source"

direct_presentation_body=$(
  sed -n \
    '/pub async fn assert_direct_presentation(/,/pub async fn assert_explicit_readback(/p' \
    "$smoke_source"
)
rg -Fq '.create_presentation_surface(' <<<"$direct_presentation_body"
rg -Fq 'WgpuPresentationAlpha::Premultiplied' <<<"$direct_presentation_body"
rg -Fq '.acquire()' <<<"$direct_presentation_body"
rg -Fq '.present(factory.begin_frame(' <<<"$direct_presentation_body"
if rg -n 'configure|recreate|map_async|finish_async' <<<"$direct_presentation_body"; then
  echo "browser-webgpu-only check found adapter recovery or CPU readback in backend presentation smoke" >&2
  exit 1
fi

readback_body=$(
  sed -n \
    '/pub async fn assert_explicit_readback(/,/pub async fn assert_imported_gpu_canvas(/p' \
    "$smoke_source"
)
rg -q '\.finish_async\(\)' <<<"$readback_body"
if rg -n 'create_presentation_surface|\.acquire\(|\.present\(' <<<"$readback_body"; then
  echo "browser-webgpu-only check found canvas presentation in explicit readback" >&2
  exit 1
fi

renderer_source="$repo_dir/crates/nuxie-renderer/src/lib.rs"
direct_submit_body=$(
  sed -n \
    '/async fn finish_to_texture_view_async(/,/async fn finish_internal(/p' \
    "$renderer_source"
)
rg -Fq 'finish_internal(false, false, false, false, Some' <<<"$direct_submit_body"

test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2.rs"
test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2_limits.rs"

echo "browser-webgpu-only summary: browser-smoke=pass gpu-smoke=pass prohibited-product-lifecycle=0 prohibited-cpu-presentation=0 typed-readback=1 surface-alpha=premultiplied"
