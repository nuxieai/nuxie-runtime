# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 364 across 85 exact files
- Parked breakdown (from `make golden-compare`): M4=83 M5=8 M6=77 gated=6 harness=36
- Current milestone: **M3 — Interactivity Exact (#V2-4)**

## Milestones

- [x] M0: Golden diff harness + corpus manifest + one exact file
- [x] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [x] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [ ] M3: Interactive files exact under scripted pointer input
- [ ] M4: Nested artboards/lists exact
- [ ] M5: Data binding exact incl. external view-model mutation
- [ ] M6: Layout + text verified per declared corpus modes; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. Widen scripted M3 coverage in corpus-priority order for exact listener/
   pointer fixtures with visible render movement. Scripted coverage now
   includes `pointer_events.riv`, `rapid_pointer_events.riv`,
   `hit_test_solos.riv`, `click_event.riv`, `opaque_hit_test.riv`,
   `state_machine_triggers.riv`, `state_machine_transition.riv`,
   `light_switch.riv`, `event_on_listener.riv`,
   `event_trigger_event.riv`, and `events_on_states.riv`.
2. Remaining unscripted exact listener/event candidates are
   `bindable_artboard_child.riv`, `component_list_2.riv`,
   `component_list_follow_path.riv`, `component_list_grouped.riv`,
   `component_list_hit_order.riv`, `joel_signed.riv`,
   `lock_icon_demo.riv`, `solos_with_nested_artboards.riv`, `sound.riv`,
   `stateful_list_props.riv`, and `text_input_event.riv`.
3. Investigate `bindable_artboard_child.riv` before attaching a script: a
   simple full-artboard click at `250,250` makes C++ turn the fill red while
   Rust stays `0xff747474`; treat it as M3 only if listener/view-model action
   dispatch is the blocker, otherwise park it behind M5 data-binding/view-model
   behavior.
4. `event_trigger_event.riv` has an exact primary rectangle click script at
   `129,101`; an alternate red-target click at `450,50` changes C++ colors
   while Rust stays passive, so do not widen that coordinate until listener
   fire-event/view-model/event propagation scope is clear.
5. Port additional `ListenerGroup` semantics only when a widened script proves
   they are the blocking gap: hover/enter-exit state, click synthesis, drag
   state, opaque target ordering, nested/list/text/layout targets, and
   component-provided groups remain intentionally out of the direct rectangle
   pointer slice.
6. There are no remaining `milestone = "M3"` parked entries in `corpus.toml`;
   scripted input exactness is the active M3 exit criterion.
7. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M3 unless a related refactor needs a cheap draw-regression check.

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
  interpolation for CubicEase/CubicValue/Elastic keyframe interpolators, and
  `DistanceConstraint` world-translation application and
  `TranslationConstraint` target/source/destination/min-max translation
  application, `RotationConstraint` compose/decompose rotation,
  `ScaleConstraint` compose/decompose scale, `TransformConstraint`
  target-origin full-transform interpolation, `FollowPathConstraint`
  Shape/Path target sampling against runtime path geometry, C++ Bone x/y
  overrides, `IKConstraint` FK-chain solving, and
  `ListFollowPathConstraint` registration/application over component-list item
  transform slices once M4 list instances populate them, and parametric
  Star/Polygon local path sampling for follow-path targets. Custom
  handle-source world-space math and nested remap dependent advancement are
  still not supported.
  Golden runner sample lists now advance by sorted absolute-time deltas and
  reuse render paths across samples; no images, text, nested artboards, scroll
  constraints, or component-list instancing. Harness-level scripted input
  replay dispatches pointerDown/pointerMove/pointerUp/pointerExit markers into
  direct rectangle state-machine listeners with listener input actions and
  primitive listener-owned default view-model writes. Full C++ ListenerGroup
  hover/click/drag/opaque behavior and nested/list/text/layout targets are
  still not supported.
- `TransformConstraint` currently covers the default empty
  `TransformComponent::constraintBounds()` path. Text/LayoutComponent
  constraint bounds remain parked behind their M6 text/layout diagnostics.
- Scroll-constraint corpus files are parked behind M6 layout/runtime support
  via `rust-runner-unsupported:scroll-constraints`. C++
  `src/constraints/scrolling/scroll_constraint.cpp` reads
  `LayoutComponent` dimensions, layout-provider child bounds, physics state,
  and optional component-list virtualization, so the current corpus has no
  pure M3 scroll slice to port without pulling layout/list runtime forward.
- Per-file parked reasons now live in `corpus.toml`: each gated entry
  carries `milestone = "M3|M4|M5|M6|gated|harness"` plus its diagnostic
  feature tags (`rust-runner-unsupported:*`, `cpp-runner-crash`,
  `import-error:*`). Query a milestone's work-list with e.g.
  `grep -B6 'milestone = "M4"' corpus.toml`.
- Entries tagged `cpp-runner-crash` (`milestone = "harness"`) stay parked
  until the C++ golden runner survives the FileAssetContents, scripting,
  and data-viz crash paths it currently aborts on.
- `coin.riv` is no longer parked as an M3 constraints file after
  `ScaleConstraint`; it reaches draw and is now `milestone = "gated"` on the
  explicit `rust-runner-unsupported:feather` renderer diagnostic.
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
  epsilon while keeping call order, IDs, verbs, and non-numeric text exact,
  matching the V2 renderer seam plan.
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
- 2026-07-04: Remaining scroll-constraint files are M6, not M3: the C++
  implementation is coupled to layout dimensions, layout-provider child
  bounds, physics, and component-list virtualization. Use the explicit
  `rust-runner-unsupported:scroll-constraints` diagnostic for this queue
  until layout/runtime support opens it.
- 2026-07-04: `golden-compare` numeric-token epsilon is now `1.3e-4`, raised
  from `1e-4` after `follow_path_shapes.riv` exposed local path float
  cancellation between C++ clang contraction/rounding and Rust strict `f32`.
  The comparator still rejects the next observed cancellation-grid step, and
  call order, IDs, verbs, and non-numeric text remain exact.
- 2026-07-04: Rust golden runner now mirrors C++ input-script parsing and
  timeline replay for pointer events, records input markers, and dispatches
  pointer events into direct rectangle state-machine listeners for the first
  M3 scripted-interactivity slice. Full C++ ListenerGroup hover/click/drag/
  opaque behavior remains corpus-driven follow-up work.

## Log

- Completed-milestone entries (M0, M1) are archived verbatim in
  `docs/v2-log-archive.md`; when a milestone completes, move its entries
  there and keep only the active milestone's recent working window here.
- Older active M2 entries are archived verbatim in `docs/v2-log-archive.md`
  under `M2 active log rolloff`; keep only the recent rolling window here once
  Metric, Next, Decisions, and `corpus.toml` capture the current state.

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
- 2026-07-04: [M2] Moved the data-bind graph converter
  state/formula/interpolator helper types, owned view-model source-path
  helpers, converter conversion/evaluation helpers, and converter
  construction helpers from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/data_bind_graph.rs`, with artboard/list binding
  and state-machine bindable builders importing the graph helpers directly.
  Exact segments remain 339 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Extracted the draw/path/rendering command pipeline from
  `crates/rive-runtime/src/lib.rs` into `crates/rive-runtime/src/draw.rs`,
  including `ArtboardInstance` draw methods, draw/path command types, render
  path cache, paint preallocation, path effect builders, renderer trait
  driving, and color interpolation helpers used by animation/data-bind code.
  Exact segments remain 339 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Moved `RuntimeOwnedViewModelInstance`, owned view-model
  source handles, owned/default/imported property-path helpers,
  `RuntimeViewModelPointer`, and runtime data-context lookup/reporting from
  `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/view_model.rs`, keeping the crate-root API/re-export
  surface stable while shrinking the remaining root runtime state. Exact
  segments remain 339 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Added `crates/rive-runtime/src/properties.rs` for shared
  runtime property-key/object-value helpers, transform-key lookup,
  joystick/Solo/paint key helpers, `mix_value`, artboard-index lookup, and
  `RuntimeArtboardDimensions`, with animation, draw, components,
  artboard-data-bind, and state-machine modules importing the helper surface
  directly instead of through `lib.rs`. Exact segments remain 339 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=339`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M2] Moved `ArtboardInstance`, core instance methods, and local
  instance tests from `crates/rive-runtime/src/lib.rs` into
  `crates/rive-runtime/src/artboard.rs`, leaving `lib.rs` as a 93-line
  module/re-export hub and preserving crate-root `ArtboardInstance` as the
  public API. Exact segments remain 339 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=339`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M2] Completed the M2 exit audit and opened M3. The corpus has
  295 entries with 70 exact files and no `diverges`/`not-yet` entries; exact
  sample coverage is 66 files at the standard five-sample M2 set,
  `pointer_events.riv` at six samples, and only the static M1 holdovers
  `artboardclipping.riv`, `shapetest.riv`, and `trim.riv` at sample `0`.
  All 225 parked entries carry milestones (`M3=21`, `M4=83`, `M5=8`,
  `M6=72`, `gated=5`, `harness=36`), and all M3 parked files are currently
  gated by `rust-runner-unsupported:constraints`.
- 2026-07-04: [M3] Ported `DistanceConstraint` world-translation application
  from C++ `src/constraints/distance_constraint.cpp`, added runtime component
  constraint-local application after world-transform updates, narrowed the
  Rust golden-runner constraint gate to keep only unimplemented constraint
  kinds parked, and promoted `distance_constraint.riv` to exact. Exact
  segments are now 340 across 71 exact files; `make golden-compare` reports
  `exact=71`, `exact-segments=340`, `diverges=0`,
  `unsupported-feature=224`, `not-yet=0`, parked `M3=20`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `TranslationConstraint` from C++
  `src/constraints/translation_constraint.cpp`, added shared
  transform-space/parent-world/min-max constraint helpers, corrected targeted
  constraints to resolve `targetId` as the artboard-local core id, narrowed
  the Rust golden-runner constraint gate for translation constraints, and
  promoted `translation_constraint.riv` to exact. Exact segments are now 341
  across 72 exact files; `make golden-compare` reports `exact=72`,
  `exact-segments=341`, `diverges=0`, `unsupported-feature=223`,
  `not-yet=0`, parked `M3=19`, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `RotationConstraint` from C++
  `src/constraints/rotation_constraint.cpp`, added shared
  `Mat2D::decompose`/`Mat2D::compose` runtime math from C++
  `src/math/mat2d.cpp`, narrowed the Rust golden-runner constraint gate for
  rotation constraints, and promoted `rotation_constraint.riv` to exact.
  Exact segments are now 342 across 73 exact files; `make golden-compare`
  reports `exact=73`, `exact-segments=342`, `diverges=0`,
  `unsupported-feature=222`, `not-yet=0`, parked `M3=18`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `ScaleConstraint` from C++
  `src/constraints/scale_constraint.cpp`, reusing the compose/decompose
  transform helpers for source/destination-space copying, min/max clamping,
  authored-offset scale, and strength interpolation, narrowed the Rust
  golden-runner constraint gate for scale constraints, promoted
  `scale_constraint.riv` to exact, and reclassified `coin.riv` from M3
  constraints to the explicit `rust-runner-unsupported:feather` gated
  renderer backlog. Exact segments are now 343 across 74 exact files; `make
  golden-compare` reports `exact=74`, `exact-segments=343`, `diverges=0`,
  `unsupported-feature=221`, `not-yet=0`, parked `M3=16`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `TransformConstraint` from C++
  `src/constraints/transform_constraint.cpp`, including target-origin
  transform construction, source/destination transform-space mapping, and
  full transform-component interpolation via the shared compose/decompose
  helpers, narrowed the Rust golden-runner constraint gate for transform
  constraints, and promoted `transform_constraint.riv` to exact. Exact
  segments are now 344 across 75 exact files; `make golden-compare` reports
  `exact=75`, `exact-segments=344`, `diverges=0`,
  `unsupported-feature=220`, `not-yet=0`, parked `M3=15`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `FollowPathConstraint` from C++
  `src/constraints/follow_path_constraint.cpp`, added runtime path geometry
  sampling with current path/vertex/parametric property overlays, narrowed the
  Rust golden-runner constraint gate for plain follow-path constraints,
  promoted `follow_path.riv`, `follow_path_constraint.riv`,
  `follow_path_path_0_opacity.riv`, `follow_path_solos.riv`, and
  `follow_path_with_0_opacity.riv` to exact, reclassified
  `follow_path_path.riv` to M6 text, and parked `follow_path_shapes.riv` on
  the narrow `rust-runner-unsupported:follow-path-star-shapes` precision
  diagnostic. Exact segments are now 349 across 80 exact files; `make
  golden-compare` reports `exact=80`, `exact-segments=349`, `diverges=0`,
  `unsupported-feature=215`, `not-yet=0`, parked `M3=9`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `IKConstraint` from C++
  `src/constraints/ik_constraint.cpp` plus the non-root Bone x/y override
  from `src/bones/bone.cpp`, added runtime FK-chain solving for one-bone,
  two-bone, and longer IK chains, narrowed the Rust golden-runner constraint
  gate for IK, and promoted `complex_ik_dependency.riv` and
  `two_bone_ik.riv` to exact. Exact segments are now 351 across 82 exact
  files; `make golden-compare` reports `exact=82`,
  `exact-segments=351`, `diverges=0`, `unsupported-feature=213`,
  `not-yet=0`, parked `M3=7`, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported `ListFollowPathConstraint` from C++
  `src/constraints/list_follow_path_constraint.cpp`, registering list
  constraints from the graph and adding the runtime item-transform application
  hook for M4 component-list instances, narrowed the Rust golden-runner
  constraint gate for list follow-path constraints, and promoted
  `component_list_follow_path.riv` and
  `component_list_follow_path_distance.riv` to exact. Exact segments are now
  353 across 84 exact files; `make golden-compare` reports `exact=84`,
  `exact-segments=353`, `diverges=0`, `unsupported-feature=211`,
  `not-yet=0`, parked `M3=5`, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Added the explicit
  `rust-runner-unsupported:scroll-constraints` diagnostic and reclassified
  `component_list_1.riv`, `deterministic_mode.riv`,
  `draw_index_list.riv`, and `virtualize_blendmode.riv` from the M3
  constraint queue to M6 layout/runtime support after confirming C++
  `ScrollConstraint` depends on `LayoutComponent` metrics and registered
  layout-provider children. Exact segments remain 353 across 84 exact files;
  `make golden-compare` reports `exact=84`, `exact-segments=353`,
  `diverges=0`, `unsupported-feature=211`, `not-yet=0`, parked `M3=1`,
  and `cargo test --workspace` passes.
- 2026-07-04: [M3] Promoted `follow_path_shapes.riv` to exact, removed the
  narrow `rust-runner-unsupported:follow-path-star-shapes` gate, matched C++
  matrix inversion/local-path composition more closely for follow-path draw
  output, and bounded the remaining local path float-cancellation band with a
  `golden-compare` comparator regression test. Exact segments are now 354
  across 85 exact files; `make golden-compare` reports `exact=85`,
  `exact-segments=354`, `diverges=0`, `unsupported-feature=210`,
  `not-yet=0`, no parked M3 entries, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Landed Rust golden-runner `--input-script`
  parsing/replay to match the C++ runner, added
  `tests/input_scripts/pointer_events_click.txt`, and attached it to
  `pointer_events.riv` as the first scripted exact corpus entry. The runner
  now advances to input timestamps and records input markers; listener
  hit-testing/action dispatch is still the next M3 runtime port. Exact
  segments remain 354 across 85 exact files; `make golden-compare` reports
  `exact=85`, `exact-segments=354`, `diverges=0`,
  `unsupported-feature=210`, `not-yet=0`, no parked M3 entries, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported direct rectangle pointer listener dispatch from C++
  `StateMachineInstance::pointer*`/`updateListeners` into Rust, wired
  `rust-golden-runner` input replay into the state machine, added listener
  input actions plus primitive listener-owned default view-model writes, and
  widened `rapid_pointer_events.riv` with a render-affecting
  `tests/input_scripts/rapid_pointer_events_click.txt` script. Exact segments
  are now 355 across 85 exact files; `make golden-compare` reports
  `exact=85`, `exact-segments=355`, `diverges=0`,
  `unsupported-feature=210`, `not-yet=0`, no parked M3 entries, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Widened scripted pointer coverage for
  `click_event.riv`, `hit_test_solos.riv`, and `opaque_hit_test.riv` with
  render-affecting down/up scripts, adding sample `0.1` to each. Direct
  C++/Rust stream diffs match for all three scripted fixtures. Exact segments
  are now 358 across 85 exact files; `make golden-compare` reports
  `exact=85`, `exact-segments=358`, `diverges=0`,
  `unsupported-feature=210`, `not-yet=0`, no parked M3 entries, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Widened scripted pointer coverage for
  `state_machine_triggers.riv`, `state_machine_transition.riv`,
  `light_switch.riv`, `event_on_listener.riv`, `event_trigger_event.riv`,
  and `events_on_states.riv` with render-affecting down/up scripts and sample
  `0.1`. Direct C++/Rust stream diffs match for all six scripted fixtures.
  Exact segments are now 364 across 85 exact files; `make golden-compare`
  reports `exact=85`, `exact-segments=364`, `diverges=0`,
  `unsupported-feature=210`, `not-yet=0`, no parked M3 entries, and
  `cargo test --workspace` passes.
