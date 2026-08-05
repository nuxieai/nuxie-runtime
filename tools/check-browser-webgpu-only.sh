#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

prohibited_pattern='webgl2|femtovg|BrowserBackendPreference|BrowserBackend|WebGl2Factory|WebGl2Frame|WebGl2GpuCanvasRenderer|fallback_reason'
prohibited_paths=(
  "$repo_dir/crates/nuxie-browser-adapter"
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

cpu_presentation_pattern='CanvasRenderingContext2d|ImageData|put_image_data|present_pixels'
if rg -n "$cpu_presentation_pattern" \
  "$repo_dir/crates/nuxie-browser-adapter/src/browser.rs" \
  "$repo_dir/crates/nuxie-renderer/src/presentation.rs" \
  "$repo_dir/crates/nuxie-renderer/Cargo.toml"; then
  echo "browser-webgpu-only check found a CPU canvas presentation path" >&2
  exit 1
fi

browser_source="$repo_dir/crates/nuxie-browser-adapter/src/browser.rs"
presentation_source="$repo_dir/crates/nuxie-renderer/src/presentation.rs"
rg -q 'WebCanvasWindowHandle' "$browser_source"
rg -q 'pub async fn present\(self\)' "$browser_source"
rg -q 'get_current_texture\(\)' "$presentation_source"
rg -q 'queue\.present\(self\.texture\)' "$presentation_source"
rg -q 'pub async fn finish_with_readback\(self\)' "$browser_source"
rg -q 'WgpuPresentationAlpha::Premultiplied => wgpu::CompositeAlphaMode::PreMultiplied' \
  "$presentation_source"

current_texture_body=$(
  sed -n \
    '/fn current_frame(&self)/,/fn acquire_current_frame(/p' \
    "$browser_source"
)
rg -Fq 'acquire_surface_texture(' <<<"$current_texture_body"
rg -Fq \
  'SurfaceRecoveryAction::ReconfigureAndRetry => self.reconfigure_surface()' \
  <<<"$current_texture_body"
rg -Fq \
  'SurfaceRecoveryAction::RecreateAndRetry => self.recreate_surface()' \
  <<<"$current_texture_body"

surface_failure_body=$(
  sed -n '/fn surface_failure(/,$p' "$browser_source"
)
rg -Fq \
  'WgpuPresentationAcquireError::Outdated => SurfaceAcquisitionFailure::Outdated' \
  <<<"$surface_failure_body"
rg -Fq \
  'WgpuPresentationAcquireError::Lost => SurfaceAcquisitionFailure::Lost' \
  <<<"$surface_failure_body"

recreate_surface_body=$(
  sed -n \
    '/fn recreate_surface(&self)/,/struct CanvasSurfaceTarget/p' \
    "$browser_source"
)
rg -Fq \
  '.recreate(CanvasSurfaceTarget(self.canvas.clone()))' \
  <<<"$recreate_surface_body"

lifecycle_source="$repo_dir/crates/nuxie-browser-adapter/src/browser_surface_lifecycle.rs"
rg -q 'SurfaceRecoveryAction::ReconfigureAndRetry' "$lifecycle_source"
rg -q 'SurfaceRecoveryAction::RecreateAndRetry' "$lifecycle_source"
rg -q 'second_failure_returns_typed_error_without_a_third_acquisition' \
  "$lifecycle_source"

surface_recovery_action_body=$(
  sed -n \
    '/fn surface_recovery_action(/,/^}/p' \
    "$lifecycle_source"
)
rg -Fq \
  'SurfaceAcquisitionFailure::Outdated => Some(SurfaceRecoveryAction::ReconfigureAndRetry)' \
  <<<"$surface_recovery_action_body"
rg -Fq \
  'SurfaceAcquisitionFailure::Lost => Some(SurfaceRecoveryAction::RecreateAndRetry)' \
  <<<"$surface_recovery_action_body"

retry_body=$(
  sed -n \
    '/pub async fn assert_surface_acquisition_retry(/,/pub async fn assert_persistent_surface_acquisition_failure(/p' \
    "$repo_dir/tools/browser-renderer-smoke/src/lib.rs"
)
if rg -n '\.resize\(' <<<"$retry_body"; then
  echo "browser-webgpu-only check found an external resize masquerading as surface recovery" >&2
  exit 1
fi

present_body=$(
  sed -n \
    '/pub async fn present(self)/,/pub async fn finish_with_readback(self)/p' \
    "$browser_source"
)
if rg -n 'finish_async|map_async|put_image_data' <<<"$present_body"; then
  echo "browser-webgpu-only check found GPU-to-CPU work in ordinary presentation" >&2
  exit 1
fi
readback_body=$(
  sed -n \
    '/pub async fn finish_with_readback(self)/,/struct BrowserPresentation/p' \
    "$browser_source"
)
if rg -n 'current_frame|\.present\(' <<<"$readback_body"; then
  echo "browser-webgpu-only check found canvas presentation in explicit readback" >&2
  exit 1
fi
rg -q 'let pixels = inner\.finish_async\(\)\.await\?' <<<"$readback_body"

renderer_source="$repo_dir/crates/nuxie-renderer/src/lib.rs"
direct_submit_body=$(
  sed -n \
    '/async fn finish_to_texture_view_async(/,/async fn finish_internal(/p' \
    "$renderer_source"
)
rg -Fq 'finish_internal(false, false, true, false, Some' <<<"$direct_submit_body"

test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2.rs"
test ! -e "$repo_dir/crates/nuxie-renderer/src/webgl2_limits.rs"

echo "browser-webgpu-only summary: browser-smoke=pass gpu-smoke=pass prohibited-surface=0 prohibited-cpu-presentation=0 typed-readback=1 surface-alpha=premultiplied recovered-surface-alpha=premultiplied surface-recovery=bounded"
