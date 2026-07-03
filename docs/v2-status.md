# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 95 across 70 exact files
- Parked breakdown (from `make golden-compare`): M3=21 M4=83 M5=8 M6=72 gated=5 harness=36
- Current milestone: **M2 — Animated Playback Exact + Real Object Model (#V2-3)**

## Milestones

- [x] M0: Golden diff harness + corpus manifest + one exact file
- [x] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [ ] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [ ] M3: Interactive files exact under scripted pointer input
- [ ] M4: Nested artboards/lists exact
- [ ] M5: Data binding exact incl. external view-model mutation
- [ ] M6: Layout + text verified per declared corpus modes; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. Continue M2 real object model work by modularizing the remaining
   animation/state-machine surfaces out of `lib.rs` while keeping generated
   `InstanceObjectStorage` as the authored-property source of truth, but only
   when it unblocks a corpus diff or removes risky coupling. Component
   dirt/runtime transform state live in
   `crates/rive-runtime/src/components.rs`, the linear animation runtime model
   and import builder live in `crates/rive-runtime/src/animation.rs`, and
   state-machine inputs/events/listener/fire actions/view-model trigger runtime
   state seed `crates/rive-runtime/src/state_machine.rs`.
2. Resume M2 sample widening after the object-model/modularization queue moves:
   pick the next small animated `exact` corpus file still pinned to sample `0`,
   add the first non-zero sample in a focused corpus, and either keep it exact
   by porting the first divergence or record the narrower blocker if it crosses
   into a later milestone.
3. Add handle-source world-space math and nested-remap dependent advancement
   to the joystick path when a corpus diff reaches those cases.

## Known Divergences

- None currently tracked for M1; remaining non-exact files are parked with
  later-milestone diagnostics or unsupported-feature gates.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until M5 external data-binding corpus files require it.
- Rust golden draw path currently supports sorted absolute-time samples,
  artboard clip/background, selected-artboard origins, solid fills/strokes, and
  `ClippingShape` clip paths, skinned `PointsPath` deformation, plus empty and
  multi-contour TrimPath effects, DashPath stroke effects, and linear/radial
  gradient shader creation, default state-machine frame-0 application for
  color/bool/uint/string keyframes, Solo active-child refresh, and
  before-update joystick animation application, keyed double/color
  interpolation for CubicEase/CubicValue/Elastic keyframe interpolators without
  custom handle-source world-space math or nested remap dependent advancement.
  Golden runner sample lists now advance by sorted absolute-time deltas and reuse render paths
  across samples;
  no images, text, nested artboards, constraints, or scripted input.
- Per-file parked reasons now live in `corpus.toml`: each gated entry
  carries `milestone = "M3|M4|M5|M6|gated|harness"` plus its diagnostic
  feature tags (`rust-runner-unsupported:*`, `cpp-runner-crash`,
  `import-error:*`). Query a milestone's work-list with e.g.
  `grep -B6 'milestone = "M4"' corpus.toml`.
- Entries tagged `cpp-runner-crash` (`milestone = "harness"`) stay parked
  until the C++ golden runner survives the FileAssetContents, scripting,
  and data-viz crash paths it currently aborts on.
- `solar-system.riv` stays gated on a Rust import gap: `blendModeValue = 5`
  rejected on Shape object 13.

## Decisions

- 2026-07-02: V2 map adopted (`docs/porting-map-v2.md`); V1 map superseded, its contract suite frozen as regression floor.
- 2026-07-02: Golden runner records decoded image payloads by size/hash for the first renderer slice; real decoded dimensions are deferred until `rive_decoders` is wired into the CLI harness build.
- 2026-07-02: Golden runner emits one accumulated stream per run with
  `source`, `input`, `sample`, and `frame` markers; `golden-compare` will split
  sample segments from that stream.
- 2026-07-02: `rive-render-api` owns the renderer seam; `rive-runtime` should
  drive those traits when static drawing moves from reports to real rendering.
- 2026-07-02: `golden-compare` validates the C++ stream for `not-yet` entries
  and refuses `exact` entries unless a Rust runner is supplied, keeping the
  exact count honest while the Rust draw path is still absent.
- 2026-07-02: First exact file is `dependency_test.riv`; the Rust runner
  preallocates source + instance render paints to mirror C++ import/clone
  paint lifetimes before drawing.
- 2026-07-02: `tools/golden-compare --bin generate-corpus` generates the
  corpus manifest from the C++ unit-test assets, preserving exact/unsupported
  annotations across regenerations.
- 2026-07-02: CI pins the reference C++ runtime to
  `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2` and builds root
  `premake5_v2.lua` debug libraries before running `make golden-compare`.
- 2026-07-02: `rive-runtime` owns static draw emission through
  `rive-render-api`; `rust-golden-runner` now only orchestrates import,
  artboard selection, stream markers, and recording output.
- 2026-07-02: Static rendering applies artboard origin as a top-level draw
  transform and preallocates clone render paints only for the selected
  artboard, matching C++ multi-artboard import/draw behavior.
- 2026-07-02: Empty effect paths are distinct from no effect path;
  `RuntimeShapePaintCommand` tracks whether a supported effect exists so C++
  empty TrimPath output is preserved.
- 2026-07-02: Effect-bearing selected-artboard paints preallocate before the
  remaining local paint order, matching C++ clone paint IDs for `trim.riv`
  without regressing `dependency_test.riv` or `shapetest.riv`.
- 2026-07-02: Corpus features prefixed `rust-runner-unsupported:` are verified
  by `golden-compare` when `--rust-runner` is supplied; use them when a
  later-phase feature would otherwise be silently omitted by Rust rendering.
- 2026-07-02: `exact` is scoped to the samples/scripts in `corpus.toml`;
  animated files may be exact at sample `0` now and still need wider M2 samples
  later.
- 2026-07-02: `golden-compare` exact stream comparison uses numeric-token
  epsilon `1e-4` while keeping call order, IDs, verbs, and non-numeric text
  exact, matching the V2 renderer seam plan.
- 2026-07-03: `rive-renderer-ffi` native mode now has a local null-context
  fallback that compiles the C++ renderer sources needed by
  `RenderContextNULL` when `librive_pls_renderer.a` is absent; the
  `ffi_null_draw` example imports `dependency_test.riv` and drew 3 calls
  through `FfiFactory`/`FfiFrame` into C++ `RiveRenderer`. Full
  visible/offscreen Metal remains blocked on the machine missing Apple's Metal
  Toolchain while building the renderer archive (`xcodebuild
  -downloadComponent MetalToolchain`).
- 2026-07-02: Instance `RenderPaint` ID allocation follows C++ import-time
  `ShapePaintMutator` object order, not Fill/Stroke object order and not draw
  order; Rust preallocates by mutator owner first, then falls back to any
  unallocated Fill/Stroke.
- 2026-07-02: Rust golden runner scene markers follow C++
  `defaultStateMachine()` selection by checking whether
  `defaultStateMachineId` was serialized on the selected artboard and treating
  the value as a state-machine index; schema default values alone do not
  select a state machine.
- 2026-07-02: Runtime composed shape paths default to C++
  `ShapePaintPath` fill rule `clockwise`; Fill paints still override the
  path fill rule immediately before draw, while Stroke paints preserve the
  composed path default.
- 2026-07-02: Imported Solo collapse mirrors `src/solo.cpp` for static state:
  constraints and clipping shapes inherit the Solo's collapse value, while
  participating children collapse unless they match the imported
  `activeComponentId` resolved through the artboard-local object table.
- 2026-07-02: Delegated subsystems (#V2-7) use Rust-native libraries instead
  of FFI, chosen by "spec-defined may swap engines; implementation-defined may
  not": Taffy (layout, behind a trait, Yoga-FFI as untriggered fallback),
  HarfRust + read-fonts/skrifa (shaping/font parsing), unicode-bidi (bidi),
  `image`-ecosystem crates (decoders), cpal/rodio (audio), and mlua+`luau`
  vendoring the official Luau VM (scripting — same VM as C++, so scripted
  files stay `exact`). `corpus.toml` gains per-entry verification modes
  `exact | tolerant(ε) | structural`; files exercising Taffy layout, HarfRust
  text, or lossy image decoding verify `tolerant`, everything else stays
  `exact`. Cross-runtime image comparison must use decoded dimensions +
  tolerant pixel sampling, never payload hashes (supersedes the size/hash
  recording decision above once Rust image support lands). Do not pin Taffy
  against Yoga behavior-by-behavior. Taffy CSS Grid is a post-M7 enhancement
  idea, not port scope.
- 2026-07-03: Metric is now segments-weighted: `golden-compare` reports
  `exact-segments` (sum of samples across exact entries) alongside the file
  count, so M2 sample widening registers as metric movement. Gated corpus
  entries carry `milestone = "M3|M4|M5|M6|gated|harness"` (preserved by
  `generate-corpus`), and the summary prints a parked-by-milestone
  breakdown, so each milestone's work-list is queryable from `corpus.toml`
  instead of backlog prose. Completed-milestone log entries are archived in
  `docs/v2-log-archive.md` to keep this file small.

## Log

- Completed-milestone entries (M0, M1) are archived verbatim in
  `docs/v2-log-archive.md`; when a milestone completes, move its entries
  there and keep only the active milestone's entries here.
- 2026-07-03: [M2] Added the first real-object-model tracer: `ArtboardInstance`
  now owns a cloned object arena built from imported slots, and schema-keyed
  color/bool/uint/string animation getters/setters mutate cloned
  `RuntimeObject` properties instead of side overlay maps. Verified
  `make golden-compare` (`exact=54`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=17`) and `cargo test --workspace`;
  next M2 work is replacing the generic arena internals with generated
  concrete object storage and generated setter/getter dispatch.
- 2026-07-03: [M2] Mirrored C++ golden default-scene startup in the Rust golden
  runner by selecting the serialized default state machine and advancing it at
  sample `0` before draw. Promoted `click_event.riv`,
  `event_trigger_event.riv`, and `sound.riv` as exact after direct stream
  comparisons; `make golden-compare` reports `exact=57`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=14`. `solo_test` and
  `solos_collapse_tests` still differ in Solo active-child refresh after
  frame-0 `KeyFrameId`.
- 2026-07-03: [M2] Ported the first generated-setter side effect into the
  runtime object arena path: `Solo.activeComponentId` uint/id writes now
  re-run C++ `Solo::propagateCollapse` using instantiated Solo child metadata.
  Promoted `solo_test.riv` and `solos_collapse_tests.riv` after direct stream
  comparisons; expected `make golden-compare` summary is `exact=59`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=12`.
- 2026-07-03: [M2] Ported the sample-0 C++ `Joystick::apply`/artboard
  `updatePass` path for joysticks that can apply before update. The Rust
  golden runner now calls `ArtboardInstance::update_pass()`, and
  `joystick_flag_test.riv` stream-matches C++ alongside the existing
  `joystick_nested_remap.riv` exact check; expected `make golden-compare`
  summary is `exact=60`, `diverges=0`, `unsupported-feature=224`,
  `not-yet=11`.
- 2026-07-03: [M2] Ported C++ golden-runner absolute sample advancement into
  the Rust runner and added a scene-long render path cache so artboard clips,
  backgrounds, clipping shapes, and draw paths retain C++ path ids across
  emitted samples. Promoted `clip_tests.riv` and `pointer_events.riv` after
  direct stream comparisons; `make golden-compare` reports
  `exact=62`, `diverges=0`, `unsupported-feature=224`, `not-yet=9`.
- 2026-07-03: [M2] Added live double-property animation writes for cloned
  runtime objects and made TrimPath effects read live `start`/`end`/`offset`
  and `modeValue` from the instance. Also ported clockwise fill path reversal
  instead of dropping reversed local-clockwise paths. Promoted
  `trim_path_linear.riv`; `make golden-compare` reports `exact=63`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=8`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Fixed keyed-property metadata lookup to use the imported
  `KeyedObject.objectId` slot rather than the remapped runtime-local id,
  allowing frame-0 `KeyFrameDouble` writes to reach TrimPath effects whose
  local ids diverge from C++ artboard-local ids. Promoted
  `fill_trim_path.riv`; `make golden-compare` reports `exact=64`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=7`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Rechecked remaining M2 `not-yet` sample-0 files after the
  live keyed-property/state-machine work and promoted `opaque_hit_test.riv`
  and `quantize_test.riv` after direct C++/Rust stream comparisons matched.
  `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Started the real object model replacement by routing cloned
  object arena writes through generated CoreRegistry setter-family metadata,
  rejecting wrong-family and non-setter/encoded property writes before
  mutating the `RuntimeObject` property bag. Exact count remains 66;
  `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Split cloned artboard object mutation off
  `RuntimeObject` by introducing runtime-local `InstanceObject` storage in
  `InstanceObjectArena`; reads still honor schema stored-field defaults and
  writes still validate generated setter families. Exact count remains 66;
  `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Moved arena mutation storage from imported
  `RuntimeProperty`/`FieldValue` objects into runtime-owned
  `InstanceProperty`/`InstancePropertyValue`, keeping binary import values as
  clone-time input only. Exact count remains 66; `make golden-compare`
  reports `exact=66`, `diverges=0`, `unsupported-feature=224`, `not-yet=5`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Extracted `InstanceObjectArena` and runtime-local instance
  property storage into `crates/rive-runtime/src/objects.rs`, leaving
  `lib.rs` to call the arena through the same typed accessors while the next
  generated-storage pass has a focused module target. Exact count remains 66;
  `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Added build-generated per-type
  `InstanceObjectStorage` for cloned artboard objects, with schema-derived
  typed fields, imported-property application, generated property-key
  getters/setters, Artboard `clip` default handling, and encoded byte payload
  storage. Exact count remains 66; `make golden-compare` reports `exact=66`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Made clone-time `RuntimeComponent` transform
  initialization read from generated `InstanceObjectStorage` through
  concrete object property-name lookup, so imported Node/vertex transform
  fields flow through the cloned arena before component state. Exact count
  remains 66; `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Routed live transform mutation through generated
  `InstanceObjectStorage` by concrete object property name before syncing the
  `RuntimeComponent` mirror, and updated runtime tests to carry generated
  synthetic Node/vertex storage. Exact count remains 66; `make golden-compare`
  reports `exact=66`, `diverges=0`, `unsupported-feature=224`, `not-yet=5`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Removed authored x/y/rotation/scale/opacity mirrors from
  `TransformRuntimeState`; transform update and render-opacity update now read
  generated `InstanceObjectStorage` through `ArtboardInstance` transform
  accessors, leaving `RuntimeComponent` with only derived local/world/render
  transform state. Exact count remains 66; `make golden-compare` reports
  `exact=66`, `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Extracted component dirt bits, runtime component transform
  state, `Mat2D`, and component update methods into
  `crates/rive-runtime/src/components.rs`, shrinking the monolithic runtime
  file while preserving the public re-exports used by probes and downstream
  crates. Exact count remains 66; `make golden-compare` reports `exact=66`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Extracted `LinearAnimationInstance` playback state and
  loop-kind handling into `crates/rive-runtime/src/animation.rs`, preserving
  the existing public re-export while leaving `lib.rs` with the remaining
  linear-animation import/keyframe model and state-machine surfaces to peel
  next. Exact count remains 66; `make golden-compare` reports `exact=66`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved `RuntimeLinearAnimation`, keyed objects/properties,
  keyframe structs, and keyframe sampling helpers into
  `crates/rive-runtime/src/animation.rs`, keeping the import-time builder in
  `lib.rs` and preserving public re-exports for the runtime probe surface.
  Exact count remains 66; `make golden-compare` reports `exact=66`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Seeded `crates/rive-runtime/src/state_machine.rs` with
  `StateMachineReportedEvent`, preserving the public re-export while moving a
  shared animation/state-machine event report surface out of `lib.rs`. Exact
  count remains 66; `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Moved `RuntimeStateMachineInput`,
  `StateMachineInputKind`, and `StateMachineInputInstance` into
  `crates/rive-runtime/src/state_machine.rs`, keeping `StateMachineInputValue`
  private behind crate-visible constructors and preserving the public input
  accessors. Exact count remains 66; `make golden-compare` reports `exact=66`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=5`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved scheduled listener actions and the shared
  `StateMachineFireOccurrence` timing enum into
  `crates/rive-runtime/src/state_machine.rs`, keeping listener import and input
  mutation beside the state-machine input runtime model while leaving
  view-model trigger fire actions in `lib.rs` until their bindable trigger
  dependencies are extracted. Exact count remains 66; `make golden-compare`
  reports `exact=66`, `diverges=0`, `unsupported-feature=224`, `not-yet=5`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved `StateMachineViewModelTriggerInstance` into
  `crates/rive-runtime/src/state_machine.rs`, keeping imported
  `RuntimeViewModelTrigger` data in `lib.rs` and routing default/imported/owned
  trigger binding through crate-visible accessors. Exact count remains 66;
  `make golden-compare` reports `exact=66`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=5`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Moved `RuntimeStateMachineFireAction`,
  `perform_state_machine_fire_actions`, and fire-trigger target resolution into
  `crates/rive-runtime/src/state_machine.rs`, now that view-model trigger
  runtime state lives there. Exact count remains 66; `make golden-compare`
  reports `exact=66`, `diverges=0`, `unsupported-feature=224`, `not-yet=5`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Ported keyed-frame interpolator application for linear
  animation sampling by resolving artboard-local `KeyFrameInterpolator`
  objects into the runtime animation model and applying CubicEase,
  CubicValue, and Elastic behavior for double/color keyframes. Promoted
  `cubic_value_test.riv` and `oneshotblend.riv` to exact;
  `make golden-compare` reports `exact=68`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=3`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Matched C++ rounded-corner midpoint precision by using
  fused `scaleAndAdd` math while keeping exact duplicate segment pruning.
  Promoted `juice.riv` to exact; `make golden-compare` reports `exact=69`,
  `diverges=0`, `unsupported-feature=224`, `not-yet=2`. Next M2 exact-count
  target is the remaining `rocket.riv` rounded path residual.
- 2026-07-03: [M2] Matched rotated local path cancellation for `rocket.riv` by
  using fused path-local composition for visibly rotated/skewed matrices while
  preserving axis-aligned cancellation for `juice.riv`. Promoted `rocket.riv`
  to exact; `make golden-compare` reports `exact=70`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=1`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Classified `interpolation_zero_duration.riv` under the M5
  data-binding transform bucket by extending the Rust golden runner diagnostic
  to interpolated shape transform binds. `make golden-compare` reports
  `exact=70`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `cubic_value_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping its CubicValue/CubicEase animated stream exact.
  Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `looping_timeline_events.riv` from sample `0` to
  samples `0` and `0.25`, keeping its callback/event timeline stream exact.
  Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `test_elastic.riv` from sample `0` to samples `0`
  and `0.25`, keeping ElasticInterpolator animated playback exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `quantize_test.riv` from sample `0` to samples `0`
  and `0.25`, keeping its quantized animated stream exact. Exact count remains
  70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `timeline_event_test.riv` from sample `0` to
  samples `0` and `0.25`, keeping callback/event timeline playback exact.
  Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `scripted_string.riv` from sample `0` to samples
  `0` and `0.25`, keeping its view-model string/state-machine playback stream
  exact. Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `multiple_state_machines.riv` from sample `0` to
  samples `0` and `0.25`, keeping multi-state-machine sample playback exact.
  Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `settler.riv` from sample `0` to samples `0` and
  `0.25`, keeping its CubicEase animated playback stream exact. Exact count
  remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `scripted_boolean.riv` from sample `0` to samples
  `0` and `0.25`, keeping its view-model boolean/state-machine playback stream
  exact. Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `oneshotblend.riv` from sample `0` to samples `0`
  and `0.25`, keeping its one-shot blend-state playback stream exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `stroke_name_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping its stroked state-machine playback stream exact.
  Exact count remains 70; focused golden compare reports `exact=1`,
  `diverges=0`, `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `state_machine_triggers.riv` from sample `0` to
  samples `0` and `0.25`, keeping trigger-transition playback exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `solo_test.riv` from sample `0` to samples `0` and
  `0.25`, keeping Solo active-child playback exact. Exact count remains 70;
  focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `dependency_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping its animated dependency playback stream exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `light_switch.riv` from sample `0` to samples `0`
  and `0.25`, keeping bool-transition state-machine playback exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `two_artboards.riv` from sample `0` to samples `0`
  and `0.25`, keeping multi-artboard animated playback exact. Exact count
  remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `event_on_listener.riv` from sample `0` to samples
  `0` and `0.25`, keeping listener-event state-machine playback exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `events_on_states.riv` from sample `0` to samples
  `0` and `0.25`, keeping state-machine fire-event playback exact. Exact count
  remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `joystick_flag_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping joystick/state-machine flag playback exact. Exact
  count remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `blend_test.riv` from sample `0` to samples `0`
  and `0.25`, keeping direct/1D blend-state playback exact. Exact count
  remains 70; focused golden compare reports `exact=1`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Tripwire fired: repeated sample-widening commits kept the
  project at `exact=70`, so the queue now pivots back to the M2 real object
  model/modularization work before harvesting more sample-only coverage.
- 2026-07-03: [M2] Modularized solo collapse runtime into `components.rs` and
  joystick runtime metadata into `animation.rs`, keeping authored-property
  mutation routed through `InstanceObjectArena`. Exact count remains 70;
  `make golden-compare` reports `diverges=0`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved `InstanceSlot` into `objects.rs` with
  `InstanceObjectArena` and moved the self-contained state-machine input
  importer into `state_machine.rs`. Exact count remains 70; `make
  golden-compare` reports `diverges=0`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved the linear animation import builder and its private
  keyframe/import helpers into `animation.rs`, leaving shared property lookups
  in `lib.rs` for the state-machine/data-binding code still parked there.
  Exact count remains 70; `make golden-compare` reports `exact=70`,
  `diverges=0`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `animation_reset_cases.riv` from sample `0` to
  samples `0` and `0.25`, keeping its reset/blend-state playback stream exact.
  Exact segments are now 94 across 70 exact files; focused golden compare
  reports `exact=1`, `exact-segments=2`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
- 2026-07-03: [M2] Widened `bindable_artboard_child.riv` from sample `0` to
  samples `0` and `0.25`, keeping its bindable artboard/state-machine playback
  stream exact. Exact segments are now 95 across 70 exact files; focused
  golden compare reports `exact=1`, `exact-segments=2`, `diverges=0`,
  `unsupported-feature=0`, `not-yet=0`.
