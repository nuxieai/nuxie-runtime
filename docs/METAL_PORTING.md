# Native Metal mechanical-port guide

This document is the operating contract for UNIV-1643 and its Apple Metal
children, UNIV-2086 through UNIV-2092. It supplements `docs/PORTING.md`; where
this guide is stricter for renderer-platform or ORE Metal work, this guide
wins. `docs/PARITY_WORKFLOW.md` defines the backend-neutral execution and
promotion workflow; this guide is its Metal-specific specialization.

The authoritative upstream checkout is `rive-app/rive-runtime` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The executable source inventory is
`docs/metal-port-manifest.toml`, and the state-bearing lifetime inventory is
`docs/metal-port-ownership.toml`. Run `make metal-port-check` before and after
every port slice.

## Outcome

Apple product artifacts use a native Rust Metal renderer-platform adapter for
built-in rendering and a separate native Rust ORE Metal adapter for
authenticated authored GPU Canvas. Final Apple artifacts contain no `wgpu`,
`wgpu-core`, `wgpu-hal`, runtime Naga, WGSL parsing or validation, or runtime MSL
generation. Pixels, failure behavior, lifetime semantics, physical work, and
performance match the pinned C++ Metal implementation under the applicable
parity contract.

This is a translation campaign, not a redesign. Preserve the existing public
`Factory`/`Renderer` seam and the already-portable logical renderer planning.
Port the upstream native Metal implementation and the relevant ORE interface
as directly as Rust and Objective-C interop allow. Do not first invent a
cross-platform HAL, normalize Metal to wgpu concepts, or reorganize the shared
renderer solely in anticipation of Vulkan.

## Authoritative modules and seams

Two seams must remain distinct:

1. **Renderer-platform seam.** The backend-neutral renderer planner supplies
   flush/resource intent. The Metal adapter owns device capabilities, resource
   realization, pipeline selection, command encoding, submission, completion,
   presentation, and readback. Pinned authorities include
   `RenderContextImpl`, `RenderContextHelperImpl`, and
   `RenderContextMetalImpl`.
2. **ORE scripting GPU seam.** The scripting layer supplies authenticated ORE
   operations and shader artifacts. The ORE Metal adapter owns authored
   buffers, textures, samplers, bind groups, pipelines, render/compute passes,
   command buffers, and frame serials. Pinned authorities include
   `rive::ore::Context` and `rive::ore::ContextMetal`.

Shared resource concepts may move behind a private internal seam only after
both concrete implementations demonstrate identical semantics. A type name or
similar-looking Metal object is not sufficient evidence that ownership,
ordering, failure, or reuse behavior is shared.

## UNIV-2074 research artifact disposition

UNIV-2074 closes the abandoned wgpu-native-MSL product direction without
discarding evidence useful to this port. Its disposition is explicit:

| Artifact | Disposition for native Metal |
| --- | --- |
| Machine-readable Apple support matrix, semantic validator, and honest pending physical-device rows | Retain as the initial qualification inventory; native Metal must replace research assumptions with measured platform results. |
| Final-Mach-O symbol/string/link-map reachability checker and tests | Retain and extend with the native Metal forbidden-dependency contract. |
| `tools/apple-msl-catalog/reviewed-inventory.json` and its 91-permutation callsite review | Retain as differential coverage and pipeline-family input, not as an authoritative native slot map. |
| Captured native-shader catalog and catalog validators | Retain only as historical/differential evidence for shader coverage and artifact identity. Pinned C++ Metal shader sources and metallibs remain authoritative. |
| Incomplete Apple product feature flip | Abandon; no shipping selector changes before UNIV-2092. |
| Vendored wgpu/wgpu-core/wgpu-hal feature split | Abandon as a product direction; final Apple closure removes those dependencies rather than maintaining a custom split. |
| Manual passthrough callsite migration and runtime materialization | Abandon; renderer callsites cannot reconstruct the backend-owned Metal resource-slot contract. |

The retained artifacts are evidence inputs, not implementation scaffolding.
Default and shipping feature graphs remain unchanged until the native adapter
passes the cutover gate.

## Mechanical translation rules

1. Keep upstream file, type, method, field, enum, and control-flow
   correspondence visible. Prefer a recognizably translated Rust module over
   an idiomatic rewrite during this campaign.
2. Record the upstream path and pin in every translated module. Update the
   corresponding source-manifest row in the same commit.
3. Preserve decision order, fallback order, resource creation timing,
   capability probes, alignment, buffer growth, frame numbering, barrier
   placement, render-pass breaks, and completion behavior.
4. Translate Objective-C ownership deliberately:
   - retained C++/Objective-C fields become owned `Retained<T>`-style values;
   - borrowed parameters remain borrows for the smallest valid scope;
   - completion handlers capture only owners that upstream intentionally keeps
     alive;
   - autorelease-pool placement is recorded where temporary object volume is
     material;
   - `unsafe` blocks state the upstream invariant they rely on.
5. Rust errors must preserve upstream observability. A recoverable native
   failure returns a typed error; an upstream-unreachable state may remain a
   hard failure only if the caller enforces the same invariant.
6. Do not translate the current wgpu implementation into Metal. It is a
   regression oracle, not the architectural authority for Metal capability,
   synchronization, or resource policy.
7. Do not clean up, deduplicate, rename broadly, optimize, or extract a future
   WebGPU/Vulkan interface in the same change as behavioral translation.
8. Do not add placeholders that make compilation green: no `todo!`,
   `unimplemented!`, success-returning no-op, empty shader, skipped manifest
   row, swallowed Metal error, or fallback to wgpu.

When Rust cannot represent an upstream shape directly, add an explicit
adaptation note beside the implementation and to the manifest evidence. The
note must state the invariant being preserved and name its focused test.

## Campaign preparation

Before a source row moves to `in-progress`:

- verify `RIVE_RUNTIME_DIR` is the exact pinned revision;
- read the complete upstream header and implementation, not only the target
  method;
- identify every state-bearing field in `metal-port-ownership.toml`;
- name the focused compiler/test work queue for the slice;
- establish the C++ Metal replay input and expected adapter identity;
- confirm the slice can remain dormant behind explicit experimental selection
  until UNIV-2092.

The source manifest is exhaustive over the native Metal, Metal shader, shared
ORE interface, and ORE Metal directories. An upstream file added under those
globs fails `metal-port-check` until classified. `ported` requires a real Rust
owner and focused green evidence. The `rust_modules` and `evidence` values are
checked-in file paths. `verified` additionally requires checked-in
`parity_evidence` from the issue-level parity gates. A time-boxed source review
can improve confidence in unobservable lifetime behavior, but it is not a
promotion requirement.

The first mechanical wave records the complete
`generate_draw_combinations.py` translation with its focused byte oracle as
`ported`. The second wave adds the complete background compiler worker and
native `newLibraryWithSource` adapter, and the offline `draw.metal` source
closure and metallib loader; those source rows are `ported` after the focused
crate tests execute the real Metal compiler and library loader. The promotion
evidence must come from a run with a system Metal device: the portability guard
that skips a live test when no device exists is not sufficient promotion
evidence by itself. For this wave, both live tests executed and passed on the
macOS Metal host, and all macOS, iOS-device, and iOS-simulator target builds
produced both offline metallibs. Their exact offline fixtures are
`crates/nuxie-renderer/tests/fixtures/native_metal/background_shader_macros.txt`,
every file under the pinned
`crates/nuxie-renderer/tests/fixtures/native_metal/background_shader_sources/`
set enumerated in the source manifest, and
`crates/nuxie-renderer/tests/fixtures/native_metal/offline_draw_shader/source_inventory.txt`.

The render-context header and implementation rows remain `in-progress`. Their
coverage now also includes retained Metal buffer rings, image-texture upload
and mip generation, the complete sampler permutation table, and construction
of retained offline draw-library and sampler owners. The third wave adds the
concrete context/frame boundary: a frame acquires one command buffer at
`begin_frame`, retains that exact owner until consuming `finish` or
abandonment, and a resize constructs a complete target generation before
replacing the factory's current one. Target generations use controlled
single-thread interior mutation because pinned upstream attaches product
textures and lazily realizes atomic storage after the reference-counted target
is shared. The fourth wave adds native weak-reference observation after an
explicit Objective-C autorelease pool drains; this proves an abandoned frame
releases its uncommitted command buffer, old target texture, and all realized
atomic storage. It also adds the dormant caller-supplied drawable seam: the
Apple caller retains layer configuration, main-actor acquisition, and frame
scheduling. One borrowed drawable becomes the frame's BGRA target; renderer work commits on the frame's
command buffer, and presentation commits on the next command buffer from the
same queue, matching the pinned product oracle. The drawable remains retained
until synchronous completion. The target
ownership row is therefore `ported`; the command-buffer row remains
`in-progress` only because a safe live Metal error has not yet been produced.
The presentation adapter remains `in-progress` until product-main-actor resize
and no-drawable behavior are exercised through the actual Apple boundary. Image
decode/upload is not wired into product behavior yet. The tracer retains
format-compatible RGBA and BGRA solid pipelines for headless and drawable
targets respectively. Because the
public `Factory` render-buffer constructor is infallible, a native Metal
allocation failure deliberately terminates at that backend boundary; it never
substitutes CPU storage or the wgpu renderer.

The image-texture leaf records one intent-preserving correction to pinned C++:
the upstream ASTC calculation adds a footprint index to
`MTLPixelFormatASTC_4x4_LDR`, but Metal reserves enum value 209 between the
6x6 and 8x5 formats. Rust uses an explicit 14-entry `MTLPixelFormat` table so
8x5 and every later footprint cross that enum gap without selecting the wrong
format. The focused
`astc_footprints_use_their_exact_metal_pixel_formats_across_enum_gap` test is
the evidence for this adaptation.

## Trial translation: UNIV-2086

UNIV-2086 is the process trial as well as the solid-render tracer. It must prove
the complete workflow before the campaign expands:

1. Create device, queue, target, offline library, solid pipeline, command
   buffer, presentation/readback, and completion through the existing product
   `Factory`/`Renderer` seam.
2. Keep the shipping default on wgpu. Selection of Rust Metal is explicit and
   experimental; failure does not fall back.
3. Compare the same recorded stream in the same run against pinned C++ Metal
   and current Rust wgpu. `renderer-metal-oracle-tracers` records C++ Metal as
   the primary reference and runs a separate Rust-wgpu secondary comparison
   into its own output directory.
4. Exercise create, submit, completion, resize, abandonment, and drop.
5. Produce a rooted test Mach-O and prove the selected path contains none of
   the forbidden dependency families. The root must execute both deterministic
   readback and the public caller-owned `CAMetalDrawable` presentation seam;
   rooting only the headless frame is insufficient product-surface evidence.
6. When the slice contains lifetime or unsafe-interop behavior that the
   differentials cannot observe, use the optional time-boxed behavioral review
   described below. Repair the guide, manifest, or harness if it exposes a
   class of mistake.

After this trial, implementation proceeds in the existing issue order. The
issues are verification checkpoints; they must not distort source
correspondence or create temporary shipping adapters.

The UNIV-2086 closeout root is
`tools/renderer-replay/src/bin/native-metal-product-root.rs`. This upper-layer
oracle caller keeps `CAMetalLayer` ownership and drawable acquisition outside
the protected renderer package. It asserts main-thread execution, then its
explicit feature-selected factory checks a deterministic solid pixel, handles
a missing drawable without creating a frame, resizes, configures a
caller-owned `CAMetalLayer`, acquires its drawable,
and renders and presents through the public Apple boundary before proving the
layer remains reusable. Focused live tests separately reject a stale-sized
drawable after resize and recycle repeatedly abandoned frames. The binary gate
requires the acquisition and presentation selectors in the final Mach-O and
rejects wgpu, Naga, and WGSL symbol/string reachability. The shipping Apple C
ABI remains on wgpu until the full cutover in UNIV-2092. The pure-runtime
boundary ratchet rejects `CAMetalLayer` and `nextDrawable` if they spread into
an unapproved protected source file.

## UNIV-2087 first resource slice

The first UNIV-2087 checkpoint ports the complete pinned
`color_ramp.metal` and `tessellate.metal` source closures into the offline
`native_metal_resources.metallib`. Its exact exported inventory is `EF`, `FF`,
`WF`, and `XF`. The concrete Metal context owns one synchronous concrete
gradient/tessellation resource generation, governed by the upstream-shaped
three-slot reservation/release policy, plus the Gaussian integral texture, the
canonical tessellation index/vertex data, and the resource pipelines. The
public `Factory`/`Renderer` seam exercises them with one closed cubic,
linear-gradient, clockwise midpoint-fan draw. Multiple simultaneously in-flight
concrete resource generations are not claimed by this checkpoint.

The primary oracle is pinned C++ Metal at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The checked-in gradient fixture
requires identical geometry/coverage occupancy and permits at most one RGBA8
LSB of same-backend rounding. Current Rust wgpu remains a secondary regression
oracle in `tools/metal-port/tracer-corpus-wgpu-secondary.toml`; its separate
35-LSB release-replay bound cannot weaken the C++ Metal gate. The live Rust
test uses a wider cross-backend guard because the default wgpu factory path has
shown up to 59 LSB on the antialiased edge.

This checkpoint deliberately does not claim the whole issue. General flush
topology, atlas resources, image draws, clip updates, more than one concurrent
resource generation, and injected live device failure remain for later
UNIV-2087 slices. The combined ownership row therefore stays `in-progress`
even though both shader-source rows and the buffer-ring source row are
`ported`. The shipping renderer remains wgpu-selected.

## UNIV-2087 upload-ring slice

The second resource checkpoint ports the distinct RenderContext upload and
uniform `BufferRingMetalImpl`; it does not modify the user-facing
`RiveRenderBuffer` implementation in `native_metal/buffer.rs`. The concrete
context now owns seven independent three-buffer shared-storage rings for flush
uniforms, gradient spans, tessellation spans, paths, paints, paint auxiliary
data, and contours. Capacities are passed to Metal verbatim and grow only when
the typed payload exceeds the current allocation. Every ring begins with the
pinned physical order 1, 2, 0. Path base-instance remains an inline binding
because that is also upstream's intentional ABI.

A pure blocking coordinator separates pre-submit ownership from submitted GPU
ownership. Dropping an unsubmitted lease abandons and wakes; submission moves
release responsibility to an exact-once completion owner. The current public
frame waits synchronously for Metal, then completes that owner. This proves the
ownership protocol without claiming that the public renderer already supports
multiple asynchronous in-flight frames or multiple concrete resource-texture
generations.

The live gradient oracle remains pixel-identical to the first checkpoint's
pinned C++ bounds under `MTL_DEBUG_LAYER=1`. Its Rust physical-work contract is
one command buffer, three render passes, seven buffer uploads totaling 944
bytes, one queue submission, three draw calls, eleven draw instances, six
tessellation spans, and four path patches. Pinned C++ defines the topology and
pixels but exposes no numeric counter API, so these values are explicitly a
deterministic Rust regression oracle rather than a claim of C++ counter
equality.

## UNIV-2087 feather-atlas slice

The third resource checkpoint ports one complete, naturally selected feather
atlas stroke through the public `Factory`/`Renderer` seam. The context owns a
nullable private `R16Float` render-target/shader-read texture, the exact `RF`
atlas vertex function with `UE` fill/Add and `VE` stroke/Max fragment
pipelines, and the final raster-order atlas-blit permutation. Canonical logical
admission, feather preparation, and atlas packing supply the geometry and
placement. A backend-neutral, solid-only serializer produces the narrow
path/paint/aux/contour records, tessellation spans, patch range, and six final
`TriangleVertex` values without constructing the WGPU-bearing general draw
owner; a focused byte-level test keeps that output equal to the canonical
writer for this admitted fixture. The Metal adapter asserts this checkpoint's
deliberate boundary of one logical flush and one atlas draw instead of
inventing a parallel general scheduler.

The upload owner now contains the upstream eighth triangle ring. This fixture
performs seven nonempty uploads totaling 1,064 bytes because the solid paint
has no gradient-span payload; the eighth owned ring still carries the six
atlas-blit vertices. Its deterministic topology is one command buffer, three
render passes, one submission, three draw calls, thirteen submitted
instances, seven tessellation spans, and five path patches. The selected
placement is a 33x33 content extent in a 41x41 physical allocation.

The platform seam stores only `PreparedTypedDrawResources`-derived records.
An earlier version retained `SolidDraw` until Metal encoding; although its
image field was `None`, Rust's unwind/drop glue still made the WGPU image owner
reachable and grew the rooted product binary. The release-size gate caught
this when pixel tests did not. The reusable rule is that a platform root must
be backend-neutral across values, ownership, destruction, and failure paths.
Shared planners should expose narrow typed outputs instead of making a native
backend retain a sum type that contains another backend's resources. The same
rule also led the single-gradient seam to consume normalized gradient inputs
without constructing `SolidDraw`. The resulting rooted Mach-O is 635,840
bytes and contains no WGPU, Naga, or WGSL markers.

The historical fixture called `first-light-atlas-feather-stroke` used feather
20 at identity scale. Pinned C++ selects the atlas only when
`16 / (feather * 1.5 * matrixScale) <= 0.5`, so that input actually exercised
direct feathering. This checkpoint corrects it to feather 24, captures a fresh
native-Metal reference from pinned revision
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, and adds an explicit threshold and
extent test. This is a reusable workflow rule: names and expected topology are
not routing evidence; every parity fixture must assert that it crossed the
intended upstream branch before its pixels can promote a port.

The primary C++ Metal comparison requires identical coverage occupancy, at
most two RGBA8 LSB, and an independently selected ceiling of one quarter of
the 64x64 frame (1,024 differing pixels); the observed run was 838 pixels at
one LSB. Rust wgpu uses the same power-of-two ceiling as a separately audited
secondary contract and observed 804 pixels at one LSB. The bound was selected
from frame geometry before promotion rather than copied from either candidate
count. Multiple atlas draws or logical flushes, relocated
flush-wide batching, atlas fills, clip/scissor overlap, gradient or image atlas
paints, advanced/HSL blends, direct feathers, atomic scheduling, general
per-batch background pipeline selection, and asynchronous presentation remain
explicit later slices. The combined resource owner therefore remains
`in-progress`.

## UNIV-2087 same-flush atlas batching slice

The next checkpoint admits multiple same-style solid atlas draws in one logical
flush. One backend-neutral planner jointly packs every placement, preserves
authored path/paint/final-blit order, and emits atlas-mask resources in the
pinned C++ order: fill before stroke, then unscissored before scissored. It
relocates all midpoint tessellation into one flush-wide range. Contiguous
unscissored patches coalesce into one `AtlasDrawBatch`; each scissored draw
remains separate. The Metal adapter consumes these exact batches instead of
inventing a second scheduler.

The live two-stroke fixture exercises two scissored stroke batches, twelve final
atlas-blit vertices, and seven concrete upload rings under one shared
coordinator reservation; the gradient-span ring is intentionally inactive
because the paints are solid. Its deterministic Rust topology is one command
buffer, three render passes, seven nonempty buffer uploads totaling 1,544 bytes,
four draw calls, twenty-two submitted instances, eleven tessellation spans, and
ten path patches. The primary pinned C++ Metal
comparison observes 503 differing pixels at one RGBA8 LSB with exact coverage
occupancy. The secondary Rust-wgpu comparison observes 583 pixels at no more
than two LSB. Both remain inside the independently selected 2,048-pixel ceiling;
the ceiling was not changed to promote the implementation.

This slice exposed an important oracle rule. The first candidate used the
precompiled fully featured AtlasBlit ubershader and produced the correct
geometry, topology, and occupancy, but 3,676 pixels differed by up to two LSB.
The pinned C++ replay forces synchronous specialization and enables default
interleaved-gradient-noise dithering. Compiling the exact
`AtlasBlit + ENABLE_DITHER + rasterOrdering` key reduced the difference to the
accepted result. Target format, render mode, compilation policy, feature mask,
miscellaneous flags, and function family are therefore all part of renderer
oracle identity. A compatible fallback pipeline is not a parity oracle even
when every pixel is geometrically correct. When a preselected tolerance fails,
trace the pipeline key before considering fixture or threshold changes.

Specialized AtlasBlit compilation remains lazy: creating a factory that never
uses a feather atlas does not start the compiler or realize these pipeline
states. First atlas allocation builds the mask pair and the specialized final
pipeline as one fail-closed replacement before publishing the resource
generation. The headless tracer now uses the same `BGRA8Unorm` target format as
the pinned C++ replay and real Apple drawables, then normalizes readback bytes
to the harness's RGBA contract.

The rooted no-WGPU Mach-O is 719,088 bytes and contains no WGPU, Naga, or WGSL
markers. This is 83,248 bytes larger than the prior single-atlas checkpoint
because the exact upstream runtime MSL specialization path is now reachable.
Keep that delta visible: parity comes first, but a later size pass should
compare this lazy source compiler with ahead-of-time specialization of the
actually reachable Apple pipeline keys instead of assuming runtime compilation
is free.

This checkpoint still deliberately excludes multiple logical flushes,
fill/stroke mixing in one native flush, gradient or image atlas paints,
non-rectangular clips, advanced/HSL blends, and general draw scheduling.

## Compiler and commit workflow

Use compiler errors as the work queue within one upstream source group:

1. Translate declarations and owned state.
2. Translate construction and destruction.
3. Translate enum/conversion helpers and resource creation.
4. Translate frame preparation, encoding, submission, and completion.
5. Make focused tests green.
6. Update the source and ownership manifests.
7. Run the issue's oracle rung.

Each commit should identify the upstream source path(s), pinned revision, and
ownership rows it advances. A source row cannot be split across untracked
temporary modules. Broad Git operations are prohibited when parallel work is
active; each implementer owns an explicit source set.

## Time-boxed behavioral review

`PORTING.md` §0 makes tests the acceptance and permits at most one time-boxed
behavioral review where differentials cannot observe the surface. Native Metal
lifetime and unsafe-interop behavior can meet that condition. A slice may use
one bounded review by a person or agent that did not implement it, using both
lenses below:

1. **Translation review:** compare C++ and Rust side by side and search for
   omitted branches, changed ordering, implicit casts, signedness/overflow
   differences, mistaken enum values, wrong defaults, altered coordinate or
   pixel formats, and incomplete source-manifest coverage.
2. **Metal lifetime review:** search for retain cycles, use-after-completion,
   early release, mutable reuse while in flight, missing autorelease scopes,
   wrong storage modes, alignment errors, incomplete device-loss handling,
   queue/command-buffer misuse, and accidental wgpu/Naga reachability.

When used, the implementer does not self-approve the review. The reviewer
reports findings; the implementer or a designated fixer resolves them. The
review is not a publication or manifest-promotion gate beyond the test
acceptance in `PORTING.md` §0, and a review that only says the tests pass is
incomplete.

## Oracle hierarchy

The three implementations have different jobs:

| Implementation | Role |
| --- | --- |
| Pinned C++ native Metal | Primary platform and behavioral authority |
| New Rust native Metal | Candidate |
| Current Rust wgpu | Secondary regression oracle for established Rust behavior |

For a Metal-specific disagreement, pinned C++ Metal wins unless an explicit
product decision records a divergence. Rust wgpu may expose a regression in
shared planning or stream replay, but it cannot override upstream Metal
capability selection, attachment policy, barriers, storage mode, or lifetime.

All dynamic GPU comparisons must report adapter identity. Candidate and
reference must run on the same adapter identity before pixel comparison. The
C++ Metal replay is selected with `--reference-backend ffi-metal`; it is never
relabeled as Dawn/WebGPU evidence. Its same-runner sidecar includes the hash of
an input manifest that pins the Rive revision and every linked upstream static
archive. Reference checks reject a moved or tracked-dirty checkout, stale
stamp, archive hash drift, or Dawn/WebGPU dynamic reachability.

## Validation ladder

Do not jump directly from compilation to the full corpus. Each rung narrows a
different class of translation error.

1. **Source and ownership completeness**
   - `make metal-port-test`
   - `make metal-port-check`
   - no unclassified upstream source or ownership row;
   - no source row promoted without a Rust module and named evidence.
2. **Compile and static contracts**
   - Apple target checks for every affected crate;
   - enum values, struct sizes/alignments, shader constants, binding slots,
     pipeline keys, and metallib provenance;
   - native bridge and upstream archives use identical layout-affecting
     defines. The pinned `tests/out/release` archive uses `WITH_RIVE_TOOLS`;
     omitting it makes inline Metal methods write the wrong object fields;
   - forbidden-dependency feature-tree and source ratchets.
3. **CPU/intermediate oracles**
   - logical flush descriptors, buffer contents, offsets, counts, atlas inputs,
     tessellation spans, pipeline-selection keys, and command topology;
   - deterministic artifacts are byte exact.
4. **Tracer pixels and lifecycle**
   - `make renderer-metal-reference-bootstrap`
   - `make renderer-metal-reference-replay`
   - `make renderer-metal-reference-check`
   - `make renderer-metal-oracle-tracers`
   - `make renderer-metal-msaa-contract` is a green negative contract proving
     that the native Metal oracle rejects the WebGPU-only MSAA mode before
     pipeline construction;
   - solid/clear/present/readback first;
   - same-runner Rust Metal versus C++ Metal is the primary comparison;
   - current Rust wgpu is recorded independently;
   - resize, abandonment, error, and completion paths are exercised.
5. **Pipeline-family corpus**
   - gradient/tessellation, raster-order, clockwise, atomic, image, clipping,
     blend, text-facing resources, and mipmaps; keep MSAA coverage in the
     WebGPU/Dawn suite rather than inventing a native Metal execution mode;
   - use existing row contracts; do not widen tolerance to close a port bug;
   - report byte identity as a secondary health metric.
6. **Hostile replay and fuzzing**
   - minimize every new mismatch into a retained fixture;
   - replay malformed streams and lifecycle interruption;
   - a mismatch fixes the implementation or improves a general harness rule,
     never a one-case bypass.
7. **Physical work and performance**
   - deterministic pass, submission, upload, draw, instance, tessellation, and
     path-patch counters where available;
   - performance at least matches the pinned C++ contract over independent
     reports; no single warm run is a release verdict.
8. **Product and distribution**
   - preserve the existing iOS 15 and macOS 12 floors;
   - iOS device arm64, simulator arm64/x86_64, macOS arm64/x86_64, including
     Intel macOS and Intel Simulator support;
   - qualify capability branches rather than marketing generations: legacy
     PowerVR iOS, modern Apple-family iOS, Intel Mac, discrete AMD Mac, Apple
     Silicon Mac, Intel-hosted Simulator, and Apple-Silicon-hosted Simulator;
   - Apple Silicon Mac, modern physical iOS, and both simulator branches gate
     renderer changes. Legacy PowerVR, Intel Mac, and discrete AMD Mac gate
     cutover, releases, and changes to capability/barrier selection;
   - every row has fresh candidate/artifact, OS, SDK, device-family, corpus,
     output, and evidence digests. Provenance-complete external device-lab or
     trusted-contributor runs are valid for scarce hardware;
   - C and Swift consumer tests;
   - final consumed Mach-Os, not intermediate `.rlib` estimates.
9. **Slim-closure deletion gate**
   - Apple selects Rust Metal with no fallback;
   - wgpu/Naga/WGSL/runtime-MSL-generation reachability is absent;
   - Apple-only wgpu paths and feature flips are deleted, not left dormant;
   - both scripting-off and scripting-on rooted Mach-O closures are at least 5%
     smaller than a freshly reproduced wgpu baseline built with identical
     roots and toolchain;
   - no Apple slice or packaged artifact grows. Record absolute and percentage
     changes for every slice and XCFramework without applying the rooted 5%
     threshold to compressed bundles;
   - release performance is recorded against the pre-cutover baseline and the
     pinned C++ comparator.

## ORE-specific fail-closed rules

ORE Metal begins only after the shared Metal resource model is proven by
UNIV-2087. The authored shader lane additionally requires:

- authenticated exact target-2 MSL bytes;
- matching target-10 BindingMap and versioned supplemental reflection;
- explicit entry interfaces, uniform sizes, binding groups/slots, texture and
  sampler classes, workgroup limits, and vertex maps;
- a fresh physical shader module and occurrence-correct resource identity;
- errors that match pinned upstream behavior where supported;
- no use of target-0 WGSL to recover from Metal validation or execution
  failure.

Keep ORE's interface limited to production-authored operations. Vulkan-only or
editor-only operations are added later when a concrete adapter requires them.
ORE Metal lives in a separate crate from the built-in Metal renderer so a
scripting-disabled product can prove its shader, reflection, and trust
machinery is absent. It may consume only the narrow device/queue service proven
by the concrete Metal adapter.

## Merge and cutover discipline

Reviewable issue slices may merge while dormant, but the Apple product default
does not change until UNIV-2092. Before cutover, every native failure is visible
and testable; it cannot silently route to wgpu. UNIV-2092 changes the default,
qualifies all distribution slices and devices, measures the final artifacts,
and deletes the obsolete Apple dependency paths in the same closeout.

After UNIV-2086 proves the concrete device, queue, frame, surface, completion,
and ownership boundary, mechanically extract the demonstrated backend-neutral
planning into `nuxie-renderer-core` and add `nuxie-renderer-metal` before
UNIV-2087 expands the implementation. Add `nuxie-ore-metal` when UNIV-2091
begins. Keep the existing `nuxie-renderer` name on the wgpu implementation for
the Metal campaign; rename or replace it only when the WebGPU phase begins.
This extraction preserves source correspondence and is not an invitation for
idiomatic cleanup.

Idiomatic refactoring begins only after the complete Metal adapter is at parity.
At that point compare the concrete Metal and WebGPU adapters, extract only
demonstrated variance behind a small interface, rerun the complete oracle
ladder, and preserve enough source correspondence for future upstream syncs.

## Upstream drift

Do not casually rebase the Metal campaign onto a newer Rive revision. A pin
advance is a separate review that:

1. enumerates every changed tracked source;
2. classifies semantic, lifetime, shader, and capability changes;
3. updates the source and ownership manifests;
4. reruns C++ Metal oracle provenance;
5. lands independently of behavioral cleanup.

## Recorded decisions

1. **Apple support floor:** preserve iOS 15, macOS 12, Intel macOS, and Intel
   Simulator during the Metal migration. Platform retirement is separate work.
2. **Required hardware matrix:** gate the distinct capability branches listed
   in validation rung 8. A failure on any supported branch blocks cutover; it
   never retains wgpu as an escape hatch. An upstream-equivalent native Metal
   execution mode is valid when it passes the same parity gates.
3. **Parity decisions:** default to exact pinned-C++ parity. If evidence exposes
   a necessary divergence, Codex presents its exact scope and recommendation;
   the user decides. Record only the resulting bounded technical contract in
   its Universe issue and enforcing test.
4. **Size success threshold:** remove all forbidden reachability and reduce
   both rooted Mach-O variants by at least 5% versus the same-toolchain wgpu
   baseline, with no Apple slice or packaged artifact growth.
5. **Crate layout:** make the first concrete adapter work in UNIV-2086, then
   perform the mechanical core/Metal split before UNIV-2087. Keep ORE Metal
   separate and defer the existing wgpu crate rename to the WebGPU phase.
6. **Native Metal execution modes:** native Metal implements raster-order
   execution where supported and an atomic fallback; it does not implement the
   WebGPU-specific `InterlockMode::msaa`. UNIV-2088 therefore proves parity for
   those two upstream-native branches and explicit MSAA non-reachability. Dawn
   MSAA remains WebGPU regression evidence and is never labeled C++ Metal.
