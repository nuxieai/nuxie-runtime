# Runtime Frame-Loop Port Status

Sole resume state for the C++-corresponding frame-loop performance closeout.

## Current

- Phase: FL-A source audit/specification complete. FL-1 rulebook validation,
  source shaping, and clean-floor verification are complete; the binding
  52-file/six-member implementation specification is adversarial-review
  green. No production owner-family translation has started.
- Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
- File closure: 0 / 337 in-scope C++ files.
- Member closure: 41 / 74 owner/member rows (the imported, already-closed
  runtime-drawing ledger); 33 frame-loop rows pending.
- Open mechanism gaps: 7 / 8.
- Current dependency wave: FL-A, Component/update ownership.
- Current FL-A landing: A1 is non-production scaffold only; A1's first
  production use must land atomically with A2's complete occurrence-graph
  replacement and legacy-path deletion.
- Current experimental changes: uncommitted KeyFrame retained-seconds and
  Component-handle candidates remain quarantined. They are not standalone
  slices and must be re-derived in FL-B/FL-A or discarded.

## FL-0 evidence

- Static closure: seeded and reviewed. Six non-overlapping source sets expand
  to 337 explicit file rows across component/update, animation, state machine,
  DataBind/Artboard, and live draw. The 103 dynamically reached rows and 234
  cold rows are machine-checked against trace evidence; each cold family stays
  in scope under its virtual-dispatch/dependency rationale.
- Dynamic reachability: captured from LLVM function-entry counters with
  construction counters reset immediately before the sample loop. C++ reached
  461 functions in 103 / 337 scoped files; Rust reached 1,087 functions in 18
  runtime modules. Full names and counts are in
  `docs/runtime-frame-loop-trace.json`.
- Deterministic structural counters: captured on the same six entries and 11
  samples against clean Rust `13aedd6d` and pinned C++. Exact pairs:
  Artboard/SMI/LinearAnimation construction 24/24, 24/24, 27/27;
  SMI advance 30/30; layer advance 31/31; animation advance 38/38; update pass
  29/29; component update 29/29; event batch 30/30; keyframe-double apply
  steps 124/124; layout compute 24/24; public/internal draw 11/11 and 30/30.
- Structural mismatches are now finite owner-family work:
  - FL-A: Component dirt additions C++ 201 vs Rust 287.
  - FL-C: transition searches 176 vs 154.
  - FL-D: Artboard DataBind batches 90 vs 113.
  - FL-A/FL-E integration: draw-order sorts 24 vs 607, clipping redundant-list
    clears 48 vs 1,214, and drawable owner lookup 0 vs 448.
  - Cross-wave allocation oracle: C++ 2,732 vs Rust 6,118 frame-loop
    allocations (debug coverage runners, identical corpus/samples, counter
    reset after construction).
  Each mismatch has a machine-checked gap row. None is a benchmark-scene
  slice.
- Deterministic renderer-feed operations are exact: 11 frames, 148 drawPath,
  134 makeEmptyRenderPath, 283 makeRenderPaint, 32 makeLinearGradient, 17
  clipPath, 146 transform, 152 save/restore, and one image decode on both.
- Cold lifecycle oracle: clean `13aedd6d` targeted tests
  `public_artboard_clone_is_cold_but_transient_layout_clone_keeps_scripts` and
  `mounted_child_backend_resources_clone_and_remount_cold` both pass (1/1
  each), preserving public clone identity separation and cold backend
  remounts. Their C++ lifecycle citations remain in the imported drawing
  ledger.
- Fail-closed checker: included in the FL-0 map commit with nine checker
  negative controls plus three summarizer unit tests. It rejects scope growth,
  overlaps, missing per-file rows, stale dynamic markers, premature close,
  unverified file promotion, missing adaptation rules, untracked counter
  mismatches, and renderer-stream work mismatches.
- Trace harness: opt-in and isolated. Instrumented C++ uses a dedicated runtime
  archive and runner name with a trace-flags stamp next to `librive.a`; Rust
  uses a dedicated Cargo target and feature. Both runners reject unavailable
  instrumentation and repeated benchmark mode rather than emitting misleading
  evidence. Ordinary runner paths remain untouched.
- Map/checker commit: `2c858676`. The clean-tree anchor correction is
  `69e89b3c`. No production behavior changed in either commit.

The prior sampled seven-divergence run used a release-linked C++ ordinary
runner and is invalid ordinary-golden evidence. Ordinary parity uses only
`env -u CPP_CONFIG -u RUST_PROFILE make golden-compare` with the checked-in
debug C++ configuration and its provenance stamp.

## FL-1 rulebook evidence

- Representative sources: complete pinned
  `src/component.cpp` + `include/rive/component.hpp`,
  `src/animation/linear_animation_instance.cpp` + header, and
  `src/animation/state_machine.cpp` + header, including the directly required
  importer/generated lifecycle sources.
- Rulebook-strict disposable translation:
  `translation.rs` SHA-256
  `b3553b81d013109c50e1d3b4ab967cb6e05ac1737ecbcd6b339f49c5148d4bc6`;
  `notes.md` SHA-256
  `8e22ba7f9913f59a12b4bf0e7dc5f49dbfa01f04ca8e16ebeca309acb5c25d2b`.
- Independent senior-Rust disposable translation:
  `translation.rs` SHA-256
  `b43c3203a41493c69e68ca320e37033c35f78793df91f212d79845fb628f4237`;
  `notes.md` SHA-256
  `0fdc69ade77bc995b7c15f17f8664dd46aa4cab1bf17a38db06d4f67e743639d`.
- Adjudication: pinned C++ selected construction-state `Option`/typestate for
  unset graph order; explicit owner mediation for Artboard back-pointers;
  preserved nullable state-machine input slots; stable non-owning animation
  definition identity; raw loop integer storage; literal time arithmetic;
  exact collection visitation; generated/base-only aggregate clone; and safe,
  explicitly ordered owner-mediated teardown. An observably uninitialized C++
  scalar is a gap/decision, not permission to invent a zero value.
- `docs/PORTING.md` now binds FLR-1..FLR-15 for definition/occurrence
  separation, owner back-pointers, construction state, dirt order, nullable
  slots, unique collections, clone, teardown, raw generated enums, literal
  arithmetic/guards, occurrence ids, lifecycle visitation, first-insert
  synchronization, event timing, and validated runtime invariants.
- Source-shaping verdict: no mechanical extraction is required before FL-A.
  The dependency-ready families already have disjoint primary owners:
  Component in `components.rs` with Artboard integration, the coupled
  KeyFrame-through-LinearAnimation family in `animation.rs`, and state-machine
  definitions/occurrences in `state_machine.rs` plus
  `state_machine/instance.rs`. Splitting inside those coupled C++ owner
  families solely for parallelism would create a new seam rather than expose
  one. Reassess only if a later complete owner family has an independently
  testable boundary.
- Both translations are disposable evidence only. Their hashes and the
  adjudication above are retained. The temporary translation trees were moved
  recoverably to
  `/Users/levi/.Trash/nuxie-fl1-disposable.MZH4pp` when FL-1 closed.
- Verification: all 12 checker unit/negative-control tests pass. The
  working-tree checker correctly rejects the quarantined KeyFrame experiment
  because it removes the committed `RuntimeKeyFrameTiming` anchor. Rerunning
  against clean committed source with the current ledger/gaps reports 337
  files, 74 members, 8 gaps, and every ratchet at its expected value. This is
  the only accepted FL-1 structural evidence.

## Baseline performance

- Last committed-tree canonical hot-loop artifact:
  `target/perf-hot-loop-13aedd6d.json`.
- Aggregate at `13aedd6d`: approximately 1.479× C++.
- This is context, not a work queue. The next checkpoint occurs only after a
  complete dependency wave.

## Gate ledger

FL-0 clean committed-tree floor, run from detached worktrees carrying only
`2c858676` plus the `69e89b3c` anchor correction:

- `cargo test -p nuxie-runtime --lib`: 414 passed, 0 failed.
- `cargo test -p nuxie --lib`: 140 passed, 0 failed.
- `env -u CPP_CONFIG -u RUST_PROFILE make golden-compare`: 317 / 317
  entries and 647 / 647 segments exact; 0 divergences, unsupported, or
  not-yet entries.
- `env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare`: 317 /
  317 entries and 647 / 647 segments exact; 0 divergences, unsupported, or
  not-yet entries. `data_viz_demo` and `db_health_tracker` both matched.
- `env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests`:
  passed with the probe built and `RIVE_CPP_PROBE` set for the workspace run.
  The explicit probe-only confirmation passed 721 / 721, 0 failed.
- `make renderer-golden`: 1,468 / 1,468 entries accepted; 0 divergences and
  0 gated failures (837 byte-exact), Apple M5 Max.
- `make capi-smoke`: passed (`draw_paths=2`, `objects=4`).
- `make apple-runtime-check`: passed, including the release panic firewall,
  66 product tests, 15 artifact-validator tests, header smoke, and deny
  clippy surface.
- `make lint-gate`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `make runtime-frame-loop-port-check`: 12 / 12 checker controls passed;
  337 file rows, 74 member rows, 8 gap rows, and all three compensation
  ratchets validated. A first clean-tree run correctly exposed one ledger
  anchor that referred to the quarantined animation experiment; `69e89b3c`
  retargets it to the committed `RuntimeKeyFrameTiming` owner and the clean
  rerun passes.
- `make size-report` at `69e89b3c`: scripting off 8,267,336 bytes
  (7.88 MiB); scripting on 9,168,392 bytes (8.74 MiB); both below the
  9,437,184-byte budget.

FL-1 clean committed-tree floor at `bb9ad75d`:

- `cargo test -p nuxie-runtime --lib`: 414 passed, 0 failed.
- `cargo test -p nuxie --lib`: 140 passed, 0 failed.
- `env -u CPP_CONFIG -u RUST_PROFILE make golden-compare`: 317 / 317
  entries and 647 / 647 segments exact; 0 divergences or failures.
- `env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare`: 317 /
  317 entries and 647 / 647 segments exact; 0 divergences or failures.
  `data_viz_demo` and `db_health_tracker` both matched.
- `env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests`:
  passed with the probe built and exported for the workspace run; the pinned
  721-test probe suite ran.
- `make runtime-frame-loop-port-check`: all 12 checker controls passed on
  clean committed source; 337 file rows, 74 member rows, and 8 gap rows match
  their ratchets.
- `make renderer-golden`: 1,468 / 1,468 entries accepted, 837 byte-exact,
  0 divergences, and 0 gated failures on Apple M5 Max.
- `make capi-smoke`: passed (`draw_paths=2`, `objects=4`).
- `make apple-runtime-check`: passed, including product tests, artifact
  validation, generated-header smoke, deny clippy, and the release panic
  firewall.
- `make lint-gate`, `cargo fmt --all -- --check`, and `git diff --check`:
  passed.
- `make size-report`: scripting off 8,267,336 bytes (7.88 MiB), SHA-256
  `4d35c3917a16ff98c6f3bbc6677d7333582dff3ab5b803b969725708db8e8d7e`;
  scripting on 9,168,392 bytes (8.74 MiB), SHA-256
  `47cf0e95bb8c8f9abc04676b3ae802ca3b4aaf401037579194c7bfaf9ca85d51`;
  both below the unchanged 9,437,184-byte budget.

## FL-A source audit and implementation specification

- Binding specification:
  `docs/runtime-frame-loop-fl-a-spec.md`.
- Coverage: exactly 52 / 52 `component-update-graph` C++ file rows,
  partitioned as 11 Component/core, 6 bones, 21 constraints/scrolling, and
  14 math rows; no missing or duplicate file. All six pending
  `component.identity`, `component.dirt`, `component.dependents`,
  `component.update_order`, `component.transforms`, and
  `component.clone_drop` rows have explicit construct/retain/dirty/update/
  clone/drop closure contracts.
- Core finding: committed Rust copies authored local-ID parent/dependent/
  constraint topology and precomputed order, then centralizes virtual family
  behavior on Artboard. Pinned C++ owns occurrence-local links, builds them
  after parenting, sorts that same retained graph, publishes accumulated dirt
  before concrete callbacks, and traverses retained owner identity.
- Constraint finding: six arithmetic families are reusable, but all 21 rows
  remain owner-divergent or missing at the committed floor. FollowPath measure,
  IK chain, ScrollPhysics, ScrollConstraint child rendezvous, virtualizer,
  draggable/proxy, and generated setter callbacks must live on concrete
  occurrences; four Artboard side vectors, per-apply reconstruction, and
  global type/property redispatch are displaced paths.
- Bones/math finding: Skin must own one Skinnable link, ordered Tendons, and
  one retained bone-transform buffer with exact accumulated-dirt callback
  order. Existing value/path math stays in its accepted modules; the two
  absent cold utilities are literal small ports, not a new math subsystem.
- One-owner rule: one `RuntimeObjectOccurrence` owns the sole generated
  backing fields, embedded Component base, and concrete subclass payload.
  Typed handles are views of that object, not links between three
  authoritative stores. Existing renderer resources remain under their
  already-closed RF ownership adaptations.
- Hybrid prevention: A1 is private/unreachable scaffold and cannot merge.
  The first production handle use is atomic with A2, which ports Component
  ownership, remaps ordinary Component plus already-closed PathComposer/
  TextVariationHelper nodes into one occurrence schedule, deletes copied
  relation/schedule reads, and adds negative ratchets in the same landing.
- Clone/drop is per owner, not a generic remap. ScrollConstraint, Skin,
  FollowPath, IK, proxies/physics, ArtboardComponentList, ScrollVirtualizer,
  and Drawable each have copied/default/rebuilt/non-owning/teardown policy.
- Artboard handoff: FL-A freezes the component construction, dependency sort,
  dirty/update, advancing/resetting, and frame-interleaving methods from
  `src/artboard.cpp` as method-level evidence while the whole Artboard file row
  remains pending for FL-D.
- Adversarial specification review:
  - owner/identity/clone/drop: PASS after resolving the forbidden hybrid,
    single-property-owner ambiguity, generic clone error, Drawable hit/clipping
    boundary, and Artboard cross-wave handoff;
  - dirt/order/advance: PASS after correcting TargetedConstraint phase,
    generated callback/no-op/deserialize semantics, mixed dependency-node
    schedule replacement, FollowPath update ownership, Scroll child
    rendezvous, and full advance/reset interleaving;
  - bones/math ownership: PASS after pinning Skin/Skinnable/Tendon/Weight
    relations, accumulated Skin callback order, Solo/Layout collapse
    exceptions, retained constraint/IK targets, and existing math-owner
    placement;
  - the final ScrollConstraint computed-property check confirms percent/index
    drag/physics/intent branches and the intentional no-mutation but repeated
    notification behavior of velocity/active writes.
- Structural preflight: `make runtime-frame-loop-port-check` remains green
  with all 12 controls; open counts remain 337 files, 74 members, and 8 gaps.
- No production behavior, gate, threshold, renderer boundary, or performance
  result changed in this audit/specification landing.

## Next

1. Implement the atomic A1/A2 occurrence-object and Component graph landing
   from `docs/runtime-frame-loop-fl-a-spec.md`; no production commit may expose
   both the copied-ID graph and the typed occurrence graph.
2. Continue A3 through A7 in dependency order, deleting each displaced owner,
   lookup, reconstruction, or broad callback path in the same landing.
3. Translate FL-A as one complete Component owner-family wave, covering all
   six pending member rows and closing or rule-backing every mapped file row.
   Delete the copied-id/rediscovery mechanisms displaced by retained
   occurrence-owned links and exact dirt/update ownership in the same landing.
4. Preserve the complete behavior/pixel/product/size floor during FL-A.
   Performance is measured only after the complete wave, never used as its
   work queue.
