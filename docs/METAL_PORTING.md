# Native Metal mechanical-port guide

This document is the operating contract for UNIV-1643 and its Apple Metal
children, UNIV-2086 through UNIV-2092. It supplements `docs/PORTING.md`; where
this guide is stricter for renderer-platform or ORE Metal work, this guide
wins.

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
   the primary reference and, once a Rust Metal candidate is selected, runs a
   separate Rust-wgpu secondary comparison into its own output directory.
4. Exercise create, submit, completion, resize, abandonment, and drop.
5. Produce a rooted test Mach-O and prove the selected path contains none of
   the forbidden dependency families.
6. When the slice contains lifetime or unsafe-interop behavior that the
   differentials cannot observe, use the optional time-boxed behavioral review
   described below. Repair the guide, manifest, or harness if it exposes a
   class of mistake.

After this trial, implementation proceeds in the existing issue order. The
issues are verification checkpoints; they must not distort source
correspondence or create temporary shipping adapters.

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
   - `make renderer-metal-msaa-probe` is a known-red capability probe, not part
     of the green smoke gate, until UNIV-2088 selects a working upstream path;
   - solid/clear/present/readback first;
   - same-runner Rust Metal versus C++ Metal is the primary comparison;
   - current Rust wgpu is recorded independently;
   - resize, abandonment, error, and completion paths are exercised.
5. **Pipeline-family corpus**
   - gradient/tessellation, MSAA, raster-order, clockwise, atomic, image,
     clipping, blend, text-facing resources, and mipmaps;
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
   - iOS device arm64, simulator arm64/x86_64, macOS arm64/x86_64;
   - required physical devices, OS floors, MSL versions, and GPU families;
   - C and Swift consumer tests;
   - final consumed Mach-Os, not intermediate `.rlib` estimates.
9. **Slim-closure deletion gate**
   - Apple selects Rust Metal with no fallback;
   - wgpu/Naga/WGSL/runtime-MSL-generation reachability is absent;
   - Apple-only wgpu paths and feature flips are deleted, not left dormant;
   - release size and performance are recorded against the pre-cutover baseline
     and the pinned C++ comparator.

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

## Merge and cutover discipline

Reviewable issue slices may merge while dormant, but the Apple product default
does not change until UNIV-2092. Before cutover, every native failure is visible
and testable; it cannot silently route to wgpu. UNIV-2092 changes the default,
qualifies all distribution slices and devices, measures the final artifacts,
and deletes the obsolete Apple dependency paths in the same closeout.

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

## Decisions requiring product ownership

Implementation can begin without resolving these, but the named gates cannot
close until they are explicit:

1. **Apple support floor:** minimum iOS and macOS versions, including whether
   Intel macOS remains a shipping requirement. This determines texture-format,
   barrier, metallib, and real-device rows.
2. **Required hardware matrix:** the oldest physical Mac GPU family and oldest
   physical iPhone/iPad family that must gate raster-order/atomic behavior.
3. **Parity exception authority:** who may approve a deliberate divergence from
   pinned C++ Metal, and where its bounded contract is recorded. Default is no
   divergence.
4. **Size success threshold:** whether success is simply removal of forbidden
   dependencies plus a measured reduction, or a numeric final Apple artifact
   target. Both measurements are required; the numeric release threshold needs
   product ownership.
5. **Post-Metal crate layout:** whether the proven backend-neutral planner moves
   to a distinct crate before the WebGPU rewrite or during it. No decision is
   needed for UNIV-2086; the concrete Metal implementation should supply the
   evidence.
6. **Native Metal MSAA authority:** at the pinned revision, direct C++ Metal
   replay aborts while creating an MSAA pipeline, and upstream's Metal test
   window does not actually propagate its `metalmsaa` selection into the frame
   descriptor. Decide whether UNIV-2088 should track a newer known-good
   upstream revision, use a different upstream Metal harness, or explicitly
   treat Rust-wgpu/Dawn MSAA as temporary secondary evidence. Do not label Dawn
   output as C++ Metal.
