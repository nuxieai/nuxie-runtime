# Whole Metal Renderer Port Plan

This plan governs completion of the native Rive Metal renderer. It applies the
file-first mechanical-translation process in `PARITY_WORKFLOW.md` and the
Metal-specific ownership and verification rules in `METAL_PORTING.md`.

The campaign is pinned to upstream Rive commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Process authority, in order, is:

1. `PARITY_WORKFLOW.md` file-first mechanical translation;
2. `METAL_PORTING.md` Metal ownership, artifact, and verification rules;
3. this campaign plan;
4. the pinned upstream source itself.

Generic implementation and test-driven-development playbooks do not govern
this campaign when they prescribe tracer bullets or feature slicing.

## Campaign rule

Source ownership units determine implementation scope. Product features,
fixtures, and failing images do not.

The following workflows are explicitly excluded during mechanical
translation:

- feature-by-feature or fixture-by-fixture implementation;
- tracer-bullet selection of the next renderer behavior;
- a test-first loop that implements only enough backend behavior for one
  fixture;
- promotion or standalone commits for isolated draw families inside an
  incomplete source ownership unit;
- designing a shared GPU abstraction before the concrete Metal backend is
  complete.

Tests remain mandatory, but they belong to the later compiler and behavior
work queues. They verify the complete translation; they do not choose which
source behavior is translated.

Editing a large source file in bounded line ranges is an implementation
logistic, not a product slice. Work always resumes at the next untranslated
source range, and no bounded range establishes whole-file completion.

## Pinned source inventory

The primary Metal ownership unit is the complete context header and
implementation:

- `renderer/include/rive/renderer/metal/render_context_metal_impl.h`
- `renderer/src/metal/render_context_metal_impl.mm`

Its directly owned supporting units are:

- `renderer/src/metal/background_shader_compiler.h`
- `renderer/src/metal/background_shader_compiler.mm`
- `renderer/include/rive/renderer/buffer_ring.hpp`
- `renderer/src/shaders/metal/color_ramp.metal`
- `renderer/src/shaders/metal/draw.metal`
- `renderer/src/shaders/metal/tessellate.metal`
- `renderer/src/shaders/metal/generate_draw_combinations.py`

`docs/metal-port-manifest.toml` remains the source-of-truth inventory.
`docs/render-context-metal-file-map.tsv` partitions every line in the primary
header and implementation, but it is a completeness ledger rather than a
feature queue.

At plan adoption, the line ledger reports:

| Source | Ported ranges | Partial ranges | Missing ranges |
| --- | ---: | ---: | ---: |
| `render_context_metal_impl.h` | 5 | 3 | 0 |
| `render_context_metal_impl.mm` | 30 | 14 | 4 |

Both primary manifest rows remain `in-progress`. Uncommitted work in the
feather-atlas range is not a promoted checkpoint and is evaluated only as part
of the complete implementation-body translation.

If translation exposes a missing dependency outside this inventory, work stops
long enough to add that dependency as a source ownership unit. The dependency
is then translated from its source; it is not replaced by a fixture-specific
Metal helper.

## Completion target

The campaign is complete only when:

1. every pinned source line and every state-bearing header field has an owned
   Rust correspondence or a documented configuration exclusion;
2. all upstream-reachable Metal branches have a compiled Rust implementation;
3. the backend consumes one general, source-shaped flush description rather
   than selecting among tracer-specific frame paths;
4. no upstream-supported path is represented by a runtime `Unsupported`
   branch or hidden WGPU fallback;
5. construction, resource ownership, pipeline lookup, encoding, submission,
   completion, abandonment, resize, and destruction match the recorded
   lifetime contracts;
6. the full compiler and behavior work queues are green;
7. the primary header and implementation manifest rows can truthfully move
   from `in-progress` to `ported`;
8. issue-level platform and product evidence supports any later `verified`
   promotion.

## Phase 1: preparation closure

No further renderer behavior is implemented until this phase is complete.

### 1.1 Re-audit the inventory

- Re-read all four primary/supporting Metal C++/Objective-C++ files at the
  pinned commit.
- Verify the manifest contains every file in the upstream Metal directories.
- Verify the line map continuously and non-overlappingly covers the complete
  primary header and implementation.
- Treat existing `ported` subranges as claims to re-audit, not assumptions.
- Record preprocessor branches separately for macOS, iOS device, iOS
  simulator, tvOS, and visionOS.

### 1.2 Complete the ownership ledger

For every state-bearing field in `RenderTargetMetal`, `ContextOptions`,
`DrawPipeline`, and `RenderContextMetalImpl`, record:

- its exact Rust owner;
- construction and publication point;
- mutation thread;
- native object and buffer lifetime;
- submission/completion transfer;
- destruction order;
- nullable or failure behavior;
- any safe-Rust adaptation.

Existing ownership rows are expanded where a single prose rule currently
covers multiple distinct fields. A field without an owner keeps the complete
header ownership unit `in-progress`.

### 1.3 Freeze translation conventions

Before mechanical translation resumes, record the exact mappings for:

- Objective-C retained objects and nullable returns;
- intrusive/reference-counted owners;
- spans, raw byte ranges, and alignment;
- enums, flags, generated shader slots, and pixel formats;
- assertions versus typed Rust errors;
- callbacks, worker shutdown, and command-buffer completion;
- preprocessor configuration branches;
- C++ member destruction order versus Rust field drop order.

No nearby behavior is redesigned while translating an owner. Corrections to
known upstream defects require a documented divergence and focused evidence.

## Phase 2: complete mechanical translation

Translate the full ownership unit in pinned source order. Do not run the
behavior corpus or add feature fixtures during this phase.

### 2.1 Header and owner shape

Translate `render_context_metal_impl.h:1-282` completely:

- nested owners and public construction surface;
- all context options and Metal capability state;
- public resource factories;
- buffer-ring and pipeline owners;
- private orchestration methods;
- every context-owned native field.

The Rust interface may be safer, but it must make every upstream owner and
state transition identifiable. Existing scattered modules remain valid when
they are deep implementation details of this complete owner; they may not
replace missing public or private owner state with fixture-specific branches.

### 2.2 Implementation body

Translate `render_context_metal_impl.mm:1-2030` sequentially:

1. platform metallib selection and pipeline helpers;
2. sampler mappings, resource pipelines, and draw-pipeline state;
3. platform/capability selection and buffer rings;
4. `MakeContext`, constructor, destructor, and exact initialization order;
5. render target, render buffer, image texture, canvas, and ORE factories;
6. texture resize and resource publication;
7. compatible-pipeline lookup and background compilation;
8. command-buffer acquisition, preparation, and submission;
9. common draw-pass creation;
10. the complete `flush` body, in source order;
11. `postFlush` and completion ownership.

The `flush` translation includes every source branch in one source-shaped
encoder path: gradient, tessellation, feather atlas, image mipmaps, main and
PLS attachments, preserve-target copy, batch policy, scissor state, image and
sampler binding, barriers, every patch/triangle/image draw type, initialize,
resolve, and MSAA boundaries.

### 2.3 General flush input

The Metal owner receives one general Rust equivalent of upstream's
`FlushDescriptor`/draw list. Existing specialized arrays in
`NativeMetalFrame` are migration inputs, not the final orchestration model.

If the canonical renderer cannot yet produce a field required by the pinned
Metal `flush`, translate the missing producer ownership unit before continuing.
Do not synthesize the field in a fixture-specific native path.

### 2.4 Translation-phase prohibitions

The translation may not contain:

- `todo!`, placeholder returns, swallowed errors, or no-op implementations;
- a fallback to WGPU or CPU rendering;
- a branch admitted only for a named fixture;
- duplicated pipeline/cache policy outside the context-owned cache;
- feature-specific resource generations that bypass the general flush owner;
- new abstractions justified only by anticipated Vulkan/WebGPU sharing.

When a native API requires a safe-Rust adaptation, implement it completely and
record it beside the corresponding source range.

## Phase 3: compiler work queue

Only after the entire mechanical translation is present:

1. wire all translated modules into the crate;
2. run one full `cargo check` for the renderer's native-Metal feature set;
3. save and group diagnostics by Rust owner/source range;
4. fix ownership, type, lifetime, native-selector, and configuration errors in
   dependency order;
5. repeat until Apple and portable configuration checks are warning-clean
   relative to the recorded baseline;
6. reject fixes that remove translated source behavior merely to compile.

Compiler errors are the queue. Fixtures are still not the queue.

## Phase 4: behavior work queue

After compilation, verify the whole backend in this order.

### 4.1 Structural/source verification

- Reconcile every line-map row with compiled Rust owners.
- Verify all header fields against the ownership ledger.
- Check enum, flag, key, shader-function, binding-slot, attachment, and
  load/store tables exhaustively.
- Verify render-pass, draw, upload, allocation, barrier, submission, and
  completion ordering against pinned source.

### 4.2 Lifecycle and failure verification

Exercise construction failure order, cache failure and fallback, resource
replacement, abandonment, concurrent in-flight rings, command failure,
context drop before completion, and destruction order. Safe-Rust adaptations
must retain the upstream invariant and fail closed.

### 4.3 Language-independent renderer verification

Run the existing same-runner C++ Metal corpus, static references, structural
oracles, and work counters. At this phase fixtures expose defects in the
complete translation. Fixes must be made in the source-corresponding owner,
not as a new fixture branch.

### 4.4 Platform matrix

Verify every configuration represented in the pinned source and available in
the campaign environment:

- Apple Silicon macOS;
- Intel/discrete macOS;
- oldest supported macOS barrier/pass-break behavior;
- iOS device and simulator;
- tvOS and visionOS, or an explicit checked configuration exclusion.

### 4.5 Product and distribution gates

- Execute the rooted native Metal product path.
- Prove the selected artifact and Cargo graph contain no WGPU, Naga, Dawn, or
  hidden shader-translation route.
- Record Mach-O size, linked libraries, shader inventory, and exact source
  provenance.

## Phase 5: independent review and promotion

Two reviews are required after behavior verification:

1. source/spec review compares the complete pinned source unit against the
   Rust translation without relying on implementation rationale;
2. ownership/standards review attacks lifetime, unsafe, threading,
   publication, failure, and repository-evidence claims.

Accepted findings are fixed in the source-corresponding owner and the complete
verification queue is rerun as appropriate.

Promotion rules:

- subrange evidence may improve the line map but does not complete the owner;
- the header and implementation remain `in-progress` while any line, field,
  configuration, or reachable branch is partial or missing;
- `ported` requires complete compiled ownership plus focused real-crate
  evidence;
- `verified` additionally requires the issue-level platform/product evidence.

## Progress reporting

Progress reports use source and ownership measures, never a feature list:

- source files and line ranges translated versus total;
- header fields with complete lifetime rows versus total;
- missing/partial/ported line-map counts;
- compiler diagnostics remaining by owner;
- behavior gates executed and exact result counts;
- platform configurations executed or explicitly excluded;
- review findings open and closed;
- final dependency and artifact-size evidence.

Phrases such as “next fixture,” “next rendering feature,” or “smallest pixel
slice” indicate process drift and must not be used to select implementation
work.
