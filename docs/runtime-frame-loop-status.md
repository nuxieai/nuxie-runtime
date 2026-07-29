# Runtime Frame-Loop Port Status

Sole resume state for the C++-corresponding frame-loop performance closeout.

## Current

- Phase: FL-C provisional implementation. FL-B1 through FL-B4 are translated
  on provisional tip
  `0b08970fccc42ff3677534d4dcece0f05f69a0bc`, but rejected candidate
  `3ef06dd5ae07c16c5dc2aa29984412b926ae5426` remains unpromoted and every
  FL-B file/member row remains pending until reacceptance.
- Coordinator direction on 2026-07-26 authorizes dependency-ordered FL-C work
  on that provisional implementation without claiming FL-B verification.
  FL-C's corrected 56-file/eight-member lane map is
  `docs/runtime-frame-loop-fl-c-spec.md`; FL-C1 inputs/listener definitions is
  implemented with a green lane-boundary floor and pending independent
  acceptance. FL-C2 transition conditions is accepted and promoted. Every one
  of its 12 C++
  owners now has a filename-corresponding Rust module; `state_machine.rs` and
  `transition_condition.rs` retain only shared dispatch/re-exports. The pinned
  comparison matrix is represented directly: two integer comparands retain
  exact `uint32` shape, trigger pairs and ViewModel-artboard literals are
  admitted, artboard runtime properties can compare to ViewModel numbers,
  `TransitionSelfComparator` accepts every bindable kind, and a successful
  transition consumes the left ViewModel source for that layer
  (`transition_viewmodel_condition.cpp:49-60,535-970,1038-1045,1066-1124`;
  `transition_property_viewmodel_comparator.cpp:50-67`). The focused C++
  probe, four-edge end-to-end differential matrix, and existing 15-family
  condition differential set are green. Independent review rejected
  `55856a453a7598be3d734d10af881001dce7cff3` because malformed focus
  comparators were dropped, which could turn a conditional transition into
  an unconditional one. The corrected family is frozen at semantic commit
  `a40b17cd1964d46e5453b7d5278dc158ebb7b64b`: every authored focus
  condition is retained, and a missing or wrong comparator evaluates false
  exactly as `transition_focus_condition.cpp:30-39` requires. Pinned-C++
  differentials cover both malformed cases and the checker permanently
  rejects the drop shape. Its behavioral, structural, packaging, and size
  battery is green on the candidate: runtime
  509 / 509; probe-armed workspace including pinned-C++ probes 736 / 736;
  ordinary and
  scripted golden each 317 / 317 entries and 647 / 647 segments with zero
  divergences, including `data_viz_demo` and `db_health_tracker`; C API smoke;
  Apple XCFramework build/package/ABI/header/C/Swift verification; size
  8,017,864 bytes without scripting and 8,918,968 bytes with scripting, both
  below 9 MiB; and the same-runner renderer referee 1,468 / 1,468 with 1,370
  byte-exact and zero divergences. The required canonical hot-loop checkpoint
  is recorded, but remains above the final program target at 1.661x C++;
  performance therefore stays open as wave evidence and does not become an
  ad hoc scene-patching queue. Independent Standards and Spec review both
  passed exact `7546f4f15b05e582c62aa52ecc93430a7048e143` with no findings.
  The 12 mapped frame-loop file rows, the three corresponding importer rows,
  and the `state_machine.conditions` plus `state_machine.transitions` member
  rows are promoted only by this acceptance.
- Stable-Rust Apple compatibility repair
  `95eb04b7cfb847f24ba77872bd8a0ee43da1af41` mechanically rewrites the
  experimental match guard in `constraint_bounds` without changing behavior.
  Fmt, focused constraint evidence (1 / 1), runtime (491 / 491), and the full
  Apple XCFramework build/package/ABI/header/Swift verification are green.
- FL-A remains independently accepted/promoted at
  `f86d5ba0146697abc996310c62fa45e1f053144b`; exact main
  `e72323c808b91d706ba3b745396beaca7accd69a` was consumed without overlap at
  FL-B boundary merge `b5d5bc8afeaa0369cbc248b85366111649cb9010`.
  FL-B remains pending reacceptance. The complete FL-C4 listener/action family
  is independently accepted at
  `0eb48976755d759c078f1f1a032bd88590e223f7` and its exact mapped rows are
  promoted.
- Active production branch: `levi/fl-c`. The former `levi/fl-b` branch name
  described the provisional stack base, not the active wave, and is no longer
  used for FL-C publication. There is no PR.
- Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
- File closure: 93 / 341. The accepted FL-C4 family adds 25 exact
  listener/action/ScriptInput owners; 248 later-wave rows remain pending.
- Member closure: 52 / 75 owner/member rows. FL-C4 adds
  `state_machine.actions` and `state_machine.events`; 23 later-wave rows
  remain.
- Open mechanism gaps: 7 / 10. FL-G02 and FL-G06 remain closed; FL-G10 records
  the user-approved D2 saturation choice for AnimationReset's otherwise
  undefined float-to-int edge.
- Current dependency wave: FL-C. FL-B's frozen 45-file/eight-member mini-map
  remains implemented but pending reacceptance. FL-C consumes that
  implementation provisionally under the coordinator override. FL-C2 is
  independently closed. The complete five-file layer/state family (FL-C3) is
  translated at semantic commit
  `78be55ef845eb8e841d9d9ef99ba8e5732120f68` against
  `docs/runtime-frame-loop-fl-c3-closure.md`. C++'s ordered state-definition
  collection and the occurrence-owned `any`, `current`, and transition-source
  triad now have direct Rust owners. The rejected candidate's three omissions
  are closed: weighted transitions use the process-global target-specific
  random provider with exact `uint32_t` arithmetic; a NestedStateMachine keeps
  its authored owner and ordered inputs when the child occurrence is null; and
  each layer constructs and runs entry callbacks before the next layer is
  constructed. The next review exposed two additional lifecycle differences,
  now closed: a selected random transition clears an earlier candidate's
  waiting-for-exit latch after changing state, and initial entry callbacks
  cannot observe DataBind facilities that C++ constructs only after layer
  initialization. Live pinned-C++ differentials cover both cases.
  Candidate `b8d1fd6f7222fcdf3f520896f1dc8e9423d69bb8` was independently
  rejected because it still synchronized the complete focus topology before
  those callbacks. The current correction retains an empty focus-manager
  identity, lazily creates only an explicitly targeted unattached FocusNode,
  and builds the complete topology after every layer initializes. Live
  target-scope and traversal differentials match pinned C++, and the complete
  constructor availability audit now covers every entry-action facility.
  Interruption/reset/advance/nested lifecycles and all 22 structural negative
  controls are closed. The self-excluding
  `docs/runtime-frame-loop-trace.json` now records committed source
  `6674aee34c07d95707bc0e2f737540a3b5633cb4`, candidate-source fingerprint
  `0936a1cf2721beb9e702845c062d748872ebe4019a6f8307575cef4e8ad9dd33`,
  and exact runner provenance. The coordinator promoted exact candidate
  `975962ccb22c3089620ab1f4a735e502e51d7ef1` after the independent production
  review passed focus, random-wait, DataBind-order, RNG, nullable nested-owner,
  and family-boundary fidelity. The five public file rows and
  `state_machine.layer` are promoted only by the dedicated reconciliation
  commit. The supporting `src/math/random.cpp` row remains pending for its
  later FL-D formula consumer.
- FL-C3's fresh once-per-candidate non-performance floor is green on the
  corrected source: runtime 521 / 521; public facade 146 / 146; probe-armed
  workspace and pinned-C++ probes 747 / 747; ordinary and scripted golden each
  317 / 317 entries and 647 / 647 segments with zero divergences, including
  `data_viz_demo` and `db_health_tracker`; same-runner pixels 1,468 / 1,468
  with 1,370 byte-exact, zero divergences, and zero gated rows; C API, native
  Apple, browser build, lint, format, and diff checks; size 8,034,568 bytes
  without scripting and 8,935,672 bytes with scripting, both below 9 MiB; and
  the full Apple XCFramework build/package/ABI/header/C/Swift floor with
  checksum
  `ca1b605a66062dce9d1cff1d139fd4125fd30fe2323f661e84daa6223885d50d`.
  The source-bound trace reports 103 / 341 reached C++ files, 29 Rust modules,
  and all 18 frame landmarks. Structural checker 37 / 37 is green. No
  performance measurement was run.
- The complete listener/action/event/focus-dispatch family (FL-C4) is
  production-complete on
  `6f008b5b8acba0b93d1405aff0f0a08583138ca9`. Immutable candidate
  `f2819cda3836846df6017cc7be747fdeb03dcb67` was rejected because unbound
  Rust performed an invented third no-context scripted-listener attempt. The
  correction stops after C++'s two cold attempts and defers occurrence three
  to the first genuine DataContext attachment. The other final lifecycle
  corrections preserve cold clone/reinit, distinguish prebound constructor
  hydration from later context binding, share one File VM/program identity
  across root and child occurrences, bind the Artboard before its
  StateMachine, and run one ignored-result detached-ViewModel tail only after
  a root StateMachineInstance host advance. The exact differential and
  parser-backed `scripted_object_unbound_constructor_enters_live_context`
  ratchet are green; the final pinned-C++ and adversarial ratchet audits are
  behavior/ownership clean.
- First evidence candidate
  `5aeb4c06aa9cc7f29a52b7d053814045f15f0539` passed independent
  behavior/oracle review but failed executable-scope review: six complete
  scalar/ViewModel ScriptInput owners were absent from the active family list,
  while the partial component-owned `script_input_artboard.cpp` row was
  assigned to FL-C instead of FL-D. The replacement packet includes those six
  complete rows in FL-C4, restores the Artboard row to FL-D, and leaves every
  candidate row pending.
- FL-C4's fresh non-performance floor is green: runtime 665 / 665; public
  facade 146 / 146; probe-armed workspace and pinned-C++ comparisons
  759 / 759; ordinary and scripted golden each 317 / 317 entries and
  647 / 647 segments with zero divergences; static pixels 1,468 / 1,468 with
  837 byte-exact and same-runner pixels 1,468 / 1,468 with 1,370 byte-exact,
  both with zero divergences; C API, native Apple, browser, lint, format, and
  diff checks; size 8,151,336 / 9,252,072 bytes under 9 MiB; Apple
  XCFramework checksum
  `316fad479f4a764610db39f94e5621330f9fc337a5d35597696aecd800b7f11c`;
  trace fingerprint
  `7f202a118e462fe298b88e7c56a76e0e8aec761e48876c12d242351180635320`;
  103 / 341 dynamically reached C++ files, 34 Rust modules, all 18 landmarks;
  and structural checker 41 / 41. No performance measurement was run.
  Independent Standards and Spec/oracle reviews both passed exact
  `0eb48976755d759c078f1f1a032bd88590e223f7` with no findings. The exact 25
  file owners and two member rows are promoted. Accepted closure is now
  93 / 341 files and 52 / 75 members, with 7 / 10 mechanism gaps open.
- FL-C1 input ownership is now source-corresponding: `state_machine_input.rs`
  owns the authored definition and `state_machine_input_instance.rs` owns the
  mutable occurrence. Each occurrence retains a handle into the one authored
  input arena and reads id/name/kind through it, matching pinned
  `SMIInput::m_input`; only bool/number/trigger state is copied into the
  occurrence. The two input files and the `state_machine.inputs` member remain
  pending until the complete 12-file input/listener lane is translated and
  independently accepted.
- FL-C1 listener qualification found four real absent branches rather than
  benchmark defects: import currently discards authored Keyboard,
  SemanticAction, Gamepad, and TextInput listener types, and the public runtime
  has no matching dispatch surface. Pinned C++ owns their constraints on the
  typed listener-input definitions and their mutable dispatch state on fresh
  per-StateMachineInstance listener groups. The definitions are complete;
  the dispatch groups now sit beside their actual listener-action and
  TextInput dependencies. No Editor or renderer workaround is permitted.
- FL-C1 map correction: pinned `KeyboardInput`, `GamepadInput`, and
  `SemanticInput` each perform a distinct importer handoff into their typed
  listener-input owner. The original 13-file map omitted those three required
  C++ sources, so the executable source set now includes `src/inputs/*.cpp`
  and the definition lane contains 12 files after the dispatch-group
  dependency correction below. This is scope completion, not a new feature
  choice; all three rows remain pending.
- FL-C1 now has direct Rust files for all four typed listener-input
  definitions and the three concrete input records. Keyboard, gamepad, and
  semantic constraints match their C++ branch order and wildcard rules.
  ViewModel listeners no longer discard every authored path after the first:
  one occurrence retains the authored listener-definition arena, owns one
  property binding per typed ViewModel input in order, and routes every
  mutation to the same parent listener queue entry
  (`listener_input_type_viewmodel.cpp`;
  `state_machine_instance.cpp:1324-1489,3021-3025`). Focused definition,
  import, and occurrence/FIFO tests plus the full 507 / 507 runtime floor are
  green.
- The later invocation owner required by those groups has been split
  behavior-preservingly from giant `scripting.rs` into the direct
  `state_machine/listener_invocation.rs` correspondence. Its existing
  pointer/reported-event/none behavior is unchanged and green under runtime
  507 / 507 plus all three scripting invocation tests. Keyboard, text,
  gamepad, focus, ViewModel, and semantic variants remain pending semantic
  translation; the row is not promoted by the file move.
- Pinned-source dependency review corrected the mini-map without changing
  total scope. Keyboard, gamepad, and semantic listener groups moved from
  FL-C1 to FL-C4 beside the invocation/action owners they call.
  `text_input_listener_group.cpp` moved from FL-C to FL-E beside
  `text_input.cpp`, because its pointer body is only drag/focus/selection
  calls on that incomplete owner. This forbids partial groups, placeholder
  `None` invocations, and invented Rust editing behavior. FL-C1 is therefore
  implementation-complete at 12 source-corresponding files; all rows remain
  pending until lane acceptance.
- FL-C1 boundary floor at `6bca78080cddf94d983ca03f9460ebb129d28477`
  is green: fmt; runtime 507 / 507; probe-armed workspace including C++ probe
  727 / 727; ordinary and scripted golden each 317 / 317 entries and 647 /
  647 segments with zero divergences; C API smoke; Apple XCFramework
  build/package/ABI/header/Swift verification; size 7.66 MiB without
  scripting and 8.51 MiB with scripting under the 9 MiB budget; and the
  same-runner Dawn pixel corpus 1,468 / 1,468 with 1,370 byte-exact and zero
  divergences. This evidence does not promote the pending FL-C1 rows.
- Editor defect RT-ED-007 overlaps the state-machine instance owner currently
  being ported. Pinned C++ retains each unresolved DataBindContext path on the
  StateMachineInstance before live data exists, then resolves it from
  `internalDataContext` when the context is attached
  (`state_machine_instance.cpp:1742-1766,2901-2905`;
  `data_bind_container.cpp:25-33`). Rust previously required the authored
  default instance to resolve the complete transition-duration source and
  silently discarded nested paths when that child was not materialized yet.
  Runtime Fix owns the correction on `levi/fl-b` in
  `state_machine/transition_duration_binding.rs`, `state_machine/bindables.rs`,
  and the existing `data_bind_graph.rs` live-context resolver. The focused
  unresolved-path regression, the unchanged nested set/fire/`advance(0)`
  occurrence-isolation Scene regression (1 / 1 in an isolated active-runtime
  overlay), and runtime 508 / 508 floor are green. The row remains pending
  until the complete StateMachineInstance owner is accepted.
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
- The typed Rust replay reinterprets color bits as C++ signed `int`, retains
  the exact serialized `float`, and converts that float back to signed `int`
  on replay. The pinned probe locks the observable non-exact case
  `0x011d1d1d -> 0x011d1d1c`. The narrow positive range that rounds to 2^31
  follows user-approved project decision D2's Rust saturation because C++ has
  no defined result there; FL-G10 records that safety boundary. The former
  integer bypass, flat cloneable entry owner, global `(object, property)`
  membership vector, and empty-entry `None` elision are deleted.
- Focused first-seen order, supported-family filtering, shared-lease, replay,
  and owned-empty-reset coverage is green. Full local floors: runtime
  491 / 491; public facade 171 / 171; probe-armed workspace including C++
  probe 727 / 727; ordinary and scripted golden each 317 / 317 entries and
  647 / 647 segments with zero divergences, including `data_viz_demo` and
  `db_health_tracker`; all 29 checker/capture/summarizer controls. Source
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
  Current local floors: runtime 491 / 491; probe-armed workspace runtime
  491 / 491, facade 171 / 171, and C++ probe 727 / 727; ordinary and scripted
  golden each 317 / 317 entries and 647 / 647 segments with zero divergences;
  source-bound trace 103 / 338 C++ files, 18 Rust modules, and 18 landmarks;
  structural checker and its negative controls 29 / 29; renderer pixel referee
  1,468 / 1,468 with 1,370 byte-exact, zero divergences, and zero gated
  failures; C API smoke green; Apple release/product floor 66 / 66 plus
  artifact validation 15 / 15; lint, format, and diff checks green.
  Committed-tree size at `89e4e3b1` is 8,017,816 bytes with scripting off and
  8,918,920 bytes with scripting on,
  both below the 9 MiB budget.
- Canonical whole-corpus FL-B `perf-hot-loop` checkpoint at `89e4e3b1`:
  `docs/evidence/perf-hot-loop-fl-b-89e4e3b1.json`, tracked with SHA-256
  `3857f02a3a6b80c21d92a871e3a7ef7862adebac7dd167e677cf8cbfb7514987`.
  Across the unchanged six-entry / 11-sample corpus and 10,000 benchmark
  repeats, the minimum aggregate is 1.628x C++ (42.982 ms C++, 69.988 ms
  Rust); individual samples range from 1.158x to 2.026x. This remains above
  the program's final <=1.0x acceptance target. Per the frozen FL-B plan it is
  wave evidence only: it neither promotes FL-B nor authorizes
  benchmark-derived work or queue reordering.

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

- User direction on 2026-07-26 supersedes family- and wave-level timing:
  run no further performance tests until every mapped FL-A-through-FL-E code
  row is ported and the complete correctness/structure floor is green. The
  historical checkpoints below remain context only. They are not current
  candidate gates and may not select implementation work.
- FL-C2 corrected-family checkpoint on exact semantic commit
  `a40b17cd1964d46e5453b7d5278dc158ebb7b64b`:
  `docs/evidence/perf-hot-loop-fl-c2-a40b17cd.json`, tracked with SHA-256
  `96fec470fb7f144daa411a9362b254aff5494fa47551fcbcaf85ae7953d40afc`.
- Canonical six-entry / 11-sample aggregate: 1.661x C++
  (41.103 ms C++, 68.289 ms Rust, minimum aggregate, 10,000 repeats).
  The unchanged 1.0x final threshold correctly rejected the measurement.
  This is an open whole-program performance result, not a transition-family
  semantic rejection and not authorization for benchmark-driven patching.
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

1. Implement FL-C5 per the committed family map: the binding checklist is
   `runtime-frame-loop-fl-c5-closure.md`, the dependency-ordered writer
   packages are `runtime-frame-loop-fl-c5-impl-spec.md` (WP0 file split
   first), and the complete source-walk evidence is
   `runtime-frame-loop-fl-c5-walk/`. One production writer; no semantic edit
   outside the packages.
2. FL-C1: five independent audit rounds. The claimed-`DataBindPath`
   corrections (non-relative `ec4d13f0`, relative name resolution
   `82e229f3`, unmapped-name-ID empty-string fallback plus boundary
   restoration `69fee252`) are all independently confirmed correct.
   Making the probes genuinely differential (`20cd8c02`) then demonstrated
   a real one-advance ViewModel-listener firing-boundary divergence (Rust
   applies the change one advance earlier than pinned C++'s
   queue-then-next-new-frame `applyEvents`). That gap is owned by the
   FL-C5 WP6 event rows as recorded gap `flc5-vm-listener-firing-boundary`;
   the claimed-path probes must pin the current divergence explicitly (no
   loosened comparisons) until WP6 restores the C++ boundary. FL-C1
   acceptance is pending one further round under that recorded-gap
   disposition. FL-B retains its separate pending acceptance state.
   Performance remains deferred until all mapped FL-A-through-FL-E code is
   ported.
