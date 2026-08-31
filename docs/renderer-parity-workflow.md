# Renderer parity validation

Renderer maintenance follows the mechanical translation and two-pass review
method in [PARITY_WORKFLOW.md](PARITY_WORKFLOW.md). The legacy Rust-WGPU
implementation and its offline Apple MSL catalog have been removed; neither is
a current oracle or fallback.

Use the pinned C++ implementation of the same backend as the behavioral
authority. Match input stream, dimensions, clear color, mode, device, and
toolchain configuration. Keep executable/source identity with comparison
results. A different backend's pixels or a faster benchmark do not justify
changing upstream semantics.

The live validation entry points are maintained with their harnesses:

- The repository Makefile defines runtime Golden, scripted Golden, Silver,
  renderer replay, and native platform validation commands.
- `.github/workflows/ci.yml` defines the browser source-oracle and platform
  parity runs. Their retained scripts live in `tools/backend-port/`; the
  directory name does not make them obsolete campaign bookkeeping.
- `tools/renderer-tracers/` retains native Metal corpus inputs used by validation.
- [Renderer exactness metrics](renderer-exactness-map.md) distinguishes
  contract-exact results from byte-exact results.
- [Browser packaging](browser-renderer-wasm-packaging.md) records the explicit
  WebGPU/WebGL2 product boundary.
- The [Apple](nux-capi-apple-release.md) and
  [Android](nux-capi-android-release.md) release contracts govern device artifacts.

Run the applicable existing harnesses after integration. A failure must be
traced to the upstream/Rust owner or approved platform adaptation; do not widen
tolerances or restore legacy rendering to make it pass. Preserve the fixture
corpus and report unavailable device coverage honestly.

Historical WGPU timing claims, capture recipes, and MSL-catalog commands are
available in Git history. They do not establish parity for the current native
renderers.
