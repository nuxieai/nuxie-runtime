# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 492 across 171 exact files
- Current compare: `make golden-compare` reports diverges=3, unsupported-feature=121, not-yet=0
- Parked breakdown: M5=0 by manifest query; `make golden-compare` reports M6=77 gated=8 harness=36
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

1. The M6 text unsupported queue is the active line now that the compact
   layout/list/data-bound-text divergence queues are closed. Query it with
   `grep -A1 -B6 'milestone = "M6"' corpus.toml | rg -B7 -A1 'rust-runner-unsupported:text'`.
   Start with `component_stateful_vm_instance_2.riv` unless a smaller
   text-only entry is identified first; run the direct C++/Rust stream pair to
   verify the first unsupported gate before porting.
2. Keep `new_text.riv` parked as a known M6 divergence until a dedicated text
   outline backend/canonicalization slice: gradient sibling admission now
   reaches draw, but Rust/Skrifa and C++ HarfBuzz emit a glyph contour with a
   different segment start/order, so exact stream comparison fails on path
   verbs/points.
3. Keep follow-path text files parked behind `TextFollowPathModifier` and
   unsupported `Text` property data-bind targets;
   `text_follow_path_shape_length.riv` currently fails first on data binding
   target `Text` global 73 before it reaches follow-path drawing.
4. Keep `text_vertical_trim_test.riv` parked behind text data-binding target
   support; after the layout-paint slice it now fails on unsupported `Text`
   property data bind target global 41 before reaching richer vertical-trim
   behavior.
5. M5 is closed for the current corpus: `grep -B6 'milestone = "M5"'
   corpus.toml` is empty. Do not reopen data-binding work unless a newly added
   corpus entry exposes a pre-text/pre-layout data-binding diagnostic.
6. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M6 unless a related refactor needs a cheap draw-regression check.

## Known Divergences

- `new_text.riv`: after admitting static text sibling `LinearGradient` /
  `GradientStop` and gradient text paints, Rust reaches draw but differs from
  C++ on glyph outline contour ordering. C++ uses HarfBuzz draw callbacks
  (`src/text/font_hb.cpp`) and `TextStylePaint::addPathClockwise`; Rust uses
  Skrifa outlines. The first stream diff is path verb/point ordering, not a
  paint/gradient mismatch.
- `relative_data_binding.riv`: after nested `TextValueRun.text` support, Rust
  reaches draw but activates a nested text/relative-position branch that C++
  suppresses at sample 0. First stream diff: a rectangle transform has x=96.5
  in Rust vs x=0 in C++, followed by extra text draw calls.
- `shared_viewmodel_instance.riv`: same newly-rendering shared-view-model text
  family; Rust emits extra nested text draw calls before the rectangle stream
  that C++ produces at sample 0.

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
  `Rectangle.width/height` 20/21 binds, nested child `TextValueRun.text`
  string binds backed by stateful child view-model values,
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
- Static text support currently covers one style or matching-metric
  multi-style text, static authored-line-break and no-break multi-run text,
  fixed-size ellipsis across multiple authored lines with bottom/middle
  vertical alignment, variation-aware no-break multi-run style outlines,
  auto-width origin offsets, and translation/rotation/opacity
  `TextModifierGroup` over C++-style
  `TextModifierRange` character, character-excluding-space, word, and static
  line range maps with runId targeting and optional cubic range
  interpolation, including C++-ordered opacity buckets, plus solid fill/stroke
  `TextStylePaint` drawing with DashPath stroke effects, and
  source-to-target `TextValueRun.text` / `SolidColor.colorValue` data binds
  around static text. Shape/follow-path/scale/origin modifiers,
  gradient/feather/other text effects, richer layout, broader `Text` property
  data binds, and text input/editing remain M6 text diagnostics.
- `TransformConstraint` currently covers Text constraint bounds for the
  supported static Text subset plus the default empty
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
  explicit `rust-runner-unsupported:feather` renderer diagnostic. `bankcard.riv`
  is also now gated on feather after clearing its `layout-component-paint`
  blocker.
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
