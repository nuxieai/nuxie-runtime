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

## Bun operating model

This campaign deliberately follows the operating model described in
[Bun's Rust rewrite](https://bun.com/blog/bun-in-rust). The relevant lesson is
not merely "use multiple agents." It is a strict separation of work queues,
roles, and evidence so a large rewrite remains a translation rather than
turning into a sequence of redesigns.

The canonical campaign is a sequence of global passes, not a per-file serial
loop:

```text
all 111 pinned files -> parallel Luna xhigh mechanical translation
all translated files -> Sol source-semantics adversarial pass
all source-reviewed files -> independent Sol ownership/lifetime pass
all reviewed files -> Sol correction pass
complete translated tree -> compiler queue -> V0-V9 parity
```

The 111 files are the primary translation denominator. The 41 manifest units
are only dependency/integration groups used to aggregate receipts and later
compiler diagnostics; they are not substitutes for file coverage. Luna workers
claim non-overlapping individual files in parallel waves. Multi-file groups
become translated only after every owned file has a source-shaped target and
file evidence. Reviews do not block later translation work. Each later review
pass is performed from the source and diff without inheriting translator
rationale, and assumes the translation is wrong.

### Why the port is all at once

An incremental backend rewrite creates temporary adapters, duplicated policy,
and behavior that exists only to cross the gap between old and new systems.
Those seams are exactly what led this campaign into fixture-specific native
paths. The Bun approach instead writes the complete source-shaped translation
before asking whether it compiles or passes the behavior suite. The initial
Rust is expected to resemble mechanically translated C++/Objective-C++, not an
idiomatic redesign.

For this renderer that means:

- the queue contains complete pinned source owners and source ranges;
- the queue never contains gradients, clips, blends, images, or individual
  corpus fixtures;
- all upstream branches are translated even when no current fixture reaches
  them;
- temporary Rust architecture intended to support only a subset is not a
  completion mechanism;
- refactoring, deduplication, abstraction cleanup, and unsafe reduction happen
  only after the complete parity matrix is green.

### Preparation before translation

Bun prepared a language mapping and an exhaustive lifetime ledger before the
bulk rewrite. This campaign's equivalent preparation artifacts are mandatory
translator inputs:

- `PARITY_WORKFLOW.md` and `METAL_PORTING.md`;
- `docs/render-context-metal-file-map.tsv`;
- `docs/render-context-metal-fields.tsv`;
- `docs/render-context-metal-configurations.tsv`;
- `docs/render-context-metal-dependencies.tsv`;
- `docs/render-context-metal-includes.tsv`;
- `docs/metal-translation-conventions.tsv`;
- `docs/metal-port-divergences.tsv`;
- `docs/metal-port-ownership.toml`;
- `docs/metal-port-manifest.toml`.

The field, configuration, translation, and ownership ledgers must be reviewed
together before a source owner is dispatched. Conflicting advice is corrected
in the ledgers first. The translator must not resolve that conflict by
inventing a local architecture.

### Role contract

#### Luna xhigh: mechanical translator

Luna xhigh is the dedicated C++/Objective-C++ to Rust translator. For each
assignment Luna receives the complete pinned source owner, its direct
dependencies, the line/field/configuration ledgers, and the frozen translation
conventions.

Luna must:

- translate every declaration, field, branch, side effect, ordering
  constraint, error route, and conditional-compilation branch in scope;
- preserve source naming and control-flow shape when Rust permits it;
- preserve intentional upstream quirks unless a divergence is already
  recorded and authorized;
- identify missing dependencies as new source owners instead of replacing them
  with fixture helpers;
- report exact source coverage and unresolved mechanical blockers;
- stop at the assigned owner boundary.

Luna must not:

- choose work from a failing image, feature, draw family, or tracer fixture;
- run a self-review or promote its own translation;
- add stubs, placeholder success, swallowed errors, fallback, or no-op bodies
  to make compilation easier;
- refactor toward an idiomatic or shared GPU architecture during translation;
- run broad Cargo/test queues while the bulk translation queue is active,
  unless Sol explicitly requests a narrow syntax check for integration safety;
- use broad Git state operations or alter files outside the declared owner.

#### Sol: orchestrator, adversarial reviewer, fixer, and driver

Sol owns campaign sequencing and integration. Sol selects the next complete
source owner from the manifest, provides Luna with the exact authority bundle,
protects file ownership, and records translation progress.

Only after the complete 111-file translation pass, Sol performs two global,
logically separate adversarial review passes:

1. **Source-semantics review** compares the diff to the pinned source and
   attacks control flow, evaluation order, error order, byte layout, flags,
   native calls, configuration branches, and source-visible quirks.
2. **Ownership review** attacks lifetimes, retain/release transfer, completion,
   thread confinement, drop order, unsafe invariants, failure publication, and
   safe-Rust adaptations.

After both full review passes, Sol applies or drives a separate correction pass
and reruns the affected review contexts. Sol does not accept an explanation
merely because the translator's reasoning sounds plausible. The source,
ledgers, and executable evidence are the authorities.

Role names in the manifest are not execution evidence. Advancement is backed
by the four checked-in receipt types defined in
`docs/metal-port-receipts/README.md`: Luna translation, Sol source review, Sol
ownership review, and Sol correction. A unit with missing receipts or open
findings stays red even if an older file happens to compile.

After every source owner is translated and reviewed, Sol changes roles from
translation orchestrator to queue driver:

- run the compiler once and save diagnostics;
- group compiler errors by source owner and dependency order;
- fix each group, adversarially review it twice, and apply corrections;
- obtain a rooted executable smoke path only after compilation is green;
- run the V0-V9 behavior, parity, platform, and product queues;
- group each failure by its source-corresponding owner rather than adding a
  feature-specific path;
- perform final independent source/spec and ownership/standards closeout.

### The nine ordered queues

The queues do not interleave merely because a later command is available:

1. **Preparation:** exhaustive source, field, configuration, lifetime,
   ownership, and translation-convention authority.
2. **Bulk translation:** parallel Luna workers translate all 111 complete pinned
   files. No adversarial review, correction, compiler, Cargo, or behavior work
   blocks this pass, and the renderer is not expected to work yet. Sol stages
   and counts each completed source-shaped target, but batches detailed receipt
   reconciliation and reverse-ledger promotion at the Phase 1 boundary rather
   than turning every file into its own closure loop.
3. **Source review:** every translated file receives the source-semantics
   adversarial pass.
4. **Ownership review:** every source-reviewed file receives the independent
   ownership/lifetime/ABI pass.
5. **Correction:** findings from both passes are resolved by source owner and
   affected review contexts rerun clean.
6. **Compiler queue:** Sol runs compilation once, saves diagnostics, groups
   them by owner, and drives the count to zero without deleting behavior.
7. **Rooted smoke queue:** link, construct the backend, render through the
   general source-shaped path, submit, complete, and read back.
8. **Behavior queue:** execute V0-V9, the full C++ Metal oracle, the full WGPU
   differential, lifecycle/failure suites, platform/configuration matrix,
   rooted no-fallback proof, logs, and rendered/diff images.
9. **Post-green cleanup:** only after parity, reduce unsafe, deduplicate,
   improve Rust idioms, and simplify interfaces while continuously rerunning
   the complete suite.

### False-start rules

The Bun rewrite exposed process failures that this campaign treats as hard
errors:

- no `git stash`, `git reset`, destructive cleanup, or broad Git operation;
- no overlapping file ownership between translator assignments;
- no repeated Cargo/test runs inside the bulk translator loop;
- no stubbing functions merely to clear compiler errors;
- no deleting, skipping, weakening, or relabeling tests;
- no paragraph-long comment whose purpose is to justify a workaround instead
  of implementing the source behavior;
- no changing tolerances from candidate output;
- no feature-specific adapter that bypasses the incomplete general owner;
- no cleanup/refactor work mixed into the literal translation.

### Adversarial semantic checklist

Compiling code can still be behaviorally wrong. Sol's source review explicitly
checks the classes of translation mistake highlighted by the Bun rewrite and
their renderer-specific equivalents:

- side effects accidentally placed inside debug-only assertions;
- eager evaluation replacing lazy source behavior;
- signed rounding, truncation, overflow, and integer-width differences;
- slice/cast/alignment behavior for odd or malformed byte lengths;
- changed bounds, capacities, sentinels, placeholder sizes, or array limits;
- compile-time source transformation accidentally moved to runtime;
- callback, completion, or asynchronous-close owners dropped too early;
- C++ reverse-member destruction changed by Rust field order;
- nullable Objective-C returns converted into panic paths;
- current-versus-submitted descriptor state confused while draining work;
- configuration branches silently collapsed into the host configuration;
- assertions with source side effects converted into erased Rust assertions.

### Green means the process completed, not that samples look good

No source owner is complete solely because a corpus image passes. The campaign
is complete only after all source owners are translated, every translation has
both adversarial reviews and its fixes, compiler diagnostics are zero, the
rooted backend runs, V0-V9 is green across the declared matrix, no test is
skipped or deleted, and the final independent reviews are clean.

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

The four paths under `renderer/src/shaders/metal` are not a closed shader
owner by themselves. The pinned Makefile invokes `minify.py` once over the
complete wildcard-expanded batch of 34 root `.glsl`, `.vert`, and `.frag`
files. Identifier counts and renamed exports are global to that batch, so a
source that is not directly included by one `.metal` file can still change the
Metal artifact. `docs/metal-shader-source-inventory.tsv` therefore pins all 40
artifact-authority files in their exact Make expansion order: the Makefile,
minifier, 34 batch inputs, draw-combination generator, and three Metal entry
sources. All 40 are full translation sources, not informal dependencies.

`metal-shader-source-batch` is renderer dispatch ordinal 32, after the complete
ORE and generic source foundations. The Metal API, background compiler, and
Metal implementation owners are ordinals 33 through 35. The mechanically
admitted path, paint, and renderer owners are ordinals 36 through 38 as
separate whole-file translation units. The Factory, Gradient, and
RiveRenderFactory owners at ordinals 39 through 41
are the final source-shaped product-route closure. Luna xhigh owns
the complete batch; separate Sol source and ownership review receipts and the
Sol correction receipt remain pending until the batch is mechanically carried
and reviewed. An existing metallib or copied minified file is historical
evidence only and does not advance those receipts.

`docs/metal-port-manifest.toml` remains the source-of-truth inventory.
`docs/render-context-metal-file-map.tsv` partitions every line in the primary
header and implementation, but it is a completeness ledger rather than a
feature queue.

`docs/render-context-metal-fields.tsv` separately accounts for all 455
state-bearing declarations inherited by the complete generic renderer and
native Metal owners. This includes the 208 shared `gpu.hpp` layout fields, the
142 `RenderContext`/`LogicalFlush`/`TessellationWriter` fields, and the
reference-counting, image, sampler, ASTC, canvas, helper, and platform fields.
The campaign checker extracts the pinned declarations and rejects missing,
duplicate, invented, configuration-drifted, or line-drifted rows.

`docs/metal-port-preprocessor-authority.tsv` accounts for every semantic
conditional block and every branch entry in all 111 pinned sources. The campaign
checker derives exactly 634 blocks and 845 branch entries; exactly six
canonical outer header guards are excluded because they prevent duplicate
inclusion rather than selecting renderer behavior. The 82-row
`docs/render-context-metal-configurations.tsv` remains a compatibility view of
the original context/dependency subset, with all 82 translation states pending
and receipt-gated to the same owning units. An omitted platform,
simulator, tools, canvas, decoder, KTX2, testing, debug, compiler, Emscripten,
feather-LUT, or disabled-diagnostic configuration fails the campaign gate.
Configuration accounting deliberately has two axes. `mapping_status` records
whether the source predicate, intended Rust owner, and exact platform semantics
are frozen for mechanical translation. `translation_status` remains pending
until that mapping is implemented and evidenced. Preparation closes only the
first axis; Luna owns the second during the whole-source pass. This prevents a
known mapping from being mistaken for working code.

`docs/render-context-metal-dependencies.tsv` freezes the 22 complete generic
source owners needed by the renderer field graph. Its source roles and
citations describe complete files, never useful subranges. The dependency
authority includes the inherited `refcnt.hpp`, `lite_rtti.hpp`,
`buffer_ring.hpp`, and `rive_render_image.hpp` owners that were absent from the
first preparation denominator. Every row freezes its unique translation unit,
source-shaped `mechanical_port/source` target, independent mapping and
translation statuses, and exact cross-link into the 455-field lifetime
authority (or an explicit state-free/source-static disposition).

`docs/metal-port-include-authority.tsv` accounts for all 366 direct `#include`
and Objective-C `#import` occurrences across 133 canonical tokens and 61 files.
The 113-occurrence/66-token `docs/render-context-metal-includes.tsv` remains a
compatibility view of the original 22 generic sources. Every exhaustive
occurrence records its active configuration, resolved
pinned source or system/generated origin, and one exact translation unit,
existing Rust owner, generated-artifact batch, or toolchain adaptation. Missing
or invented occurrences and unresolved owners keep Preparation red.

Every configuration also carries a disposition. All pinned branches currently
have `translation_disposition=required`; host hardware availability may change
how a branch is validated, never whether Luna translates it. Executable,
compile/link-only, and checked-exclusion are validation outcomes, not work
selection mechanisms.

The frozen Apple platform correspondence is source-shaped and seven-way:
`IosDevice`, `IosSimulator`, `XrosDevice`, `XrosSimulator`,
`AppleTvOsDevice`, `AppleTvOsSimulator`, and `MacOs`. Derived predicates retain
the pinned distinctions: physical mobile, mobile simulator, physical iOS only,
iOS family, embedded Apple, and macOS. In particular, `TARGET_OS_IPHONE`
includes iOS, tvOS, and visionOS SDK families, so BC7 is macOS-only; using
`not(target_os="ios")` would be a semantic mistranslation.

`docs/metal-translation-conventions.tsv` freezes the eight mechanical mappings
used during the literal source pass: Objective-C retention/nullability,
intrusive ownership, byte ranges/alignment, enums/flags/generated slots,
assertions/errors, callbacks/completion, preprocessor configurations, and
destruction/drop order. These rules prohibit source-convenient redesign while
the owner is being translated.

`docs/metal-port-divergences.tsv` is the adversarial disposition queue for any
Rust behavior that is safer or deeper than a literal translation. A safety
argument alone does not approve a redesign. Each row records pinned source
behavior, Rust behavior, observability, separate Sol source and ownership
review receipts, and the correction receipt. Until both reviews accept a row,
Luna treats the pinned source behavior as authoritative.

At plan adoption, the line ledger reports:

| Source | Ported ranges | Partial ranges | Missing ranges |
| --- | ---: | ---: | ---: |
| `render_context_metal_impl.h` | 5 | 3 | 0 |
| `render_context_metal_impl.mm` | 30 | 14 | 4 |

All 111 primary manifest rows and all 41 translation units are `in-progress`; no
mechanical translation or canonical receipt is claimed at plan adoption.
Historical and uncommitted Rust code is comparison evidence only and is
evaluated as part of the complete source-owned translation after dispatch.

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

The complete validation matrix and the distinction between the primary C++
Metal oracle and the secondary Rust-WGPU differential are specified in
`METAL_RENDERER_VALIDATION.md`. In particular, the current four-row WGPU
secondary corpus is regression evidence only. Whole-renderer promotion
requires the complete Metal-compatible `clockwise-atomic` corpus differential,
which currently contains 736 rows.

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

## Continually updated progress dashboard

`docs/metal-renderer-progress.html` is the visual campaign record. It is
generated by `tools/metal-port/generate_progress.py`, not manually edited.

Its red, amber, and green state comes directly from:

- `docs/render-context-metal-file-map.tsv` for line-weighted source closure;
- `docs/render-context-metal-fields.tsv` for state-bearing field preparation;
- `docs/metal-port-manifest.toml` for complete source-file state;
- `docs/metal-port-ownership.toml` for lifetime/owner state;
- `docs/metal-renderer-progress.toml` for the active phase and dated reports;
- `docs/METAL_RENDERER_VALIDATION.md` and the validation-suite records in
  `docs/metal-renderer-progress.toml` for executable exit gates;
- the Metal corpus manifests for checked-in rendered evidence.

The dashboard must be regenerated whenever any of those inputs changes:

```text
make metal-port-progress
```

`make metal-port-progress-check` deterministically regenerates the page and
fails when the checked-in copy is stale. `make metal-port-check` includes that
staleness gate.

Every campaign checkpoint adds a dated progress report containing:

- the exact source/ownership counts that changed;
- commands executed and their complete result summaries;
- links to retained raw logs or evidence reports;
- the commit or source state that produced the evidence;
- open failures and the exact source owners responsible for them.

Behavior-verification checkpoints also retain images. For image comparisons,
archive or link the pinned reference, Rust Metal output, and a visual diff when
the output is not byte-exact. The dashboard may display existing corpus images
during translation, but labels them as regression evidence rather than
implementation scope. A green image never changes an amber or red source row.

The dashboard is updated through every phase until all source, ownership,
platform, behavior, and product gates are green. It is not a substitute for
the machine-readable ledgers or raw evidence.
