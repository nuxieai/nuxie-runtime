# C++ WebGPU Atlas-Mask Oracle

This harness produces a deterministic readback of the C++ renderer's WebGPU
`R16Float` feather atlas. It temporarily injects a single C++ executable into
`RIVE_RUNTIME_DIR`, applies `runtime.patch`, builds the exact `--with-dawn`
renderer configuration, then reverses the patch and removes only the injected
source directory.

The exporter draws four coordinated fixtures:

* render target: `64 x 64`
* stroke fixture: closed square `(16,16) -> (48,16) -> (48,48) -> (16,48)`,
  thickness `8`, miter join, butt cap, feather `20`
* circle-fill fixture: clockwise four-cubic circle bounded by `(16,16)..(48,48)`,
  feather `20`; this exercises C++'s uniform-tangent-rotation softening pass
* cusp-fill fixture: clockwise cubic from `(16,48)` to `(48,48)` with controls
  `(51.2,16)` and `(12.8,16)`, feather `20`; this exercises convex/cusp
  preparation and the short-line cusp crossing
* frame: 4x MSAA, which selects atlas feather rendering
* atlas contract: `39 x 39` logical content at `(2,2)`, in the complete
  `48 x 48` physical allocation produced by C++'s 125% resource growth

The harness emits a mask, tessellation input, and final blit for each fixture.
The masks (`atlas-mask.r16f`, `atlas-fill-mask.r16f`, and
`atlas-cusp-mask.r16f`) use the exact `RIVEMSK` version 1
Rust interchange format: a 20-byte
little-endian header (`magic`, `version`, `width`, `height`) followed by a
canonical, tightly row-packed `R16Float` payload. WebGPU's 256-byte copy rows
are stripped during export. The complete physical C++ atlas, including its
cleared unused tail, must be exactly `48 x 48`, making the canonical file
exactly `4628` bytes. The exporter validates the frame, logical allocation,
placement, and physical allocation, then fails on drift without cropping,
padding, or normalization.

`atlas-inputs.bin`, `atlas-fill-inputs.bin`, and `atlas-cusp-inputs.bin` use
the `RIVEATI` version 1 contract. Their 40-byte
little-endian header records the atlas batch range, contour count, and
tessellation dimensions, followed by canonical 16-byte contour records and
the complete tightly packed `RGBA32Uint` tessellation texture. All artifacts
come from the same submitted C++ frame.

`softened-cusp.bin` uses the `RIVESFT` version 1 contract: verb and point
counts followed by canonical C++ `PathVerb` values and raw XY float bits. It
captures the dedicated cusp source after C++ fill softening at feather `1` and
matrix scale `1.46300006`, before direct tessellation.

`direct-cusp-inputs.bin` uses the same `RIVEATI` contract for the severe
`feather_cusp` cell: duplicate moves, controls `(133.635864,0)` and
`(-33.6358566,0)`, paint feather `1`, and matrix scale `1.46300006`. It
captures both contour records and the complete double-sided tessellation
texture after path softening and before coverage. Dawn WebGPU advertises
general atomics but not `clockwiseAtomic`, so this artifact is an
intermediate-geometry oracle only. `direct-cusp-blit.rgba` retains the general
atomic final target for diagnosis; it is not compared to Rust's
clockwise-atomic output. Native Metal stream replay remains the final-pixel
oracle for that mode.

`direct-polyshark-inputs.bin` uses the same contract for row 0, shark cell
(stream lines 14 and 28) of `feather_polyshapes`. During each build,
`generate_polyshark_stream_path.py` validates that canonical stream record and
emits its exact 315 f32 point literals into the temporary C++ source tree.
The harness therefore uses the serialized `RawPath` sequence, including its
sign-sensitive feather joins, rather than independently reproducing
`FeatherPolyShapesGM` polygonization. The stream's `1.46300006` top-level
scale remains the direct-render transform. It captures every emitted contour
record and the complete `RGBA32Uint` tessellation texture before coverage.
The configured Rust comparator treats LEFT/RIGHT as equivalent only when both
records are otherwise-identical feather joins. Dawn WebGPU and wgpu classify
that side oppositely for this polygon, while native Metal and wgpu produce
isolated final pixels with no channel delta beyond 2; all other packed flags
remain strict.

`direct-grid-inputs.bin` is the bounded `direct-grid` atomic preparation
oracle. It reproduces the 100 contours in
`fixtures/renderer/streams/gm/largeclippedpath_clockwise_nested.rive-stream`
line 10: 50 horizontal then 50 vertical 20px strips, with alternating winding.
It uses a `1000 x 1000` frame, zero feathering to select production interior
triangulation, and `clockwiseFillOverride=true`; the mode rejects any result
that is not atomic, lacks exactly 100 contours, or lacks interior triangle
records.

`direct-grid-inputs.bin` uses the `RIVEDGI` version 1 little-endian format.
Its 64-byte header is `magic[8]`, then fourteen `u32` values: `version=1`,
`headerBytes=64`, `flags=1` (`clockwiseFillOverride`), `interlockMode=1`
(production `InterlockMode::atomics`),
`drawBatchCount`, `tessWidth`, `tessHeight`, `contourCount=100`,
`triangleVertexCount`, `drawBatchStride=20`, `contourStride=16`,
`triangleVertexStride=12`, `tessTexelStride=16`, and `reserved=0`. The payload
is exactly: draw-schedule records (`drawType`, `shaderFeatures`,
`shaderMiscFlags`, `baseElement`, `elementCount`; five `u32`), contour records
(`x` and `y` raw float bits, `pathID`, `vertexIndex0`; four `u32`), interior
triangle records (`x` and `y` raw float bits, packed signed-weight/unsigned
path-ID word; three `u32`), and the complete row-packed `RGBA32Uint`
tessellation texture. The temporary runtime patch snapshots each interior
`TriangleVertex` from its still-mapped CPU production buffer after
triangulation and before `unmapResourceBuffers()` transfers it to the backend.
No WebGPU row padding, normalization, or omitted records are permitted.

The schedule is exactly four records in production `DrawType` order:
`renderPassInitialize=15`, `outerCurvePatches=2`,
`interiorTriangulation=3`, and `renderPassResolve=16`. The outer-cubic record
must have a nonzero, non-overflowing `baseElement + elementCount` range whose
17-vertex production patch spans fit in the tessellation texture, and the
interior record's `elementCount` must equal `triangleVertexCount`. `build.sh`
parses the generated artifact with these rules before reporting success.

`direct-flower-inputs.bin` is the bounded `direct-flower` preparation oracle
for line 7 of
`fixtures/renderer/streams/gm/largeclippedpath_clockwise_nested.rive-stream`.
It reproduces the exact first clip path: one 9-cubic flower contour followed by
its inner 4-cubic oval contour. Like `direct-grid`, it uses a `1000 x 1000`
frame, zero feathering, `clockwiseFillOverride=true`, production atomic
interlock, and the same pre-backend contour and `TriangleVertex` capture hooks.
It isolates the global-triangulation inputs around the remaining oval-boundary
pixel delta without replaying the second 100-contour grid clip.

The flower artifact uses the separate `RIVEDFI` version 1 little-endian magic
and otherwise has the same 64-byte header, record strides, payload order, and
canonical four-draw schedule as `RIVEDGI`. Its parser requires exactly 2
contours, a nonempty triangle count divisible by 3, a coherent outer-cubic
range, and an interior draw `elementCount` equal to `triangleVertexCount`.
`build.sh` emits and validates both direct artifacts independently.

`direct-bad-skin-inputs.bin` is the bounded `direct-bad-skin` preparation
oracle for the hair draw at lines 327-330 of
`fixtures/renderer/streams/riv/bad_skin.rive-stream`. It reproduces the
stream's single 13-cubic contour, transform
`[1.00501573,0.116219193,-0.11621917,1.00501561,550.433167,361.510925]`, and
`999 x 720` frame. It preserves the stream-authored `fillRule=0` (non-zero)
while the frame has `clockwiseFillOverride=true`, with zero feathering so
production atomic interior preparation is selected. It rejects a non-atomic
capture, anything but exactly one contour, or any draw schedule other than
initialize, outer cubics, interior triangulation, resolve.

The artifact uses the distinct `RIVEDBI` version 1 little-endian magic and the
same 64-byte header, canonical four-draw schedule, contour and
`TriangleVertex` records, and complete row-packed `RGBA32Uint` tessellation
texture as the other direct preparation artifacts. The configured Rust test
reconstructs the exact `RawPath` and transform, runs
`build_interior_tessellation`, then compares contour records, canonical
triangle records, dimensions, and every texture texel without tolerances.
`build.sh` writes and validates it independently.

`atlas-blit.rgba`, `atlas-clipped-blit.rgba`,
`atlas-path-clipped-blit.rgba`, `atlas-changing-path-clipped-blit.rgba`,
`atlas-nested-path-clipped-blit.rgba`,
`atlas-nested-evenodd-path-clipped-blit.rgba`,
`atlas-nested-clockwise-path-clipped-blit.rgba`,
`atlas-advanced-blend-blit.rgba`,
`atomic-advanced-blend.rgba`, and `atlas-fill-blit.rgba`
use the `RIVEABL` version 1 contract: a 20-byte
little-endian header (`magic`, `version`, `width`, `height`) followed by the
complete tightly packed `64 x 64` RGBA8 render target. Since the paired input
and mask oracles already prove the atlas contents, this artifact isolates the
final atlas sampling, paint application, and output path.

`atlas-advanced-blend-blit.rgba` uses clear color `0xff204080` and a clockwise
square fill with color `0xc0e08040`, feather `20`, and `ColorDodge`. The
exporter requires one MSAA `atlasBlit` batch with
`ENABLE_ADVANCED_BLEND | ENABLE_DITHER`, no fixed-function color-output flag,
and `DrawContents::advancedBlend`. On unextended WebGPU this forces the C++
renderer to end and resolve the MSAA pass, copy the draw's destination bounds
to its single-sample destination texture, restart with MSAA attachments loaded,
and finish RGB in the generated destination-reading shader while hardware
src-over blending finishes alpha.

`atomic-advanced-blend.rgba` uses the same clear, path, paint, and blend mode
with direct atomic feather rendering. It requires the exact initialize,
`midpointFanCenterAAPatches`, resolve schedule, atomic interlock, and shader
color output. This isolates the generated destination-reading atomic path
shader with `ENABLE_ADVANCED_BLEND | ENABLE_FEATHER | ENABLE_DITHER`.
The configured Dawn-versus-wgpu comparator allows only the observed backend
quantization envelope: at most eight pixels may differ, and no channel may
differ by more than one. Either cap is independently enforced by unit tests.

```sh
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh --preflight
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh
python3 tools/cpp-atlas-mask-oracle/format_test.py
RIVE_CPP_ATLAS_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-mask.r16f" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atlas_mask_oracle_matches_fixed_rust_mask_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atlas_input_oracle_matches_fixed_rust_inputs_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_FILL_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-fill-mask.r16f" \
RIVE_CPP_ATLAS_FILL_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-fill-inputs.bin" \
  cargo test -p nuxie-renderer cpp_webgpu_atlas_fill_ \
  -- --ignored --nocapture
RIVE_CPP_ATLAS_CUSP_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-cusp-mask.r16f" \
RIVE_CPP_ATLAS_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-cusp-inputs.bin" \
  cargo test -p nuxie-renderer cpp_webgpu_atlas_cusp_ \
  -- --ignored --nocapture
RIVE_CPP_SOFTENED_CUSP="$PWD/tools/cpp-atlas-mask-oracle/out/softened-cusp.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_softened_cusp_path_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_cusp_input_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_POLYSHARK_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-polyshark-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_polyshark_input_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_BAD_SKIN_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-bad-skin-inputs.bin" \
  cargo test -p nuxie-renderer \
  direct_grid_oracle::tests::configured_cpp_bad_skin_preparation_matches_record_for_record \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_blit_oracle_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-path-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_path_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-changing-path-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_changing_path_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-path-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_nested_path_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-evenodd-path-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_nested_even_odd_path_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-clockwise-path-clipped-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_nested_clockwise_path_clipped_blit_matches_fixed_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-advanced-blend-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_msaa_atlas_advanced_blend_matches_rust_output_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATOMIC_ADVANCED_BLEND="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-advanced-blend.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atomic_advanced_blend_matches_within_backend_quantization_when_configured \
  -- --exact --ignored --nocapture
```

The configured comparator is ignored by ordinary test suites and requires a
nonempty absolute `RIVE_CPP_ATLAS_MASK`, `RIVE_CPP_ATLAS_INPUTS`,
`RIVE_CPP_ATLAS_FILL_MASK`, `RIVE_CPP_ATLAS_FILL_INPUTS`, or
`RIVE_CPP_ATLAS_CUSP_MASK`, `RIVE_CPP_ATLAS_CUSP_INPUTS`, or
`RIVE_CPP_SOFTENED_CUSP`, or
`RIVE_CPP_DIRECT_CUSP_INPUTS`, `RIVE_CPP_DIRECT_POLYSHARK_INPUTS`, or
`RIVE_CPP_DIRECT_BAD_SKIN_INPUTS`, `RIVE_CPP_ATLAS_BLIT`, or
`RIVE_CPP_ATLAS_CLIPPED_BLIT`, `RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT`,
`RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT`, or
`RIVE_CPP_ATOMIC_ADVANCED_BLEND` path;
invoking either test without its variable is an error.

`--preflight` proves that the temporary patch applies and reports each missing
Dawn prerequisite without building or changing the runtime checkout.
It also requires Naga exactly at version `30.0.0`, which the renderer's WGSL
shader-generation step invokes while Premake generates the isolated build
files. By default the harness uses `$HOME/.cargo/bin/naga`, matching
`tools/generate-renderer-shaders.sh`, and prepends that executable's directory
to the build `PATH`; the caller's `PATH` does not need to include Cargo's bin
directory. `RIVE_ATLAS_MASK_NAGA=/absolute/path/to/naga` selects another
executable named `naga`, still subject to the exact version check.

On macOS with Xcode 26 or later, `build.sh` temporarily changes Dawn
PartitionAlloc's `mac_no_default_new_delete_symbols` setting from
`-fvisibility-global-new-delete=force-hidden` to an empty `cflags` list.
Xcode 26's SDK libc++ declares these symbols with default visibility, so
forcing hidden visibility causes the known declaration mismatch. The patch is
checked before use, skipped when Dawn is already compatible, and reversed on
exit.

The same Xcode-26 branch temporarily appends
`treat_warnings_as_errors=false` to Dawn's generated `out/release/args.gn`.
This keeps legacy unsafe-buffer diagnostics visible but prevents the new clang
default from promoting them to build-stopping errors. An explicit user value is
never overwritten. It also sets `use_lld=false`, making Dawn emit regular
archives that the Premake executable's Apple `ld` link step can consume. Before
either temporary edit, the harness snapshots `args.gn`; its exit trap restores
that snapshot and verifies byte equality with `cmp`, including blank lines.
