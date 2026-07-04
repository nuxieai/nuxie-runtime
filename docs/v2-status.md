# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 339 across 70 exact files
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
   the authored-property source of truth. Component dirt/runtime transform
   state live in `crates/rive-runtime/src/components.rs`, the linear
   animation runtime model and import builder live in
   `crates/rive-runtime/src/animation.rs`, and state-machine import data,
   bindables, transition conditions, layer advancement, and
   `StateMachineInstance` orchestration live under
   `crates/rive-runtime/src/state_machine/`. Artboard data-bind propagation
   and list-binding queries, plus the adjacent binding structs/builders, live
   in `crates/rive-runtime/src/artboard_data_bind.rs`. Default and imported
   view-model source handle types and imported context storage/mutation methods
   live in `crates/rive-runtime/src/view_model.rs`. Data-bind graph state,
   context keys, default-binding records, source/target handles, source/target
   nodes, converter/value types, apply phases, stateful-advance records, and
   formula random-source state live in
   `crates/rive-runtime/src/data_bind_graph.rs`; the
   `RuntimeDataBindGraphValue` owned/imported view-model resolution impl lives
   there too. Data-bind flag helpers and the target mutator bridge also live
   there, along with the converter-state bridge, graph execution impls, and
   source-node execution impl. Continue with the converter
   state/formula/interpolator helper types and converter
   conversion/construction helpers, preserving the current golden set.
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

- 2026-07-03: [M2] Widened `component_list_grouped.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping grouped component-list/view-model-list playback exact across
  the fifth sample while leaving active list/layout mutation in later
  milestones. Exact segments are now 280 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=280`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `component_list_hit_order.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive component-list hit-order/listener playback exact
  across the fifth sample while leaving scripted input in M3 scope. Exact
  segments are now 281 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=281`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `cubic_value_test.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping CubicValue/CubicEase keyed double animation playback exact across
  the fifth sample. Exact segments are now 282 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=282`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `data_bind_solo.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive data-bind/Solo/view-model playback exact across the fifth
  sample while leaving external mutation and active text behavior in later
  milestones. Exact segments are now 283 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=283`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `data_binding_test_2.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive data-bind converter and state-machine playback exact
  across the fifth sample while leaving external view-model mutation in M5
  scope. Exact segments are now 284 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=284`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `dependency_test.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping the foundational vector dependency fixture exact across the fifth
  sample. Exact segments are now 285 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=285`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `draw_rule_cycle.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping animated draw-rule cycle playback exact across the fifth
  sample. Exact segments are now 286 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=286`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `event_on_listener.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive listener event/open-url state-machine playback exact
  across the fifth sample while leaving scripted pointer/event dispatch in M3
  scope. Exact segments are now 287 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=287`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `event_trigger_event.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive trigger/fire-event and view-model condition playback
  exact across the fifth sample while leaving scripted pointer/event dispatch
  in M3 scope. Exact segments are now 288 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=288`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `events_on_states.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive listener events-on-states playback exact across the
  fifth sample while leaving scripted pointer/event dispatch in M3 scope.
  Exact segments are now 289 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=289`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `fill_trim_path.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping animated multi-shape TrimPath fill playback exact across the fifth
  sample. Exact segments are now 290 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=290`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `fix_rectangle.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping animated rectangle/path geometry playback exact across the fifth
  sample. Exact segments are now 291 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=291`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `hit_test_solos.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive hit-test Solo/bool state-machine playback exact across the
  fifth sample while leaving scripted pointer dispatch in M3 scope. Exact
  segments are now 292 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=292`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joel_signed.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping the large signed-Joel skin, constraint, direct-blend animation, and
  passive listener/data-bind fixture exact across the fifth sample. Exact
  segments are now 293 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=293`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joystick_flag_test.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive joystick flag animation playback exact across the
  fifth sample before opening scripted pointer input in M3. Exact segments
  are now 294 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=294`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joystick_nested_remap.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive joystick nested-remap animation playback exact
  across the fifth sample without opening M4 nested-artboard advancement.
  Exact segments are now 295 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=295`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `juice.riv` from samples `0`, `0.25`, `0.5`,
  and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`, keeping the
  animated gradient/vertex path fixture exact across the fifth sample. Exact
  segments are now 296 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=296`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `keyboard_event_to_script.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive script-asset/focus-data playback exact before active
  keyboard/script input opens in M3/M6. Exact segments are now 297 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=297`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Widened `library_data_enum_test.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive custom-enum/view-model state-machine playback exact
  across the fifth sample. Exact segments are now 298 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=298`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M2] Widened `light_switch.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive listener/bool transition playback exact across the fifth
  sample. Exact segments are now 299 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=299`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `list_to_path.riv`, `lock_icon_demo.riv`,
  `long_name.riv`, `looping_timeline_events.riv`, and
  `multiple_state_machines.riv` from samples `0`, `0.25`, `0.5`, and
  `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`, keeping list
  path, skinned lock icon, long-name static animation, looping timeline
  events, and passive multi-state-machine playback exact across the fifth
  sample. Exact segments are now 304 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=304`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `nested_solo.riv`, `off_road_car.riv`,
  `oneshotblend.riv`, `opaque_hit_test.riv`, and `quantize_test.riv` from
  samples `0`, `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`,
  `0.75`, and `1.0`, keeping Solo collapse, the large off-road car skin/
  draw-rule fixture, one-shot blend, opaque hit-test, and quantized keyed
  animation playback exact across the fifth sample. Exact segments are now
  309 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=309`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Widened `rapid_pointer_events.riv`,
  `remove_from_list.riv`, `rocket.riv`, `script_paths_opacity_test.riv`, and
  `script_paths_test.riv` from samples `0`, `0.25`, `0.5`, and `0.75` to
  samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`, keeping passive pointer
  listener/data-bind playback, list-removal metadata, the rocket draw-rule
  fixture, and passive script-path animation exact across the fifth sample.
  Exact segments are now 314 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=314`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `scripted_boolean.riv`,
  `scripted_enum.riv`, `scripted_graph.riv`, `scripted_string.riv`, and
  `settler.riv` from samples `0`, `0.25`, `0.5`, and `0.75` to samples `0`,
  `0.25`, `0.5`, `0.75`, and `1.0`, keeping passive scripted view-model
  playback and CubicEase keyed double animation exact across the fifth
  sample. Exact segments are now 319 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=319`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `solo_test.riv`,
  `solos_collapse_tests.riv`, `solos_with_nested_artboards.riv`,
  `sound.riv`, and `sound2.riv` from samples `0`, `0.25`, `0.5`, and `0.75`
  to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`, keeping Solo active
  child/collapse playback, passive nested-artboard metadata, and audio/open-url
  event metadata exact across the fifth sample. Exact segments are now 324
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=324`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Widened `stacked_path_effects.riv`,
  `state_machine_transition.riv`, and `state_machine_triggers.riv` from
  samples `0`, `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`,
  `0.75`, and `1.0`, keeping stacked trim/dash path effects and passive
  trigger/bool state-machine transition playback exact across the fifth
  sample. Exact segments are now 327 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=327`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `stateful_list_props.riv`,
  `stroke_name_test.riv`, `test_elastic.riv`, `text_input_event.riv`, and
  `timeline_event_test.riv` from samples `0`, `0.25`, `0.5`, and `0.75` to
  samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`, keeping passive
  stateful-list/view-model playback, stroke/fill naming, ElasticInterpolator,
  text-input listener metadata, and timeline callback events exact across the
  fifth sample. Exact segments are now 332 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=332`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Widened `trim_path.riv`, `trim_path_linear.riv`,
  `two_artboards.riv`, and `viewmodel_runtime_file.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping TrimPath, linear TrimPath, selected-artboard animation, and
  passive view-model metadata playback exact across the fifth sample. Exact
  segments are now 336 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=336`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Widened `pointer_events.riv` from samples `0`, `0.1`,
  and `0.25` to samples `0`, `0.1`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive listener/bool pointer-event playback exact across the
  standard M2 sample set while leaving scripted pointer dispatch in M3 scope.
  Exact segments are now 339 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Extracted ArtboardInstance artboard data-bind propagation
  and list-binding query methods from `crates/rive-runtime/src/lib.rs` to
  `crates/rive-runtime/src/artboard_data_bind.rs`, reducing root runtime
  coupling while preserving the generated `InstanceObjectStorage` mutation
  path. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the artboard data-bind binding structs and import
  builders from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/artboard_data_bind.rs`, keeping root runtime state
  construction thin while preserving the generated `InstanceObjectStorage`
  authored-property path. Exact segments remain 339 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=339`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M2] Moved the default view-model source handle types from
  `crates/rive-runtime/src/lib.rs` into `crates/rive-runtime/src/view_model.rs`
  and re-exported them from the crate root, starting the data-bind
  graph/default-view-model bridge extraction without changing graph execution.
  Exact segments remain 339 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the imported view-model source handle types from
  `crates/rive-runtime/src/lib.rs` into `crates/rive-runtime/src/view_model.rs`
  and re-exported them from the crate root, leaving imported context mutation
  behavior in place while shrinking the root data-bind bridge. Exact segments
  remain 339 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=339`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Moved `RuntimeImportedViewModelInstanceContext` storage and
  public mutation methods from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/view_model.rs`, re-exporting the context from the
  crate root while keeping the data-bind graph bridge in place for the next
  extraction slice. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Added `crates/rive-runtime/src/data_bind_graph.rs` for
  data-bind graph state, imported-context keys, override keys, default-binding
  records, source/target handles, and formula random-source state while leaving
  behavior-heavy graph impls in `crates/rive-runtime/src/lib.rs` for the next
  extraction slice. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the data-bind graph source/target node,
  converter/value, apply-phase, and stateful-advance type definitions from
  `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, leaving graph value resolution,
  graph behavior, and target mutator bridge impls in `lib.rs` for the next
  extraction slices. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the `RuntimeDataBindGraphValue` owned/imported
  view-model resolution impl from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, keeping resolver methods
  crate-visible while the remaining graph execution and target mutator bridge
  are extracted. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved data-bind direction flag helpers and
  `RuntimeDataBindGraphTargetsMut` target application from
  `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, leaving the remaining graph
  execution/converter-state bridge as the next extraction slice. Exact
  segments remain 339 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the `RuntimeDataBindGraphConverterState` bridge impl
  from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, keeping the conversion
  engine helpers in `lib.rs` for the next extraction slice while shrinking the
  root graph bridge. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved the `RuntimeDataBindGraph` and
  `RuntimeDataBindGraphSourceNode` execution impls from
  `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, keeping converter
  state/formula/interpolator helper types and converter construction helpers
  in `lib.rs` for the next extraction slice. Exact segments remain 339 across
  70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=339`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
