# Bug: artboard content lost after the first draw in the Vulkan product backend

## Reproducer (1s, on this branch android/vulkan-capi)

export RUSTC=$(rustup which rustc --toolchain stable)
export NUXIE_MOLTENVK_LIBRARY=/Users/levi/dev/oss/rive-runtime/renderer/dependencies/MoltenVK/Package/Release/MoltenVK/dynamic/dylib/macOS/libMoltenVK.dylib
NUX_PROBE_RIV=/Users/levi/dev/nuxie-dev/.claude/worktrees/composer-options-placement-dd7d14/sdks/nuxie-android/nuxie-android/src/androidTest/assets/data_binding_test.riv \
  cargo test -p nux-capi --features android-vulkan,scripting --test android_vulkan_content_probe

Currently FAILS: the rendered frame contains exactly 2 colors (clear +
artboard background). Expected: the fixture's panel, rows, and text
(reference render: the same riv through the old wgpu renderer showed a
gray panel, white text rows, toggle squares).

## Facts established (do not re-derive)

1. All 14 artboard draw_path calls reach RiveRenderer::drawPath with
   valid RiveRenderPathHandle/RiveRenderPaintHandle downcasts (temporary
   NUX_PROBE_TRACE eprintlns are in exact_source_adapter.rs draw_path/
   clip_path - keep or extend while debugging, REMOVE before finishing).
   Zero clip_path calls. Only the FIRST visible result (the artboard
   background rectangle) appears; the other 13 draws are invisible.
2. Direct Renderer-trait probes through NativeVulkanFactory/Frame render
   correctly, including multi-draw with save/transform/restore + clip
   (tests native_vulkan::tests::direct_rect_draw_probe and
   artboard_like_draw_probe on this branch - keep them, they are sound).
3. FIT transform is irrelevant (FIT_NONE reproduces identically; the
   probe file currently has FIT_NONE - restore FIT_CONTAIN_CENTER when
   fixing the test's final form).
4. The same exact_source_adapter + artboard machinery drives the browser
   WebGPU path (editor product) where artboard content renders fine, so
   the adapter and runtime artboard draw path are proven; the loss is in
   crates/nuxie-renderer/src/mechanical_port/vulkan/ (VulkanProductBackend)
   - likely draw accumulation/flush/pipeline state after the first draw
   within one frame, or paint/path state resolution for retained objects.
5. The same riv renders fully through the retired wgpu path and iOS's
   native Metal arm, so the fixture is good.

## Task

Root-cause and fix the Vulkan backend so the probe passes with >= 4
distinct colors and visually correct content. Compare against the
native replay path (tools/renderer-replay renders recorded streams
through the same backend - if replay streams also drop draws 2..n when
driven with multiple draw_path calls in one frame, that is the bug
locus; if replay renders multi-draw frames fine, diff what the replay
drive does differently from artboard draws). Write/extend a regression
test at the nuxie-renderer level that draws >= 3 overlapping solid
paths in one frame and asserts each lands (the existing probes pass -
find the delta the artboard triggers, e.g. paint reuse across draws,
identical paint object drawn twice, path reuse, blend modes, opacity
modulation, or feather/atlas resources).

Acceptance:
- The content probe passes (restore FIT_CONTAIN_CENTER + keep the
  >= 4 colors assertion).
- cargo test -p nuxie-renderer --features renderer-vulkan --lib green.
- cargo ndk -t arm64-v8a -t x86_64 build --release -p nux-capi
  --features android-vulkan,scripting green.
- Remove the temporary NUX_PROBE_TRACE eprintlns.
- Commit in logical commits on this branch; do not push.
