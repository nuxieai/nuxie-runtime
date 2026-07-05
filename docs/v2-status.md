# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 457 across 136 exact files
- Parked breakdown: M5=0 by manifest query; `make golden-compare` reports M6=116 gated=7 harness=36
- Current milestone: **M6 — Layout + Text Verified Per Declared Corpus Modes (#V2-7)**

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

1. Inspect `modifier_to_run.riv`: the generic text gate is gone and the first
   blocker is now `TextModifierRange.unitsValue = 2`, which means the next
   decision is whether to port word/line range maps plus run-scoped/multi-run
   text now, or keep that broader text-layout slice parked with a sharper
   diagnostic.
2. Keep `new_text.riv` parked for now: it has five
   `Text` objects, multiple runs/styles, gradients/strokes, clipping, and text
   keyframes, so it is not the next narrow static tracer.
3. M5 is closed for the current corpus: `grep -B6 'milestone = "M5"'
   corpus.toml` is empty. Do not reopen data-binding work unless a newly added
   corpus entry exposes a pre-text/pre-layout data-binding diagnostic.
4. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M6 unless a related refactor needs a cheap draw-regression check.

## Known Divergences

- None currently tracked for M1/M2; remaining non-exact files are parked with
  later-milestone diagnostics or unsupported-feature gates.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until a future external data-binding corpus file requires it.
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
  transform slices once layout-backed list instances populate them, and
  parametric
  Star/Polygon local path sampling for follow-path targets, plus static plain
  `NestedArtboard` host draw with child root opacity inheritance, default
  nested simple-animation/state-machine hosts backed by persistent child
  artboard instances, stateful child `ViewModelInstance` subtree admission
  under plain nested hosts, nested child unbound SolidColor data-bind defaults,
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
  `Rectangle.width/height` 20/21 binds,
  authored-transparent Backboard/background draw suppression,
  custom-property trigger keyed-callback target-to-source binding,
  custom-property enum target-to-source binding, live data-bound nested host
  `isPaused` mutation, plus no-input recursive nested `ListenerAlignTarget`
  fixtures where the action is unexercised.
  Custom handle-source world-space math, data-bound nested host controls beyond
  generated defaults (external/live pause/speed/quantize mutation), remaining
  nested child data-bind targets beyond the current number/color/default bind
  set, and broader bound stateful child view-model propagation remain
  corpus-driven follow-up work if a future file exposes them. Focus data,
  input-driven recursive
  `ListenerAlignTarget` and nested pointer/listener hit propagation beyond
  reported `Event` listeners, `NestedArtboardLayout` / `NestedArtboardLeaf`,
  and layout-backed or virtualized component-list instancing remain M6 or
  later diagnostics.
  Golden runner sample lists now advance by sorted absolute-time deltas and
  reuse render paths across samples; no images, richer text layout/editing,
  live data-bound nested host controls/artboard swaps, nested layout/leaf,
  scroll constraints, or layout-backed/virtualized component-list instancing.
  Harness-level scripted input replay dispatches
  pointerDown/pointerMove/pointerUp/pointerExit markers into direct rectangle
  state-machine listeners with listener input actions, direct rectangle
  enter/exit hover state, direct rectangle click synthesis, and listener-owned
  default view-model trigger target-to-source writes. Full C++ ListenerGroup
  drag/opaque behavior and input-driven nested align-target/list/text/layout
  targets are still not supported.
- Static text modifier support currently covers translation-only
  `TextModifierGroup` over character-unit `TextModifierRange` coverage with an
  optional `CubicInterpolatorComponent`. Word/line units, range maps, runId
  targeting, multi-run/multi-style text, shape/follow-path/rotation/scale/
  origin/opacity modifiers, and text input/editing remain M6 text diagnostics.
- `TransformConstraint` currently covers Text constraint bounds for the
  supported static one-run Text subset plus the default empty
  `TransformComponent::constraintBounds()` path. LayoutComponent bounds remain
  parked behind M6 layout diagnostics.
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
  `grep -B6 'milestone = "M5"' corpus.toml`.
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
- 2026-07-04: M4 is corpus-closed after a direct `rust-golden-runner` sweep:
  no `milestone = "M4"` entries remain. Former M4 parked files now carry their
  first verified later diagnostic: M5 data-binding nested child/host or
  custom-property trigger paths, and M6 text/images/nested-artboard-layout/
  focus/layout-component-paint paths. This opens M5 without hiding the later
  text/layout/list work.

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
