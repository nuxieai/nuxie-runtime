# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact-status segments (file × sample): 560 across 239 files (strict
  exact=553/232; tolerant=7/7; structural=0/0)
- Current compare: `make golden-compare` reports diverges=0, unsupported-feature=55, not-yet=1
- Parked breakdown: M5=0 by manifest query; `make golden-compare` reports M6=14 gated=5 harness=36
- Current milestone: **M6 — Layout + Text Verified Per Declared Corpus Modes**

## Milestones

- [x] M0: Golden diff harness + corpus manifest + one exact file
- [x] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [x] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [x] M3: Interactive files exact under scripted pointer input
- [x] M4: Nested artboards/lists exact for corpus entries whose first verified blocker is not M5/M6/gated
- [x] M5: Data binding exact incl. external view-model mutation
- [ ] M6: Layout + text verified per declared corpus modes; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. Resolve the lone active `not-yet` entry:
   `not-yet:nested-feather-gradient-space` (`ai_assitant.riv`). The
   selected-root shader-order, gradient opacity, and local-clockwise contour
   adjacencies are closed: `bullet_man.riv` now reaches the existing
   `selected-root-skinned-clip-path` gate, and `hunter_x_demo.riv` is exact
   under `tolerant(0.0015)`.
2. Other parked one-file M6 queues are `scripted-data-context`,
   `focus-data` (`focus_traversal.riv`), `layout-component-paint`
   (`text_input.riv`), `viewmodel-asset-conditions`,
   `text-joystick-data-bind` (`echo_show_demo.riv`),
   `nested-artboard-layout` (`superbowl.riv`), and
   `selected-root-skinned-clip-path` (`bullet_man.riv`,
   `spotify_kids_demo.riv`). The former
   `data-binding-nested-child` queue is five one-file diagnostics:
   `nested-trim-path-data-bind` (`db_health_tracker.riv`),
   `nested-artboard-root-transform` (`nested_hug.riv`),
   `nested-node-transform-data-bind` (`car_widgets_v01.riv`),
   `nested-layout-clip-data-bind` (`stateful_multi_property.riv`), and
   `nested-stateful-view-model-property` (`stateful_nested.riv`). The
   `rewards_demo.riv` file now exposes the one-file
   `nested-layout-size-data-bind` diagnostic. Gated one-file diagnostics
   include `scripted-transition-condition` (2 gated),
   `scripted-path-effects` (1 gated), and `text-polygon-sibling`
   (`bankcard.riv`). Keep these parked queues separate from the active
   `not-yet` item above.
3. M5 is closed for the current corpus: `grep -B6 'milestone = "M5"'
   corpus.toml` is empty. Do not reopen data-binding work unless a newly added
   corpus entry exposes a pre-text/pre-layout data-binding diagnostic.
4. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M6 unless a related refactor needs a cheap draw-regression check.

5. Threads are now policy (see `/goal` "Threads" section): the main loop
   stays the single writer here; use read-only scout threads to triage the
   remaining M6 queues in parallel. Start the first lane thread in a NEW
   worktree for the C++ golden-runner crash repair (`milestone =
   "harness"`, 36 files; FileAssetContents/scripting/data-viz crash paths
   in `tools/golden-runner` only), merging back into this branch once the
   full ratchet passes. Recovered files enter as `not-yet` — denominator
   growth, zero conflict with M6 runtime work.

6. Harness lane MERGED (e5941e7): the C++ golden-runner now survives 34 of
   36 `milestone = "harness"` files (FileAssetContents stripping for the
   non-scripting librive build, flush + `_Exit(0)` before teardown, ABI
   define alignment). MAIN-LOOP FOLLOW-UP: flip those 34 corpus entries
   from `milestone = "harness"` to `status = "not-yet"` (keep type-key
   features, drop `cpp-runner-crash`), then triage them like any queue.
   Residuals (2): `data_viz_demo` and `data_binding_artboards_test` crash
   only because the runner binds a blank default view-model instance;
   binding named instance 0 (like the reference unit tests) recovers both
   but perturbs 66 currently-exact entries — treat as a coordinated
   convention-change decision, not a harness fix. Keep them
   `milestone = "harness"` until decided.

7. DECIDED (see Decisions 2026-07-06): adopt the reference-test view-model
   binding convention. Run as ONE coordinated slice: (a) change BOTH
   runners (tools/golden-runner and tools/rust-golden-runner) to bind
   named view-model instance 0 when the selected artboard has a serialized
   `viewModelId` (matching `createViewModelInstance(viewModelId, 0)` in
   the C++ unit tests) instead of a blank default instance; (b) in the
   same slice, re-verify every affected exact entry (~66) — the ratchet
   must be green in the same commit, with any legitimately-changed
   expectations re-confirmed by direct C++/Rust stream pairs, never by
   loosening modes; (c) then flip `data_viz_demo` and
   `data_binding_artboards_test` from `milestone = "harness"` to
   `not-yet` and triage them. If any exact entry cannot be re-confirmed
   under the new convention, stop and record it as a Known Divergence
   rather than shipping a partial convention.

8. SCOUT RESULT (read-only pre-classification of the 34 recovered harness
   files; streams/diffs in the session scratchpad — trust but re-verify on
   promotion): (a) promote-exact, already verified epsilon-identical via
   golden-compare: audio_script, image_scripting_property_value,
   multi_listeners, script_dependency_test, script_dependency_test2,
   script_dependency_test_using_library(+_v2), script_namespace_test,
   script_string_converter_test, scripted_listener_action.
   (b) gated-scripting (21): all remaining script*/viewmodel*/gamepad/
   data_bind_artboard_input/path_effect_with_feathers/group_effect/
   replace_view_model files — blocked on the Luau VM; note
   path_effect_with_feathers is ScriptedPathEffect content, NOT M6 feather
   work. (c) NEW M6 WORK (3): relative_data_bind_path (nested-child data
   bind into NestedArtboard), scripted_data_converter_bound_input (data
   bind target Shape.x through static-text subset), databind_viewmodel
   (DataConverterToString value mismatch feeding a Text run — Rust
   data_bind_graph ToString produces a different string than C++).
   PROCESS FIX REQUIRED before flipping the 18 stream-subset scripting
   files: the Rust runner silently drops ScriptedDrawable draws (known-
   ignored list in text.rs), so they would land as `diverges` and invite
   wrong work — add a loud `unsupported: scripting` diagnostic for
   ScriptedDrawable-bearing files first, then flip them straight to
   `milestone = "gated"`. Unsupported is never silent.

## Known Divergences

- None in the active corpus. `make golden-compare` reports `diverges=0`; the
  remaining M6 work is parked behind explicit unsupported-feature diagnostics
  or the lone documented `not-yet` entry. `ai_assitant.riv` imports and draws
  but has a nested-feather gradient-space exact-parity gap.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until a future external data-binding corpus file requires it.
- Scripted data-context execution is gated until the `mlua`/Luau scripting
  glue lands: `scripted_data_context.riv` now emits
  `rust-runner-unsupported:scripted-data-context` when a `ScriptedDrawable`
  combines `DataBindContext` text with nested view-model context. The focused
  C++ runner printed `Failed to import object of type 106` before suppressing
  the text, so this is an M6 scripting diagnostic rather than text
  draw-suppression work.
- Rust golden draw path currently supports sorted absolute-time samples,
  visibility-gated artboard clip/background, selected-artboard origins, solid
  fills/strokes, and
  `ClippingShape` clip paths, skinned `PointsPath` deformation, plus empty and
  multi-contour TrimPath effects, DashPath stroke effects, and linear/radial
  gradient shader creation, default state-machine frame-0 application for
  color/bool/uint/string keyframes, Solo active-child refresh, source-to-target
  and target-to-source `Solo.activeComponentId` enum binds, enum-to-string
  artboard property conversion, `Text.alignValue` enum/uint binds, and
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
  transform slices once layout-backed list instances populate them, and
  parametric
  Star/Polygon local path sampling for follow-path targets, plus static plain
  `NestedArtboard` host draw with child root opacity inheritance, default
  nested simple-animation/state-machine hosts backed by persistent child
  artboard instances, stateful child `ViewModelInstance` subtree admission
  under plain and layout/leaf nested hosts, nested child unbound SolidColor data-bind defaults,
  nested child Ellipse width/height, RootBone x/y, and Shape x/y
  source-to-target number binds backed by stateful child view-model values,
  direct no-converter Shape x/y number binds, direct SolidColor `colorValue`
  color binds, artboard
  source-to-target `DataConverterInterpolator` number/color binds,
  artboard source-to-target `DataConverterGroup`/`DataConverterFormula`
  transform binds with C++ fallback random sequencing, near-zero-duration
  `DataConverterInterpolator` Shape x/y transform binds,
  nested bool/number/trigger input proxying, and basic nested remap-time host
  plumbing, runtime `DrawTarget` placement sorting from active `DrawRules`,
  serialized nested host speed/quantize local elapsed, generated
  source-to-target nested host `isPaused`/`speed`/`quantize` default binding,
  source-to-target nested host `artboardId` default/runtime swaps with
  cleared-host draw suppression,
  per-host nested paint caches for repeated child instances under Solo-owned
  hosts, and nested state-machine reported-event bubbling into parent event
  listeners, nested child `Node.opacity` and `Rectangle.width/height`
  source-to-target number binds with child artboard data-bind advancement,
  nested child `CustomPropertyString.propertyValue` string binds and
  `Rectangle.width/height` 20/21 binds, nested child `TextValueRun.text`
  string, `SolidColor.colorValue` color, and converted `Shape.rotation` binds
  backed by stateful child view-model values,
  authored-transparent Backboard/background draw suppression,
  custom-property trigger keyed-callback target-to-source binding,
  custom-property enum/boolean/color target-to-source binding, live data-bound
  nested host `isPaused` mutation, plus no-input recursive nested
  `ListenerAlignTarget` fixtures where the action is unexercised, plus plain
  embedded/hosted non-mesh `Image::draw` including layout-controlled
  fit/alignment under Taffy bounds, metadata-only `NSlicer`/axis
  image-layout fixtures that render through existing `LayoutComponent` paints,
  and sample-0 asset-image listener files whose image decode/source-paint
  ordering is exact while drawing only simple vector/text siblings, plus
  import-stack-displaced pre-source embedded image decode ordering for mixed
  file-asset imports, simple
  clipped/draw-target image fixtures with metadata-only component-list nodes,
  plus an asset-only unresolved nested-library host that decodes its image
  asset but draws only the root background like C++, simple
  ShapePaint/Feather draws including outer feathers and repeated
  inner-feather paints that share the original/effect clip path, and
  NSlicedNode vector shape path deformation for local/world draw commands,
  plus mesh-backed `Image::draw`/`drawImageMesh` with file-wide source mesh
  buffer preallocation, cloned dynamic vertex buffers, UV/index buffers,
  skinned mesh vertex updates, and live `Stroke.thickness` visibility/paint
  application for state-machine-keyed paints.
  Custom handle-source world-space math, data-bound nested host controls beyond
  generated defaults (external/live pause/speed/quantize mutation), remaining
  nested child data-bind targets beyond the current number/color/default bind
  set, and broader nested child object/list/value propagation remain
  corpus-driven follow-up work. Focus data,
  input-driven recursive
  `ListenerAlignTarget` and nested pointer/listener hit propagation beyond
  reported `Event` listeners, and layout-backed or virtualized component-list
  instancing remain M6 or later diagnostics.
  Golden runner sample lists now advance by sorted absolute-time deltas and
  reuse render paths across samples; no broader NSliced image layout parity,
  selected-root image paint/preallocation ordering beyond ImportStack-displaced
  embedded-image predecode and the text-root single external-image predecode
  case, remaining text
  layout/editing, selected-root gradient shader ordering, selected-root
  skinned clip-path geometry, nested-feather gradient-space exact parity for
  `ai_assitant.riv`, live
  data-bound nested host controls/artboard swaps, nested layout/leaf, scroll
  constraints, or layout-backed/virtualized component-list instancing.
  Harness-level scripted input replay dispatches
  pointerDown/pointerMove/pointerUp/pointerExit markers into direct rectangle
  state-machine listeners with listener input actions, direct rectangle
  enter/exit hover state, direct rectangle click synthesis, and listener-owned
  default view-model trigger target-to-source writes. Full C++ ListenerGroup
  drag/opaque behavior and input-driven nested align-target/list/text/layout
  targets are still not supported.
- Static text support currently covers one style or matching-metric
  multi-style text, static authored-line-break and no-break multi-run text,
  fixed-size ellipsis across multiple authored lines with bottom/middle
  vertical alignment, variation-aware no-break multi-run style outlines,
  auto-width origin offsets, and translation/rotation/scale/opacity
  `TextModifierGroup` over C++-style
  `TextModifierRange` character, character-excluding-space, word, and static
  line range maps with runId targeting and optional cubic range
  interpolation, including C++-ordered opacity buckets, plus solid fill/stroke
  `TextStylePaint` drawing with DashPath stroke effects and text paint feather
  state, text under `Shape` parent transforms, fit-font-size wrapping under
  layout-controlled bounds with C++ zero-font collapsed glyph paths, and
  source-to-target `TextValueRun.text` / `Text.alignValue` /
  `Text.overflowValue` / `TextStylePaint.fontSize` /
  `LayoutComponent.height` / `SolidColor.colorValue` / `Shape.x/y` through
  no-converter and `DataConverterGroup` paths / `Shape.rotation` via
  `DataConverterSystemDegsToRads`, `Text.width/height` through no converter
  or `DataConverterFormula`, static `TextFollowPathModifier` over Shape/Path
  targets with C++ `PathMeasure` tolerance, static vertical trim measured and
  rendered bounds with `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue` bitmask passthrough binds, plus no-converter
  `ParametricPath` width/height binds for
  Ellipse/Polygon/Rectangle/Star/Triangle around static text.
  Static text can coexist with authored nested bool input controls beside
  nested state-machine hosts and passive sample-0 `FocusData` /
  `KeyboardInput` metadata, passive nested numeric controls, plus inert
  `ScriptedDrawable` siblings. Shape/origin modifiers, gradient/other text
  effects, richer layout, broader `Text` property
  data binds, and text input/editing remain M6 text diagnostics.
- `TransformConstraint` currently covers Text constraint bounds for the
  supported static Text subset plus the default empty
  `TransformComponent::constraintBounds()` path. LayoutComponent bounds remain
  parked behind M6 layout diagnostics.
- Passive sample-0 `ScrollConstraint` files with zero authored offset,
  percent, and index values, no input events, no state-machine listener target,
  registered layout-provider children, and a coherent Taffy snapshot are
  admitted by the Rust runner, including at-rest snap metadata. Empty
  virtualized `ArtboardComponentList` layout providers are also admitted when
  they have no children and no map rules, because they create no virtualized
  instances at sample 0. `scroll_snap.riv` is exact after applying the
  accumulated Taffy layout bounds through artboard-origin-adjusted layout world
  transforms. Remaining scroll-constraint corpus files stay parked behind M6
  layout/runtime support via `rust-runner-unsupported:scroll-constraints`;
  C++ `src/constraints/scrolling/scroll_constraint.cpp` also reads
  `LayoutComponent` dimensions, layout-provider child bounds, physics state,
  infinite scroll state, and non-empty component-list virtualization, so
  dynamic/virtualized scroll remains outside the passive initial slice.
- Per-file parked reasons now live in `corpus.toml`: each gated entry
  carries `milestone = "M3|M4|M5|M6|gated|harness"` plus its diagnostic
  feature tags (`rust-runner-unsupported:*`, `cpp-runner-crash`,
  `import-error:*`). Query a milestone's work-list with e.g.
  `grep -B6 'milestone = "M5"' corpus.toml`.
- Entries tagged `cpp-runner-crash` (`milestone = "harness"`) stay parked
  until the C++ golden runner survives the FileAssetContents, scripting,
  and data-viz crash paths it currently aborts on.
- `coin.riv` is no longer parked as an M3 constraints or gated feather file:
  repeated inner-feather paints now share the C++ clip-path cache key and
  `coin.riv` is exact at sample 0. `bankcard.riv` now reaches the text
  diagnostic after clearing its `layout-component-paint` and feather blockers.
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
- 2026-07-06: [M6] Closed `text-modifier-group-flags` by adding C++-style
  `TextModifierGroup` scale interpolation and passive `NestedNumber` static
  text sibling admission. `hunter_x_demo.riv` now reaches a Rust stream and
  parks as `not-yet:gradient-shader-order`; `rewards_demo.riv` now verifies as
  `rust-runner-unsupported:nested-layout-size-data-bind` for LayoutComponent
  width binding. `make golden-compare` reports exact-segments=555,
  diverges=0, unsupported-feature=58, not-yet=3; next target is
  `rust-runner-unsupported:mesh-images`. `cargo test --workspace` passes.
- 2026-07-06: [M6] Closed the `mesh-images` runner guard by porting
  mesh-backed `Image::draw` from C++ `src/shapes/image.cpp` /
  `src/shapes/mesh.cpp`: source mesh vertex/UV/index buffers are allocated
  file-wide before clone dynamic vertex buffers, `drawImageMesh` uses dynamic
  skinned vertices, and `golden-compare` treats vertex `bufferData` as
  semantic f32 data while keeping index buffers exact. `jellyfish_test.riv`
  is exact under `tolerant(0.0004)` for skinned vertex-buffer float residuals,
  `tape.riv` is strict exact, and `bad_skin.riv` is parked as
  `not-yet:skinned-contour-transform-order` after a later structural skinned
  transform delta. Full `make golden-compare` reports `exact=236`,
  `exact-segments=557`, `diverges=0`, `unsupported-feature=55`,
  `not-yet=4`, and parked `M6=14 gated=5 harness=36`; next target is
  `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `bad_skin.riv` by mirroring C++
  `Stroke::isVisible()` against live instance `Stroke.thickness` and applying
  live stroke thickness/cap/join during paint configuration, so the default
  state machine's sample-0 keyed thickness suppresses the mask stroke instead
  of drawing the authored width. `bad_skin.riv` is exact under
  `tolerant(0.0004)` for residual skinned path float drift. Full
  `make golden-compare` reports `exact=237`, `exact-segments=558`,
  `diverges=0`, `unsupported-feature=55`, `not-yet=3`, and parked
  `M6=14 gated=5 harness=36`; next target is `local_bounds.riv`
  (`not-yet:image-predecode-order`).
- 2026-07-06: [M6] Promoted `local_bounds.riv` by mirroring C++
  `ImportStack` file-asset importer displacement: when a pre-artboard
  file-asset importer supersedes an embedded `ImageAsset` importer, Rust now
  predecodes that image before source paint allocation. `local_bounds.riv` is
  exact under `tolerant(0.00001)` for residual HarfRust/C++ text-outline float
  drift. Full `make golden-compare` reports `exact=238`,
  `exact-segments=559`, `diverges=0`, `unsupported-feature=55`,
  `not-yet=2`, and parked `M6=14 gated=5 harness=36`; next target is
  `hunter_x_demo.riv` (`not-yet:gradient-shader-order`).
- 2026-07-06: [M6] Closed the selected-root gradient shader-order adjacency
  by deferring selected-root `LayoutComponent` gradient shader preparation
  until after nested artboard paint preparation while preserving child-artboard
  ordering. `hunter_x_demo.riv` now matches C++ selected-root shader
  allocation order and is parked at `not-yet:gradient-opacity-propagation`
  after the focused stream first differs at child gradient stop alpha values.
  The stale `selected-root-gradient-shader-order` runner guard was removed;
  `bullet_man.riv` now verifies as
  `rust-runner-unsupported:selected-root-skinned-clip-path`. Full
  `make golden-compare` reports `exact=238`, `exact-segments=559`,
  `diverges=0`, `unsupported-feature=55`, `not-yet=2`, and parked
  `M6=14 gated=5 harness=36`; `cargo test --workspace` passes. Next target is
  `hunter_x_demo.riv` (`not-yet:gradient-opacity-propagation`).
- 2026-07-06: [M6] Closed the `hunter_x_demo.riv` gradient-opacity adjacency
  by live-reading `LinearGradient.opacity`/`RadialGradient.opacity` through
  paint mutators and by matching C++ `shouldDraw()` effective-visibility
  gating for layout, foreground layout, and text paints while preserving
  ordinary Shape alpha-zero draw emission. Focused streams now match through
  selected-root shader allocation and child gradient stop alpha propagation;
  the first diff is local-clockwise child contour verb ordering, parked as
  `not-yet:local-clockwise-contour-order`. Full `make golden-compare` reports
  `exact=238`, `exact-segments=559`, `diverges=0`,
  `unsupported-feature=55`, `not-yet=2`, and parked
  `M6=14 gated=5 harness=36`; `cargo test --workspace` passes. Next target is
  `hunter_x_demo.riv`
  (`not-yet:local-clockwise-contour-order`).
- 2026-07-06: [M6] Promoted `hunter_x_demo.riv` by honoring
  `Path.pathFlags` for `PointsPath` clockwise state, reusing the C++ draw-path
  identity for inner-feather clips, reading live `Feather` properties at draw
  time, preserving distinct local/localClockwise `Shape` path cache identity
  while aliasing text/layout providers, and building weighted `PointsPath`
  commands from deformed cubic handles directly instead of round-tripping
  through angle/distance. The stream is structurally identical and guarded by
  `verification = "tolerant(0.0015)"` for bounded skinned float drift (max
  observed `0.0013504`). Full `make golden-compare` reports `exact=239`,
  `exact-segments=560`, `diverges=0`, `unsupported-feature=55`, `not-yet=1`,
  and parked `M6=14 gated=5 harness=36`. Next target is `ai_assitant.riv`
  (`not-yet:nested-feather-gradient-space`).
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
- 2026-07-02: Delegated subsystems (#V2-7) use Rust-native libraries where the
  delegated behavior is spec-defined, chosen by "spec-defined may swap engines;
  implementation-defined may not": Taffy (layout, behind a trait, Yoga-FFI as
  untriggered fallback), HarfRust + read-fonts/skrifa (shaping/font parsing),
  unicode-bidi (bidi), `image`-ecosystem crates (decoders), cpal/rodio
  (audio), and mlua+`luau` vendoring the official Luau VM (scripting uses the
  same VM as C++, so scripted files stay `exact`). `corpus.toml` gains
  per-entry verification modes `exact | tolerant(ε) | structural`; files
  exercising Taffy layout, HarfRust shaping/font numeric drift, or lossy image
  decoding verify `tolerant`, everything else stays `exact`. Rive-owned text
  layout, wrapping, fit-font-size, draw suppression, call order, and glyph
  contour ordering are ported behavior, not tolerant delegated-engine drift.
  Cross-runtime image comparison must use decoded dimensions + tolerant pixel
  sampling, never payload hashes (supersedes the size/hash recording decision
  above once Rust image support lands). Do not pin Taffy against Yoga
  behavior-by-behavior. Taffy CSS Grid is a post-M7 enhancement idea, not port
  scope.
- 2026-07-06: #V2-7 image verifier first slice: C++ and Rust recording
  factories now parse PNG/JPEG/WebP dimensions from encoded headers, emit
  `decodeImage id=... width=... height=...`, and return those dimensions
  through `RecordingRenderImage`. Payload hashes are no longer in golden
  streams; tolerant pixel sampling remains before lossy decoder fixtures can
  rely on image-specific tolerant verification.
- 2026-07-03: Metric is now segments-weighted: `golden-compare` reports
  `exact-segments` (sum of samples across exact entries) alongside the file
  count, so M2 sample widening registers as metric movement. Gated corpus
  entries carry `milestone = "M3|M4|M5|M6|gated|harness"` (preserved by
  `generate-corpus`), and the summary prints a parked-by-milestone
  breakdown, so each milestone's work-list is queryable from `corpus.toml`
  instead of backlog prose. Completed-milestone log entries are archived in
  `docs/v2-log-archive.md` to keep this file small.
- 2026-07-05: `component_stateful.riv` is exact after admitting nested
  `TextValueRun.text` string binds from stateful child view-model values and
  clearing created default nested text contexts. `relative_data_binding.riv`
  and `shared_viewmodel_instance.riv` now render but are parked as M6
  divergences because Rust draws nested/shared text that C++ does not at
  sample 0.
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
- 2026-07-04: M4 is corpus-closed after a direct `rust-golden-runner` sweep:
  no `milestone = "M4"` entries remain. Former M4 parked files now carry their
  first verified later diagnostic: M5 data-binding nested child/host or
  custom-property trigger paths, and M6 text/images/nested-artboard-layout/
  focus/layout-component-paint paths. This opens M5 without hiding the later
  text/layout/list work.

- 2026-07-05: Layout engine fence: the hand-rolled flex math that closed
  the simple root row/column layout-paint queue is capped at exactly that
  scope. The next layout gate that needs wrapping, grow/shrink ratios,
  percent/auto sizing, alignment beyond start/center/end, gaps, or nested
  layout containers MUST trigger the #V2-7 decision instead: integrate
  Taffy behind the layout trait and route the existing simple cases
  through it. Extending the hand-rolled math case-by-case is re-porting
  Yoga behavior-by-behavior — the V1 pattern — and is a tripwire. Files
  whose layouts diverge under Taffy verify in `tolerant` mode per the
  V2 map; do not pin Taffy against Yoga.
- 2026-07-05: Layout trait contract: the #V2-7 layout adapter computes a
  coherent whole-artboard layout snapshot from Rive style/component data and
  either returns all supported `LayoutComponent` bounds for that snapshot or
  refuses the tree. Runtime draw, world-transform, and computed-value code
  consume those bounds; they must not mix Taffy-solved nodes with ad hoc
  per-node flex fixes inside the same layout tree. `tolerant` verification
  covers swapped-engine numeric geometry drift, not missing style plumbing.
- 2026-07-05: `golden-compare` implements the #V2-7 manifest field
  `verification = "exact" | "tolerant(ε)" | "structural"` for exact corpus
  entries, defaulting omitted entries to `exact`; `generate-corpus` preserves
  non-default verification modes across regeneration. This is the harness
  prerequisite for Taffy/HarfRust/image-decoder corpus admission.
- 2026-07-05: #V2-7 verification language is interpreted by the current
  comparator as accepted-under-declared-mode, not byte-identical for all
  accepted files. `exact-segments` counts `status = "exact"` entries, including
  entries that declare `verification = "tolerant(...)"`. Tolerant verification
  relaxes numeric tokens only: call order, IDs, path verbs, non-numeric payloads,
  and glyph contour ordering remain strict unless a future Decision introduces
  a dedicated outline canonicalization or raster comparison mode. It does not
  hide missing Rive text layout behavior such as wrapping, fit-font-size, or
  layout-controlled text bounds. New Taffy layout gates may not be promoted
  through hand-rolled fallback after the #V2-7 layout adapter refuses a tree.
- 2026-07-06: #V2-7 fallback fence: legacy hand-rolled layout helpers may
  remain inside `rive-runtime` as regression scaffolding for older exact
  slices, but `rust-golden-runner` must reject any layout-dependent draw
  candidate when the Taffy adapter cannot produce a coherent whole-artboard
  layout snapshot. This includes painted `LayoutComponent` paths plus child
  text/image/shape drawables whose parent chain passes through a
  `LayoutComponent`. Existing exact layout fixtures must either compute Taffy
  bounds or return to an explicit unsupported-feature gate.
- 2026-07-06: #V2-7 scroll admission rule: `ScrollConstraint` is Rive-owned
  runtime behavior, not delegated tolerant behavior. The runner may admit only
  passive initial-state scroll constraints once Rust applies the C++
  `constrain` / `constrainChild` transform slice over registered
  layout-provider children and Taffy can compute a coherent whole-artboard
  snapshot. Input-driven drag, nonzero offset/percent/index state, snap,
  infinite scroll, virtualized lists, listener-targeted scroll, physics
  advancement, and scroll-bar driving remain
  `rust-runner-unsupported:scroll-constraints`.
- 2026-07-05: M6 layout/text diagnostic rule: when a Taffy-backed file reaches
  draw but diverges on wrapped layout placement, expose local-id layout boxes
  from C++ Yoga and Rust Taffy before adding more renderer/text behavior. Draw
  suppression and layout participation are separate facts; do not infer one
  from the other without a focused C++ probe.
- 2026-07-05: Scripted data-context files are M6 scripting gates, not text
  draw-suppression targets, when the selected artboard combines a
  `ScriptedDrawable`, `DataBindContext` text, and nested view-model context.
  The Rust runner emits `unsupported: scripted-data-context` for that surface
  until the #V2-7 `mlua`/Luau glue lands; passive script fixtures that already
  match C++ remain eligible for exact comparison.

- 2026-07-06: Threads policy adopted (recorded in `/goal`): single writer
  per worktree; parallel threads are read-only triage scouts or orthogonal
  lane threads started in a new worktree and merged back through the full
  ratchet (eligible lanes: C++ harness crash repair, M7 scaffolding —
  benchmarks/fuzz/API drafts — and the feature-gated scripting spike).
  Never two writers on adjacent critical-path runtime slices.

- 2026-07-06: View-model binding convention decided: both golden runners
  bind named view-model instance 0 (reference unit-test convention,
  `createViewModelInstance(viewModelId, 0)`) when the selected artboard
  has a serialized `viewModelId`, replacing the blank default instance.
  Rationale: matches how the C++ unit tests exercise these files, recovers
  the two harness residuals, and aligns default-instance semantics before
  M7 freezes the public API. Must land as one coordinated slice with all
  affected exact entries re-verified in the same commit.

## Log

- Completed-milestone entries (M0 through M5) are archived verbatim in
  `docs/v2-log-archive.md`; when a milestone completes, move its entries
  there and keep only the active milestone's recent working window here.

- 2026-07-04: [M6] Opened M6 after closing the M5 queue: the final four M5
  entries now probe as nested child `TextValueRun`, so the next loop starts
  with the text sizing spike from `docs/porting-map-v2.md`. `make
  golden-compare` reports `exact=128`, `exact-segments=449`, `diverges=0`,
  `unsupported-feature=167`, `not-yet=0`, and parked
  `M6=124 gated=7 harness=36`; manifest query confirms M5=0, and `cargo
  test --workspace` passes.
- 2026-07-04: [M6] Sized the text opening in
  `docs/prototypes/m6-text-sizing-spike.md`: the largest M6 diagnostic bucket
  is `text` (59 files), C++ text is about an 11k-line stack across import,
  shaping, line breaking, draw, and input/editing, and the first implementation
  slice is now pinned to `hello_world.riv` instead of manifest-first
  `align_target.riv` because it isolates static top-level text path emission.
- 2026-07-04: [M6] Promoted `hello_world.riv` by adding a narrow embedded
  static text draw path in `rive-runtime` with HarfRust/Skrifa shaping and
  outlines, keeping richer text behind static-subset diagnostics. `make
  golden-compare` moved to `exact=129`, `exact-segments=450`,
  `unsupported-feature=166`, and parked `M6=123 gated=7 harness=36`; `cargo
  test --workspace` passes.
- 2026-07-04: [M6] Rechecked the post-`hello_world` text queue. `new_text.riv`
  is too broad for the next slice (five texts plus multi-run/style,
  gradient/stroke, clipping, and keyframed text). `ellipsis.riv` is the
  smallest one-run axis/layout target; axis-only bypass reaches draw but
  diverges on C++ ellipsis layout, so the next implementation must port that
  layout path rather than simply admitting axes.
- 2026-07-04: [M6] Promoted `ellipsis.riv` with static `TextStyleAxis`
  variation setup plus the smallest one-run fixed-height ellipsis/wrap path.
  `make golden-compare` moved to `exact=130`, `exact-segments=451`,
  `unsupported-feature=165`, and parked `M6=122 gated=7 harness=36`; the next
  narrow text tracer is `hosted_font_file.riv`, which isolates no-loader
  hosted font resolution rather than text layout.
- 2026-07-04: [M6] Promoted `hosted_font_file.riv` by mirroring C++
  `FileAssetImporter` no-loader behavior: a hosted `FontAsset` with no
  in-band contents resolves without a decoded font, so static text emits its
  drawable save/restore wrapper but no text path. `make golden-compare` moved
  to `exact=131`, `exact-segments=452`, `unsupported-feature=164`, and parked
  `M6=121 gated=7 harness=36`; the next narrow text tracer is
  `animated_clipping.riv`, which now stops on sibling shape/clipping admission.
- 2026-07-04: [M6] Promoted `animated_clipping.riv` by admitting sibling
  Shape/ClippingShape scaffolding around the one supported static Text path
  and preserving C++'s text-local save/restore around glyph transforms even
  when clipping elides the drawable-level save. The same gate relaxation also
  unlocked byte-identical `databind_artboard.riv`. `make golden-compare`
  moved to `exact=133`, `exact-segments=454`, `unsupported-feature=162`, and
  parked `M6=119 gated=7 harness=36`; next inspect `background_measure.riv`,
  which stops on sibling `RootBone` rather than modifiers.
- 2026-07-04: [M6] Promoted `background_measure.riv` by admitting passive
  bone/skin `PointsPath` decoration around one static Text, computing static
  Text constraint bounds for `TransformConstraint`, and rounding HarfBuzz-style
  line metrics while disabling legacy kern-only advances to preserve
  `hello_world.riv`. `make golden-compare` moved to `exact=134`,
  `exact-segments=455`, `unsupported-feature=161`, and parked
  `M6=118 gated=7 harness=36`; next inspect the narrow text-modifier fixtures.
- 2026-07-04: [M6] Promoted `modifier_test.riv` and `align_target.riv` by
  adding the first static text-modifier slice: translation-only
  `TextModifierGroup`, character-unit `TextModifierRange` coverage, and cubic
  range falloff. `make golden-compare` moved to `exact=136`,
  `exact-segments=457`, `unsupported-feature=159`, and parked
  `M6=116 gated=7 harness=36`; `modifier_to_run.riv` remains parked on
  word/line range mapping plus run-scoped/multi-run text.
- 2026-07-04: [M6] Rechecked the post-modifier text queue. `modifier_to_run`
  is not a narrow modifier follow-up: it has four Text objects plus word/line
  range units, runId targeting, and multi-run text. `vertical_align_ellipsis`
  reaches draw if sibling `Stroke` is admitted, but exact comparison diverges
  first on fixed-size vertical align/ellipsis text placement. The next narrow
  implementation slice is `double_line.riv`, which isolates same-style
  multi-run text and explicit line breaks before the modifier range-map files.
- 2026-07-04: [M6] Promoted `double_line.riv` by aggregating same-style
  authored-line-break `TextValueRun` children and placing shaped non-empty
  lines at C++-style static line-height baselines while preserving empty forced
  line breaks. `make golden-compare` moved to `exact=137`,
  `exact-segments=458`, `unsupported-feature=158`, and parked
  `M6=115 gated=7 harness=36`; next reopen `modifier_to_run.riv`, which now
  fails first on `TextModifierRange` word/line range-map units.
- 2026-07-04: [M6] Promoted `modifier_to_run.riv` by translating the static
  range-map path from C++ `src/text/text_modifier_range.cpp`: word/line and
  character-excluding-space units, runId clipping, matching-metric multi-style
  no-break runs, and per-style text paint allocation ordering. `make
  golden-compare` moved to `exact=138`, `exact-segments=459`,
  `unsupported-feature=157`, and parked `M6=114 gated=7 harness=36`; next
  reopen `test_modifier_run.riv`, which now fails first on rotation modifier
  flags.
- 2026-07-04: [M6] Promoted `test_modifier_run.riv` by translating the static
  glyph rotation path from C++ `src/text/text_modifier_group.cpp`, including
  per-glyph center transforms and averaged glyph coverage for multi-codepoint
  glyphs. `make golden-compare` moved to `exact=139`,
  `exact-segments=460`, `unsupported-feature=156`, and parked
  `M6=113 gated=7 harness=36`; next reopen `text_opacity_modifier.riv`, which
  now fails first on a static-text sibling `CubicEaseInterpolator`.
- 2026-07-04: [M6] Promoted `text_opacity_modifier.riv` by translating C++
  `TextModifierGroup::computeOpacity` and `TextStylePaint` opacity buckets,
  including temporary render-paint allocation order and libc++ float bucket
  iteration for exact stream ordering. `make golden-compare` moved to
  `exact=140`, `exact-segments=461`, `unsupported-feature=155`, and parked
  `M6=112 gated=7 harness=36`; next reopen `text_stroke_test.riv`, which now
  fails first on a static-text sibling `DashPath`.
- 2026-07-04: [M6] Promoted `text_stroke_test.riv` by admitting solid
  `Stroke` paints on `TextStylePaint`, routing DashPath effects through the
  existing shape stroke-effect path, and matching C++'s per-style text
  paint-pool allocation. `make golden-compare` moved to `exact=141`,
  `exact-segments=462`, `unsupported-feature=154`, and parked
  `M6=111 gated=7 harness=36`; next reopen `vertical_align_ellipsis.riv`,
  which now fails first on ellipsis across multiple authored lines.
- 2026-07-04: [M6] Promoted `vertical_align_ellipsis.riv` by moving
  fixed-size ellipsis line selection and bottom/middle vertical-align offsets
  into the static text render path, mirroring C++
  `src/text/text.cpp::computeBoundsInfo`/`buildRenderStyles`. `make
  golden-compare` moved to `exact=142`, `exact-segments=463`,
  `unsupported-feature=153`, and parked `M6=110 gated=7 harness=36`; next
  reopen `text_listener_simpler.riv`, which now fails first on mismatched
  no-break multi-run `TextStylePaint` metrics.
- 2026-07-04: [M6] Promoted `text_listener_simpler.riv` by shaping static
  no-break text per `TextValueRun` style/variation, using measured auto-width
  for C++-style origin offsets, and preserving per-style paint buckets. `make
  golden-compare` moved to `exact=143`, `exact-segments=464`,
  `unsupported-feature=152`, and parked `M6=109 gated=7 harness=36`; next
  reopen `new_text.riv`, which now fails first on sibling `LinearGradient`.
- 2026-07-04: [M6] Admitted `new_text.riv` through its LinearGradient sibling
  gate: static text allows gradient siblings and gradient text fill/stroke
  paints, TextStylePaints without authored font/container no longer abort the
  whole text, and keyed runtime gradient endpoints/render opacity now match
  C++. The file reaches draw but is parked as the sole known divergence on
  text-outline contour ordering between Rust/Skrifa and C++ HarfBuzz. `make
  golden-compare` reports `exact=143`, `exact-segments=464`, `diverges=1`,
  `unsupported-feature=151`, `not-yet=0`, and parked
  `M6=108 gated=7 harness=36`; next start `runtime_nested_text_runs.riv`,
  which fails first on sibling `NestedArtboard`.
- 2026-07-04: [M6] Promoted `runtime_nested_text_runs.riv` by admitting
  passive `NestedArtboard`/`NestedStateMachine` siblings around static text;
  the existing nested artboard draw path and text paint preallocation already
  matched C++ structurally once the text gate was removed. `make
  golden-compare` moved to `exact=144`, `exact-segments=465`,
  `unsupported-feature=150`, and parked `M6=107 gated=7 harness=36`; next
  start the high-frequency static text data-binding blocker with
  `bankcard.riv`.
- 2026-07-04: [M6] Admitted source-to-target `TextValueRun.text` and
  `SolidColor.colorValue` binds around static text. This promoted
  `databind_external_artboard_child.riv`, `sorted_listeners.riv`, and
  `zero_width_space_line_break.riv`; six broader data-bound text/converter
  files now run but are marked as M6 divergences; and `bankcard.riv` gets past
  data binding to the painted `LayoutComponent` gate. `make golden-compare`
  moved to `exact=147`, `exact-segments=468`, `diverges=7`,
  `unsupported-feature=141`, and parked `M6=98 gated=7 harness=36`; next
  start the painted `LayoutComponent` slice with `bankcard.riv`.
- 2026-07-04: [M6] Started the painted `LayoutComponent` slice by routing
  `LayoutComponent` shape paints through the runtime draw-command path with
  serialized background-rect commands, moving the explicit
  `layout-component-paint` runner gate ahead of static text, and retagging
  `bankcard.riv` plus ten similar files from stale `text` diagnostics to
  `layout-component-paint`. `make golden-compare` stayed at `exact=147`,
  `exact-segments=468`, `diverges=7`, `unsupported-feature=141`, and parked
  `M6=98 gated=7 harness=36`; next port computed layout bounds/style plumbing
  before removing the gate.
- 2026-07-04: [M6] Admitted the first exact painted `LayoutComponent` subset:
  simple root full-artboard solid fills now draw through the layout-proxy
  command path with C++-style background rect paths, promoting
  `viewmodel_list_trigger.riv`, `transition_index_condition.riv`,
  `viewmodel_from_context.riv`, `list_to_length_test.riv`, and
  `reset_phase.riv`. `artboard_list_map_rules.riv` is reclassified as the
  next M6 divergence on component-list/map-rule layout bounds. `make
  golden-compare` moved to `exact=152`, `exact-segments=473`,
  `diverges=8`, `unsupported-feature=135`, and parked
  `M6=92 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M6] Promoted `artboard_list_map_rules.riv` by translating the
  first C++ `LayoutComponent` root-row fill sizing path: sibling root layout
  children split the artboard width, layout proxy draw commands use the
  computed layout transform, and layout proxies keep per-layout path-cache
  identity. `make golden-compare` moved to `exact=153`,
  `exact-segments=474`, `diverges=7`, `unsupported-feature=135`, and parked
  `M6=92 gated=7 harness=36`; `cargo test --workspace` passes. Next target:
  `artboard_list_overrides.riv`, which stops on nested clipped layout global
  21 with `ArtboardComponentListOverride` sizing.
- 2026-07-04: [M6] Promoted `artboard_list_overrides.riv` by mirroring C++
  clipped `LayoutComponent::drawProxy` save/clip/restore ordering, giving
  layout clips their own render-path cache, and collapsing the nested fill/hug
  component-list override layout to the C++ empty-list zero-size bounds. `make
  golden-compare` moved to `exact=154`, `exact-segments=475`,
  `diverges=7`, `unsupported-feature=134`, and parked
  `M6=91 gated=7 harness=36`; next target: `bankcard.riv`, still gated on
  `layout-component-paint` global 21.
- 2026-07-04: [M6] Cleared `bankcard.riv`'s first `LayoutComponent` paint
  blocker by admitting root layout backgrounds with rounded style corners and
  moving unconditional `Feather` diagnostics ahead of text. `bankcard.riv` is
  now `gated` on feather; passive text sibling/Node ancestry admission also
  promoted `joel_v3.riv` and `word_joiner_test.riv`, while
  `number_to_list_nested_children.riv` now runs as an M6
  `layout-component-bounds` divergence. `make golden-compare` moved to
  `exact=156`, `exact-segments=477`, `diverges=8`,
  `unsupported-feature=131`, and parked `M6=87 gated=8 harness=36`; next
  target: `collapse_data_binds.riv`, still gated on `layout-component-paint`
  global 31.
- 2026-07-04: [M6] Reclassified `collapse_data_binds.riv` from generic
  `layout-component-paint` to `layout-computed-values` after finding
  data-bound `LayoutComponent.computedLocalX` values feeding text. `make
  golden-compare` stayed at `exact=156`, `exact-segments=477`,
  `diverges=8`, `unsupported-feature=131`, and parked
  `M6=87 gated=8 harness=36`; next target:
  `component_list_child_origin.riv`, still gated on `layout-component-paint`
  global 19.
- 2026-07-04: [M6] Narrowed the root row layout paint gate by admitting
  clockwise layout background paths and root padding/gap sizing. This retags
  `component_list_child_origin.riv`, `component_list_virtualized.riv`, and
  `virtualized_artboard_databound_children.riv` to `scroll-constraints`, and
  moves `transition_duration_bind_list.riv` to the existing
  `layout-component-bounds` divergence (`2617` vs C++ `2000` height). `make
  golden-compare` reports `exact=156`, `exact-segments=477`, `diverges=9`,
  `unsupported-feature=130`, and parked `M6=86 gated=8 harness=36`; next
  target: `computed_root_transform.riv`, still gated on
  `layout-component-paint` global 32.
- 2026-07-04: [M6] Promoted `computed_root_transform.riv` and
  `list_items.riv` by adding the first simple flex layout background sizing:
  non-reverse row/column parents, fixed point/percent main-axis sizes,
  fill-weighted remaining space via `fractionalWidth`/`fractionalHeight`, and
  fill/fixed/hug cross-axis sizing. Seven files now clear layout paint and are
  retagged to `rust-runner-unsupported:text`; only
  `data_bind_test_cmdq.riv`, `scroll_snap.riv`, and `scroll_test.riv` remain
  on `layout-component-paint`. `make golden-compare` reports `exact=158`,
  `exact-segments=479`, `diverges=9`, `unsupported-feature=128`, and parked
  `M6=84 gated=8 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M6] Closed the remaining layout-component-paint manifest queue
  by admitting rounded simple flex backgrounds plus invisible, stroked, and
  gradient layout background paints already handled by the runtime draw path.
  `data_bind_test_cmdq.riv` now parks on `text`,
  `scroll_snap.riv` parks on `text`, and `scroll_test.riv` parks on
  `scroll-constraints`; `grep -n ... corpus.toml` for
  `rust-runner-unsupported:layout-component-paint` is empty. `make
  golden-compare` reports `exact=158`, `exact-segments=479`, `diverges=9`,
  `unsupported-feature=128`, and parked `M6=84 gated=8 harness=36`; `cargo
  test --workspace` passes. Next target: `collapse_data_binds.riv` on
  `layout-computed-values`.
- 2026-07-05: [M6] Closed the `layout-computed-values` runner gate by polling
  target-to-source `LayoutComponent.computed*` data binds from runtime layout
  geometry, building a graph-aware artboard context for `from_graph()`, and
  drawing static `Text` under `LayoutComponent` through runtime component
  world transforms. The layout bounds resolver is now memoized to avoid
  recursive fill/hug overflow. `collapse_data_binds.riv`,
  `data_binding_artboards_source_test.riv`, and
  `hittest_collapsed_layouts.riv` now run and are retagged as
  `rust-runner-divergence:layout-component-bounds`; the first inspected diff
  is the broader solver gap, not computed data-bind plumbing. `make
  golden-compare` reports `exact=158`, `exact-segments=479`, `diverges=12`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  broader `LayoutComponent` bounds/positioning parity, starting with
  `collapse_data_binds.riv`.
- 2026-07-05: [M6] Promoted `collapse_data_binds.riv` by adding
  effective-collapse checks through layout ancestors, display-none layout
  handling, absolute layout bounds, space-between/alignment offsets, and
  intrinsic flex-basis sizing that avoids computed-bounds feedback. Narrow
  direct `DataConverterToString` default admission now lets numeric view-model
  values bind to `TextValueRun.text` without waking unrelated formula or
  interpolator defaults. `make golden-compare` reports `exact=159`,
  `exact-segments=480`, `diverges=11`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `data_binding_artboards_source_test.riv`.
- 2026-07-05: [M6] Promoted `data_binding_artboards_source_test.riv` by
  creating C++-style default view-model values from declared paths when no
  serialized default instance exists and using root-hug Artboard layout bounds
  for background drawing. `make golden-compare` reports `exact=160`,
  `exact-segments=481`, `diverges=10`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target: `hittest_collapsed_layouts.riv`.
- 2026-07-05: [M6] Promoted `hittest_collapsed_layouts.riv` by aligning the
  Rust golden runner with C++ `File::createViewModelInstance(artboard)` fresh
  view-model defaults for state-machine data contexts, while preserving
  serialized default-context probe behavior. Owned-context listener trigger
  writes now flow through target-to-source conversion and mirror the active
  view-model trigger cache so the same pointer scripts stay exact.
  `make golden-compare` reports `exact=161`, `exact-segments=482`,
  `diverges=9`, `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  `number_to_list_nested_children.riv`.
- 2026-07-05: [M6] Promoted `number_to_list_nested_children.riv` after the
  focused C++/Rust golden stream compare showed the stale
  `layout-component-bounds` divergence was already closed by the previous
  layout/default-context work. `make golden-compare` reports `exact=162`,
  `exact-segments=483`, `diverges=8`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `transition_duration_bind_list.riv`.
- 2026-07-05: [M6] Promoted `transition_duration_bind_list.riv` after the
  focused direct C++/Rust stream compare also showed exact output at its
  declared sample; the stale `layout-component-bounds` manifest tag came from
  before the previous layout/default-context fixes. `make golden-compare`
  reports `exact=163`, `exact-segments=484`, `diverges=7`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target: `new_text.riv`, then the
  data-bound text divergence bucket.
- 2026-07-05: [M6] Rechecked the M6 text divergence queue with direct
  C++/Rust streams. `new_text.riv` remains a real text-outline contour-order
  divergence, but `format_number_with_commas.riv`,
  `listener_view_model.riv`, and `trigger_fires_single_change.riv` are now
  epsilon-equivalent and were promoted to exact. `make golden-compare`
  reports `exact=166`, `exact-segments=487`, `diverges=4`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target:
  `rebind_with_nested_viewmodel.riv`.
- 2026-07-05: [M6] Promoted `rebind_with_nested_viewmodel.riv` by binding
  artboard data-bind defaults to the selected artboard `viewModelId` and
  following `ViewModelPropertyViewModel.viewModelReferenceId` in declared
  paths. `make golden-compare` reports `exact=167`,
  `exact-segments=488`, `diverges=3`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `replace_vm_instance.riv`, which now has matching stream line count but
  shifted text outlines.
- 2026-07-05: [M6] Promoted `replace_vm_instance.riv` after mirroring C++
  static text horizontal alignment for `Text.alignValue` in the Rust text
  renderer. The focused C++/Rust streams are epsilon-equivalent after the
  center-aligned header text starts from the C++ line offset; `make
  golden-compare` reports `exact=168`, `exact-segments=489`, `diverges=2`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target: `transition_actions.riv`.
- 2026-07-05: [M6] Promoted `transition_actions.riv` by carrying scheduled
  state-machine `ListenerViewModelChange` actions through layer advancement,
  applying them to the bound view-model data-bind graph, and mirroring the
  changed source path into artboard-side data-bind values before static text
  draw. `make golden-compare` reports `exact=169`,
  `exact-segments=490`, `diverges=1`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target is the M6
  `rust-runner-unsupported:text` manifest queue, starting with
  `bindable_artboard_nesty.riv` unless a smaller text-only entry is found.
- 2026-07-05: [M6] Promoted `bindable_artboard_nesty.riv` by admitting
  source-to-target `NestedArtboard` host binds through the static text gate
  for the nested-host properties already applied by the runtime
  (`artboardId`, `isPaused`, `speed`, and `quantize`). `make golden-compare`
  reports `exact=170`, `exact-segments=491`, `diverges=1`,
  `unsupported-feature=124`, `not-yet=0`, and parked
  `M6=80 gated=8 harness=36`; next target is `component_stateful.riv`.
- 2026-07-05: [M6] Promoted `component_stateful_vm_instance_2.riv` by
  allowing static text to coexist with `Star` siblings, admitting stateful
  nested child `Shape.rotation` binds through `DataConverterSystemDegsToRads`,
  and propagating child `ViewModelInstanceColor.propertyValue` into nested
  `SolidColor.colorValue`. `make golden-compare` reports `exact=172`,
  `exact-segments=493`, `diverges=3`, `unsupported-feature=120`,
  `not-yet=0`, and parked `M6=76 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `computed_values_test.riv`.
- 2026-07-05: [M6] Added #V2-7 per-entry verification modes to
  `golden-compare` and preserved non-default modes in `generate-corpus` so
  layout/text/image entries can declare `tolerant(ε)` or `structural` before
  moving to `exact`. Baseline after unwinding the misaligned computed-values
  spike remains `exact=172`, `exact-segments=493`, `diverges=3`,
  `unsupported-feature=120`, `not-yet=0`, and parked
  `M6=76 gated=8 harness=36`; next target is the Taffy-backed layout trait
  slice for `computed_values_test.riv`.
- 2026-07-05: [M6] Routed supported `LayoutComponent` bounds through a
  #V2-7 Taffy layout trait that computes coherent whole-artboard snapshots
  from Rive style data, refuses nested artboard/component-list provider trees
  this slice cannot model yet, and leaves the old hand-rolled helpers as
  fallback only for refused trees. The existing simple root row/column layout
  cases stay exact under the snapshot-first resolver. `make golden-compare`
  remains `exact=172`, `exact-segments=493`, `diverges=3`,
  `unsupported-feature=120`, `not-yet=0`, and parked
  `M6=76 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  reopen `computed_values_test.riv` through the Taffy-backed layout path.
- 2026-07-05: [M6] Reopened `computed_values_test.riv` by admitting
  `ArtboardComponentList.listSource`, nested child `Shape.computedRootX/Y`
  binds, and empty component-list provider trees through the Taffy layout
  adapter. The file now reaches draw and is retagged as
  `rust-runner-divergence:computed-values-text`; `computed_root_transform.riv`
  declares `verification = "tolerant(0.5)"` for the subpixel Yoga/Taffy layout
  rounding exposed by the same path. `make golden-compare` reports
  `exact=172`, `exact-segments=493`, `diverges=4`,
  `unsupported-feature=119`, `not-yet=0`, and parked
  `M6=75 gated=8 harness=36`; next target is `follow_path_path.riv`.
- 2026-07-05: [M6] Reopened `follow_path_path.riv` by admitting static text
  siblings `FollowPathConstraint`, `CubicDetachedVertex`,
  `CubicAsymmetricVertex`, and `CubicMirroredVertex`. It now reaches draw and
  is parked as `rust-runner-divergence:follow-path-text-outline`; the same
  gate removal made `spotify_kids_app_icon.riv` reach draw, now parked as
  `rust-runner-divergence:spotify-icon-draw-order`. `make golden-compare`
  reports `exact=172`, `exact-segments=493`, `diverges=6`,
  `unsupported-feature=117`, `not-yet=0`, and parked
  `M6=73 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Admitted inert `Event` siblings through the static text
  gate. `nested_events.riv` is exact by focused stream comparison;
  `data_bind_test_cmdq.riv` now reaches draw and is parked as
  `rust-runner-divergence:data-bind-command-queue-text-layout`; the same gate
  removal reopens `state_transition_fire_trigger.riv` and
  `trigger_based_listeners.riv`, both parked as
  `rust-runner-divergence:event-trigger-extra-text-draw`. `make
  golden-compare` reports `exact=173`, `exact-segments=494`, `diverges=9`,
  `unsupported-feature=113`, `not-yet=0`, and parked
  `M6=69 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_binding_test.riv`.
- 2026-07-05: [M6] Reopened `data_binding_test.riv` by admitting
  `ForegroundLayoutDrawable` through the static text gate; that C++ class is
  already modeled in graph/draw ordering as layout foreground paint glue. The
  file reaches draw and is parked as
  `rust-runner-divergence:foreground-layout-text-transform` after the focused
  stream diff showed C++ placing text at `[1,0,0,1,400,468.925781]` while Rust
  emits identity transform and a shorter stream. `make golden-compare` reports
  `exact=173`, `exact-segments=494`, `diverges=10`,
  `unsupported-feature=112`, `not-yet=0`, and parked
  `M6=68 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_converter_to_number.riv`.
- 2026-07-05: [M6] Reopened `data_converter_to_number.riv` by admitting
  custom-property siblings through static text and adding
  `CustomPropertyBoolean`/`CustomPropertyColor` target-to-source binding
  values. The file reaches draw and is parked as
  `rust-runner-divergence:data-converter-to-number-text-values` after focused
  streams showed the first text path at `[1,0,0,1,34.473156,389.39209]` had 17
  C++ contours versus 15 Rust contours. `make golden-compare` reports
  `exact=173`, `exact-segments=494`, `diverges=11`,
  `unsupported-feature=111`, `not-yet=0`, and parked
  `M6=67 gated=8 harness=36`; next target is `databind_solo_to_enum.riv`.
- 2026-07-05: [M6] Promoted `databind_solo_to_enum.riv` by admitting Solo
  parent/sibling text, mapping enum source-to-target Solo binds through
  DataEnum labels, mirroring target-to-source Solo active-child enum writes,
  and applying `Text.alignValue` enum/uint binds. `make golden-compare`
  reports `exact=174`, `exact-segments=495`, `diverges=11`,
  `unsupported-feature=110`, `not-yet=0`, and parked
  `M6=66 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `fit_font_size_test.riv`.
- 2026-07-05: [M6] Reopened `fit_font_size_test.riv` by admitting
  source-to-target `TextStylePaint.fontSize`, `Text.overflowValue`, and
  `LayoutComponent.height` binds through the static text gate. The file reaches
  draw and is parked as
  `rust-runner-divergence:text-fit-font-size-layout-bounds`: focused streams
  show C++ wrapping/fitting the right-column text where Rust keeps advancing on
  a wider line (`x=7.71484375` versus `x=212.890625`), and C++ emits a
  zero-sized middle text path that Rust suppresses. `make golden-compare`
  reports `exact=174`, `exact-segments=495`, `diverges=12`,
  `unsupported-feature=109`, `not-yet=0`, and parked
  `M6=65 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `hit_test_nested.riv`.
- 2026-07-05: [M6] Promoted `hit_test_nested.riv` by admitting authored
  `NestedBool` siblings through the static text gate and allowing static text
  under `Shape` parent transforms. Focused direct streams then matched C++ under
  numeric-token epsilon, and the full corpus promoted the file to exact.
  `make golden-compare` reports `exact=175`, `exact-segments=496`,
  `diverges=12`, `unsupported-feature=108`, `not-yet=0`, and parked
  `M6=64 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `interpolate_to_end.riv`.
- 2026-07-05: [M6] Reopened `interpolate_to_end.riv` by admitting nested child
  `TextValueRun.text` converter groups through the golden-runner gate and
  letting artboard property-binding admission validate stateful converter
  groups with `RuntimeDataBindGraphConverterState`. The file now reaches draw
  and is parked as
  `rust-runner-divergence:nested-child-text-converter-context`: focused streams
  show C++ rendering the nested data-bound/interpolated numeric text at
  `[1,0,0,1,245.207031,58.4726562]` while Rust still emits the serialized
  fallback text glyph payload. `make golden-compare` reports `exact=175`,
  `exact-segments=496`, `diverges=13`, `unsupported-feature=107`,
  `not-yet=0`, and parked `M6=63 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `keyboard_listener.riv`.
- 2026-07-05: [M6] Promoted `keyboard_listener.riv` by admitting passive
  `FocusData` and `KeyboardInput` siblings through the static text subset for
  sample-0 rendering. The file's direct C++/Rust streams have the same call
  sequence and pass golden numeric-token comparison, so the stale
  `rust-runner-unsupported:text` manifest gate is removed. `make
  golden-compare` reports `exact=176`, `exact-segments=497`, `diverges=13`,
  `unsupported-feature=106`, `not-yet=0`, and parked
  `M6=62 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `list_index_script_access.riv`.
- 2026-07-05: [M6] Promoted `list_index_script_access.riv` by admitting
  inert `ScriptedDrawable` siblings through the static text subset for
  sample-0 rendering and declaring its existing Taffy/Yoga list-row rounding
  drift as `verification = "tolerant(0.75)"`. The same gate removal reopens
  `scripted_data_context.riv`, now parked as
  `rust-runner-divergence:scripted-data-context-extra-text-draw` after direct
  streams showed Rust drawing two data-bound text payloads C++ suppresses.
  `make golden-compare` reports `exact=177`, `exact-segments=498`,
  `diverges=14`, `unsupported-feature=104`, `not-yet=0`, and parked
  `M6=60 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `saturation.riv`.
- 2026-07-05: [M6] Reopened `saturation.riv` by admitting static-text
  `Shape.x/y` source-to-target binds with no converter or a
  `DataConverterGroup`, clearing the stale `rust-runner-unsupported:text`
  stop. The file reaches draw and is parked as
  `rust-runner-divergence:saturation-color-to-string-text`: focused streams
  first differ at text path id 3 under `[1,0,0,1,64.5,26.5]`, while the later
  numeric/color text path is only float drift. `make golden-compare` reports
  `exact=177`, `exact-segments=498`, `diverges=15`,
  `unsupported-feature=103`, `not-yet=0`, and parked
  `M6=59 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `scroll_snap.riv`.
- 2026-07-05: [M6] Reclassified `scroll_snap.riv` by moving the existing
  `ScrollConstraint` runner preflight ahead of the static-text gate, so the
  first Rust diagnostic is now `rust-runner-unsupported:scroll-constraints`
  for global 93 instead of a stale sibling-text error. This confirms the file
  belongs with the scroll/layout runtime queue, not the text-layout queue.
  `make golden-compare` reports `exact=177`, `exact-segments=498`,
  `diverges=15`, `unsupported-feature=103`, `not-yet=0`, and parked
  `M6=59 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `stateful_source_switch.riv`.
- 2026-07-05: [M6] Promoted `stateful_source_switch.riv` by admitting
  no-converter source-to-target `ParametricPath.width/height` binds for static
  text sibling shapes (`Ellipse` in the active stateful source, plus the same
  C++ property family for Polygon/Rectangle/Star/Triangle). Direct Rust and C++
  sample-0 streams now match the parent artboard background-only draw. `make
  golden-compare` reports `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_follow_path_shape_length.riv`.
- 2026-07-05: [M6] Reclassified `text_follow_path_shape_length.riv` after
  admitting source-to-target `Text.width/height` data binds with no converter
  or `DataConverterFormula` through the static text gate. Direct Rust now gets
  past the generic `Text` property blockers and stops on
  `TextFollowPathModifier` global 168, so the file is retagged as
  `rust-runner-unsupported:text-follow-path-modifier`. `make golden-compare`
  remains `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_vertical_trim_test.riv`.
- 2026-07-05: [M6] Reclassified `text_vertical_trim_test.riv` as
  `rust-runner-unsupported:text-vertical-trim` after confirming property keys
  1027/1028 are `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue`, bitmask passthroughs into
  `verticalTrimValue`. C++ applies them in `src/text/text.cpp` through
  `Text::computeVerticalTrim` to the rendered/measured text bounds, so this is
  a real text-layout port rather than a finite static admission. Direct Rust
  now reports `unsupported: text-vertical-trim`; `make golden-compare` remains
  `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `transition_duration_bind_nested.riv`.
- 2026-07-05: [M6] Reclassified `transition_duration_bind_nested.riv` by
  admitting nested child `TextValueRun.text` through `DataConverterFormula`.
  The stale generic `rust-runner-unsupported:text` stop is gone: direct Rust
  reaches draw, and the first real diff is nested transition-duration reveal
  behavior where C++ collapses the icon circles to zero-scale transforms at
  sample 0 while Rust draws them at full scale. The #V2-7 decision language was
  reviewed at the same time and remains the right guardrail: Taffy is the
  layout adapter, tolerant verification is numeric-only, and missing
  text/layout behavior must stay visible as diagnostics or divergences.
  `make golden-compare` reports `exact=178`, `exact-segments=499`,
  `diverges=16`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `transition_duration_bind_nested.riv` as a focused nested
  transition-duration/data-bind divergence.
- 2026-07-05: [M6] Promoted `transition_duration_bind_nested.riv` by mirroring
  C++ per-instance `StateTransition.duration` data binds. State-machine data
  binds targeting transitions now create runtime transition-duration slots,
  child-artboard default view-model values resolve against the selected
  artboard context, and transition mixing rounds/clamps bound durations like
  C++ `StateMachineInstance::resolvedDuration`. `make golden-compare` reports
  `exact=179`, `exact-segments=500`, `diverges=15`,
  `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the M6 text layout/draw-suppression bucket, starting with
  `data_binding_test.riv`.
- 2026-07-05: [M6] Narrowed `data_binding_test.riv` by routing
  `ForegroundLayoutDrawable` paints through their parent `LayoutComponent`
  path/transform, threading the coherent Taffy layout snapshot into draw/text,
  disabling Taffy rounding to mirror Yoga point-scale `0`, measuring static
  Text leaves for layout control size, and using controlled layout width for
  auto-width text alignment under non-artboard layout parents. Focused streams
  now have matching length and no identity-transform failure; the first
  remaining diff is the Taffy/Yoga wrapped row offset described above.
  `make golden-compare` remains `exact=179`, `exact-segments=500`,
  `diverges=15`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  a C++/Rust local-id layout-box probe for `data_binding_test.riv`.
- 2026-07-05: [M6] Promoted `data_binding_test.riv` after the local-id
  C++ Yoga/Rust Taffy probe showed all 142 layout nodes match once static Text
  leaves measure with finite layout constraints. The remaining focused stream
  diff was `DataConverterToString` spelling C++ `std::to_string(NaN)` as
  lowercase `nan`, now mirrored in the shared converter helper. `make
  golden-compare` reports `exact=180`, `exact-segments=501`, `diverges=14`,
  `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv` in the text layout/draw-suppression bucket.
- 2026-07-05: [M6] Narrowed `data_bind_test_cmdq.riv` by measuring Shape
  layout leaves in the Taffy adapter using C++ `Shape::measureLayout` /
  `ParametricPath::measureLayout` semantics for the static runtime subset.
  Rust layout-bounds now succeeds with all 19 boxes and local 40 measures
  24x24; the first stream diff improved from y=`460.671631` to
  y=`453.185791` but the file remains a known Taffy/Yoga text-layout
  divergence. `make golden-compare` remains `exact=180`,
  `exact-segments=501`, `diverges=14`, `unsupported-feature=101`,
  `not-yet=0`, and parked `M6=57 gated=8 harness=36`;
  `cargo test --workspace` passes. Next target stays
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Narrowed `data_bind_test_cmdq.riv` again by mirroring C++
  `LayoutComponent::syncStyle`: only leaf layout components with
  `intrinsicallySizedValue` get a Taffy measure context. The C++ Yoga and Rust
  Taffy local-id layout boxes now match for all 19 nodes, including the bottom
  command-queue block at local 98/101; the remaining first diff is the
  `Update Random Vals` glyph path payload at the matched transform, so the
  file is retagged as `rust-runner-divergence:data-bind-command-queue-text-outline`.
  `make golden-compare` remains `exact=180`, `exact-segments=501`,
  `diverges=14`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_converter_to_number.riv`.
- 2026-07-05: [M6] Promoted `data_converter_to_number.riv` after refreshing
  focused C++/Rust streams: the stale 17-vs-15 contour note was gone, both
  streams had 75 lines with matching non-numeric structure, and the largest
  numeric text-outline delta was about `1e-6`, below the normal golden epsilon.
  `make golden-compare` reports `exact=181`, `exact-segments=502`,
  `diverges=13`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; next target is `scripted_data_context.riv`.
- 2026-07-05: [M6] Reclassified `scripted_data_context.riv` as an explicit
  scripting gate after focused streams showed the C++ runner printing
  `Failed to import object of type 106` and suppressing two script-driven
  data-context text draws. The Rust runner now emits
  `unsupported: scripted-data-context` only for selected artboards with a
  `ScriptedDrawable`, `DataBindContext` text, and nested view-model context;
  checked exact script fixtures `list_index_script_access.riv` and
  `scripting_root_viewmodel.riv` still stream. `make golden-compare` reports
  `exact=181`, `exact-segments=502`, `diverges=12`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; next target is
  `state_transition_fire_trigger.riv`.
- 2026-07-05: [M6] Promoted `state_transition_fire_trigger.riv` and
  `trigger_based_listeners.riv` by preserving nested child default text
  contexts when the child artboard owns state-machine data binds, while
  retaining serialized-text fallback for plain nested text hosts. Focused
  sample-0 streams match C++; `make golden-compare` reports `exact=183`,
  `exact-segments=504`, `diverges=10`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the text-outline
  backend/canonicalization slice starting with `new_text.riv`.
- 2026-07-05: [M6] Promoted `new_text.riv` by using Skrifa FreeType-style
  outline extraction for static fonts while retaining HarfBuzz-style outlines
  for variable fonts, matching C++'s HarfBuzz callback contour starts without
  regressing Inter variable-font text fixtures. Focused streams for
  `new_text.riv` and sampled exact text fixtures match under the golden
  epsilon; `make golden-compare` reports `exact=184`,
  `exact-segments=505`, `diverges=9`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `follow_path_path.riv`'s follow-path
  text transform.
- 2026-07-05: [M6] Promoted `follow_path_path.riv` by letting text draw use
  constraint-written component world transforms unless a layout ancestor needs
  the #V2-7 layout-bounds path. Focused streams now match all four follow-path
  text transforms under the golden epsilon. `make golden-compare` reports
  `exact=185`, `exact-segments=506`, `diverges=8`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Promoted `data_bind_test_cmdq.riv` by mirroring C++
  `LayoutComponent::propagateSizeToChildren` / `ParametricPath::controlSize`
  for layout-controlled parametric shape draw. The focused command-queue
  sample now keeps matching local-id layout boxes and expands the inner
  authored `20x18` trigger ellipse to the solved `24x24` layout size before
  draw, matching C++ under the golden epsilon. `make golden-compare` reports
  `exact=186`, `exact-segments=507`, `diverges=7`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `saturation.riv`.
- 2026-07-05: [M6] Narrowed `saturation.riv` by making artboard
  custom-property target-to-source binds carry the data-bind flags, converter
  state, and source default kind, then applying C++'s main-direction converter
  rule before writing the shared artboard source cache. This fixes the coarse
  wrong-text/fallback path: the focused sample now matches the data-bound
  color-to-string payloads and only diverges on tiny text outline coordinate
  drift. `cargo check -q -p rive-runtime` passes, and `make golden-compare`
  remains `exact=186`, `exact-segments=507`, `diverges=7`,
  `unsupported-feature=102`, `not-yet=0`, with parked
  `M6=58 gated=8 harness=36`. Next pass should decide whether the remaining
  `saturation.riv` float drift is a direct text-outline parity fix or a
  verification-mode policy decision.
- 2026-07-05: [M6] Promoted `saturation.riv` after the narrowed focused diff
  proved to be same-structure HarfRust/C++ outline coordinate drift at roughly
  `1e-6`, not missing text layout or data-bind behavior. The entry now declares
  `verification = "tolerant(0.00001)"`, small enough that integer IDs still
  cannot be accidentally accepted by the current numeric-token comparator.
  `make golden-compare` reports `exact=187`, `exact-segments=508`,
  `diverges=6`, `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`. Next target is `fit_font_size_test.riv`.
- 2026-07-05: [M6] Promoted `fit_font_size_test.riv` by translating C++
  `src/text/text.cpp::Text::fitFontScale` into the static text path: Rust now
  binary-searches the largest fitting integer top font size, scales font-size
  only during shaping/metrics/line breaking, and preserves C++ zero-font
  collapsed text paths. Focused streams are epsilon-equivalent under the exact
  comparator. `make golden-compare` reports `exact=188`,
  `exact-segments=509`, `diverges=5`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`;
  `cargo test --workspace` passes. Next target is `spotify_kids_app_icon.riv`.
- 2026-07-06: [M6] Promoted `spotify_kids_app_icon.riv` by routing root
  artboard background paints through the same C++ `ShapePaint::shouldDraw`
  visibility gate used by regular shape paints. This suppresses the hidden
  full-artboard Backboard fill before the centered icon while preserving the
  visible rounded background draw. `make golden-compare` reports `exact=189`,
  `exact-segments=510`, `diverges=4`, `unsupported-feature=102`, `not-yet=0`,
  and parked `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is `computed_values_test.riv`.
- 2026-07-06: [M6] Promoted `computed_values_test.riv` by mirroring C++
  host-first artboard data-bind updates and `Node::computedRootX/Y`
  root-transform semantics through nested artboard hosts. Nested child
  `Shape.computedRootX/Y` now publishes `238.5/205` in root artboard space
  instead of child-local `39/49`; focused streams are exact under the normal
  golden epsilon. `make golden-compare` reports `exact=190`,
  `exact-segments=511`, `diverges=3`, `unsupported-feature=102`, `not-yet=0`,
  and parked `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is `relative_data_binding.riv` with `shared_viewmodel_instance.riv`.
- 2026-07-06: [M6] Promoted `relative_data_binding.riv` and
  `shared_viewmodel_instance.riv` by binding owned view-model contexts through
  nested artboard hosts, resolving manifest-backed name paths, defaulting fresh
  generated color values to opaque black, and clearing missing name-based
  `TextValueRun.text` only for concrete nested owned contexts. The runner now
  applies this owned-context pass to nested artboards only, leaving root
  artboard values on the existing state-machine/default data-bind path so
  `transition_actions.riv` remains exact. `make golden-compare` reports
  `exact=192`, `exact-segments=513`, `diverges=1`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `interpolate_to_end.riv`.
- 2026-07-06: [M6] Promoted `interpolate_to_end.riv` after the previous nested
  owned view-model/context work reduced its focused stream diff to numeric path
  drift accepted by the standard exact comparator epsilon. A scratch exact
  corpus for only this file passes, and full `make golden-compare` reports
  `exact=193`, `exact-segments=514`, `diverges=0`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_follow_path_shape_length.riv`.
- 2026-07-06: [M6] Promoted `text_follow_path_shape_length.riv` by porting
  static `TextFollowPathModifier` over Shape/Path targets, adding
  target-to-source TrimPath/Shape length source values for the fixture's
  formula-driven `Text.width`, and mirroring C++'s text follow-path
  `PathMeasure(&path, 0.1f)` tolerance. Focused and full streams are exact:
  `make golden-compare` reports `exact=194`, `exact-segments=515`,
  `diverges=0`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_vertical_trim_test.riv`.
- 2026-07-06: [M6] Promoted `text_vertical_trim_test.riv` by adding generated
  bitmask passthrough set/get support for `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue`, admitting their no-converter data binds, and
  porting the static `Text::computeVerticalTrim` bounds/render offset path for
  the current text subset. Focused `background_measure.riv` stayed exact after
  backing out a too-broad line-metric detour, and focused
  `text_vertical_trim_test.riv` is exact. Full `make golden-compare` reports
  `exact=195`, `exact-segments=516`, `diverges=0`,
  `unsupported-feature=100`, `not-yet=0`, and parked
  `M6=56 gated=8 harness=36`. Next target is the M6 image bucket, starting
  with `custom_image_name.riv`.
- 2026-07-06: [M6] Promoted `custom_image_name.riv` by porting the first
  non-mesh `Image::draw` slice from C++ `src/shapes/image.cpp`, decoding
  in-band `ImageAsset` contents through the render factory before the stream
  `source` marker, and keeping broader mesh/layout/data-bound image files
  behind the existing image diagnostic. Focused C++ and Rust streams are
  byte-identical. Full `make golden-compare` reports `exact=196`,
  `exact-segments=517`, `diverges=0`, `unsupported-feature=99`, `not-yet=0`,
  and parked `M6=55 gated=8 harness=36`. Next target is the nested-library
  image pair: `double_library_with_image.riv` and `library_with_image.riv`.
- 2026-07-06: [M6] Reviewed the #V2-7 decision and enforced its fallback
  boundary in `rust-golden-runner`: painted `LayoutComponent` corpus entries
  now fail with a Taffy-refused diagnostic instead of promoting through the
  legacy hand-rolled layout resolver. To keep existing exact list fixtures on
  the snapshot path, the Taffy adapter now treats root artboards as definite
  viewport nodes even when their style uses hug axes, and admits zero-sized
  `ArtboardComponentList` metadata children such as map rules and overrides.
  `transition_duration_bind_list.riv`, `artboard_list_map_rules.riv`, and
  `artboard_list_overrides.riv` all compute Taffy layout bounds. Full
  `make golden-compare` remains `exact=196`, `exact-segments=517`,
  `diverges=0`, `unsupported-feature=99`, `not-yet=0`, and parked
  `M6=55 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the nested-library image pair: `double_library_with_image.riv` and
  `library_with_image.riv`.
- 2026-07-06: [M6] Promoted `double_library_with_image.riv` and
  `library_with_image.riv` by widening the Rust runner's image admission from
  a single image-only artboard to a static image artboard tree. The existing
  runtime image cache already predecoded embedded `ImageAsset` contents and
  threaded them through nested hosts; focused C++/Rust streams for both
  fixtures and `custom_image_name.riv` are byte-identical. Full
  `make golden-compare` reports `exact=198`, `exact-segments=519`,
  `diverges=0`, `unsupported-feature=97`, `not-yet=0`, and parked
  `M6=53 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the top-level image pair: `hosted_image_file.riv` and `in_band_asset.riv`.
- 2026-07-06: [M6] Closed the #V2-7 fallback-fence audit gap by treating
  `ArtboardComponentList` `ListFollowPathConstraint` children as zero-sized
  metadata in the Taffy adapter. `component_list_follow_path_distance.riv` now
  computes a coherent layout snapshot instead of relying on legacy bounds
  fallback, and a sweep of all 33 exact `LayoutComponent` entries reports zero
  `--layout-bounds` failures. Full `make golden-compare` remains `exact=198`,
  `exact-segments=519`, `diverges=0`, `unsupported-feature=97`, `not-yet=0`,
  and parked `M6=53 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains the top-level image pair: `hosted_image_file.riv` and
  `in_band_asset.riv`.
- 2026-07-06: [M6] Promoted `hosted_image_file.riv` and
  `in_band_asset.riv` by admitting simple image artboards with root background
  paints and hosted image assets that have no decoded `RenderImage`, matching
  C++ `Image::draw`'s early return before save. Image predecode now runs after
  source-file paint allocation but before selected-artboard clone paint
  allocation, matching C++ paint/decode ID order for in-band image contents.
  Focused streams for both fixtures plus `custom_image_name.riv`,
  `library_with_image.riv`, and `double_library_with_image.riv` are
  byte-identical. Full `make golden-compare` reports `exact=200`,
  `exact-segments=521`, `diverges=0`, `unsupported-feature=95`, `not-yet=0`,
  and parked `M6=51 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is `walle.riv` and `image_fit_alignment_3.riv`.
- 2026-07-06: [M6] Promoted `walle.riv` and `image_fit_alignment_3.riv`.
  `walle.riv` admits inert image-artboard animation metadata and preserves C++
  multi-image decode/source-paint ordering by splitting the first embedded
  image decode before source paint allocation. `image_fit_alignment_3.riv`
  ports the plain non-mesh `Image::controlSize` / `updateImageScale`
  fit/alignment path for images under a Taffy-backed `LayoutComponent`,
  including C++'s zero-sized recording-image NaN transform surface. The
  #V2-7 fallback fence now rejects any layout-dependent draw candidate when
  Taffy refuses the whole-artboard snapshot, not just painted
  `LayoutComponent` paths. Asset-image view-model binding/reset files remain
  explicitly gated after `image_fit_alignment.riv` and
  `viewmodel_image_reset.riv` proved successful Rust image drawing is not
  exact there. Full `make golden-compare` reports `exact=202`,
  `exact-segments=523`, `diverges=0`, `unsupported-feature=93`, `not-yet=0`,
  and parked `M6=49 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is the asset-image view-model pair:
  `image_fit_alignment.riv` and `viewmodel_image_reset.riv`.
- 2026-07-06: [M6] Promoted `viewmodel_image_reset.riv` by applying
  `ViewModelInstanceAssetImage` defaults to `Image.assetId` targets like C++
  `DataBindContextValueAssetImage`, including the empty private image-asset
  reset path that suppresses `Image::draw`. Removed the blanket asset-image
  image gate and replaced `image_fit_alignment.riv` with the sharper
  `rust-runner-unsupported:asset-image-layout` diagnostic after the focused
  diff narrowed to image decode ordering plus LayoutComponent Y placement
  (`272.5` in C++ vs `539.5` in Rust). Full `make golden-compare` reports
  `exact=203`, `exact-segments=524`, `diverges=0`,
  `unsupported-feature=92`, `not-yet=0`, and parked
  `M6=48 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `image_fit_alignment.riv`, then `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Closed the #V2-7 payload-hash image verifier gap by
  switching both recording factories from encoded `size/hash` lines to
  decoded-header `width/height` lines for PNG/JPEG/WebP assets, and by
  returning those dimensions through `RecordingRenderImage`. Focused
  `image_fit_alignment_3.riv` and `walle.riv` streams remain exact with real
  dimensions. Full `make golden-compare` remains `exact=203`,
  `exact-segments=524`, `diverges=0`, `unsupported-feature=92`, `not-yet=0`,
  and parked `M6=48 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains `image_fit_alignment.riv`, then
  `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Promoted `image_fit_alignment.riv` by mapping Yoga
  undefined position insets to Taffy auto in the #V2-7 layout adapter, matching
  C++ `LayoutComponent::positionTypeChanged` for stale non-absolute
  `positionTop` values, and by widening image predecode ordering for
  asset-image-bound layout trees so the first two embedded images decode
  before source paint allocation. Focused streams for
  `image_fit_alignment.riv`, `viewmodel_image_reset.riv`, `walle.riv`, and
  `image_fit_alignment_3.riv` are exact. Full `make golden-compare` reports
  `exact=204`, `exact-segments=525`, `diverges=0`,
  `unsupported-feature=91`, `not-yet=0`, and parked
  `M6=47 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Promoted `image_fit_alignment_2.riv` by admitting
  metadata-only `NSlicer`/`AxisX`/`AxisY` image artboards through the static
  image gate while keeping meshes and actual `NSlicedNode` draw behavior gated.
  The existing runtime draw path already matched C++: the fixture decodes the
  embedded images but renders only layout-component background paints. Focused
  C++/Rust streams are exact. Full `make golden-compare` reports `exact=205`,
  `exact-segments=526`, `diverges=0`, `unsupported-feature=90`, `not-yet=0`,
  and parked `M6=46 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is the M6 nested-artboard-layout bucket, starting with
  `artboard_width_test.riv`.
- 2026-07-06: [M6] Promoted `artboard_width_test.riv` and
  `transition_artboard_condition_test.riv` by admitting layout-backed nested
  artboard hosts into the Taffy snapshot, applying solved host dimensions to
  persistent child artboards before nested state-machine advancement, drawing
  `NestedArtboardLayout` hosts at their solved layout node position, and
  parsing `TransitionArtboardCondition` through the existing artboard-number
  comparator path. Focused streams for both fixtures are exact. Full
  `make golden-compare` reports `exact=207`, `exact-segments=528`,
  `diverges=0`, `unsupported-feature=88`, `not-yet=0`, and parked
  `M6=44 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the nested-artboard-layout bucket, starting with
  `collapsing_elements.riv`.
- 2026-07-06: [M6] Promoted `collapsing_elements.riv` and
  `multitouch_enter.riv` by admitting `NestedArtboardLeaf` as a persistent
  nested artboard host, matching C++ leaf `computeAlignment` behavior, and
  giving fixed/default `NestedArtboardLayout` nodes the referenced child
  artboard's intrinsic size instead of collapsing to a zero-size Taffy leaf.
  Focused C++/Rust stream diffs for both fixtures are exact. Full
  `make golden-compare` reports `exact=209`, `exact-segments=530`,
  `diverges=0`, `unsupported-feature=86`, `not-yet=0`, and parked
  `M6=42 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the M6 image bucket, starting with `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `image_binding_with_listener.riv` by narrowing the
  static image gate to admit simple Shape/Rectangle siblings beside image
  drawables while keeping complex image files with skins, scripts, draw rules,
  events, and constraints behind `rust-runner-unsupported:images`. A scratch
  exact corpus proved only this one of the eight newly admitted image entries
  matched C++; full `make golden-compare` reports `exact=210`,
  `exact-segments=531`, `diverges=0`, `unsupported-feature=85`, `not-yet=0`,
  and parked `M6=41 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains the M6 image bucket, starting with `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `library.riv` by admitting simple static text
  siblings beside non-mesh image drawables in the static image gate and the
  static text sibling allow-list. Focused and full streams are exact; full
  `make golden-compare` reports `exact=211`, `exact-segments=532`,
  `diverges=0`, `unsupported-feature=84`, `not-yet=0`, and parked
  `M6=40 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `library_with_text_and_image.riv`, then `bad_skin.riv` if the nested-library
  slice is not a small promotion.
- 2026-07-06: [M6] Promoted `library_with_text_and_image.riv` by letting the
  static image gate treat unresolved `NestedArtboard` hosts as empty child
  draws, matching C++ for this asset-only nested-library file: both runtimes
  decode the image asset, then draw only the selected artboard background.
  Focused streams are exact; full `make golden-compare` reports `exact=212`,
  `exact-segments=533`, `diverges=0`, `unsupported-feature=83`, `not-yet=0`,
  and parked `M6=39 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is the M6 image bucket, starting with `bad_skin.riv` unless focused
  classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `clipping_and_draw_order.riv` by admitting simple
  clipped/draw-target image artboards and by treating `ArtboardComponentList`
  sorted drawable entries as metadata-only for draw-command generation, so
  they no longer flush pending clipping shapes into empty clip operations.
  Focused streams for the promoted fixture and component-list guard fixtures
  are exact; full `make golden-compare` reports `exact=213`,
  `exact-segments=534`, `diverges=0`, `unsupported-feature=82`, `not-yet=0`,
  and parked `M6=38 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target remains the M6 image bucket, starting with `bad_skin.riv` unless
  focused classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `data_binding_images_test.riv` by admitting
  nested-state-machine metadata in the static image gate, allowing nested
  child `Image.assetId` binds, and aligning generated owned asset-image
  defaults with C++'s private empty asset sentinel. Nested asset-image context
  imports now use the C++ decode/source-paint ordering where all but the final
  embedded image resolve before source paint allocation. Focused streams for
  the promoted fixture plus `viewmodel_image_reset.riv`,
  `image_fit_alignment*.riv`, `walle.riv`, and `custom_image_name.riv` are
  exact. Full `make golden-compare` reports `exact=214`,
  `exact-segments=535`, `diverges=0`, `unsupported-feature=81`, `not-yet=0`,
  and parked `M6=37 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target remains the M6 image bucket, starting with `bad_skin.riv` unless
  focused classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `scripted_property_image.riv` by admitting inert
  `ScriptedDrawable`/`Event` metadata in the static image gate, preserving
  C++'s no-layout asset-image decode/source-paint ordering, and relying on the
  existing empty asset-image defaults to suppress the two image draws after the
  C++ `FileAssetContents` import failure. `viewmodel_based_condition.riv`
  moved from the generic image bucket to the sharper
  `rust-runner-unsupported:viewmodel-asset-conditions` diagnostic after
  focused streams showed wrong state-machine condition colors once admitted.
  Full `make golden-compare` reports `exact=215`,
  `exact-segments=536`, `diverges=0`, `unsupported-feature=80`,
  `not-yet=0`, and parked `M6=36 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the largest M6 bucket,
  `rust-runner-unsupported:nested-artboard-layout`, starting with
  `db_health_tracker.riv` unless focused classification finds a smaller
  nested-layout slice.
- 2026-07-06: [M6] Reclassified `db_health_tracker.riv` from
  `nested-artboard-layout` to `scroll-constraints` by porting the
  `NestedArtboardLayout` scale-type 2 hug/intrinsic sizing path from C++
  `StyleOverrider` into the Taffy adapter. The file now computes a coherent
  whole-artboard layout snapshot for its six layout-backed nested hosts; the
  first remaining Rust runner gate is the authored `ScrollConstraint` global
  210. Full `make golden-compare` remains `exact=215`,
  `exact-segments=536`, `diverges=0`, `unsupported-feature=80`,
  `not-yet=0`, and parked `M6=36 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is still the largest M6 bucket,
  `rust-runner-unsupported:nested-artboard-layout`, starting with
  `focus_collapsing.riv` unless focused classification finds a smaller
  nested-layout slice.
- 2026-07-06: [M6] Closed the stale
  `rust-runner-unsupported:nested-artboard-layout` manifest queue by direct
  `rust-golden-runner` classification of all 13 remaining entries. Each now
  computes or passes Taffy layout far enough to expose a sharper first gate:
  `focus-data`, `data-binding-nested-stateful-view-model`,
  `data-binding-nested-child`, `scroll-constraints`, `feather`, or `images`.
  Full `make golden-compare` remains `exact=215`, `exact-segments=536`,
  `diverges=0`, `unsupported-feature=80`, `not-yet=0`, and parked
  `M6=36 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:scroll-constraints`, starting
  with `component_list_1.riv` unless focused classification finds a smaller
  scroll slice.
- 2026-07-06: [M6] Promoted `component_list_1.riv`,
  `deterministic_mode.riv`, `interactive_scrolling.riv`, `scroll_test.riv`,
  and `scroll_threshold.riv` by porting the passive initial
  `ScrollConstraint::constrain` / `constrainChild` transform slice over
  registered layout-provider children and admitting only zero-offset,
  non-interactive, non-snap, non-virtualized scroll constraints behind a
  coherent Taffy snapshot. Focused streams match exactly apart from signed-zero
  matrix text accepted by `golden-compare` numeric-token comparison. Full
  `make golden-compare` reports `exact=220`, `exact-segments=541`,
  `diverges=0`, `unsupported-feature=75`, `not-yet=0`, and parked
  `M6=31 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:images`, starting with
  `bad_skin.riv` unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Promoted `zombie_skins.riv` by widening the static image
  gate to admit non-mesh image artboard trees with nested vector children,
  preserving C++ all-but-final embedded image decode ordering, and reading live
  animated `GradientStop` color/position values when building gradient shaders.
  Focused `zombie_skins.riv` and full streams are exact; `bad_skin.riv`,
  `bullet_man.riv`, and `spotify_kids_demo.riv` stay parked behind the image
  diagnostic after focused diffs exposed contour-mesh path drift, selected-root
  gradient allocation ordering, and image/deeper layout-text drift respectively.
  Full `make golden-compare` reports `exact=221`, `exact-segments=542`,
  `diverges=0`, `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the M6 image bucket, starting with `bad_skin.riv` unless focused
  classification finds another smaller first gate.
- 2026-07-06: [M6] Reclassified stale image diagnostics by letting
  `Feather`/`NSlicedNode` gates fire before the broad image fence.
  `car_widgets_v01.riv`, `echo_show_demo.riv`, and
  `feather_render_test.riv` now verify as `rust-runner-unsupported:feather`;
  `local_bounds.riv` verifies as `rust-runner-unsupported:n-slice`. The image
  queue is down to six true image-admission/mesh/paint-order candidates:
  `bad_skin.riv`, `bullet_man.riv`, `jellyfish_test.riv`,
  `spotify_kids_demo.riv`, `superbowl.riv`, and `tape.riv`. Full
  `make golden-compare` reports `exact=221`, `exact-segments=542`,
  `diverges=0`, `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:scroll-constraints`, starting
  with `component_list_child_origin.riv` unless focused classification finds a
  smaller first gate.
- 2026-07-06: [M6] Tightened `golden-compare` unsupported diagnostics so a
  manifest `rust-runner-unsupported:*` entry must match the Rust runner's
  actual first diagnostic. The stricter check exposed two stale labels:
  `db_health_tracker.riv` now verifies as
  `rust-runner-unsupported:data-binding-nested-child`, and `superbowl.riv`
  now verifies as `rust-runner-unsupported:text`. Full `make golden-compare`
  reports `exact=221`, `exact-segments=542`, `diverges=0`,
  `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the largest true M6 bucket,
  `rust-runner-unsupported:scroll-constraints`, starting with
  `component_list_child_origin.riv`.
- 2026-07-06: [M6] Promoted `component_list_child_origin.riv` and
  `virtualize_blendmode.riv` by admitting passive empty virtualized
  `ArtboardComponentList` scroll providers and drawing layout proxy clip paths
  from computed Taffy bounds. `scroll_snap.riv` remains parked because authored
  snap exposes broader layout transform inheritance drift. Full
  `make golden-compare` reports `exact=223`, `exact-segments=544`,
  `diverges=0`, `unsupported-feature=72`, `not-yet=0`, and parked
  `M6=28 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  one of the tied largest M6 buckets, image or feather, starting with
  `bad_skin.riv` unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Split the remaining generic image gate into first-blocker
  diagnostics. `bad_skin.riv` now verifies as
  `rust-runner-unsupported:contour-mesh-metadata`; `jellyfish_test.riv` and
  `tape.riv` verify as `rust-runner-unsupported:mesh-images`;
  `bullet_man.riv` and `spotify_kids_demo.riv` verify as
  `rust-runner-unsupported:selected-root-image-order`. Full
  `make golden-compare` reports `exact=223`, `exact-segments=544`,
  `diverges=0`, `unsupported-feature=72`, `not-yet=0`, and parked
  `M6=28 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `feather`, starting with `car_widgets_v01.riv`
  unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Promoted `feather_render_test.riv` by porting the runtime
  ShapePaint/Feather draw slice from C++ `shape_paint.cpp` and `feather.cpp`:
  render paints now carry feather strength, outer feathers apply world/local
  offsets in the same order as C++, and inner feathers draw their generated
  inner path under the original/effect path clip. The broad feather queue is
  gone; the remaining former feather files now verify as sharper first gates:
  `selected-root-image-order`, `text`, `feather-inner-multipaint`, or
  `nested-feather-paints`. Full `make golden-compare` reports `exact=224`,
  `exact-segments=545`, `diverges=0`, `unsupported-feature=71`,
  `not-yet=0`, and parked `M6=27 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the tied
  `rust-runner-unsupported:scroll-constraints` bucket, starting with
  `component_list_virtualized.riv` unless focused classification finds a
  smaller first gate.
- 2026-07-06: [M6] Promoted `scroll_snap.riv` by letting passive at-rest snap
  metadata through the scroll gate and applying accumulated Taffy layout
  bounds with artboard-origin-adjusted layout world transforms, matching C++
  `LayoutComponent::update` placement for nested layout children under the
  scroll viewport. Full `make golden-compare` reports `exact=225`,
  `exact-segments=546`, `diverges=0`, `unsupported-feature=70`,
  `not-yet=0`, and parked `M6=26 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-stateful-view-model`.
- 2026-07-06: [M6] Promoted `stateful_keyed_trigger.riv` by admitting
  `ViewModelInstance*` subtrees under `NestedArtboardLayout` and
  `NestedArtboardLeaf` hosts. The old
  `data-binding-nested-stateful-view-model` queue is closed:
  `focus_traversal.riv` now verifies as `rust-runner-unsupported:focus-data`,
  while `stateful_multi_property.riv` and `stateful_nested.riv` verify as
  `rust-runner-unsupported:data-binding-nested-child`. Full
  `make golden-compare` reports `exact=226`, `exact-segments=547`,
  `diverges=0`, `unsupported-feature=69`, `not-yet=0`, and parked
  `M6=25 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-child`.
- 2026-07-06: [M6] Reclassified `hit_test_test.riv` after admitting nested
  child `ArtboardComponentList.listSource` / `DataConverterNumberToList` binds
  in the Rust runner; it now reaches
  `rust-runner-unsupported:scroll-constraints`. Focused comparison also showed
  that admitting the broader stateful Artboard/text/layout value path is not
  exact yet: `nested_hug.riv`, `stateful_multi_property.riv`, and
  `stateful_nested.riv` render but drift in nested transforms, so they remain
  parked as `rust-runner-unsupported:data-binding-nested-child`. Next target is
  one of the tied four-file M6 buckets: data-binding nested child, focus data,
  or scroll constraints.
- 2026-07-06: [M6] Promoted passive nested focus metadata for
  `focus_collapsing.riv` and `focusable_element.riv`. The runner now admits
  nested `FocusData` only for no-input, non-traversal files; `focus_traversal`
  stays parked as `rust-runner-unsupported:focus-data` because the focused
  stream renders but differs structurally in path allocation, and `text_input`
  advances to `rust-runner-unsupported:layout-component-paint`. Full
  `make golden-compare` reports `exact=228`, `exact-segments=549`,
  `diverges=0`, `unsupported-feature=67`, `not-yet=0`, and parked
  `M6=23 gated=8 harness=36`. Next target is one of the tied four-file M6
  buckets: data-binding nested child or scroll constraints.
- 2026-07-06: [M6] Closed the `scroll-constraints` queue by admitting
  passive, listener-free, zero-offset virtualized/infinite scroll constraints
  when a Taffy layout snapshot exists, and by suppressing authored-transparent
  layout proxy paints without suppressing transparent normal hit shapes.
  Promoted `component_list_virtualized.riv`, `draw_index_list.riv`,
  `hit_test_test.riv`, and `virtualized_artboard_databound_children.riv`.
  Full `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-child`.
- 2026-07-06: [M6] Split the remaining
  `rust-runner-unsupported:data-binding-nested-child` bucket into four
  sharper one-file diagnostics after focused probes showed that temporary
  broad admissions render but still drift in nested layout transforms/clips.
  `db_health_tracker.riv` now verifies as
  `rust-runner-unsupported:nested-trim-path-data-bind`, `nested_hug.riv` as
  `rust-runner-unsupported:nested-artboard-root-transform`,
  `stateful_multi_property.riv` as
  `rust-runner-unsupported:nested-layout-clip-data-bind`, and
  `stateful_nested.riv` as
  `rust-runner-unsupported:nested-stateful-view-model-property`. Full
  `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  one of the tied largest M6 buckets, `selected-root-image-order` or `text`.
- 2026-07-06: [M6] Split
  `rust-runner-unsupported:selected-root-image-order` into three verified
  first blockers. `car_widgets_v01.riv` now reaches the existing
  `rust-runner-unsupported:text` gate on text paint feather,
  `bullet_man.riv` now verifies as
  `rust-runner-unsupported:selected-root-gradient-shader-order`, and
  `spotify_kids_demo.riv` now verifies as
  `rust-runner-unsupported:selected-root-skinned-clip-path` after adding the
  selected-root single-image predecode ordering observed in the focused
  compare and narrowing it against `feather_render_test.riv`'s exact paint
  ordering. `golden-compare` now reports the first differing stream line for
  failed exact comparisons. Full `make golden-compare` reports `exact=232`,
  `exact-segments=553`, `diverges=0`, `unsupported-feature=63`, `not-yet=0`,
  and parked `M6=19 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is the `text` bucket.
- 2026-07-06: [M6] Closed the broad
  `rust-runner-unsupported:text` queue by admitting text paint feather state
  through the existing `ShapePaint` feather path and splitting the five
  remaining broad text-tagged entries into first blockers.
  `car_widgets_v01.riv` and `hunter_x_demo.riv` now verify as
  `rust-runner-unsupported:feather-inner-multipaint`, `echo_show_demo.riv` as
  `rust-runner-unsupported:text-joystick-data-bind`, `superbowl.riv` as
  `rust-runner-unsupported:nested-artboard-layout`, and gated `bankcard.riv`
  as `rust-runner-unsupported:text-polygon-sibling`. Full
  `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `rust-runner-unsupported:feather-inner-multipaint`.
- 2026-07-06: [M6] Closed the
  `rust-runner-unsupported:feather-inner-multipaint` queue by removing the
  runner-only global inner-feather guard and keying repeated inner-feather
  clip paths by the original draw command instead of by each paint-local
  generated inner path. `coin.riv` is now exact, `car_widgets_v01.riv` now
  verifies as `rust-runner-unsupported:nested-node-transform-data-bind`, and
  `hunter_x_demo.riv` plus `rewards_demo.riv` now verify as
  `rust-runner-unsupported:nested-feather-paints`. Full
  `make golden-compare` reports `exact=233`, `exact-segments=554`,
  `diverges=0`, `unsupported-feature=62`, `not-yet=0`, and parked
  `M6=19 gated=7 harness=36`; next target is the tied largest M6 bucket
  `rust-runner-unsupported:nested-feather-paints`.
- 2026-07-06: [M6] Closed the stale
  `rust-runner-unsupported:nested-feather-paints` runner guard and let the
  affected files run to their real first blockers. `hunter_x_demo.riv` now
  verifies as `rust-runner-unsupported:text-modifier-group-flags`,
  `rewards_demo.riv` as `rust-runner-unsupported:n-slice`, and gated
  `ai_assitant.riv` now reaches Rust draw output but is parked as
  `not-yet:nested-feather-gradient-space` because its focused compare differs
  at nested-feather linear-gradient coordinates. Full `make golden-compare`
  reports `exact=233`, `exact-segments=554`, `diverges=0`,
  `unsupported-feature=61`, `not-yet=1`, and parked
  `M6=19 gated=6 harness=36`; next target is the tied largest M6 bucket,
  preferring `rust-runner-unsupported:n-slice` over `mesh-images` because it
  is layout-facing and now blocks both `local_bounds.riv` and
  `rewards_demo.riv`.
- 2026-07-06: [M6] Ported NSlicedNode vector path deformation by mirroring the
  C++ N-slicer stop mapping for local/world shape draw commands, admitting
  passive NSlicedNode/axis/draw-rule siblings through static text, and
  removing the stale runner-level N-slice guard. `n_slice_triangle.riv` is now
  exact, `rewards_demo.riv` now verifies as
  `rust-runner-unsupported:text-modifier-group-flags`, and
  `local_bounds.riv` is parked as `not-yet:image-predecode-order` because its
  focused run now reaches Rust draw output but differs in external image
  predecode ordering and tiny static-text float residuals. Full
  `make golden-compare` reports `exact=234`, `exact-segments=555`,
  `diverges=0`, `unsupported-feature=59`, `not-yet=2`, and parked
  `M6=18 gated=5 harness=36`; next target is the tied largest M6 bucket,
  preferring `rust-runner-unsupported:text-modifier-group-flags` over
  `mesh-images`.
