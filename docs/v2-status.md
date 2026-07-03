# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 151 across 70 exact files
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

1. Continue M2 real object model work by modularizing the remaining runtime
   surfaces out of `lib.rs` while keeping generated `InstanceObjectStorage` as
   the authored-property source of truth, but only when it unblocks a corpus
   diff or removes risky coupling. Component dirt/runtime transform state live
   in `crates/rive-runtime/src/components.rs`, the linear animation runtime
   model and import builder live in `crates/rive-runtime/src/animation.rs`,
   and state-machine import data, bindables, transition conditions, layer
   advancement, and `StateMachineInstance` orchestration live under
   `crates/rive-runtime/src/state_machine/`. Remaining root coupling is mostly
   the data-bind graph/default-view-model bridge and artboard-level data-bind
   helpers; move those only with a corpus diff or a clear M2 coupling payoff.
2. Add handle-source world-space math and nested-remap dependent advancement
   to the joystick path when a corpus diff reaches those cases.
3. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them for M2 unless a related refactor needs a cheap draw-regression check.

## Known Divergences

- None currently tracked for M1/M2; remaining non-exact files are parked with
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
  there and keep only the active milestone's recent working window here.
- Older active M2 entries are archived verbatim in `docs/v2-log-archive.md`
  under `M2 active log rolloff`; keep only the recent rolling window here once
  Metric, Next, Decisions, and `corpus.toml` capture the current state.

- 2026-07-03: [M2] Widened `off_road_car.riv` from sample `0` to samples `0`
  and `0.25`, keeping its animated skinned vector/path playback stream exact.
  Exact segments are now 129 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=129`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `script_paths_opacity_test.riv` from sample `0` to
  samples `0` and `0.25`, keeping its scripted-drawable opacity/keyed-double
  playback stream exact before M6 scripting work. Exact segments are now 130
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=130`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `script_paths_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping its scripted-drawable/keyed-double playback stream
  exact before M6 scripting work. Exact segments are now 131 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=131`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `sound.riv` from sample `0` to samples `0` and
  `0.25`, keeping its passive audio-event/state-machine render stream exact
  before M6 audio work. Exact segments are now 132 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=132`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `sound2.riv` from sample `0` to samples `0` and
  `0.25`, keeping its passive audio/open-url/nested-state-machine render
  stream exact before M4 nested and M6 audio work. Exact segments are now 133
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=133`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Ported live DashPath/Dash path-effect property reads so
  animated `DashPath.offset` and `Dash.length` come from cloned instance
  storage during draw, matching C++'s live effect objects. Widened
  `stacked_path_effects.riv` from sample `0` to samples `0` and `0.25`;
  exact segments are now 134 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=134`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `remove_from_list.riv` from sample `0` to samples
  `0` and `0.25`, keeping its passive text/list/scripted-drawable playback
  stream exact before M4/M5/M6 list, data-binding, and scripting work. Exact
  segments are now 135 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=135`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `stateful_list_props.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive view-model/list/state-machine
  playback stream exact before M4/M5/M6 list, data-binding, and text work.
  Exact segments are now 136 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=136`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joel_signed.riv` from sample `0` to samples `0`
  and `0.25`, keeping its heavy keyed-animation/skin/constraint/blend-state
  render stream exact before M3 constraints/input and later data-binding work.
  Exact segments are now 137 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=137`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Moved `RuntimeStateMachine` and the
  `build_state_machines` import builder out of `lib.rs` and into
  `state_machine.rs`, keeping the public crate-root re-export unchanged while
  shrinking the remaining state-machine surface in the monolith. Exact
  segments remain 137 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=137`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Moved `RuntimeTransitionCondition` and its
  component/view-model comparand helpers out of `lib.rs` and into
  `crates/rive-runtime/src/state_machine/transition_conditions.rs`, leaving
  shared schema property-by-key helpers in the crate root for animation and
  transition reuse. Exact segments remain 137 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=137`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved the `StateMachineBindable*Instance` structs and
  bindable value helpers out of `lib.rs` and into
  `crates/rive-runtime/src/state_machine/bindables.rs`, sharing the same
  bindable state between transition conditions and state-machine layer
  orchestration while leaving `StateMachineInstance` data-binding ownership in
  `lib.rs`. Exact segments remain 137 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=137`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved the state-machine bindable import builders and
  default view-model trigger builder out of `lib.rs` and into
  `crates/rive-runtime/src/state_machine/bindables.rs`, keeping the
  data-bind graph/converter helpers in `lib.rs` for the remaining
  `StateMachineInstance` data-context orchestration. Exact segments remain
  137 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=137`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved the `RuntimeBindable*` import model structs,
  default-source records, trigger source enum, view-model source enum, and
  default view-model trigger record out of `lib.rs` and into
  `crates/rive-runtime/src/state_machine/bindables.rs`, leaving the root
  data-bind graph to read the same crate-visible fields until
  `StateMachineInstance` data-context orchestration is split. Also aligned the
  checked-in port map and `/goal` command wording around `exact-segments` as
  the health metric. Exact segments remain 137 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=137`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `animation_reset_cases.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping its reset/blend-state
  playback stream exact after the state-machine modularization run. Exact
  segments are now 138 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=138`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `bindable_artboard_child.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping its passive
  bindable/view-model/state-machine playback stream exact. Exact segments are
  now 139 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=139`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `blend_test.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping its direct/1D blend-state playback
  stream exact. Exact segments are now 140 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=140`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `circle_clips.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping its animated clipping playback
  stream exact. Exact segments are now 141 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=141`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `clear_viewmodel_list.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping its passive
  view-model/list playback stream exact before later M4/M5 list and data-bind
  work. Exact segments are now 142 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=142`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `click_event.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping its default state-machine/event
  playback stream exact. Exact segments are now 143 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=143`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `clip_tests.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping its animated clipping/state-machine
  playback stream exact. Exact segments are now 144 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=144`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_based_conditions.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping component-comparator
  state-machine playback exact. Exact segments are now 145 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=145`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_list_2.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping component-list state
  playback exact. Exact segments are now 146 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=146`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_list_grouped.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping grouped component-list
  playback exact. Exact segments are now 147 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=147`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_list_hit_order.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping component-list hit
  ordering playback exact. Exact segments are now 148 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=148`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `cubic_value_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping CubicValue interpolator
  playback exact. Exact segments are now 149 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=149`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Ported the artboard-level default view-model bridge for
  animated custom-property target-to-source binds feeding Solo
  `activeComponentId` source-to-target binds, including recursive Solo collapse
  dirt for newly active descendants. Widened `data_bind_solo.riv` from samples
  `0` and `0.25` to samples `0`, `0.25`, and `0.5`; exact segments are now
  150 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=150`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Moved `StateMachineInstance` orchestration out of
  `lib.rs` and into `crates/rive-runtime/src/state_machine/instance.rs`,
  leaving the artboard root to construct/advance instances through
  crate-visible methods while the remaining data-bind graph stays in the root
  until a corpus diff or clear M2 coupling payoff justifies moving it. Exact
  segments remain 150 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=150`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `data_binding_test_2.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping its animated custom
  property/data-bind converter playback stream exact. Exact segments are now
  151 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=151`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
