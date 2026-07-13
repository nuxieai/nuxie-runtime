# GPU Semantic-Trap Audit

Date: 2026-07-13
Status: In progress

This audit is the first R3 entry gate required by
`docs/renderer-port-map.md`. It reviews semantic differences introduced by the
renderer shader path rather than re-reviewing already translated renderer
algorithms. Corpus promotion and tolerance changes remain out of scope.

## Compared Paths

- Shared WebGPU lineage: upstream GLSL to SPIR-V to WGSL through Naga 30 with
  `--keep-coordinate-space`. Rust consumes the 50 raw WGSL modules. Upstream's
  `wgsl_to_header.py` minifies and renames that WGSL into 50 C++ headers before
  Dawn consumes it. Both artifact sets are generated and pinned; their compiler
  input bytes are intentionally not identical.
- Clockwise-atomic fork: Rust generates ten additional WGSL modules because
  upstream does not emit a WebGPU clockwise-atomic family. Eight are direct
  upstream-source variants; two replace PLS input-attachment reads with
  sampled clip-texture reads and omit the corresponding no-op store.
- Native reference: C++ Metal output, used as a third signal for the
  clockwise-atomic fork and decoder/color ingress. It does not by itself
  identify which compiler or backend caused a disagreement.

The upstream runtime revision, shader inputs, generated artifacts, adapters,
and comparison artifacts must be pinned whenever an executable cross-path
oracle is used.

## Review Rubric

Every finding must record:

1. The upstream GLSL/SPIR-V source and C++ consumption path.
2. The generated WGSL and Rust pipeline/binding path.
3. The concrete semantic risk, not only a textual difference.
4. An executable oracle, validation test, or corpus entry that can fail.
5. A disposition: fixed, accepted with rationale, deferred behind a named
   gate, or rejected as unreachable.

The audit covers:

- float precision, `RelaxedPrecision`, denormals, conversion, and fused math;
- texture and sampler typing, filtering, mip selection, and address modes;
- coordinate-space, Y origin, pixel-center, derivative, and clip conventions;
- matrix packing/majorness, uniform/storage layout, alignment, and padding;
- signed/unsigned conversion, integer wrap, shifts, bitfields, and atomics;
- uniformity analysis, derivative/control-flow restrictions, and barriers;
- multisample coverage, sample masks, resolve, and load/store behavior.

## Findings

### R3-ST-01: Shader provenance drift

**Verified, fixed.** Regeneration previously accepted an unpinned upstream
checkout and whichever shader tools were on `PATH`. Parse validation therefore
did not prove source reproducibility.

`make renderer-shaders-check` now rejects dirty shader inputs and rebuilds from
runtime `7c778d13`, Naga 30.0.0, glslang 16.2.0, SPIRV-Tools 2026.1, and ply
3.11 into isolated directories. It also pins `PYTHONHASHSEED=0` because the
upstream header minifier otherwise assigns identifiers nondeterministically.
The gate byte-compares and digest-pins all 60 Rust WGSL modules plus the 50
canonical minified C++ compiler-input headers. CI runs the same gate. The clean
result is:

```text
renderer shaders reproducible: rust-modules=60 rust-digest=ad5eb6f60c30d74e34871e9dafdb2095906dc945c2bd6e765e6e107140fa2e44 cpp-headers=50 cpp-digest=0ba25987fbf839d5eedf88de61e11b0ea1b9321a242ade5d912a7f40ae000708
```

### R3-ST-02: Atomic intermediate precision

**Verified residual, named gate.** `dstreadshuffle` and `interleavedfeather`
exercise advanced blends after colors have passed through atomic/fixed-point
representations. The existing C++ Dawn oracles prove exact coverage while
bounding the small color-word differences. They do not discriminate f16,
denormals, contraction, or another backend lowering as the cause.

Disposition: retain exactly two
`metal-webgpu-atomic-intermediate-precision` gates. Do not create separate f16,
denormal, or FMA gates without a discriminating oracle, and do not change the
native references or `2/32` contract.

### R3-ST-03: Sampled clockwise-atomic clip plane

**Verified source fork, closed by production readout oracle.**
`tools/renderer-shaders/clockwise_atomic_path_webgpu.main` replaces C++ Metal
PLS input-attachment reads with
`texelFetch(..., floor(gl_FragCoord.xy), 0)` and omits a no-op PLS store. It is
reachable in nested clockwise clips.

`first-light-nested-clip-probe-clockwise-atomic` forces one large asymmetric
compound outer clip, one nested arbitrary clip update, and an opaque white
full-frame draw. A focused Rust test records the production draw kinds as
`OutermostClip`, `NestedClip`, and `ClippedContent`, then proves the complete
captured clip update equals the final probe bytes. Because the clip paths are
pixel-aligned and binary, that draw is an identity readout of every
semantically consumed clip-plane red byte: C++ Metal reads PLS and Rust reads
the sampled texture. The pinned 640x640 native Metal reference and Rust output
match at zero delta across all 409,600 pixels. The stream SHA-256 is
`efbb8df4b4c1bf877b5723154da980a28c84f064df90cbb92fef2fceed7798dc`;
the reference SHA-256 is
`f064bdf9fd879e7161123c127a39e50e4fe833379d4bb15f255785370045f4ea`.
This closes the semantic fork without exposing private backend storage or
normalizing physical RGBA8-versus-packed-u32 representations.

### R3-ST-04: Decoded-image color ingress

**Closed: production decode buffers measured.** The native FFI oracle calls the
same C++ `Bitmap::decode` and `RGBAPremul` conversion as
`RenderContext::decodeImage`, then compares that buffer directly with Rust's
pre-upload decode result. `make renderer-decoder-oracle` reproduces the check.

On macOS 26.4.1 with runtime `7c778d13`, the reachable 278x278 JPEG
(`62e087df734fa3a0f57524db98a4d5aa30a8628ede9a7d59ed67981cc71823de`)
differs at 35,652 pixels and 78,669 channel bytes, with max delta 37 and exact
alpha; 12,509 source pixels exceed the corpus channel threshold of 2. This
independently confirms a decoder-level difference on the same image, supporting
decoder attribution for the rendered image-interior delta. It does not
numerically derive the rendered 9,494-pixel/max-18 result; the existing
10,000-pixel frame cap remains its separately measured empirical contract and
was not widened.

The 319x320 ICC PNG
(`a72cd2314ae2cb861da62dfb9782e323337fc600e44200cd16bd150d7c15f2cb`)
differs at 4,950 pixels and 5,013 channel bytes, with max delta 2 and exact
alpha. Color ingress therefore already fits the corpus channel threshold; any
remaining greater-than-2 rendered samples arise after decode rather than from
an unimplemented color transform. The executable contract pins both encoded
fixture hashes, formats, dimensions, ICC presence, the runtime revision, clean
decoder sources, and a non-stale decoder archive. It permits bounded Apple
decoder drift while requiring exact alpha, JPEG max delta at most 40, and ICC
PNG max delta at most 2.

### R3-ST-05: Non-finite replay input

**Verified by the dual-renderer fuzz-replay gate.** Render-stream `f32` parsing
admits `NaN` and infinity into transform state. R3-FZ-01 replays NaN,
positive/negative infinity, and `f32::MAX` transforms under save/restore, then
draws an opaque finite control region. Rust/wgpu and pinned C++/Metal complete
under the process deadline, preserve the control region, and are exact outside
its footprint.

The broader gate also found and fixed a Rust debug-overflow panic for an absurd
but finite stroke width. The complete matrix, pass contract, and remaining
named hostile-input deltas are recorded in
[`renderer-fuzz-replay.md`](renderer-fuzz-replay.md). This remains an input-
contract result, not evidence of a shader mismatch.

### R3-ST-06: Signed C++ packing

**Verified upstream oracle limitation, Rust behavior accepted.** Upstream C++
left-shifts potentially negative signed values when packing triangle weights
and tessellation spans. Rust deliberately casts through unsigned integers and
tests the intended two's-complement words. No Rust or WGSL change is warranted.
An upstream UBSan test would improve the C++ oracle but does not gate this port.

## Cleared Surfaces

- **Shared translation:** the 50 raw WGSL modules and their 50 upstream-minified
  C++ headers share one pinned lineage; Naga 30 parse/validation and clean
  regeneration both pass.
- **Precision vocabulary:** all 60 modules use `f32`; none enables f16 or emits
  `fma`. `RelaxedPrecision` is dropped in the one shared generation path.
- **Textures and samplers:** reachable formats, separate bindings, address
  modes, nearest/bilinear filters, nearest mip selection, and the shared mip
  shader match. No storage, array, cube, comparison, or sRGB GPU textures are
  reachable.
- **Coordinates:** both paths retain upstream clip-space math with Naga's
  coordinate transform disabled; Rust supplies the expected negative inverse
  viewport Y. No derivative intrinsic is emitted.
- **ABI and matrices:** translated records use flattened arrays and explicit
  padding. `gpu_upload_records_match_cpp_abi` covers every translated upload
  record, including image-rectangle vertices.
- **Integers, atomics, and uniformity:** generated atomics share upstream
  fixed-point limits. No shader barriers, derivative-dependent divergent
  control flow, sample-index, or sample-mask builtins are emitted.
- **MSAA:** 4x sample count, full sample mask, disabled alpha-to-coverage,
  resolve target behavior, and destination-copy load behavior match C++ Dawn.

## Required Gates

- [x] Generated WGSL validates with Naga 30 and is reproducible from the pinned
  upstream runtime.
- [x] Cross-language ABI/layout tests cover every translated shader-visible
  Rust upload record.
- [x] Focused pixel or intermediate-plane oracles cover every accepted
  precision/compiler boundary.
- [x] The renderer corpus remains at or above its committed ratchet with no
  `.riv` regression.
- [ ] Every residual semantic difference has a named corpus or milestone gate;
  no generic `algorithm-core` diagnostic may hide a shader-stack issue.

## Verification

- `make renderer-shaders-check`
- `cargo test -p nuxie-renderer`
- `cargo test --workspace`
- `make renderer-golden`
- `make golden-compare`
- `make scripted-golden-compare`
- `cargo fmt --all --check`
- `git diff --check`
