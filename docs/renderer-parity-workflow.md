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

## Portable shader generation on the pool

The `d278866b` source checkpoint updates runtime/source-oracle and shader
source/archive/tool pins. Translation and both independent reviews, including
correction rereviews, are complete. Pool generation succeeded in job
`f4940aed-22d9-48b5-a07e-70d5fae11a23`; the imported 66 WGSL files, 96 SPIR-V
headers, 24 embedded GL strings and 54 WebGPU GLSL/export files carry actual
`d278866b` provenance. Historical Metal captures remain unchanged. Linux/WSL
pool WebGL2 library validation passed 574 tests with zero failures and six
ignored; Vulkan and Metal consumer checks passed. The actual wasm32 pool check
failed before renderer compilation on the previously observed `rules_rust`
CopyFile remote-only execution-strategy error. Browser and ANGLE/EGL 4x-MSAA
pixels remain unqualified. The sync map records job identities, the reviewed
constructor correction, unavailable coverage and unchanged Metal assertions.
The pool target selects its pinned source archive. Standalone shader checks and
importers must use the clean source checkout matching their current explicit pin
in `RIVE_RUNTIME_DIR`; do not relabel older compiled products as newly generated.
Historical artifacts and captures retain their original provenance.

Submit `//tools/shader-generation:renderer-shaders` with `nuxie-bazel
//tools/shader-generation:renderer-shaders --platform linux --directory
<artifact-directory>`. This builds the pinned Naga, glslang and SPIR-V tools on
the pool; it does not add Naga to the runtime dependency graph. Do not substitute
locally installed shader compilers for this target.

The output tree contains `wgsl/`, `upstream/` (GLSL, minified headers and SPIR-V),
and `provenance.json`. It does not produce Apple metallibs or establish device
rendering parity. Import and validate generated products through the existing
backend tooling before updating retained artifacts.

When advancing the shader input revision, update the immutable archive revision
and checksum in `bazel/shader-sources.bzl`, the complete per-file input hashes in
`bazel/shader-source-manifest.json`, and the existing generator's revision pin
together. The action rejects missing, extra or changed inputs. Keep compiler
versions/checksums pinned in `bazel/shader-tools.bzl`; changing a toolchain is a
separate deliberate change, not a workaround for a generation failure.
