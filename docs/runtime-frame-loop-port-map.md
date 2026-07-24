# Runtime Frame-Loop Port Map

This is the finite execution map for the C++-corresponding runtime frame-loop
port. It replaces benchmark-scene ranking as the work queue. Performance is a
wave-level verification oracle after structural correspondence, never the
source of the next slice.

Pinned source: `rive-app/rive-runtime` at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

## Boundary

The port begins at `StateMachineInstance::advanceAndApply`, follows events,
DataBind, layers, transitions, actions, animation/keyframes/resets, Artboard
dirt and update traversal, and live Artboard draw, then stops at the existing
`Renderer` / `RenderFactory` interface.

Renderer backends, wgpu, Dawn, shaders, atlases, tessellation, GPU batching,
renderer algorithms, and renderer API design are outside this map.

## Mechanical closure

`docs/runtime-frame-loop-ownership.toml` is executable:

- source-set globs expand against the pinned C++ checkout;
- every expanded file has its own ledger row naming its source set, wave,
  target Rust module(s), dynamic-reachability result, and disposition;
- every file row must also exist in `file-correspondence-manifest.toml`;
- no file may belong to two source sets;
- the exact expanded status counts are ratchets;
- every state-bearing member row names its C++ files, Rust owner, dependency
  wave, disposition, and—when closed—all eight lifecycle phases;
- `adapted` rows cite a binding AF/RF rule;
- `divergent-by-decision` rows cite a user-approved D-row;
- closed mode rejects pending/compensation rows, open gaps, and nonzero legacy
  ratchets.

The imported 41-owner runtime-drawing ledger is already closed. It remains part
of the total proof without reopening faithful draw-owner code.

## Dependency waves

| wave | C++ ownership boundary | retention boundary crossed |
|---|---|---|
| FL-A | Component, dependents, dirt, dependency order, transforms, constraints | ids/scans become retained owner links; one dirt source and one consume order |
| FL-B | KeyFrame, KeyedProperty, KeyedObject, LinearAnimation, AnimationReset, blend animations | authored timing, targets, and reset values live on their C++-corresponding owner |
| FL-C | StateMachineInstance, layers, transitions, conditions, actions, listeners, inputs | definitions and instance collections are retained once; advance performs no rediscovery |
| FL-D | Event, DataBind, DataContext, ViewModel, Artboard and nested settlement | dirty batches and next-frame event semantics flow through the owning Artboard/SMI |
| FL-E | live Artboard draw and the existing 41-owner draw ledger | update-owned state is read live through `Renderer`; no scene replay/materialization |

The order is binding: FL-A → FL-B → FL-C → FL-D → FL-E. A later wave may
start only when all of its dependency rows are closed and its floor run is
green.

## FL-0 evidence protocol

The canonical hot-loop corpus is the six pinned entries selected by
`PERF_CORPUS_IDS`: `advance_blend_mode`, `ai_assitant`, `align_target`,
`animated_clipping`, `animation_reset_cases`, and `spotify_kids_demo`.

FL-0 has three evidence layers:

1. Dynamic reachability from both runner binaries, recording concrete
   frame-loop functions/files for every entry and sample.
2. Deterministic work counts for owner construction, dirt add/consume, update
   passes, target/property application, collection clones/searches, owner
   lookups, geometry/text/layout rebuilds, allocations, event/DataBind batches,
   and clone/remount identity.
3. Static virtual-dispatch/dependency closure. The source sets are the union,
   so cold-but-valid frame-loop branches remain in scope.

The six source sets each carry their static-closure rationale. Their 337
explicit file rows mark 103 files dynamically reached and 234 cold in this
capture. A cold row remains in scope until pinned C++ proves that its virtual
branch cannot be reached from the frame-loop boundary; corpus coverage alone
can never remove it.

Timing profiles may explain a counter mismatch. They do not add, remove, or
reorder ownership rows.

The initial capture is committed as `docs/runtime-frame-loop-trace.json`;
the isolated, provenance-bound runner and capture contract is documented in
`tools/runtime-frame-loop-port/README.md`.
LLVM counters are reset after construction and immediately before the sample
loop, so the frame-only counts exclude import/instantiation. A second full-run
capture supplies construction counts. The clean Rust production ref is
`13aedd6d`; runner-only coverage hooks do not alter runtime behavior.

The initial high-level result is deliberately structural:

- ten advance/update/event/keyframe/layout/draw landmarks match exactly;
- three construction landmarks match exactly;
- every renderer-feed operation count matches exactly;
- the seven mismatched counters are assigned to owner-family gaps:
  `Component::addDirt` 201/287, transition search 176/154, Artboard DataBind
  batches 90/113, draw-order sort 24/607, clipping-list clear 48/1,214,
  drawable owner lookup 0/448, and frame-loop allocations 2,732/6,118
  (C++/Rust).

These enter FL-A through FL-E with their complete corresponding owner
families. They do not make any of the six corpus entries a work slice.

## Per-wave landing contract

For every dependency-ready owner family:

1. Read its complete pinned headers and sources.
2. Translate construct, retain, dirty, update, advance, draw, clone, and drop.
3. Preserve C++ owner identity and ordering through binding PORTING rules.
4. Delete the displaced Rust path in the same landing.
5. Add deterministic lifecycle/counter tests.
6. Run independent ownership/identity and dirt/order reviews.
7. Run all applicable floors and `make runtime-frame-loop-port-check`.
8. Commit with `[FL-1]`.
9. Run one canonical whole-corpus performance checkpoint and record it in the
   status file; continue from the dependency map.

No benchmark entry becomes a slice. No renderer-boundary change, invented
optimization, gate change, or new deliberate divergence is authorized.

## Completion

Completion is `closed members / total members`, with:

- zero pending and compensation rows;
- every gap closed or backed by an approved adaptation/decision;
- all behavioral, pixel, product, size, and structural floors green;
- canonical `perf-hot-loop` aggregate ≤ 1.0× C++;
- unchanged provenance, A-B-B-A, drift, and candidate-repeat checks.

If the structure is closed but ≤1.0× requires departing from C++ ownership,
dirt, update, or traversal, stop for the user.
