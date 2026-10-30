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

The `6285c6fe` translation merges path and paint GPU records. Its shader
inputs/products were regenerated on the pool in job
`d90790e3-0f57-4ac8-8936-2505428eb98f`: 66 WGSL modules, 96 Vulkan
SPIR-V headers, and corresponding minified GLSL inputs and exports. The
artifact's `provenance.json` records the exact source manifest, generator,
overlay, and compiler hashes at `6285c6fee237e464e21a19e75017501691cc3472`.
The sampled-clip WebGPU overlay follows this batch's binding-set alias `M3`.
This generation does not establish renderer device parity or produce Apple
metallibs. Historical native Metal captures and earlier checks retain their
original identity; production Metal compilation uses the translated source owners.
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

## Planets local-path GPU comparison

`tools/silver-corpus/examples/planets_gpu_capture.rs` captures the complete
164-frame planets replay with production CSS unchanged. Its three SRIV files
contain the ordinary cached paths, forced fresh local paths for the nine owners
identified in the Silver audit, and the pinned producer's local payloads for
those same nine resources. The third control preserves every other Rust command,
including CSS transforms and paints; it is not a complete C++ rendering oracle.
The capture verifies actual recomposition before every use and exact consumer
ordering, including multiple paints and clips.

Use the existing pool frontend with new, empty output directories:

```sh
RIVE_RUNTIME_DIR=/absolute/path/to/pinned/rive-runtime \
  python3 tools/bazel/runtime.py run -p silver-corpus \
  --example planets_gpu_capture -- /absolute/path/to/captures

python3 tools/bazel/runtime.py run -p renderer-replay \
  --features native-metal,native-ore-metal --bin native-metal-sriv-compare -- \
  --cached /absolute/path/to/captures/cached.sriv \
  --fresh /absolute/path/to/captures/fresh.sriv \
  --comparison cache-rebuild --mode raster-ordering \
  --output /absolute/path/to/cache-gpu-result
```

Repeat with `upstream-local.sriv` and `--comparison upstream-local`, using a
separate output directory. `--mode clockwise-atomic` selects the other native
Metal mode. The frontend declares the input/output closure and routes GPU runs
to the macOS pool. Do not enable extra diagnostic environment output modes for
these commands; their output paths are not part of this contract.

The comparator keeps SRIV resources alive across frames, flushes the last frame
before destroying them, checks the actual device and mode, and compares RGBA
pixels exactly. It requires nonblank frames with physical draws and a visible
positive control. Its report records input hashes, device identity, per-frame
results and the comparison's scope. Frame-zero samples and the first mismatch
are saved; temporary matched-frame PNGs are removed as comparison progresses.
Nonempty output directories are rejected to prevent stale results from being
mistaken for a completed run. This is a focused cache/geometry experiment,
not a waiver for the rest of the planets Silver divergence.

For a repeatability diagnostic, supply byte-identical inputs and
`--comparison repeatability`. This mode requires identical bytes; the geometry
comparison modes still reject them. It preserves exact pixel comparison and the
visible positive control. Use `--frames N` only with an input containing exactly
N nonempty frames.

The tool explicitly opts into upstream's infinite triangulation budget on both
lanes and the positive control, and records `triangulation_policy` in the report.
This retains geometric eligibility checks but removes timing-based admission
and threshold adjustment. Production factories retain their adaptive budget.
Use this fixed policy for geometry comparisons: adaptive timing can select
rendering paths differently even for byte-identical replay inputs.
