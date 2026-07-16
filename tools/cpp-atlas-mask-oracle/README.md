# C++ WebGPU Atlas-Mask Oracle

This harness produces a deterministic readback of the C++ renderer's WebGPU
`R16Float` feather atlas. It temporarily injects a single C++ executable into
`RIVE_RUNTIME_DIR`, applies `runtime.patch`, builds the exact `--with-dawn`
renderer configuration, then reverses the patch and removes only the injected
source directory.

## Parallel MSAA reference capture

The generic `msaa-reference` mode replays the strict cases in
`msaa-reference-corpus.toml` through C++ Dawn WebGPU on Metal. Build the
shared binary once, then run the independent captures with bounded
parallelism:

GM rows replay their single terminal frame. RIV rows pin `profile`, `frame`,
the stable source suffix, `artboard`, and `sample_seconds`; the compiler
retains declarations from prior frames but executes only the selected frame's
draw commands on a fresh renderer. `inventory_msaa_references.py
--sync-manifest` deterministically appends newly supported gated rows in corpus
order.

`--build-only` skips only the fixture-completeness test needed to bootstrap a
new registry. Full preflight and the normal format suite still require every
manifest PNG and provenance record to validate.

```sh
tools/cpp-atlas-mask-oracle/build.sh --build-only
cargo build -p pixel-compare --bin riveabl-to-png
python3 tools/cpp-atlas-mask-oracle/generate_msaa_reference_registry.py \
  --manifest tools/cpp-atlas-mask-oracle/msaa-reference-corpus.toml \
  --repo-root . \
  --runtime-revision 7c778d13c5d903b3b74eec1dd6bb68a811dea5f2 \
  --dawn-revision 211333b2e3e429c3508f25c81c547f602adf448c \
  --output target/generated_msaa_reference_registry.inc
python3 tools/cpp-atlas-mask-oracle/capture_msaa_references.py \
  --binary /Users/levi/dev/oss/rive-runtime/renderer/out/cpp-atlas-mask-oracle/rive_atlas_mask_oracle \
  --converter target/debug/riveabl-to-png \
  --manifest tools/cpp-atlas-mask-oracle/msaa-reference-corpus.toml \
  --output-dir tools/cpp-atlas-mask-oracle/out/msaa-reference \
  --repo-root . --jobs 4 --case-timeout-seconds 120 \
  --runtime-revision 7c778d13c5d903b3b74eec1dd6bb68a811dea5f2 \
  --dawn-revision 211333b2e3e429c3508f25c81c547f602adf448c \
  --registry-sha256 "$(python3 tools/cpp-atlas-mask-oracle/generate_msaa_reference_registry.py \
    --manifest tools/cpp-atlas-mask-oracle/msaa-reference-corpus.toml \
    --repo-root . \
    --runtime-revision 7c778d13c5d903b3b74eec1dd6bb68a811dea5f2 \
    --dawn-revision 211333b2e3e429c3508f25c81c547f602adf448c \
    --print-registry-sha256)"
```

The coordinator validates every stream digest, RIVEABL payload, PNG, and
provenance record before atomically installing the output directory. Results
print in manifest order even when captures finish out of order. A failed wave
is retained under an isolated `.failed-*` directory and never replaces a
completed reference set. Use `RENDERER_JOBS=4 make renderer-golden` to apply
the same bounded parallelism to local Rust corpus verification; the default
remains one GPU process.

The exporter draws five coordinated fixtures:

* render target: `64 x 64`
* stroke fixture: closed square `(16,16) -> (48,16) -> (48,48) -> (16,48)`,
  thickness `8`, miter join, butt cap, feather `20`
* circle-fill fixture: clockwise four-cubic circle bounded by `(16,16)..(48,48)`,
  feather `20`; this exercises C++'s uniform-tangent-rotation softening pass
* cusp-fill fixture: clockwise cubic from `(16,48)` to `(48,48)` with controls
  `(51.2,16)` and `(12.8,16)`, feather `20`; this exercises convex/cusp
  preparation and the short-line cusp crossing
* empty-stroke fixture: a move-only path centered at `(32,32)`, thickness `8`,
  miter join, round cap, feather `20`; this isolates synthetic cap coverage
* frame: 4x MSAA, which selects atlas feather rendering
* atlas contract: `39 x 39` logical content at `(2,2)`, in the complete
  `48 x 48` physical allocation produced by C++'s 125% resource growth

The harness emits a mask, tessellation input, and final blit for each fixture.
The masks (`atlas-mask.r16f`, `atlas-fill-mask.r16f`,
`atlas-cusp-mask.r16f`, and `atlas-empty-stroke-mask.r16f`) use the exact
`RIVEMSK` version 1
Rust interchange format: a 20-byte
little-endian header (`magic`, `version`, `width`, `height`) followed by a
canonical, tightly row-packed `R16Float` payload. WebGPU's 256-byte copy rows
are stripped during export. The complete physical C++ atlas, including its
cleared unused tail, must be exactly `48 x 48`, making the canonical file
exactly `4628` bytes. The exporter validates the frame, logical allocation,
placement, and physical allocation, then fails on drift without cropping,
padding, or normalization.

`atlas-inputs.bin`, `atlas-fill-inputs.bin`, `atlas-cusp-inputs.bin`, and
`atlas-empty-stroke-inputs.bin` use the `RIVEATI` version 1 contract. Their
40-byte
little-endian header records the atlas batch range, contour count, and
tessellation dimensions, followed by canonical 16-byte contour records and
the complete tightly packed `RGBA32Uint` tessellation texture. All artifacts
come from the same submitted C++ frame.

The empty-stroke input pins one contour with `basePatch=1` and `patchCount=5`.
Its mask pins the nonzero inner coverage at the center of the synthetic round
cap.
`atlas-empty-stroke-overlap-blit.rgba` adds an opaque marker before the stroke
and pins the cap's scheduled MSAA depth against that earlier draw.

The two `atlas-large-feather-*` fixture families reproduce the strongest
residual contours from `gm-feather_cusp-msaa` and
`gm-feather_shapes-msaa` at paint feather `403.428802` and frame scale
`1.46300006`. Each family emits the complete `RIVEATI` tessellation input,
physical `RIVEMSK` atlas, and `RIVEABL` 1756-by-2048 final frame. Its
88-byte `RIVEATP` version 1 placement record contains the frame size, clipped
pixel bounds, atlas origin, logical and physical extents, raw scale and
translation float bits, and scissor. The paired Rust test requires exact
placement and exact signed zero/nonzero mask topology, bounds every nonzero
R16 sample including negative near-degenerate coverage at `2^-9` (the smallest
passing power-of-two budget; `2^-10` fails), and retains the corpus `2/32`
final-pixel contract.

## Hunter X general-atomic diagnostic

The build also compiles the complete `hunter_x_demo` frame after strict linear
and radial gradient reconstruction. This is a same-device C++ Dawn diagnostic
for the retained native-Metal advanced-feather gate; Dawn exposes general
atomics rather than native clockwise atomics, so its pixels are evidence, not
a promotion reference.

```sh
tools/cpp-atlas-mask-oracle/build.sh --build-only
"${RIVE_RUNTIME_DIR:?}/renderer/out/cpp-atlas-mask-oracle/rive_atlas_mask_oracle" \
  /dev/null /dev/null /tmp/hunter-dawn.rgba atomic-hunter-x-full \
  /tmp/hunter-dawn.provenance
cargo run -q -p pixel-compare --bin riveabl-to-png -- \
  --artifact /tmp/hunter-dawn.rgba --output /tmp/hunter-dawn.png
```

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
atomic final target and is compared to Rust's matching generic-atomic draw
path. Native Metal stream replay remains the full-GM final-pixel oracle for the
clockwise-atomic mode.

`direct-cusp-coverage.bin` is the corresponding atomic PLS coverage-plane
sub-oracle. It uses `RIVEAPC` version 1: an exact 24-byte little-endian header
of `magic[8]`, `version`, `width`, `height`, and `wordCount`, followed by every
coverage `u32` encoded little-endian in native backing-buffer index order.
The direct-cusp harness reads it after rendering through a temporary
`CopySrc` capability on the production coverage backing buffer. The build
requires exactly `64 x 64 = 4096` words, so the canonical file is `16408`
bytes; a changed capacity, header, word count, order, or payload length fails
validation. The Rust comparison normalizes only C++'s untouched fixed-point
zero sentinel (`65536`) at transparent-black final pixels to Rust's raw-zero
clear representation; every drawn coverage word must then match exactly.

`direct-strokes-round-spans.bin`, `direct-strokes-round-inputs.bin`, and
`direct-strokes-round-blit.rgba` isolate draw 38 from
`fixtures/renderer/streams/gm/strokes_round.rive-stream`. The exact closed
stroke path is:

```text
moveTo(25.5016327, 70.300293)
lineTo(67.7646637, 70.300293)
cubicTo(79.4274673, 70.300293, 88.8961792, 80.9101868, 88.8961792, 89.5240784)
lineTo(88.8961792, 127.971649)
cubicTo(88.8961792, 138.581543, 79.4274673, 147.195435, 67.7646637, 147.195435)
lineTo(25.5016327, 147.195435)
cubicTo(16.0329189, 147.195435, 4.37011719, 138.581543, 4.37011719, 127.971649)
lineTo(4.37011719, 89.5240784)
cubicTo(4.37011719, 80.9101868, 16.0329189, 70.300293, 25.5016327, 70.300293)
close()
```

It preserves the stream stroke's thickness `4.5`, miter join, butt cap, and
zero feathering in a `400 x 400` frame cleared white. The `RIVEATS` version 1
artifact is the exact CPU-side `TessVertexSpan` range copied from C++'s mapped
production ring before unmap. Its 28-byte header pins `firstSpan=0`,
`spanCount=11`, and a 64-byte record stride; the payload stores every raw field
in production order. This is the strict pre-raster topology oracle.

The `RIVEATI` artifact captures post-tessellation contour and texture state.
All non-angle fields compare exactly; the backend-computed tangent angle has a
bounded `0.00035` radian allowance. The `RIVEABL` artifact is the C++ Dawn
final frame and remains a cross-backend diagnostic, not the native-Metal
corpus promotion gate. `build.sh` requires exactly one contour, patch range
`1+10`, a nonempty canonical span artifact, and a final-frame artifact of
exactly `640020` bytes.

`direct-rawtext-spans.bin`, `direct-rawtext-inputs.bin`, and
`direct-rawtext-blit.rgba` isolate draw 1 from
`fixtures/renderer/streams/gm/rawtext.rive-stream`. During each build,
`generate_rawtext_stream_path.py` validates the canonical stream record and
emits its exact 506 verbs and 1,096 f32 point literals into the temporary C++
source tree. The fixture preserves all 36 contours, clockwise fill, zero
feathering, and the stream's `400 x 335` white frame. The `RIVEATS` artifact
captures the complete `firstSpan=0`, `spanCount=438` CPU `TessVertexSpan`
range before unmap. The `RIVEATI` artifact pins patch range `1+318`, every
contour record, and the complete `2048 x 2` post-tessellation `RGBA32Uint`
texture. Their canonical sizes are `28060` and `66152` bytes, respectively;
the `400 x 335` diagnostic blit is exactly `536020` bytes. Together they
distinguish path and midpoint-fan preparation from downstream
native-Metal-versus-wgpu raster edges. The `RIVEABL` final frame is diagnostic
only; native Metal replay remains the corpus pixel oracle for clockwise-atomic
mode.

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

`direct-bug339297-inputs.bin` isolates the large single-contour fill from
`fixtures/renderer/streams/gm/bug339297.rive-stream`. It uses the stream's
exact `640 x 480` frame, `[1,0,0,1,258,10365663]` transform, non-zero fill,
and million-scale path coordinates, including its two authored zero-length
lines. This is the counter-parity case where the local single-contour fallback
emitted 200 excess patches; production C++ instead sends the contour through
the global interior triangulator.

The artifact uses `RIVED39` version 1 and the same direct-preparation layout as
the grid, flower, and bad-skin captures. Its parser requires atomic interlock,
one contour, a canonical initialize/outer/interior/resolve schedule, and a
nonempty triangle count divisible by three. The configured Rust test compares
the contour and triangle records in exact order and compares every generated
`RGBA32Uint` tessellation texel. `build.sh` emits and validates the capture on
every full oracle run.

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

`atomic-colorburn-pair.{rgba,color,coverage}` isolates draws 13-14 from
`interleavedfeather.rive-stream`. The build-time generator validates the
recorded draw and emits every path point as its exact f32 literal; the harness
preserves the stream transforms, paints, clockwise fill, transparent frame,
and the exact initialize/fill/stroke/resolve schedule. `RIVEACO` and
`RIVEAPC` version 1 use a 24-byte little-endian header followed by all
`1024 x 1024` native-order `u32` words from the atomic color and coverage
planes. The paired `RIVEABL` file contains the complete resolved RGBA8 frame.

The configured Rust comparison normalizes only C++'s untouched fixed-point
zero sentinel at transparent-black pixels, then requires every raw coverage
word to match. The packed color plane may differ at no more than three words
with max byte delta one; the resolved output may differ at no more than those
same three deswizzled coordinates with max channel delta 15. The current exact
fixture has two coupled words/pixels and max resolved delta seven.

`atomic-spotify-kids-app-icon-full.{rgba,coverage,clip,provenance}` replays the
complete pinned `.riv` stream through Dawn WebGPU on Metal. The strict RIV
profile validates the stream digest, source/artboard/scene/sample metadata,
14 draws, 6 clips, 15 transforms, 18 balanced saves/restores, 20 paths, 48
paints, and every path/paint snapshot. `RIVEABL` contains the logical
`1024 x 1436` final target. `RIVEAPC` and `RIVEACL` contain every word in the
physical `1024 x 1440` tiled coverage and clip backings. The harness requires
one 24-batch atomic flush, fixed-function color output, and no packed atomic
color backing; provenance pins all artifact, schedule, replay, stream,
runtime, Dawn, and adapter identities.

Rust partitions the same stream into one generic and one clockwise atomic run,
so raw coverage words retain different path IDs and stale per-run state and
are not compared as a cross-schedule format. Both implementations must keep
the padded coverage rows untouched, the final clip backing must match exactly,
and final alpha participates in the unchanged `2/32` full-frame comparison.
The configured test then shows both WebGPU implementations fail that unchanged
contract against the same native-Metal PNG with nearly identical residual
masks. This supports isolating the fixed-function backend color-output boundary
from the packed-color intermediate-precision family without claiming a
specific hardware blending mechanism.

`msaa-intersection-groups` is a schedule-only runtime assertion; it emits no
artifact. It authors three non-feathered MSAA fills with distinct captured
`DrawContents` identities: opaque clockwise draw 0, translucent non-zero draw
1, and translucent clockwise draw 2. Draw 0 overlaps draw 1; draw 2 is disjoint
from both. Draw 0 is the opaque MSAA fast path
(`prepassCount=3`, `subpassCount=0`), so the production scheduler must reserve
three intersection-board layers via `max(prepassCount, subpassCount)`. Draws 1
and 2 are both positive-key three-pass fast fills, so prepass polarity cannot
order them. The exact positive schedule is draw 2's types `8,9,10` followed by
draw 1's types `8,9,10`. In particular, draw 2's group-3 reset/type `10`
precedes draw 1's lower borrowed-pass/type `8`. Because draw-group bits outrank
draw-type bits, type `8` would sort first if draw 1 incorrectly started at
group 3. The asserted order therefore proves that draw 0 reserved all three
layers and forced overlapping draw 1 to start at group 4, while disjoint draw
2 started at group 1. The proof does not depend on final pixels. The current
oracle hook does not expose logical draw-group indices or barrier placement,
so it cannot separately pin the unextended
WebGPU advanced-destination-copy collapse-to-one-layer rule without an
invasive runtime hook; the existing `advanced-blend` assertion remains limited
to its one destination-reading MSAA atlas batch.

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
RIVE_CPP_ATLAS_LARGE_FEATHER_CUSP_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-cusp-mask.r16f" \
RIVE_CPP_ATLAS_LARGE_FEATHER_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-cusp-inputs.bin" \
RIVE_CPP_ATLAS_LARGE_FEATHER_CUSP_PLACEMENT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-cusp-placement.bin" \
RIVE_CPP_ATLAS_LARGE_FEATHER_CUSP_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-cusp-blit.rgba" \
RIVE_CPP_ATLAS_LARGE_FEATHER_SHAPES_CUSP_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-shapes-cusp-mask.r16f" \
RIVE_CPP_ATLAS_LARGE_FEATHER_SHAPES_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-shapes-cusp-inputs.bin" \
RIVE_CPP_ATLAS_LARGE_FEATHER_SHAPES_CUSP_PLACEMENT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-shapes-cusp-placement.bin" \
RIVE_CPP_ATLAS_LARGE_FEATHER_SHAPES_CUSP_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-large-feather-shapes-cusp-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_large_radius_feather_atlas_stages_match_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_SOFTENED_CUSP="$PWD/tools/cpp-atlas-mask-oracle/out/softened-cusp.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_softened_cusp_path_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_cusp_input_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
python3 tools/cpp-atlas-mask-oracle/format_test.py \
  --validate-direct-cusp-coverage \
  "$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-coverage.bin"
RIVE_CPP_DIRECT_CUSP_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-blit.rgba" \
RIVE_CPP_DIRECT_CUSP_COVERAGE="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-coverage.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_cusp_atomic_coverage_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_CUSP_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-blit.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_cusp_blit_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_POLYSHARK_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-polyshark-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_polyshark_input_oracle_matches_rust_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-strokes-round-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_strokes_round_tessellation_matches_bounded_tangent_angles \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_STROKES_ROUND_SPANS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-strokes-round-spans.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_direct_strokes_round_cpu_spans_match_rust_record_for_record \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_RAWTEXT_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-rawtext-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_direct_rawtext_tessellation_matches_rust \
  -- --exact --ignored --nocapture
RIVE_CPP_DIRECT_RAWTEXT_SPANS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-rawtext-spans.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_direct_rawtext_cpu_spans_match_rust_record_for_record \
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
RIVE_CPP_ATOMIC_COLORBURN_PAIR_COLOR="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-colorburn-pair.color" \
RIVE_CPP_ATOMIC_COLORBURN_PAIR_COVERAGE="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-colorburn-pair.coverage" \
RIVE_CPP_ATOMIC_COLORBURN_PAIR_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-colorburn-pair.rgba" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atomic_colorburn_pair_has_only_coupled_quantization_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATOMIC_SPOTIFY_FULL="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-spotify-kids-app-icon-full.rgba" \
RIVE_CPP_ATOMIC_SPOTIFY_COVERAGE="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-spotify-kids-app-icon-full.coverage" \
RIVE_CPP_ATOMIC_SPOTIFY_CLIP="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-spotify-kids-app-icon-full.clip" \
RIVE_CPP_ATOMIC_SPOTIFY_PROVENANCE="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-spotify-kids-app-icon-full.provenance" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atomic_spotify_kids_app_icon_is_fixed_color_backend_residual \
  -- --exact --ignored --nocapture
```

The configured comparator is ignored by ordinary test suites and requires a
nonempty absolute `RIVE_CPP_ATLAS_MASK`, `RIVE_CPP_ATLAS_INPUTS`,
`RIVE_CPP_ATLAS_FILL_MASK`, `RIVE_CPP_ATLAS_FILL_INPUTS`, or
`RIVE_CPP_ATLAS_CUSP_MASK`, `RIVE_CPP_ATLAS_CUSP_INPUTS`, or
`RIVE_CPP_SOFTENED_CUSP`, or
`RIVE_CPP_DIRECT_CUSP_INPUTS`, `RIVE_CPP_DIRECT_POLYSHARK_INPUTS`, or
`RIVE_CPP_DIRECT_BAD_SKIN_INPUTS`, `RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS`,
`RIVE_CPP_DIRECT_STROKES_ROUND_SPANS`, `RIVE_CPP_DIRECT_RAWTEXT_INPUTS`,
`RIVE_CPP_DIRECT_RAWTEXT_SPANS`, `RIVE_CPP_ATLAS_BLIT`, or
`RIVE_CPP_ATLAS_CLIPPED_BLIT`, `RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT`,
`RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT`, or
`RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT`, or
`RIVE_CPP_ATOMIC_ADVANCED_BLEND`, or the three
`RIVE_CPP_ATOMIC_COLORBURN_PAIR_{COLOR,COVERAGE,BLIT}` paths, or the four
`RIVE_CPP_ATOMIC_SPOTIFY_{FULL,COVERAGE,CLIP,PROVENANCE}` paths;
invoking a configured test without its variable is an error.

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
