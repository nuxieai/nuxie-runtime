# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 435 across 114 exact files
- Parked breakdown (from `make golden-compare`): M4=50 M5=8 M6=81 gated=6 harness=36
- Current milestone: **M4 — Nested Artboards And Lists Exact (#V2-5)**

## Milestones

- [x] M0: Golden diff harness + corpus manifest + one exact file
- [x] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [x] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [x] M3: Interactive files exact under scripted pointer input
- [ ] M4: Nested artboards/lists exact
- [ ] M5: Data binding exact incl. external view-model mutation
- [ ] M6: Layout + text verified per declared corpus modes; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. Continue M4 with the smallest remaining pure nested-artboard runtime slice.
   Query the queue with `grep -B6 'milestone = "M4"' corpus.toml`; the raw
   queue now starts with `ai_assitant.riv`, but that file carries skin/mesh/
   feather-ish complexity and should wait for a narrower entry point. Nested
   reported-event bubbling is closed for `nested_event_test.riv`; prefer the
   next file whose first failing diagnostic names a single nested host/list
   behavior instead of pulling M5 data binding or M6 text/layout forward.
   Static sample-0 files with recursive nested `ListenerAlignTarget` actions
   are no longer parked when the corpus entry has no input script, because
   those actions are unexercised during static draw; input-driven align-target
   behavior remains out of scope for this slice. `align_target.riv` moved to
   M6 because its first remaining Rust diagnostic is text.
2. Nested remap with runtime `DrawTarget` placement is closed for
   `death_knight.riv` at sample `0.0`: draw order now reads active
   `DrawRules.drawTargetId` / `DrawTarget.placementValue` during draw,
   replays clipping proxy insertion, and mirrors C++ child-before-parent
   render-paint preallocation for the nested tree. The next useful M4 slice is
   another small nested/list file from the M4 queue; avoid pulling M5 data
   binding or M6 layout/text/image support forward.
3. Nested host serialized `speed`/`quantize` local elapsed and generated
   source-to-target host-control defaults are closed for
   `nested_artboard_quantize_and_speed.riv` through samples `0.0, 0.25, 0.5,
   0.75, 1.0`; external/live host-control mutation belongs to M5.
4. Solo-owned repeated nested child paint allocation is closed for
   `pointer_events_nested_artboards_in_solos.riv` through samples `0.0, 0.1,
   0.25, 0.5, 0.75, 1.0, 1.25, 1.5`: Rust now keeps per-host nested paint
   caches, so repeated child instances do not share mutable paint/shader state.
5. Plain static `NestedArtboard` draw, default nested
   simple-animation/state-machine host advancement are closed for the current
   sample-0 corpus slice, and nested child unbound SolidColor data-bind
   defaults are closed for `library_vmtest_1_host.riv` and
   `unbound_stateful_component.riv`. Exact promoted files now include the
   earlier static set plus `library_export_animation_test.riv`,
   `library_export_state_machine_test.riv`, `ball_test.riv`,
   `data_binding_test_3.riv`, `data_binding_test_triggers.riv`,
   `databind_external_artboard_main.riv`, `drag_event.riv`, `multitouch.riv`,
   `nested_needs_advance.riv`, `scripted_listener_context.riv`,
   `pointer_events_nested_artboards_in_solos.riv`,
   `nested_artboard_quantize_and_speed.riv`,
   `nested_event_test.riv`,
   `death_knight.riv`,
   `pointer_exit.riv`,
   `scripting_root_viewmodel.riv`, `solid_affects_has_changed.riv`,
   `target_event.riv`, and `transition_self_comparator_test.riv`.
6. M3 is closed: all `milestone = "M3"` parked entries are gone, all scripted
   direct-pointer corpus entries are exact through sample `1.5`, and the
   remaining unscripted exact listener files showed no C++ render delta on a
   bounded coarse click/hover probe or require M4/M5/M6 domains.
7. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M4 unless a related refactor needs a cheap draw-regression check.

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
  Star/Polygon local path sampling for follow-path targets, plus static plain
  `NestedArtboard` host draw with child root opacity inheritance, default
  nested simple-animation/state-machine hosts backed by persistent child
  artboard instances, stateful child `ViewModelInstance` subtree admission
  under plain nested hosts, nested child unbound SolidColor data-bind defaults,
  nested bool/number/trigger input proxying, and basic nested remap-time host
  plumbing, runtime `DrawTarget` placement sorting from active `DrawRules`,
  serialized nested host speed/quantize local elapsed, generated
  source-to-target nested host `isPaused`/`speed`/`quantize` default binding,
  per-host nested paint caches for repeated child instances under Solo-owned
  hosts, and nested state-machine reported-event bubbling into parent event
  listeners, plus no-input recursive nested `ListenerAlignTarget` fixtures
  where the action is unexercised.
  Custom handle-source world-space math, data-bound nested host controls beyond
  generated defaults
  (`artboardId` runtime swaps and external/live pause/speed/quantize
  mutation), nested child non-color data-bind targets, focus data, bound
  stateful child view-model propagation, input-driven recursive
  `ListenerAlignTarget` and nested pointer/listener hit propagation beyond
  reported `Event` listeners, `NestedArtboardLayout` / `NestedArtboardLeaf`,
  and
  layout-backed or virtualized component-list instancing are still not
  supported.
  Golden runner sample lists now advance by sorted absolute-time deltas and
  reuse render paths across samples; no images, text, live data-bound nested
  host controls/artboard swaps, nested layout/leaf, scroll constraints, or
  layout-backed/virtualized component-list instancing.
  Harness-level scripted input replay dispatches
  pointerDown/pointerMove/pointerUp/pointerExit markers into direct rectangle
  state-machine listeners with listener input actions, direct rectangle
  enter/exit hover state, direct rectangle click synthesis, and listener-owned
  default view-model trigger target-to-source writes. Full C++ ListenerGroup
  drag/opaque behavior and input-driven nested align-target/list/text/layout
  targets are still not supported.
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
- 2026-07-04: Runtime draw order is dynamic once `DrawTarget` rules can be
  driven by animations or nested remap time: Rust derives sorted drawables
  from active `DrawRules.drawTargetId` and `DrawTarget.placementValue` during
  draw, then recomputes clipping proxies and save-operation elision.

## Log

- Completed-milestone entries (M0 through M3) are archived verbatim in
  `docs/v2-log-archive.md`; when a milestone completes, move its entries
  there and keep only the active milestone's recent working window here.

- 2026-07-04: [M4] Ported the first static plain `NestedArtboard` draw slice
  from the C++ `ArtboardHost`/`NestedArtboard::draw` shape: Rust now resolves
  referenced child artboards during draw, applies the host world transform,
  draws children without the top-level artboard-origin transform, inherits host
  render opacity into the child root, and preallocates child instance paints in
  host object order. Promoted `entry.riv`, `library_export_test.riv`,
  `magic_alley_db_reduced_export.riv`, `nested_artboard_opacity.riv`, and
  `stateful_artboard_swap.riv` to exact; moved the three now image-blocked
  library fixtures to M6 `rust-runner-unsupported:images`. `make
  golden-compare` reports `exact=90`, `exact-segments=398`, `diverges=0`,
  `unsupported-feature=205`, `not-yet=0`, and parked
  `M4=75 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Ported default nested animation/state-machine host
  instances from the C++ `NestedAnimation`, `NestedSimpleAnimation`, and
  `NestedStateMachine` shape: selected artboards now build persistent nested
  child instances, advance nested simple animations/state machines before
  drawing, sync host render opacity into child roots, and call child
  `drawInternal` without an unconditional wrapper save. Promoted
  `library_export_animation_test.riv`, `library_export_state_machine_test.riv`,
  and 12 newly unblocked nested-host corpus files to exact. Added runner
  diagnostics for still-parked nested host controls: remap/input hosts,
  data-bound host controls, stateful child view-model binding, nested
  listener/event propagation, nested layout/leaf, and component-list paths.
  `make golden-compare` reports `exact=104`, `exact-segments=412`,
  `diverges=0`, `unsupported-feature=191`, `not-yet=0`, and parked
  `M4=61 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Narrowed the nested stateful view-model guard to allow
  authored child `ViewModelInstance` subtrees under plain `NestedArtboard`
  hosts, and mirrored C++ unbound artboard-owned SolidColor
  `DataBindContext` import defaults to opaque black for child artboard
  instances. Promoted `library_vmtest_1_host.riv` and
  `unbound_stateful_component.riv` to exact; kept nested child non-color
  data-bind targets and focus data behind nested-artboards diagnostics. `make
  golden-compare` reports `exact=106`, `exact-segments=414`, `diverges=0`,
  `unsupported-feature=189`, `not-yet=0`, and parked
  `M4=59 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Ported nested input proxying from the C++ `NestedInput`,
  `NestedBool`, `NestedNumber`, and `NestedTrigger` shape plus
  `NestedRemapAnimation` time/apply plumbing: hosted child state machines now
  receive authored/keyed nested bool/number/trigger values, remap hosts use
  global-to-local animation time, and the runner has narrower diagnostics for
  DrawTarget-heavy remap and Solo-owned nested listener children. Promoted
  `advance_blend_mode.riv`, `runtime_nested_inputs.riv`, and `smi_test.riv`
  to exact. `make golden-compare` reports `exact=109`, `exact-segments=419`,
  `diverges=0`, `unsupported-feature=186`, `not-yet=0`, and parked
  `M4=56 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Ported sample-0 nested child paint allocation for
  repeated nested artboard instances under Solo hosts: tree preallocation now
  consumes `RenderPaint` allocation per child artboard instance while
  preserving the first source-global paint mapping used by current draw
  lookup. Promoted `pointer_events_nested_artboards_in_solos.riv` to exact.
  `make golden-compare` reports `exact=110`, `exact-segments=420`,
  `diverges=0`, `unsupported-feature=185`, `not-yet=0`, and parked
  `M4=55 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Closed per-host nested paint caches for repeated
  Solo-owned nested artboard instances: Rust render paint state now lives in a
  recursive `RuntimeRenderPaintCache`, and the golden runner prepares/draws
  nested children through matching per-host paint caches instead of reusing a
  child artboard's global paint map. Widened
  `pointer_events_nested_artboards_in_solos.riv` from sample `0.0` to samples
  `0.0, 0.1, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5`, raising `exact-segments` to
  427 while `exact` remains 110. At that point, `death_knight.riv` was still
  gated on nested remap `DrawTarget` rules: the C++ runner creates
  transparent child shaders for Death Up but never draws that child, while
  Rust must not bypass the existing diagnostic until DrawTarget rules are
  ported. `make
  golden-compare` reports `exact=110`, `exact-segments=427`, `diverges=0`,
  `unsupported-feature=185`, `not-yet=0`, and parked
  `M4=55 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Mirrored C++ nested host local elapsed for serialized
  `NestedArtboard.speed` and `NestedArtboard.quantize`: nested child
  animations and child artboard advancement now run through
  `NestedArtboard::calculateLocalElapsedSeconds` semantics, including paused
  hosts and quantized accumulated time. Narrowed the golden-runner host-control
  guard so generated speed/quantize properties no longer park otherwise exact
  files, while live pause/data-bound host mutation stays gated. Promoted
  `nested_artboard_quantize_and_speed.riv` to exact. `make golden-compare`
  reports `exact=111`, `exact-segments=428`, `diverges=0`,
  `unsupported-feature=184`, `not-yet=0`, and parked
  `M4=54 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Closed generated source-to-target nested host
  `isPaused`/`speed`/`quantize` defaults for the artboard-owned
  `File::createViewModelInstance()` path while preserving serialized default
  handling for component-list bindings. Widened
  `nested_artboard_quantize_and_speed.riv` from sample `0.0` to samples `0.0,
  0.25, 0.5, 0.75, 1.0`, raising `exact-segments` to 432 while `exact`
  remains 111. `make golden-compare` reports `exact=111`,
  `exact-segments=432`, `diverges=0`, `unsupported-feature=184`,
  `not-yet=0`, and parked `M4=54 M5=8 M6=80 gated=6 harness=36`;
  `cargo test --workspace` passes.
- 2026-07-04: [M4] Closed `death_knight.riv` sample-0 nested remap
  `DrawTarget` ordering: Rust draw emission now rebuilds runtime draw order
  from active draw rules/placement values, mirrors C++ clipping proxy/save
  elision for that order, preallocates nested child paint caches before parent
  mutator paints, and defers the same-pass child update only for newly
  uncollapsed remap hosts. Promoted `death_knight.riv` to exact. `make
  golden-compare` reports `exact=112`, `exact-segments=433`, `diverges=0`,
  `unsupported-feature=183`, `not-yet=0`, and parked
  `M4=53 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Ported nested reported-event bubbling from C++
  `StateMachineInstance::notifyEventListeners`/`nestedEventListeners`: child
  state-machine reported events are collected during nested host advancement,
  parent event listeners no longer require hit paths, and parent listener
  actions settle with a zero-time advance only when a nested event actually
  changes the root state machine. Promoted `nested_event_test.riv` to exact.
  `make golden-compare` reports `exact=113`, `exact-segments=434`,
  `diverges=0`, `unsupported-feature=182`, `not-yet=0`, and parked
  `M4=52 M5=8 M6=80 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M4] Narrowed the recursive nested `ListenerAlignTarget`
  diagnostic to runs with input scripts, matching sample-0 static draw scope:
  unexercised align-target listener actions no longer park static nested
  files. Promoted `pointer_exit.riv` to exact and moved `align_target.riv` to
  M6 `rust-runner-unsupported:text`; input-driven recursive align-target
  behavior remains gated. `make golden-compare` reports `exact=114`,
  `exact-segments=435`, `diverges=0`, `unsupported-feature=181`,
  `not-yet=0`, and parked `M4=50 M5=8 M6=81 gated=6 harness=36`;
  `cargo test --workspace` passes.
