# Native Metal renderer port: process postmortem

Date: 2026-08-22  
Scope: UNIV-1643 and the native Metal work through UNIV-2091  
Upstream authority: Rive `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Executive summary

The 39-hour native Metal port succeeded, but not by following one plan from
start to finish. It succeeded because the process was willing to invalidate
its own apparent progress.

The first half of the work used issue slices, named renderer fixtures, focused
tests, and locally complete resource owners. That produced real value: a live
Metal path, useful lifecycle machinery, exact atomic-rendering checkpoints,
and a rooted Mach-O with no runtime WGPU fallback. It also produced the wrong
shape of confidence. At the last checkpoint before the process reset, 24
native tracer tests passed, eight C++ Metal comparisons were byte-exact, and
the rooted binary scan was clean, while the full renderer owner was still
partial and surrounded by duplicated policy and fixture-shaped paths.

The decisive change was to stop treating visible rendering behavior as the
implementation queue. The campaign switched to complete pinned source owners,
then enforced global phase barriers:

```text
inventory -> mechanical translation -> source review -> ownership review
          -> correction -> compiler -> rooted execution -> V0-V9 -> rereview
```

That reset exposed 60 raw defects in the first two independent review passes,
including 14 P0 findings. Two primary implementation files were comment-only
shells; retained native owners were zero-sized markers; several safe APIs hid
unbounded raw-pointer operations; configuration branches were missing; and
early “safer” Rust adaptations had changed publication, fallback, and teardown
semantics.

The final result is strong: 111 pinned files in 41 translation units are
verified, 993 required-live tests pass, all nine Apple compile configurations
are warning-clean, 736/736 native-Metal-compatible corpus rows match pinned C++
Metal, and the rooted product graph and binary contain no WGPU/Naga/Dawn
fallback. The main lesson is not that an all-at-once rewrite is universally
better. It is that the unit of implementation and the unit of verification
must match the authority being ported. For this renderer, that unit was the
source owner, not the feature or fixture.

## How this postmortem was reconstructed

This account uses the merged 54-commit history, the source and ownership
ledgers, the dated progress records, 264 checked receipt files, the two red
adversarial-review reports, and the V0-V9 closeout reports. The 39 hours are the
active-work figure for the port. Git timestamps span 78 hours 20 minutes, from
the first campaign scaffold at 2026-08-19 14:31 PDT to the final integration
commit at 2026-08-22 20:52 PDT.

The distinction matters. Git records publication checkpoints, not active
agent time, and the final 52.5 wall-clock hours were collapsed into one large
commit. The repository supports a reliable phase-level reconstruction, but
not a minute-by-minute allocation of translator, reviewer, correction, build,
and test time.

## Outcome at a glance

| Measure | Result |
| --- | ---: |
| Active effort | 39 hours |
| Git wall-clock span | 78 hours 20 minutes |
| Commits | 54 |
| Final cumulative change | 604 files, 186,555 additions, 664 deletions |
| Source-shaped mechanical modules | 120 files, 105,617 additions |
| Pinned source denominator | 111 files in 41 translation units |
| State-bearing declarations | 455 |
| Preprocessor authority | 634 blocks, 845 branch entries |
| Include/import authority | 366 occurrences, 133 canonical tokens |
| Receipt evidence | 264 TOML receipts, 1,446 evidence items |
| Initial independent review findings | 60: 14 P0, 26 P1, 17 P2, 3 P3 |
| Required-live validation | 993 passed, 0 failed, 40 declared ignores |
| Pinned C++ Metal parity | 736/736 accepted; 698 byte-identical |
| Apple configuration matrix | 9/9 warning-clean |
| Rooted product | 986,880-byte arm64 Mach-O; no forbidden backend route |

This closed the native renderer's mechanical translation and validation
campaign. It did not change the shipping Apple backend selector; product
cutover remains the separate UNIV-2092 decision.

## Timeline

| Period | Commits | What happened | Disposition |
| --- | ---: | --- | --- |
| Aug 19, 14:31–16:39 | 8 | Campaign scaffold, pinned oracle, native tracer, initial semantic fixes | Good foundation, incomplete work model |
| Aug 19, 17:24–Aug 20, 06:08 | 22 | Native Metal resource, lifecycle, atlas, and atomic feature slices | Real checkpoints, but feature/fixture-driven |
| Aug 20, 06:55–15:24 | 17 | File-first ORE workflow, ORE owners, product roots, then more renderer slices | First successful owner-oriented correction |
| Aug 20, 15:38–16:22 | 6 | Whole-renderer plan, dashboard, V0-V9 contract, exhaustive field/configuration/convention ledgers | Full process reset |
| Aug 20, 16:22–Aug 22, 20:52 | 1 integration commit | Bulk translation, two red reviews, corrections, compiler work, V0-V9, repeated promotion stops, final rereviews | Complete and green |

The commit distribution tells its own story. Before the whole-renderer reset,
47 incremental commits added 46,911 lines. The six preparation commits added
3,163 lines of control-plane material. The final integration commit changed
517 files with 139,587 additions and 3,366 deletions.

## What happened

### 1. The campaign began with the right authority and the wrong queue

The initial guide got several foundational decisions right:

- pin one upstream Rive revision;
- treat C++ Metal as the primary correctness oracle;
- keep Rust-WGPU as a secondary diagnostic only;
- preserve source naming, control flow, ownership, and failure order;
- forbid silent WGPU fallback and premature shipping cutover;
- keep renderer-platform Metal and ORE Metal as separate concrete seams.

But the implementation workflow contradicted those principles. Work proceeded
in issue order through “first resource,” upload-ring, feather-atlas, batching,
and generic-atomic slices. Named fixtures selected the next behavior. Focused
tests and a source row with local evidence could promote a slice, while a
source review was optional and time-boxed.

That made it possible to be locally faithful and globally wrong. A feature
slice needed enough resource and policy machinery to execute before the full
source owner existed, so it naturally grew temporary adapters, duplicated
state, and publication rules. The code was often reasonable Rust. The problem
was that it was not always the same program as the pinned C++ owner.

### 2. Narrow green checkpoints concealed global incompleteness

The last fully verified pre-reset checkpoint was not fake. It had:

- 24/24 native Metal tracer tests;
- 8/8 byte-exact same-runner C++ Metal comparisons;
- a rooted 802,576-byte Mach-O that passed the forbidden-dependency scan;
- a green manifest and ownership checker for the denominator then known.

Those checks proved the implemented paths. They did not prove that the
denominator was complete. At plan adoption, the primary Metal header still had
partial ranges, the implementation had four missing ranges and fourteen
partial ranges, and the renderer depended on generic owners not represented in
the original campaign graph.

The process had confused “all tests for the current slice pass” with “the
source-owned backend is complete.” The dashboard correction explicitly
reclassified those green images as regression evidence rather than progress
authority.

### 3. ORE demonstrated the better unit of work

The first process correction happened before the full reset. The ORE phase
adopted file-oriented, dependency-ordered ownership units. Translators received
exclusive source sets; the integrator wired modules later; fixtures validated
completed batches instead of choosing behavior to implement.

This worked better because ORE's file and ownership boundaries were visible
and comparatively tractable. It produced canonical context, buffer, texture,
pipeline, bind-group, render-pass, and authenticated GPU-canvas roots without
inventing a cross-backend HAL.

The ORE experience became the proof that the same discipline needed to govern
the main renderer, where the temptation to chase visible image differences was
much stronger.

### 4. The whole-renderer reset changed the denominator and the order of work

The reset froze four things before more behavior work:

1. **A complete denominator.** The first preparation audit expanded the
   campaign from a partial renderer/ORE view to complete generic, shader-build,
   and native Metal authorities. The final denominator reached 111 files and
   41 units.
2. **A complete state model.** The field ledger grew from a small Metal-only
   inventory to 455 direct, inherited, and reachable declarations.
3. **Configuration and build authority.** Platform predicates, 40 globally
   minified shader inputs, includes/imports, generated artifacts, and dispatch
   prerequisites became explicit data rather than ambient knowledge.
4. **Role and phase separation.** Mechanical translation, source review,
   ownership review, correction, compilation, and behavior verification became
   separate global queues.

This was the most important decision in the port. It removed the feedback loop
that had made every failing fixture an invitation to add another local path.
Compiler errors and pixels were deliberately postponed until the complete
source-shaped tree existed. The queue and role separation was explicitly
adapted from [Bun's Rust rewrite](https://bun.com/blog/bun-in-rust), with local
ledgers and native-ownership reviews added for this renderer.

### 5. Mechanical translation produced coverage, not correctness

The first global source review covered 98 then-known source-target pairs and
found 28 defects: 2 P0, 12 P1, 11 P2, and 3 P3. The two P0s were stark:
`render_context.cpp` and `render_context_metal_impl.mm` had targets whose
purported implementations were inert comments.

Other findings included:

- a shader Makefile represented as provenance rather than executable build
  behavior;
- a translated lexer using regex features unsupported by Rust's regex engine;
- disconnected declaration and implementation owner types;
- nonexistent Cargo features standing in for real platform branches;
- empty or uninhabited placeholders for shared renderer types;
- lost virtual dispatch and Objective-C exception publication;
- changed integer wrapping, defaults, assertion modes, and move semantics.

The second, independent ownership/lifetime/ABI review found another 32 defects:
12 P0, 14 P1, and 6 P2. Its P0s included zero-sized retained native owners,
missing command-buffer completion ownership, a dropped retain during a move,
raw back-pointers into movable values, safe APIs manufacturing references from
arbitrary pointers, invalid intrusive-owner casts, and lifetime-untracked ORE
raw pointers.

The lesson is subtle: postponing compilation was useful because it kept the
translation queue source-shaped, but source coverage alone was not a quality
gate. “Mechanical” could still mean comment copying, placeholder types, or a
syntactically plausible ownership model. The independent reviews were not
polish. They were part of implementation.

### 6. Corrections removed early architectural inventions

Six explicit divergence records capture the main ways the slice-driven design
had become a different program:

- transactional publication of the three atomic planes;
- eager immutable command-queue injection;
- a copied deep context-options cache owner;
- transactional aggregate resource generations;
- paired atlas texture/pipeline publication;
- a deep standalone compatible-pipeline cache.

Each design had a defensible safety or modularity rationale. Each also changed
observable allocation timing, retry behavior, cache warmth, identity,
nullability, replacement, or teardown. The final implementation de-rooted
those parallel owners and restored the pinned source order. Safer Rust
adaptations remained only where they preserved the same externally observable
contract and had explicit evidence.

The final fix receipts contain 35 non-divergence resolution entries. Several
close families of raw findings were corrected through one source-owned cause,
but none of their observable acceptance requirements was dropped.

### 7. Green was repeatedly revoked during closeout

The evidence system's strongest result was not a test pass. It was its ability
to say “still red” after impressive passes:

- After the canonical runtime suites and most of V5-V8 passed, V8 remained red
  because the native-only Cargo graph still compiled 20 WGPU/Naga-family rows,
  even though the rooted Mach-O scan itself was clean.
- After 736/736 pinned C++ Metal rows passed, V9 remained red because active
  source ownership was split across executor and host wrappers and mandatory
  promotion receipts were absent.
- After owner flattening, a fresh pinned-source review remained red because
  the Metal context still copied the source flush graph into a second DTO
  universe and delegated source-owned platform, capability, and lifetime
  choices to adapters.

These were not bureaucratic failures. They were examples of different
evidence lanes observing different classes of truth. Pixel parity could not
prove owner identity. A final binary scan could not prove that forbidden code
was absent from the build graph. Compilation could not prove Objective-C local
lifetime or selector roles.

### 8. The last defect justified the final frozen-byte rereview

The closing bug was one missing Objective-C message receiver.
`generateMipmapsForTexture:` passed only the texture handle to the execution
adapter, which required argument zero to be the blit encoder and argument one
to be the texture. The call was syntactically valid and most images did not
expose it. A downscaled transparent-image comparison did.

The corrected sequence passes `[encoder, texture]`, then performs
`endEncoding`, clears the dirty flag, and releases the encoder in the pinned
order. The focused regression passed, and the revealing image became
byte-exact with zero differing pixels.

This defect is a compact example of why final review must inspect the exact
bytes that passed parity. The relevant semantic unit was not “mipmaps work.” It
was receiver, argument roles, pre-pass placement, dirty-state transition, and
native-owner release order.

## What worked well

### The primary oracle never moved

Pinned C++ Metal remained the acceptance authority throughout. Rust-WGPU ran
all 736 compatible rows as a useful diagnostic and retained 58 WGPU-only pixel
differences, but it could neither approve nor reject Metal behavior. This
prevented the candidate from drifting toward the existing Rust backend simply
because that implementation was easier to compare.

### Review independence was real

The source-semantics and ownership/lifetime/ABI passes used separate review
contexts, made no edits, and did not use compiler errors or fixtures to choose
what to inspect. They found different defect populations. Combining them into
one general “code review” would almost certainly have hidden ownership and ABI
issues behind the already-large source comparison.

### The campaign tolerated red states

Several apparent finishes were revoked. That is evidence the status model was
doing useful work rather than narrating success. A green behavior lane never
overrode a red source, graph, ownership, or promotion lane.

### Machine-readable authority scaled better than prose

The final checker can independently derive and mutation-test source coverage,
field ownership, configurations, includes/imports, dispatch prerequisites,
assertion semantics, compiled target roots, test census, and receipt digests.
The dashboard is generated from those authorities rather than hand-colored.
This made “complete” falsifiable.

### Rooted-product verification caught a distinct class of failure

The port did not stop at library tests or images. It linked and ran the actual
native path, scanned its Cargo graphs, linked libraries, symbols, and binary
tokens, and required real Metal drawable markers. That separated “the backend
works in tests” from “the selected product cannot secretly reach the old
backend.”

## What did not work well

### The initial unit of work was too small in the wrong dimension

The early slices were small by feature, not by source ownership. That made them
easy to demonstrate but forced them to reconstruct fragments of a larger state
machine. The result was duplicate DTOs, cache owners, resource generations,
and publication policy that later had to be de-rooted.

### The original promotion rule was too permissive

The first guide allowed focused evidence to promote a source row and made
independent source review optional. For a renderer with native ownership,
configuration, ABI, and asynchronous completion semantics, tests cannot
observe enough of the contract to make that safe.

### Translation admission did not reject inert or fake owners early enough

The bulk queue accepted comment-only bodies, disconnected types, and zero-sized
owner markers into the review barrier. The later reviews caught them, but the
translator handoff should have rejected obvious non-executable or
non-owning representations before counting file coverage as complete.

### Evidence granularity was better than commit granularity

The final campaign had excellent per-unit receipts but poor Git checkpointing.
The last integration commit contained 139,587 additions across 517 files. That
made ordinary diff review, bisection, and temporal reconstruction harder than
necessary. Global phase barriers do not require a single final commit; the
translation barrier, each review/correction barrier, compiler green, and
parity green could each have been immutable commits.

### The evidence layer was expensive

The repository contains 264 receipt TOMLs and 7,110 receipt lines, plus large
ledgers and reports. That expense bought real protection, especially against
premature promotion, but much of the data is repetitive. Future campaigns
should preserve the same invariants while generating more of the receipt
surface from a normalized event log or attestation database.

### Timing data was not captured at the phase level

The repository proves what happened and in what order, but not how the 39
active hours split among translation, review, correction, compilation, and
GPU validation. Without that data, this postmortem cannot quantify the cost of
the false start or the return on each evidence lane.

### Hardware closure is still asymmetric

The final matrix compiled all nine Apple configurations and preserved Intel,
AMD, old-macOS, device, and simulator policy. Live execution happened on an
Apple M5 Max. Intel and discrete-GPU behavior was compile- and policy-checked,
not executed on matching hardware. That is an honest limitation, not a hidden
green.

## Root cause

The primary process failure was a mismatch between the unit of implementation
and the unit of authority.

The authority was a set of complete, stateful C++/Objective-C++ owners whose
behavior depended on field identity, construction and destruction order,
configuration branches, asynchronous completion, and globally generated
shader artifacts. The initial implementation unit was a visible feature or
fixture. Every slice therefore had to invent enough of the missing owner to
run, and those inventions accumulated into a parallel architecture.

Contributing factors were:

- an incomplete initial denominator;
- tests acting as both work selector and promotion gate;
- optional rather than mandatory independent review;
- “safer Rust” changes accepted before their observable equivalence was
  established;
- checks that proved the rooted binary but not initially the complete build
  graph;
- translation coverage that could count inert comments and placeholder owner
  shapes;
- insufficient phase-level Git and timing checkpoints.

The complexity of Rust/Objective-C ownership and Metal synchronization made
the consequences severe, but it was not the root cause. The process allowed
locally successful code to outrun the model it was supposed to translate.

## Lessons for the next port

1. **Choose the authority denominator before writing behavior.** Inventory
   complete source files, generated inputs, configuration branches, and
   state-bearing fields first. If the denominator expands during translation,
   stop and repair preparation.
2. **Use source owners as work items and fixtures as later probes.** Small work
   is still possible through non-overlapping line ranges, but completion
   belongs to the whole owner.
3. **Treat translation, review, correction, compilation, and behavior as
   different cognitive jobs.** Mixing them encourages compiler- or
   fixture-driven deletion of source behavior.
4. **Make independent source and ownership reviews mandatory.** Native ports
   need both. Neither compilation nor pixels can substitute for them.
5. **Require executable ownership at translator handoff.** Comment-only bodies,
   zero-sized native owners, disconnected declaration/implementation types,
   and placeholder success paths should fail before the review barrier.
6. **Run graph and rooted-binary checks.** They answer different questions and
   both are necessary.
7. **Freeze and rereview the bytes that passed parity.** Corrections made during
   compiler and behavior work can reintroduce source or ownership drift.
8. **Preserve progress in Git at phase barriers.** Evidence receipts should
   complement reviewable commits, not replace them.
9. **Record queue timing automatically.** Active time per phase, rerun count,
   and compute time should be first-class postmortem data.
10. **Defer cleanup, but make the debt explicit.** Mechanical, source-shaped
    Rust was the right closeout target. Idiomatic consolidation and unsafe
    reduction are separate post-green work, not something to forget.

## Follow-up actions

### P0 — before another large native port

- Add a reusable preparation gate that rejects translation until source,
  field, configuration, include, generated-input, and ownership denominators
  are exhaustive.
- Extend translation admission checks to reject comment-only function bodies,
  constructible placeholder owner types, disconnected declaration/definition
  identities, and compiler-inert targets.
- Require immutable commits at the translation, source-review correction,
  ownership-review correction, compiler-green, and parity-green barriers.
- Run the forbidden Cargo graph gate with the first rooted executable and
  again at final V8 closure.

### P1 — improve safety and evidence efficiency

- Replace untyped Objective-C selector argument assembly with typed receiver
  and argument-role helpers where practical; retain the mipmap regression as a
  mutation-sensitive test.
- Record phase start/end times, agent/reviewer run IDs, command duration, and
  rerun counts in the generated progress data.
- Generate repetitive receipt fields from a normalized campaign event log
  while retaining per-unit digests, independent review identities, citations,
  and replayable commands.
- Open explicit post-green work for unsafe-boundary reduction, deduplication,
  and idiomatic Rust cleanup, continuously guarded by V0-V9.

### P2 — broaden physical qualification

- Execute the V3/V4/V7 lifecycle, parity, and barrier-policy lanes on Intel
  and discrete AMD Macs when hardware is available; retain the current compile
  rows until then.

## Final assessment

The port was successful because the campaign eventually optimized for
falsifiability instead of momentum. It pinned one authority, separated
translation from judgment, made ownership a first-class review surface, and
allowed strong green signals to be overruled by a different red lane.

The avoidable cost came before that discipline was universal: feature slices
created a locally convincing parallel design, the first bulk translation
admitted obviously inert owners, and the final work was compressed into a
single enormous commit. The durable process is therefore not “always rewrite
everything at once.” It is:

> Freeze the complete authority, translate by owner, review by failure domain,
> and let no single kind of green claim completion.

## Primary evidence

- [Whole Metal renderer port plan](METAL_RENDERER_PORT_PLAN.md)
- [Validation contract](METAL_RENDERER_VALIDATION.md)
- [Initial source-semantics review](metal-port-reports/phase2-source-review.md)
- [Initial ownership/lifetime/ABI review](metal-port-reports/phase3-ownership-review.md)
- [V0-V9 closeout](metal-port-reports/v9-independent-closeout.md)
- [Mechanical translation receipt contract](metal-port-receipts/README.md)
- [Dated progress record](metal-renderer-progress.toml)
- [Metal mechanical-port guide](METAL_PORTING.md)
