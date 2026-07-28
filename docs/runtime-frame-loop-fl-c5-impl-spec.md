# FL-C5 single-writer implementation specification

This specification is for one production writer. Work packages are strictly
dependency ordered: finish and accept each package before starting the next.
The first package is a behavior-neutral file split; semantic work begins only
after that split is green.

The binding checklist is `docs/runtime-frame-loop-fl-c5-closure.md`. If this specification
and an inventory appear to conflict, `docs/runtime-frame-loop-fl-c5-walk/directives.md` wins.

FL-C5 covers exactly the four pinned files:

- `include/rive/animation/state_machine.hpp`
- `src/animation/state_machine.cpp`
- `include/rive/animation/state_machine_instance.hpp`
- `src/animation/state_machine_instance.cpp`

The separately handled FL-C1 claimed-`DataBindPath` correction (W6) is
excluded. Do not reopen, duplicate, or fold that fix into FL-C5.

## Allowed production ownership

The writer may change production code only where required by these packages:

- create
  `crates/nuxie-runtime/src/state_machine/state_machine.rs`;
- create
  `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`;
- reduce `crates/nuxie-runtime/src/state_machine.rs` to a thin module entry
  point/re-export while preserving all public definition APIs;
- reduce `crates/nuxie-runtime/src/state_machine/instance.rs` to a thin module
  entry point/re-export while preserving all public instance APIs;
- change
  `crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs`
  only for the W4-confirmed retained per-layer changed flag, current-state
  access, and corresponding call plumbing;
- change `crates/nuxie-runtime/src/artboard.rs` only to replace leaked
  StateMachineInstance orchestration with thin calls into the new instance
  owner;
- change directly coupled state-machine module declarations/imports needed for
  the split;
- add focused tests, pinned-C++ probe cases, and FL-C5 structural checker
  rules/negative controls; and
- update the FL-C5 closure/status/ownership/manifest evidence only after the
  corresponding production behavior and proof are green.

No package authorizes a broad cleanup, formatting sweep, public API redesign,
or opportunistic port from a later family.

## Acceptance command conventions

Each package names tests that the writer must add if they do not already exist.
Run the exact focused commands first. `cargo test` filters may match more than
one deliberately grouped test; the receipt must list the actual tests run.

`make runtime-frame-loop-port-check` requires the pinned Rive checkout through
the repository’s normal `RIVE_RUNTIME_DIR` configuration. Do not weaken the
checker when a package fails.

No performance command belongs in any package.

## WP0 — behavior-neutral owner-file split

### Production work

1. Create `state_machine/state_machine.rs` and move the complete existing
   definition implementation from the root `state_machine.rs` into it without
   changing behavior.
2. Create `state_machine/state_machine_instance.rs` and move the complete
   existing instance implementation from `state_machine/instance.rs` into it
   without changing behavior.
3. Make the two old files thin entry points/re-exports. Preserve every public
   item and module path from W4 §C.
4. Do not move `StateMachineLayerInstance`; its accepted owner remains
   `state_machine_layer_instance.rs`.
5. Do not consolidate artboard orchestration yet. This package must be
   reviewable as a pure ownership/file-layout change.

### Acceptance

```sh
cargo check -p nuxie-runtime
cargo test -p nuxie-runtime --lib state_machine
cargo test -p nuxie --test public_api state_machine
cargo test -p nux-capi
make runtime-frame-loop-port-test
cargo fmt --all -- --check
```

Add and run a compile-time `fl_c5_public_reexports_survive_file_split` test
that imports every W4 §C group through its pre-split public path:

```sh
cargo test -p nuxie-runtime --lib fl_c5_public_reexports_survive_file_split -- --exact
```

### Package stop condition

Do not begin semantic work while any downstream import changed, any public
visibility narrowed, or either old entry file still owns substantive
implementation.

## WP1 — StateMachine definition collections and listener-slot retention

Depends on WP0.

### Production work

1. Make `state_machine/state_machine.rs` the direct owner of the complete
   `state_machine.hpp/.cpp` adaptation.
2. Replace listener `filter_map` compaction with one retained slot per authored
   listener. An unbuildable listener is inert but preserves its authored
   index.
3. Preserve layer/input/listener/data-bind/scripted-object authored order,
   duplicates, and supported null slots.
4. Add definition-level count/index/name behavior or documented immutable
   equivalents needed to close every definition row.
5. Represent `import`, `onAddedDirty`, and `onAddedClean` through the existing
   Rust import architecture. Preserve their observable attachment/status and
   inputs → layers → listeners first-error ordering; do not introduce a second
   importer framework.
6. Keep typed bindable arrays, transition-duration occurrences, and the action
   owner arena as documented Rust adaptations.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_definition_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_state_machine_definition_
cargo test -p nuxie-runtime --test cpp_probe malformed_listener_retains_authored_index -- --exact
cargo test -p nuxie-runtime --test cpp_probe state_machine_added_phases_match_cpp -- --exact
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include empty machines, duplicate names/pointers, exact
case-sensitive first match, null input count/index behavior, malformed listener
followed by valid listener, missing importer, dirty/clean first failure, and no
rollback.

### Structural negative controls

- Reintroduce `filter_map` over listeners.
- Flatten `Option` input slots.
- Replace an authored collection with a set/map.
- Leave a definition implementation in the root entry file.

Each injected form must make the checker fail.

## WP2 — instance fields, construction order, and lifecycle skeleton

Depends on WP1.

### Production work

1. Move all `StateMachineInstance`-owned fields and helper types into the new
   instance owner with explicit Rust initialization for C++ fields lacking
   header initializers.
2. Establish constructor phase boundaries in the pinned order:
   inputs; layers/Any/Entry; machine binds; authored listener categories;
   component-provided groups; nested/list/text hits; scripted clones and
   facilities; hit sort; focus tree.
3. Preserve null/unsupported input slots and all duplicate occurrence
   identities.
4. Retain the index/arena definition identity, stable listener-definition
   arena, typed data-bind occurrences, script lifecycle maps, generation
   counters, notification queue, and terminal script-error adaptation.
5. Add explicit `dispose`/nested-event detach plumbing and observable Drop
   ordering. Keep snapshot `Clone`, but prove every mutable collection and
   registration is non-aliased; script occurrence state stays cold where the
   existing public snapshot contract requires it.
6. Do not implement pointer hit behavior in this package beyond the ownership
   skeleton required by WP3.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_constructor_order_
cargo test -p nuxie-runtime --lib fl_c5_clone_teardown_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_constructor_order_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_dispose_nested_event_
cargo test -p nuxie --lib scripted_listener_action
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include Entry actions before ordinary binds/listeners, duplicate
transition-property binds, unresolved pointer targets retaining groups,
partial construction after nested registration, internal versus external
manager teardown, repeated dispose, snapshot pending queues versus cold
remount, regenerated trigger-layer IDs, and non-aliased pointer/script state.

### Structural negative controls

- Add `Default` to queued focus/semantic values with a meaningful sentinel.
- Shallow-clone a queue, registration, pointer table, layer trigger ID, or
  script table.
- Remove explicit nested detach.
- Reorder bind/layer/script teardown.

## WP3 — complete hit-component hierarchy and pointer routing

Depends on WP2. This is the largest semantic package.

### Production work

1. Implement a `HitComponent` trait and concrete `HitDrawable`,
   `HitExpandable`, `HitTextRun`, `HitLayout`, `HitNestedArtboard`, and
   `HitComponentList` types in `state_machine_instance.rs`.
2. Implement tri-state `HitResult` (`none`, `hit`, `hitOpaque`).
3. Implement the three complete `updateListeners` passes: reset every group,
   prepare every hit owner, then process sorted hit owners. After the first
   opaque result, continue processing later owners with `canHit=false`.
4. Release the pointer’s group state after Exit processing.
5. Implement `addToHitLookup` with component identity reuse, recursive child
   traversal, exact shape/text dirt side effects, duplicate listener append,
   and the layout-versus-shape/text opacity rules.
6. Implement constructor sort and re-sort when the artboard draw-order counter
   changes. Match the swap-derived C++ order; do not substitute a stable sort.
7. Route nested-artboard events in authored nested-animation order. Route
   component-list events in reverse `orderedListIndices()` order, preserve
   strongest-result aggregation, and convert occluded down/up/move/exit work
   to child Exit cleanup.
8. Implement drag start/end enable/disable walks and the exact timestamp rules.
9. Put existing Rust pointer capture/history behind a ListenerGroup-shaped
   internal seam. Keep that state until FL-D. Delete the old per-listener hit
   traversal displaced by the new owners.
10. C++-corresponding pointer methods must forward NaN, infinities, signed
    zero, and negative timestamps. Existing validating Rust convenience
    methods may remain only as distinct entry points.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_hit_
cargo test -p nuxie-runtime --lib fl_c5_pointer_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_hit_component_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_nested_pointer_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_component_list_pointer_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_pointer_fp_
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

The differential set must cover shared targets, duplicate groups, click-only
down/up needs, enter/opacity early-out behavior, dynamic target opacity,
scroll→opaque, hidden/no-paint shapes, layout bounds and singular transforms,
text runs, mixed recursive containers, nested first-hit/later-miss overwrite,
reverse list routing, opaque cleanup, duplicate indices, drag timestamp
discard/follow-up Move, frame-origin coordinates, draw-order changes, and
nonfinite values.

### Structural negative controls

- Replace tri-state result with `bool`.
- Fuse prepare and process, or process before all preparation completes.
- Break after opaque instead of continuing with `canHit=false`.
- Traverse component lists forward.
- Omit any concrete hit type, draw-order counter, exit release, or
  enable/disable walk.
- Retain the displaced per-listener dispatch loop.

## WP4 — per-layer state reporting and instance query surface

Depends on WP2 and may be completed after WP3; do not combine it with broader
FL-C3 refactoring.

### Production work

1. In `state_machine_layer_instance.rs`, retain
   `state_changed_on_advance` on each layer occurrence. Clear it only on a new
   frame and preserve it through same-frame zero-time convergence.
2. Add the required current-state access used by `currentState`, testing
   `layerState`, and `stateChangedByIndex`.
3. Make `stateChangedCount` scan retained flags or use only a demonstrably
   derived cache. Delete cached-only state.
4. Preserve compressed authored-layer order for changed states and current
   animations.
5. Keep per-layer random-weight scratch and monotonic trigger-layer identity.
6. Do not create Rust instance-level implementations for the three stale
   undefined private declarations in the C++ header.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_state_changed_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_state_changed_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_current_state_and_animation_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_random_transition_edges_
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include layers 0 and 2 changed, several transitions in one
layer, a same-frame follow-up, reset during an active transition, two layers
using one animation, non-animation interleaving, weighted boundaries,
wraparound, and two-instance random-scratch isolation.

## WP5 — C++-shaped bind and DataContext family

Depends on WP2 and WP4.

### Production work

Implement these as the primary instance structure, preserving exact order and
null distinctions:

1. `setViewModelInstance`: null no-op; stage main without binding.
2. `setGlobalViewModelInstance`: validate file/name/global slot and replace
   only that slot.
3. `completeViewModelInstances`: missing main first, then missing globals in
   file-global order; do not replace an occupied cross-model slot.
4. `bind`: no context no-op; complete, bind artboard, then bind machine.
5. `bindViewModelInstance`: non-null set+bind; null clears machine
   context/listener cells and calls artboard unbind, without inventing an
   explicit machine-bind unbind.
6. `bindDataContext`: clear old machine registration, register the supplied
   non-null context, clear/bind artboard, then bind machine. Null is not a safe
   clear.
7. `inheritDataContext`: null no-op; register/apply without clearing a prior
   context, preserving the C++ stale-registration hazard.
8. `dataContext` setter/getter, `rebind`, `clearDataContext`,
   `relinkDataContext`, `rebuildDataBind`, `unbind`, and
   `internalDataContext` in the exact W3 order.
9. Bind listener cells before scripted context/init passes as specified by the
   pinned member order.
10. Convert all existing typed Rust context APIs into delegating adaptations;
    preserve their signatures and boolean/result shapes.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_bind_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_bind_family_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_bind_null_matrix_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_inherit_context_a_then_b_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_complete_view_models_
cargo test -p nuxie --lib data_context
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

### Structural negative controls

- Merge `setViewModelInstance(nullptr)` with
  `bindViewModelInstance(nullptr)`.
- Accept null in `bindDataContext`.
- Clear before `inheritDataContext`.
- Bind the machine before the artboard.
- Reorder listener-cell and scripted-context passes.
- Let a typed convenience bind bypass the primary family.

## WP6 — events, ListenerViewModel reporting, and bubbling

Depends on WP5.

### Production work

1. Keep the existing dual-cursor event adaptation and notification queue.
2. Port the trigger-zero suppression guard into `reportToStateMachine`.
3. Preserve `applyEvents`: update binds; snapshot/clear both pending queues;
   event callbacks; ViewModel callbacks; repeat through exactly 100 batches.
4. Make pending count/index inspection exclude the batch currently being
   dispatched.
5. Preserve listener-major/event-minor order, single-listener first-match
   break, multi-input per-event scan, duplicate reports, and immediate nested
   bubbling.
6. Order local dispatch → bubbling → recorded audio seam. Port
   `playsAudio == true`; do not implement the deferred audio owner.
7. Preserve the live host-report projection so payload changes before and
   after fire are not frozen.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_event_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_apply_events_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_apply_events_100_batches -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_event_mid_callback_visibility -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_trigger_zero_suppression -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_event_bubbling_audio_seam_order -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_live_event_projection -- --exact
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include exactly 100 finite batches, a 101st pending batch,
event-generated event+VM work, VM-generated event work, callback count/At,
`[A,A]` against single and multi listeners, nested target mismatch, duplicate
listener cells, trigger `0→1→0` including signed zero, host drain isolation,
and nested audio reports through two ancestors up to the seam.

## WP7 — advance consolidation and floating-point policy

Depends on WP3, WP4, WP5, and WP6.

### Production work

1. Make `advance(seconds,newFrame)` and both `advanceAndApply` forms members of
   the new instance owner. They take the artboard explicitly as the documented
   Rust borrow-model adaptation.
2. Reduce `artboard.rs` to thin delegating wrappers for this work.
3. Preserve raw order: draw-order re-sort; focus snapshot; semantic snapshot;
   apply event/VM batches; clear scheduling latch; pre-layer bind update;
   authored layer advance; converter/bind advance; every input `advanced()`.
4. Preserve `newFrame=false` behavior, per-layer changed flags, deferred focus/
   semantic batches, report timing, and the pinned raw return terms.
5. Preserve facade order and the five-pass settlement limit, including
   zero-time state-machine/artboard follow-ups and optional VM advancement.
6. Delete the clean zero-delta fast path and its solely supporting
   `has_advanced_once` state.
7. Delete `requires_post_update_state_probe`,
   `post_update_probe_pending`, and the definition-level capability scan.
   Probe transitions unconditionally on every settlement pass.
8. Preserve exact `seconds == 0.0f` forcing for both signs.
9. Forward NaN, infinities, signed zero, and negative seconds through every
   C++-corresponding path. Keep validation only on separately named Rust
   convenience entry points.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_advance_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_raw_advance_order_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_zero_delta_bookkeeping -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_advance_fp_matrix -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_advance_return_terms -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_five_pass_unconditional_probe -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_advance_view_models_false -- --exact
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include idle `+0/-0`, fired trigger at `-0`, clean zero still
running bind/layer/input bookkeeping, NaN one-shot, infinities on loop and
ping-pong fixtures under a bounded probe harness, `newFrame=false` after true
continuation, focus chaining during focus processing, a hidden focus target,
reports created during layer/bind advance, persistent dirt requiring six
passes, and `advanceViewModels=false`.

### Structural negative controls

- Add any clean-zero return.
- Add any capability-gated transition probe.
- Reject nonfinite values in the C++-corresponding path.
- Omit pending event/VM terms or exact-zero forcing from return behavior.
- Put settlement implementation back in `artboard.rs`.

## WP8 — keyframe DataBind construction, advancement, and removal

Depends on WP5 and WP7.

### Production work

1. Close `keyFrameHolderPropertyKey`, `makeKeyFrameValueHolder`,
   `buildStateKeyFrameBinds`, and `removeStateKeyFrameBinds` against the pinned
   ordering.
2. Support exactly number, color, boolean, and string holder types.
3. Select the first source-artboard bind per keyframe target.
4. Preserve traversal order and the holder → clone → file → retarget/property
   key → initialize → converter → container enrollment → state tracking
   sequence.
5. If already bound, bind/update the new occurrence immediately through the
   normal container.
6. Preserve duplicate-build behavior and ensure state teardown removes tracked
   binds before holders/state drop.
7. Keep the reusable Rust keyframe graph cache only as a documented
   occurrence-isolating adaptation; it must reproduce the live C++ behavior.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_keyframe_data_bind_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_keyframe_data_bind_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_keyframe_first_source_bind -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_keyframe_initialize_converter_order -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_keyframe_bound_context_lifecycle -- --exact
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include every supported holder, unsupported and null keyframe,
duplicate source binds, duplicate build, already-bound context, converter
keep-going, remove unknown state, multiple-bind removal order, attempted
reentrant removal, and destructor with active binds.

## WP9 — focus/semantic orchestration and recorded dependency boundaries

Depends on WP2, WP6, and WP7.

### Production work

1. Close queue/process ordering for focus and semantic events defined in
   `state_machine_instance.cpp`.
2. Close the instance-defined selection/call ordering for focus-manager
   replacement, focus set/state/traversal, semantic enable/replacement, and
   `fireSemanticAction` up to the recorded manager/data seams.
3. Snapshot and clear deferred focus/semantic batches before callbacks.
   Focus-to-focus and semantic-to-semantic work queued during processing waits
   for a later new frame; preserve the W4-observed focus-then-semantic phase
   relationship.
4. Add `FocusState` host polling and the missing C++-surface accessors where
   the Rust ownership model can expose an owner-safe adaptation.
5. Record, but do not implement, manager/tree/node internals owned by
   `focus_manager.cpp`, `semantic_manager.cpp`, and `semantic_data.cpp`.

### Acceptance

```sh
cargo test -p nuxie-runtime --lib fl_c5_focus_semantic_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_focus_queue_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_semantic_queue_
cargo test -p nuxie-runtime --test cpp_probe fl_c5_focus_manager_switch_order -- --exact
cargo test -p nuxie-runtime --test cpp_probe fl_c5_focus_state -- --exact
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

Required cases include duplicate focus/semantic events, chained focus,
focus-generated semantic work, null semantic groups/listeners, hidden focused
targets, same-manager identity no-op, external→internal fallback, null
FocusData/node behavior, focused non-keyboard node, and missing semantic
manager/node/data no-op at the recorded seam.

## WP10 — API, compensation, structural, and publication closeout

Depends on WP0–WP9. No new semantic design belongs here.

### Production/evidence work

1. Run the generated W4 §C public API inventory and repair re-export-only
   breakage without redesigning APIs.
2. Verify every compensation `KEEP` is documented and tested. Verify each
   `DELETE` mechanism is absent and has a differential proving the C++
   behavior it masked.
3. Add all structural negative controls from the closure draft and run them.
4. Verify every out-of-scope seam is `RECORDED` with its owning row and that no
   deferred row was promoted.
5. Fill the closure packet with exact test/checker counts, source citations,
   trace receipt, and immutable candidate identity only after all gates pass.

### Focused acceptance

```sh
cargo test -p nuxie-runtime --lib
cargo test -p nuxie-runtime --test cpp_probe
cargo test -p nuxie --lib
cargo test -p nuxie --test public_api
cargo test -p nux-capi
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
cargo fmt --all -- --check
```

### Whole-family non-performance floor

Run the repository’s current complete correctness/structure publication floor
required by the family procedure: probe-armed workspace, ordinary and scripted
goldens, pixel referee, C API, native Apple/XCFramework/ABI/header checks,
browser WebGPU-only checks, lint/format/diff checks, committed-tree size gate,
and source-bound trace/provenance validation.

Do not run a performance target. FL-C5 acceptance cannot use timing to select,
reject, or reorder implementation work.

## What not to touch

The following are hard boundaries for every package:

- Do not modify or re-solve the W6 FL-C1 claimed-`DataBindPath` correction.
- Do not port `src/listener_group.cpp` internals. Keep the existing pointer
  capture/history representation behind the internal seam until FL-D.
- Do not port `src/animation/text_input_listener_group.cpp`; it remains FL-E.
- Do not implement the `src/input/gamepad_batch.cpp` byte-buffer parser or
  claim its row.
- Do not port `src/input/focus_manager.cpp` internals or redesign `focus.rs`.
- Do not implement `semantic_manager.cpp` or `semantic_data.cpp` manager/tree
  internals.
- Do not implement audio playback; stop at the recorded `audio_event.cpp`
  seam.
- Do not rewrite importer owners. Represent `state_machine.cpp` import/onAdded
  behavior through the accepted Rust import architecture.
- Do not move or refactor the accepted FL-C3
  `StateMachineLayerInstance` beyond the specific W4 divergence named in WP4.
- Do not change accepted FL-C1–FL-C4 behavior or public APIs except for the
  minimum delegation/re-export plumbing required by this file split.
- Do not remove any W4 §C public API, narrow its visibility, or silently change
  its signature/result contract.
- Do not move substantive instance orchestration into `artboard.rs`; it may
  contain only borrow-model delegating wrappers for FL-C5.
- Do not delete the approved Rust adaptations: index/arena identity, snapshot
  Clone, event cursors, notification queue, script lifecycle/generation state,
  terminal script error, per-layer random scratch/trigger IDs, typed bindable
  structures, live host report projection, or probe APIs.
- Do not retain the three rejected compensations: clean zero-delta fast path,
  capability-gated post-update probe state/scan, or cached-only changed-state
  count.
- Do not add finite validation to a C++-corresponding advance, pointer, or
  report path.
- Do not promote a recorded seam to faithful, change unrelated ownership/
  manifest rows, regenerate unrelated artifacts, update dependencies, reformat
  unrelated files, or perform broad naming/style cleanup.
- Do not run performance measurements.

## Final handoff requirement

The writer hands off one immutable FL-C5 candidate only after all ten packages
are accepted in order and every closure row is checked. The handoff must state:

- the exact production files changed;
- the focused test/probe names and counts by package;
- structural checker and injected-negative counts;
- public API preservation result;
- recorded seams left open;
- compensation KEEP/DELETE verification;
- complete non-performance floor receipts; and
- confirmation that W6 and all other out-of-scope owners were untouched.

