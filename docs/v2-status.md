# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact segments (file × sample): 243 across 70 exact files
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

1. The two-sample exact widening queue is exhausted. Continue the fourth-sample
   M2 sweep over exact entries with samples `0`, `0.25`, and `0.5`, starting
   after `quantize_test.riv` (next candidate: `rapid_pointer_events.riv`),
   and stop on the first real divergence to localize runtime code.
2. Continue M2 real object model work by modularizing the remaining runtime
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
3. Add handle-source world-space math and nested-remap dependent advancement
   to the joystick path when a corpus diff reaches those cases.
4. Remaining exact entries pinned to sample `0` are static M1 holdovers:
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

- 2026-07-03: [M2] Widened `viewmodel_runtime_file.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive view-model
  property import plus animation/state-machine playback exact while leaving
  external data-binding mutation in M5 scope. Exact segments are now 204
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=204`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Exhausted the exact two-sample widening queue and widened
  `animation_reset_cases.riv` from samples `0`, `0.25`, and `0.5` to samples
  `0`, `0.25`, `0.5`, and `0.75`, starting the fourth-sample M2 sweep with
  blend/reset state-machine playback still exact. Exact segments are now 205
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=205`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `bindable_artboard_child.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  passive bindable artboard-child state-machine render path exact while
  leaving interactive listener/data-binding behavior in later M3/M5 scope.
  Exact segments are now 206 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=206`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `blend_test.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping direct and 1D
  blend-state playback exact across the wider sample set. Exact segments are
  now 207 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=207`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `circle_clips.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping animated
  clipping-shape playback exact across the wider sample set. Exact segments
  are now 208 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=208`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `clear_viewmodel_list.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  passive scripted/list/view-model render path exact while leaving list
  mutation, data binding, and scripting behavior in later M4/M5/M6 scope.
  Exact segments are now 209 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=209`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `click_event.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  listener/event state-machine playback exact while leaving scripted pointer
  event dispatch in M3 scope. Exact segments are now 210 across 70 exact
  files; `make golden-compare` reports `exact=70`,
  `exact-segments=210`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `clip_tests.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping clipping playback
  exact across the wider sample set. Exact segments are now 211 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=211`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_based_conditions.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  component-based transition conditions exact across the wider sample set.
  Exact segments are now 212 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=212`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `component_list_2.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the passive
  component-list/view-model render path exact while leaving list mutation in
  later M4/M5 scope. Exact segments are now 213 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=213`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_list_grouped.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  grouped component-list/view-model render path exact while leaving list
  mutation in later M4/M5 scope. Exact segments are now 214 across 70 exact
  files; `make golden-compare` reports `exact=70`,
  `exact-segments=214`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `component_list_hit_order.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  passive component-list hit-order render path exact while leaving scripted
  input dispatch in M3 scope. Exact segments are now 215 across 70 exact
  files; `make golden-compare` reports `exact=70`,
  `exact-segments=215`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `cubic_value_test.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping CubicValue
  interpolator playback exact across the wider sample set. Exact segments are
  now 216 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=216`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `data_bind_solo.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  data-bind/Solo playback exact while leaving external mutation in M5 scope.
  Exact segments are now 217 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=217`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `data_binding_test_2.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive data-binding playback exact while leaving external mutation in M5
  scope. Exact segments are now 218 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=218`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `dependency_test.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the base
  vector playback stream exact across the wider sample set. Exact segments
  are now 219 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=219`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `draw_rule_cycle.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping draw-rule
  ordering exact across the wider animated sample set. Exact segments are now
  220 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=220`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `event_on_listener.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  listener fire-event playback exact while leaving scripted input dispatch in
  M3 scope. Exact segments are now 221 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=221`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `event_trigger_event.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive event-trigger playback exact while leaving scripted input dispatch
  in M3 scope. Exact segments are now 222 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=222`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `events_on_states.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  state event playback exact while leaving scripted input dispatch in M3
  scope. Exact segments are now 223 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=223`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `fill_trim_path.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping fill trim
  path playback exact across the wider sample set. Exact segments are now 224
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=224`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `fix_rectangle.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping animated
  rectangle playback exact across the wider sample set. Exact segments are now
  225 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=225`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `hit_test_solos.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive Solo
  hit-test playback exact while leaving scripted pointer dispatch in M3
  scope. Exact segments are now 226 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=226`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `joel_signed.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping skinned vector
  playback exact across the wider sample set. Exact segments are now 227
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=227`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `joystick_flag_test.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  joystick flag playback exact across the wider sample set. Exact segments
  are now 228 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=228`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joystick_nested_remap.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  nested joystick remap playback exact across the wider sample set. Exact
  segments are now 229 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=229`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `juice.riv` from samples `0`, `0.25`, and `0.5`
  to samples `0`, `0.25`, `0.5`, and `0.75`, keeping its larger vector
  playback stream exact across the wider sample set. Exact segments are now
  230 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=230`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `keyboard_event_to_script.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  passive scripted/focus state-machine stream exact without opening scripted
  keyboard input behavior. Exact segments are now 231 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=231`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `library_data_enum_test.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the
  passive data-enum/view-model state-machine stream exact across the wider
  sample set. Exact segments are now 232 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=232`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `light_switch.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive listener
  bool-change state-machine playback exact across the wider sample set. Exact
  segments are now 233 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=233`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `list_to_path.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive ListPath
  and view-model-list playback exact across the wider sample set. Exact
  segments are now 234 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=234`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `lock_icon_demo.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping skinned
  vector, TrimPath, and passive bool-listener playback exact across the wider
  sample set. Exact segments are now 235 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=235`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `long_name.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping simple
  rectangle animation playback exact across the wider sample set. Exact
  segments are now 236 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=236`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `looping_timeline_events.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive looping callback-event timeline playback exact across the wider
  sample set. Exact segments are now 237 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=237`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `multiple_state_machines.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive multi-state-machine playback exact across the wider sample set.
  Exact segments are now 238 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=238`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `nested_solo.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping Solo
  state-machine playback exact across the wider sample set. Exact segments
  are now 239 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=239`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `off_road_car.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the larger
  skinned vector, clipping, and gradient playback stream exact across the
  wider sample set. Exact segments are now 240 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=240`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `oneshotblend.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping one-shot 1D
  blend-state playback exact across the wider sample set. Exact segments are
  now 241 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=241`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `opaque_hit_test.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  nested-bool and draw-rule playback exact across the wider sample set.
  Exact segments are now 242 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=242`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `quantize_test.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping quantized
  keyframe playback exact across the wider sample set. Exact segments are now
  243 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=243`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
