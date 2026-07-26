# Runtime Frame-Loop Port Status

Sole resume state for the C++-corresponding frame-loop performance closeout.

## Current

- Phase: FL-B independent candidate verification. FL-B1 through FL-B4 are
  translated;
  the focused C++ probes, runtime and probe-armed workspace floors,
  ordinary/scripted differential gates, renderer pixel referee, C API, Apple
  product/release checks, trace/checker, lint, format, and diff checks are
  green. Committed-tree size is below 9 MiB, and the canonical performance
  checkpoint is recorded. The exact candidate awaits independent acceptance.
  FL-B1 through FL-B3 are locally gated, and
  FL-A was independently
  accepted and promoted at
  `f86d5ba0146697abc996310c62fa45e1f053144b`; exact main
  `e72323c808b91d706ba3b745396beaca7accd69a` was consumed without overlap at
  FL-B boundary merge `b5d5bc8afeaa0369cbc248b85366111649cb9010`.
- Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
- File closure: 52 / 338. All 52 `component-update-graph` rows are `faithful`
  and `orchestrator-verified`; the 286 later-wave rows remain pending.
- Member closure: 47 / 75 owner/member rows (41 imported runtime-drawing
  owners plus all six FL-A Component rows); 28 later-wave rows remain.
- Open mechanism gaps: 7 / 9. FL-G02 is closed; FL-G06 remained closed.
- Current dependency wave: FL-B. Its frozen 45-file/eight-member mini-map is
  `docs/runtime-frame-loop-fl-b-spec.md`; FL-B1 through FL-B4 are implemented,
  FL-B4 is in whole-wave verification, and
  every FL-B row remains pending until whole-wave independent verification.
- The pre-advance `LinearAnimationInstance::m_didLoop` decision is resolved:
  safe Rust retains `false` as the FLR-3 binding adaptation and matches every
  defined post-advance C++ result. No `Option<bool>` API break or
  indeterminate-memory emulation is permitted.
- Current FL-A landing: promotion
  `f86d5ba0146697abc996310c62fa45e1f053144b`, published on `levi/fl-a`.

## FL-B1 KeyFrame/keyed-definition evidence

- Pinned ownership translated from `keyframe.cpp`,
  `keyed_property.cpp`, `keyed_object.cpp`, and
  `keyed_property_importer.cpp`: every concrete KeyFrame occurrence retains
  attachment-time seconds and one KeyedProperty owns one insertion-ordered
  concrete-frame sequence.
- Rust CoreRegistry binding is one `RuntimeKeyedPropertyTarget` enum, not
  parallel family flags. The old six frame vectors, read-time
  `seconds(..., fps)`, family booleans, and import-time double/color/bool
  fallback snapshots are deleted.
- Zero-fps seconds use the same float division as pinned C++ (`0/0 -> NaN`,
  nonzero/0 -> infinity); no Rust-only zero guard remains.
- Focused evidence covers retained seconds after fps mutation, mixed concrete
  owner order, duplicate/exact-offset binary search, effective bound values,
  full-mix lazy reads, and uint/id non-bindability.
- Source-bound trace refreshed on the candidate: canonical
  `keyframe_double_apply_steps` remains exactly 124 / 124 C++/Rust. The trace
  harness now counts the single target-dispatch instantiation and excludes
  its partial-mix current-value closure.
- Local floors: runtime 484 / 484; public facade 146 / 146; probe-armed
  workspace including C++ probe 726 / 726; ordinary and scripted golden each
  317 / 317 entries and 647 / 647 segments with zero divergences; all 24
  checker/capture/summarizer controls; structural checker 338 files,
  75 members, 9 gaps, and all three new FL-B1 zero-ratchets green. Rows and
  FL-G01 remain pending/open until whole-wave independent acceptance.

## FL-B2 LinearAnimation definition/occurrence evidence

- Pinned ownership translated from `linear_animation.cpp`,
  `linear_animation_instance.cpp`, and the nested linear/simple/remap owners:
  the Artboard now retains one immutable `Arc<Vec<RuntimeLinearAnimation>>`
  definition arena and each occurrence retains one
  `RuntimeLinearAnimationHandle`; `RuntimeLinearAnimation` is no longer
  cloneable.
- `LinearAnimationInstance` now retains the C++ raw signed `-1` loop sentinel.
  Its copy retains only the C++ copy-constructor state and starts without
  keyframe holders or cloned bind/converter state.
- Quantize, seconds conversion, global-to-local time, and advance no longer
  contain Rust-only zero-fps, zero-duration, or zero-range early returns.
  Float arithmetic follows the pinned expressions. Where pinned C++ has no
  defined result, Rust keeps its language-defined safe result rather than
  inventing a different control-flow branch.
- The approved binding adaptation is narrow: construction exposes
  `did_loop=false` before the first advance; zero-delta and every nonzero
  advance write the exact pinned post-advance result. Direction, time,
  total/last/spilled time, reset, raw loop override, and copy behavior have
  focused lifecycle coverage.
- Local floors: runtime 488 / 488; public facade 146 / 146; all 11
  `linear_animation_*` C++ probe comparisons green; probe-armed workspace
  including C++ probe 726 / 726; ordinary and scripted golden each
  317 / 317 entries and 647 / 647 segments with zero divergences; all 25
  checker/capture/summarizer controls; source-bound trace 103 / 338 C++
  files, 18 Rust modules, and 18 landmarks; structural checker 338 files,
  75 members, 9 gaps, with all four FL-B2 zero-ratchets green. Rows and
  FL-G01 remain pending/open until whole-wave independent acceptance.

## FL-B3 AnimationReset factory-lifecycle evidence

- Pinned ownership translated from `animation_reset.cpp` and
  `animation_reset_factory.cpp`: each reset retains first-seen object order,
  each object retains first-seen double/color property order in an owner-local
  set, and an optional first-animation baseline reads the exact first
  KeyFrame occurrence.
- Reset construction always returns an owned lease, including an empty reset.
  The final lease owner clears and returns its entry allocation to one global
  synchronized pool. This matches C++ factory acquire/release instead of
  relying on drop-only disposal.
- Rust's public `StateMachineInstance::clone` snapshot extension shares the
  immutable reset lease. It neither clones reset entries nor creates a second
  factory resource; the final snapshot owner performs the release.
- The typed Rust replay retains color values in the integer representation
  consumed by the generated setter; the exact golden oracle rejected an
  additional Rust-side float round trip. The former flat cloneable entry
  owner, global `(object, property)` membership vector, and empty-entry `None`
  elision are deleted.
- Focused first-seen order, supported-family filtering, shared-lease, replay,
  and owned-empty-reset coverage is green. Full local floors: runtime
  489 / 489; public facade 171 / 171; probe-armed workspace including C++
  probe 726 / 726; ordinary and scripted golden each 317 / 317 entries and
  647 / 647 segments with zero divergences, including `data_viz_demo` and
  `db_health_tracker`; all 26 checker/capture/summarizer controls. Source
  trace and structural checks were refreshed on the candidate. All FL-B rows
  and FL-G01 remain pending/open until whole-wave independent acceptance.

## FL-B4 blend definition/occurrence evidence

- Pinned ownership translated from `BlendState`,
  `BlendStateInstance<K,T>`, `BlendAnimation`, `AnimationStateInstance`,
  `BlendState1DInstance`, and `BlendStateDirectInstance`: each BlendState owns
  one ordered definition vector, and each embedded occurrence retains one
  stable handle to the corresponding definition plus its
  `LinearAnimationInstance` and mix.
- One-dimensional occurrences no longer copy authored threshold values or
  their state source. Direct occurrences no longer copy their source. Advance
  reads those properties through the owning state definition, and transition
  exit-animation lookup compares definition identity rather than treating
  occurrence payload as an independent descriptor.
- Invalid animation IDs retain their BlendAnimation/AnimationState definition
  and resolve to one shared empty animation, matching pinned C++ instead of
  compacting ordered children or erasing an otherwise-valid state. The empty
  definition uses the exact generated C++ defaults and has no keyed objects.
- Storage adaptation: pinned C++ uses process-global static empty animations.
  Rust's complete animation definition type can retain single-threaded
  scripting handles and therefore cannot safely be a `Sync` static. One
  immutable empty definition is owned by each Artboard arena and shared by all
  of its unresolved occurrences and clones. There is no cross-Artboard
  observable empty-animation identity; behavior and owner sharing are exact
  under FLR-1/FLR-2.
- Focused evidence proves ordered valid+empty occurrence retention, definition
  handles, shared-empty resolution, retained from/to identity, live threshold
  and direct-source reads, and AnimationState's required no-op occurrence.
  Pinned debug C++ probes are green for all nine `blend_state_*` cases plus
  direct-transition, animation-state, and mutable-bind-source cases (12 / 12).
  Current local floors: runtime 490 / 490; probe-armed workspace runtime
  490 / 490, facade 171 / 171, and C++ probe 726 / 726; ordinary and scripted
  golden each 317 / 317 entries and 647 / 647 segments with zero divergences;
  source-bound trace 103 / 338 C++ files, 18 Rust modules, and 18 landmarks;
  structural checker and its negative controls 27 / 27; renderer pixel referee
  1,468 / 1,468 with zero divergences and zero gated failures; C API smoke
  green; Apple release/product floor 66 / 66 plus artifact validation 15 / 15;
  lint, format, and diff checks green. Committed-tree size at `10bc5b23` is
  8,017,800 bytes with scripting off and 8,918,904 bytes with scripting on,
  both below the 9 MiB budget.
- Canonical whole-corpus FL-B `perf-hot-loop` checkpoint at `10bc5b23`:
  `target/perf-hot-loop-fl-b-10bc5b23.json`. Across the unchanged six-entry /
  11-sample corpus and 10,000 benchmark repeats, the minimum aggregate is
  1.684x C++ (41.707 ms C++, 70.222 ms Rust); individual samples range from
  1.416x to 2.025x. This remains above the program's final <=1.0x acceptance
  target. Per the frozen FL-B plan it is wave evidence only: it neither
  promotes FL-B nor authorizes benchmark-derived work or queue reordering.

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

- FL-A whole-wave checkpoint on runtime-identical FL-B boundary
  `b5d5bc8afeaa0369cbc248b85366111649cb9010`:
  `target/perf-hot-loop-fl-a-b5d5bc8a.json`.
- Canonical six-entry / 11-sample aggregate: 1.664× C++
  (41.340 ms C++, 68.769 ms Rust, minimum aggregate, 10,000 repeats).
- Worst total sample: `ai_assitant@0` at 2.085×. This is unfinished-wave
  acceptance evidence, not a work queue or an authorization for a Rust-only
  optimization.
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

FL-A candidate-tree floor before the publish commit:

- `cargo test -p nuxie-runtime --lib`: 478 / 478 passed.
- `env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests`:
  the full workspace passed with the probe built and `RIVE_CPP_PROBE`
  exported; the pinned probe suite ran 726 / 726 with 0 failures.
- `env -u CPP_CONFIG -u RUST_PROFILE make golden-compare`: 317 / 317
  entries and 647 / 647 segments exact; 0 divergences or failures.
- `env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare`: 317 /
  317 entries and 647 / 647 segments exact; 0 divergences or failures.
  `data_viz_demo` and `db_health_tracker` both matched.
- `make renderer-golden`: 1,468 / 1,468 entries accepted, 837 byte-exact,
  0 divergences, and 0 gated failures on Apple M5 Max.
- `make capi-smoke`: passed (`draw_paths=2`, `objects=4`).
- `make apple-runtime-check`: passed, including locked debug and
  `release-apple` product tests plus the release panic firewall.
- `make lint-gate`, `cargo fmt --all -- --check`, and `git diff --check`:
  passed.
- `make runtime-frame-loop-port-check`: all 23 checker, capture, and
  summarizer controls passed; 338 file rows, 75 member rows, 9 gap rows, and
  every zero-ratchet matched.
- Independent Standards review: PASS with no remaining findings.
- Independent Spec review: PASS with no remaining findings.
- `make size-report` at `a71814b3`: scripting off 8,300,616 bytes (7.92 MiB),
  SHA-256
  `dd889631e55568ffba8ac3dfb48f5e3ba9fb39fb74108b462818a7937f90c2ba`;
  scripting on 9,201,592 bytes (8.78 MiB), SHA-256
  `448c54d910b1ab4ec76996323bc1bb1e9abfb2462d291061afa1b22232603665`;
  both below the unchanged 9,437,184-byte budget.

FL-A post-rebase floor, refreshed after final independent review:

- `env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests`:
  the full workspace passed with the probe built and `RIVE_CPP_PROBE`
  exported; `nuxie-runtime` ran 479 / 479 and the pinned probe suite ran
  726 / 726 with 0 failures.
- Ordinary and scripted `make golden-compare` lanes each passed 317 / 317
  entries and 647 / 647 segments with 0 divergences or failures;
  `data_viz_demo` and `db_health_tracker` both matched in the scripted lane.
- `RENDERER_JOBS=4 make renderer-golden`: 1,468 / 1,468 entries accepted,
  837 byte-exact, 0 divergences, and 0 gated failures on Apple M5 Max. Worker
  parallelism was the only execution change; manifest, backend, references,
  tolerances, and timeout were unchanged.
- `make capi-smoke`: passed (`draw_paths=2`, `objects=4`).
- `make apple-runtime-check`: passed the generated-header smoke, 66 / 66
  product tests, 15 / 15 artifact-validator tests, deny clippy, and the
  `release-apple` panic firewall.
- `make lint-gate`, `cargo fmt --all -- --check`, and `git diff --check`:
  passed.
- Fresh `make runtime-frame-loop-trace` plus
  `make runtime-frame-loop-port-check`: all 23 checker/capture/summarizer
  controls passed; 338 file rows, 75 member rows, 9 gap rows, and every
  zero-ratchet matched the final rebased source fingerprint.
- `make size-report` at `f439ef50`: scripting off 8,017,784 bytes (7.65 MiB),
  SHA-256
  `c7a41481bce8d8fc4d64cf1af626f143b5c8a17b738148d9369ddcd06c9e97ec`;
  scripting on 8,918,904 bytes (8.51 MiB), SHA-256
  `ce8e248bffee61589e0c523021b64bf0905edc8828a8a802c8d1daabb153c2b2`;
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

## FL-A production translation evidence

- One occurrence graph: `InstanceObjectArena` now owns authored Components,
  PathComposer dependency nodes, and TextVariationHelper dependency nodes in
  one typed-handle address domain and one retained dependency schedule.
  Parent, child, dependent, constraint, collapsable, layout-ancestor, bone,
  tendon, advancing, resetting, scrolling, and virtualization relations are
  occurrence-local links; the copied-id/runtime-order sidecars are ratcheted
  to zero.
- One dirt owner: the root Artboard uses its inherited root
  `RuntimeComponent.dirt`; the duplicate `ArtboardInstance` dirt mask was
  removed and is guarded by the `artboard_duplicate_component_dirt` zero
  ratchet. The update loop clears only the root Components bit before walking
  the retained schedule, matching `Artboard::updateComponents`.
- Exact lifecycle: accumulated dirt is published before concrete callbacks,
  Artboard mediation, and dependent recursion. Collapse registration and
  initial synchronization preserve insertion order. Dependency sorting
  preserves C++ partial-order publication on cycles. Clone builds a fresh
  occurrence arena, clears runtime links, reconstructs concrete relations and
  schedules, and replays authored Solo/Layout collapse in authored order.
- State-machine hit testing now crosses the same construction boundary as
  C++: creating an instance mutably mediates through its Artboard, sets
  `neverDeferUpdate`, and immediately publishes recursive Shape Path dirt.
  The deferred Rust-only hit-shape queue was deleted and is guarded at zero.
  State-machine definitions are borrowed from a locally cloned retained
  `Arc` owner, eliminating the raw-pointer/`unsafe` bridge while preserving
  the exact authored definition occurrence.
- Complete FL-A family: Component/container/transform/node/Drawable-facing
  identity, bones/Skin, all mapped constraint and scrolling families,
  ParentTraversal, advancing/resetting/virtualizing owners, and the two cold
  math utilities are translated under the 52-file specification. DrawTarget
  and DrawRules renderer-order edges are excluded from the Component
  dependency schedule as in pinned C++.
- Runtime tests: `cargo test -p nuxie-runtime --lib` is 479 / 479 green after
  root-dirt unification and retained DataBind target-dirt remediation. The
  focused owner regressions clear the root mask, dirty the retained source,
  and prove PATH/Components publication on the exact retained Component. The
  custom-source queue regression proves duplicate Bindings dirt does not
  republish target dirt before the queued occurrence is consumed, matching
  C++ `DataBind::addDirt`.
- Fresh deterministic trace:
  - construction owner resolutions 1,565 / 1,565, dependency builds
    1,455 / 1,455, and dependency sorts 24 / 24;
  - mechanism construction owner resolutions 239 / 239, dependency builds
    227 / 227, dependency sorts 8 / 8, and IK chain builds 1 / 1;
  - advancing/resetting dispatches 348 / 348 and 6 / 6; constraint
    applications 21 / 21; FollowPath rebuilds 2 / 2; Scroll child,
    physics, and virtualizer settlements 115 / 115, 6 / 6, and 2 / 2; Skin
    buffer rebuilds 1 / 1; internal owner rediscovery 0 / 0;
  - component dirt consumptions are 234 / 234 on the mechanism corpus and
    714 / 713 on the canonical corpus. The sole canonical delta is the
    pending FL-C `ListenerAlignTarget` owner.
  - successful Component dirt publications are 159 / 155 on the mechanism
    corpus and 4 / 0 on the canonical corpus. C++ breakpoint evidence assigns
    the mechanism delta to the pending FL-E
    `Text::buildRenderStyles -> Node -> LayoutComponent -> Artboard`
    callback chain; the canonical delta is two FL-C align-target writes plus
    two FL-E root-layout publications. FL-G03 and FL-G07 now own those exact
    downstream call sites.
  - unchanged-frame derived-state work is zero on both sides for dirt
    consumption, constraints, FollowPath, Skin, sorting, clipping, layout,
    and owner rediscovery. Unchanged-frame allocations remain 0 / 107 and
    stay open under FL-G07.
- Structural checker: all 23 unit/negative controls pass; current ledger
  counts are 52 faithful and 286 pending files, 47 / 75 members closed, 9 gaps
  with 7 open, and all FL-A zero-ratchets hold.
- Independent orchestrator acceptance: PROMOTE exact
  `249a66015b41190c0ec927a367c763b428f82306`. The detached acceptance checkout
  verified the corrective delta and pinned-C++ parent ownership, focused
  rejection, runtime 480 / 480, probe-armed workspace runtime 480 / 480 plus
  C++ probe 726 / 726, checker 23 / 23 with source fingerprint, format/diff,
  ordinary 317 / 317 and 647 / 647 zero, and scripted 317 / 317 and 647 / 647
  zero including `data_viz_demo` and `db_health_tracker`.
- Trace evidence is source-bound: capture fingerprints the complete tracked
  plus intended-untracked candidate (including content, executable mode,
  symlink target, and deletion state), verifies it did not change during the
  run, and the checker recomputes it. Trace/status/generated artifacts and the
  four local-only fixture symlinks are explicitly excluded; stale production
  or harness source fails closed.
- LOC-009 follow-up evidence added the FL-E `scripting.render_context` owner
  and FL-G09: pinned C++ installs one persistent ScriptingContext render
  factory before import, whereas Rust currently scopes it to selected
  callbacks. FL-E must make listener/input/path-effect/DataConverter/draw
  callbacks observe one retained factory; this is runtime callback plumbing,
  not renderer-backend ownership.
- A post-rebase probe found a separate Scene-authoring boundary outside FL-A:
  Scene-authored ComponentList padding/gap values omit explicit Yoga units,
  and mapped plain Artboard items omit the root `LayoutComponentStyle` that
  pinned C++ requires for hosted Yoga size. Rust currently compensates by
  interpreting undefined units as lengths and falling back to Artboard
  dimensions. The exploratory exact-unit/root-style production edits were
  discarded. This remains a later Scene-authoring/Layout closure; FL-A does
  not claim or alter it.
- F-ED-03B's ordinary layout/text-style characterization is absorbed by the
  existing FL-G07/FL-E owner-family closure, not FL-A. FL-E must replace broad
  `layout_epoch`/global layout-and-text invalidation for
  `LayoutComponentStyle` padding keys 512-515 and unit keys 617-620, and
  `TextStyle`/`TextStylePaint` font-size/line-height keys 274/370, with the pinned owner-local
  LayoutComponent/Yoga and Text shape-dirt callback chains. Direct and
  DataBind writes share the same acceptance matrix; the file-disjoint Scene
  authoring slice waits for that runtime owner landing.
- Final spec review removed a prematurely included LOC-007 callback slice:
  ParametricPath, Path, PathComposer, and ShapePaintPath remain one complete
  FL-E family. Per the FL-A integration boundary in
  `runtime-frame-loop-fl-a-spec.md`, FL-A does not broaden the preexisting
  Trim/Dash/Feather allowlist; its schedule adapter routes only that existing
  classifier through `RuntimeShapeList`'s already-resolved paint-owner mapping
  so the binding retained-effect floor survives occurrence identity. The full
  generated callback and Path-to-Shape-to-paint owner family, plus deletion of
  the allowlist, remain pending in the FL-E `dash.cpp`, `shape_paint.cpp`, and
  `stroke_effect.cpp` rows.
- The FL-A C++ probe no longer reads `Component::m_GraphOrder` from the
  ordinary import snapshot because that C++ member has no construction
  default. The scheduled-only runtime-update probe remains the graph-order
  oracle. Post-rebase verification exposed and removed one stale Rust
  deserializer requirement for the deleted ordinary-snapshot field.

## Next

1. Publish the current FL-B branch tip carrying production candidate
   `e8cfdc63` and evidence snapshot `10bc5b23`, without promoting pending rows.
2. Run independent acceptance; only the orchestrator verdict may promote the
   FL-B rows and close FL-G01.
3. At accepted FL-B boundary, reconcile the top-level program/status protocol
   to one canonical NEXT pointer before selecting FL-C. Performance remains
   verification evidence, never the source of a work slice.
