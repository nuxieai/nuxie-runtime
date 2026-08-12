# UNIV-2073 authored Apple Metal contract

This change adds a dormant, explicitly selected authored-shader profile. It
does not change the shipping `apple-runtime` feature. Default factories,
browser WebGPU, and the macOS editor continue to request RSTB targets 0/16.
Only `WgpuFactory::new_with_trusted_apple_metal_shaders` requests targets 2/10
and wgpu's `PASSTHROUGH_SHADERS` feature.

## Trust boundary

Rust `SignedContent` parsing only reports that 64 signature bytes are present;
it does not verify them. Native source is therefore unavailable to ordinary
`File::import` and to direct scripting-VM ShaderAsset registration.

The accepted provenance is the existing opaque `ScriptExecutionCapability`.
An unsafe product boundary authenticates an exact `.riv` byte sequence, verifies
that its native shader payloads are valid output of the product's trusted MSL
compiler/exporter, and mints a dedicated capability containing its size and
SHA-256. Authentication alone is insufficient because arbitrary authenticated
MSL can violate wgpu's unsafe passthrough preconditions. Import rechecks those
exact bytes before decoding. Only that admitted import, with validated execution
limits, can mint private ShaderAsset provenance for its owned payload. Generic
script-execution capabilities—including safe test-support compatibility
imports—remain native-inert. The native decoder rechecks the payload size and
SHA-256. The safe render API cannot mint or retarget either proof. A merely set
SignedContent bit, random signature, capability for another file, generic script
capability, or ordinary import fails before MSL passthrough.

This is an equivalent fail-closed host provenance boundary, not Rive production
key verification. If a later product requires ordinary production-signed files,
it must retain `FileAssetContents.signature` and reproduce Rive's aggregate
libhydrogen verification; Ed25519 is not compatible with that format.

## RSTB selection and reflection

Trusted Apple Metal selects the exact target-2 whole-module MSL container and
target-10 BindingMap. It never probes or falls back to a valid target 0/16.
The pinned HAL compiles passthrough source with the device-selected MSL language
version and enables `preserveInvariance` only when the source contains an
invariant position, matching its normal Naga-generated compilation path.
The repo-owned supplemental reflection extension is RSTB section tag `2`.
Duplicates are malformed. All integers are little-endian; strings are a `u16`
byte length followed by UTF-8.

Version 1 is:

| Field | Encoding |
| --- | --- |
| version | `u8`, exactly `1` |
| source digest | SHA-256 of the exact target-2 variant bytes |
| map digest | SHA-256 of the exact target-10 variant bytes |
| entry count | `u8`, exactly the target-2 entry count |
| each entry | stage `u8`; logical and physical strings; workgroup `u32[3]`; input and output counts `u8`; then interface records |
| interface record | binding kind `u8`; location/builtin `u16`; type `u8`; interpolation `u8`; sampling `u8` |
| binding count | `u16`, exactly the target-10 row count |
| each binding | group `u8`; binding `u8`; array count `u16`; minimum buffer size `u64` |

Stages are vertex/fragment/compute `0/1/2`. Interface binding kind `0` is a
location and kind `1` is a builtin. `255` means absent interpolation or
sampling. Types are `f32`, `vec2/3/4<f32>`, the corresponding signed and
unsigned 32-bit scalar/vectors, then `bool` as values `0..12`. Builtins are
vertex index, instance index, position, front facing, fragment depth, sample
index, and sample mask as values `0..6`.

The decoder rejects unknown values, truncation, trailing data, duplicates,
zero array counts, zero buffer minimums, non-buffer minimums, digest mismatch,
entry or binding mismatch, and illegal workgroup dimensions. The renderer then
checks stage/direction/type rules for builtins, location type/interpolation
rules, vertex-to-fragment linkage, color outputs, vertex layouts, uniform
minimums, and device limits before its single unsafe passthrough leaf.

## Metal slots and arrays

`wgpu-hal` now owns a pure `MetalBindingSlotAllocator`; the production Metal
pipeline-layout implementation and the target-10 validator call that same
function. It models independent per-stage buffer, texture, and sampler tables,
group ordering, visibility, immediates, and Metal argument-buffer allocation.
When a pipeline selects vertex and fragment entries from different assets, the
renderer runs that allocator again over the final merged layout and requires
each selected stage's target-10 spaces and slots to match exactly.
Any binding array consumes one buffer slot regardless of element kind, matching
the pinned HAL path. Unit cases cover buffers, textures, samplers, arrays,
multiple groups, sparse bindings, stage visibility, and single-slot mutation.

The current GPU-canvas plan has no resource-array value model. Arrays are
therefore validated for exact target-10/HAL equivalence and then fail closed;
they cannot be executed accidentally as scalar resources. Adding executable
array bindings belongs to the shipping cutover, not this trust/decoder slice.

## Evidence

- Decoder fixtures contain targets 0/16 and 2/10 together, multi-stage MSL,
  multiple groups, mixed buffers/textures/samplers, arrays, compute workgroups,
  and digest-bound reflection. Every corrupt-native case must fail even though
  its target 0/16 pair is valid.
- The real-Metal passthrough probe compiles the exact native artifact through
  the dormant factory with two uniform buffers across sparse groups 0 and 2,
  renders it, renders equivalent WGSL through the default factory, and requires
  byte-identical pixels.
- `apple-authored-msl` gates native decoding, SHA-256, HAL access, and the
  factory constructor. The shipping `nux-apple-product-extension` graph does
  not enable it.

The eventual RSTB producer must emit this frozen tag-2/version-1 schema and bind
the exact target-2/10 bytes. The repository fixtures are the conformance oracle
until that exporter is integrated; UNIV-2074 owns production selection.
