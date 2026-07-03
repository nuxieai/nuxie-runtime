# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Corpus files `exact`: 59
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

1. Port C++ `Joystick::apply`/artboard advance ordering enough for
   `joystick_flag_test` sample `0`; current first diff is joystick-driven
   transform positions after default state-machine startup.
2. Continue M2 real object model work by replacing the generic cloned
   `RuntimeObject` arena tracer with generated concrete object storage plus
   schema-generated setter side effects beyond the hand-ported Solo uint/id
   path.
3. `clip_tests` is raw-exact at sample `0`, but its manifest includes sample
   `0.25`; keep it parked until M2 non-zero sample support or an explicit
   sample-scope split.
4. Keep `fill_trim_path` and `trim_path_linear` parked for M2 keyframe and
   non-zero sample support.

## Known Divergences

- None currently tracked for M1; remaining non-exact files are parked with
  later-milestone diagnostics or unsupported-feature gates.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until M5 external data-binding corpus files require it.
- Rust golden draw path currently supports sample `0`, artboard
  clip/background, selected-artboard origins, solid fills/strokes, and
  `ClippingShape` clip paths, skinned `PointsPath` deformation, plus empty and
  multi-contour TrimPath effects, DashPath stroke effects, and linear/radial
  gradient shader creation, default state-machine frame-0 application for
  color/bool/uint/string keyframes, and Solo active-child refresh;
  no non-zero samples, images, text, nested artboards, constraints, or scripted
  input.
- `fill_trim_path.riv` is parked for M2 even at sample `0`: C++ applies
  keyframes to TrimPath `offset`/`end` before drawing, so imported static
  values cannot match without animation application.
- `juice.riv` and `rocket.riv` are parked for M2 at sample `0`: after gradient
  shader creation matched C++, their first diffs traced to frame-0 keyed
  transform/geometry application from default animations/state machines, while
  Rust still draws imported static values.
- `scripted_color.riv` is parked for M5 at sample `0`: C++ binds the default
  `ViewModelPropertyColor` through `DataBindContext` to a `SolidColor`, while
  static Rust still draws the imported color.
- `databind_external_artboard_main.riv`, `databind_external_artboard_child.riv`,
  and `viewmodel_image_reset.riv` are parked for nested-artboard, text, and
  image support respectively; their sample-0 first diffs are covered by the
  existing Rust runner diagnostics.
- `component_list_1.riv` is parked for constraint support, and
  `custom_property_enum.riv` is parked for M5 custom-property enum data
  binding; both have verified Rust runner diagnostics.
- `scripted_transition_condition.riv` is parked for scripting support;
  `data_converter_interpolator_reset.riv` is parked for M5 color data binding;
  `stateful_keyed_trigger.riv`, `unbound_stateful_component.riv`, and
  `scripting_root_viewmodel.riv` are parked for nested-artboard support.
- `bidirectional_precedence.riv` and `collapsable_data_binding.riv` are parked
  for M5 data-binding transform/color application; `zero_width_space_line_break.riv`
  is parked for text support.
- `viewmodel_from_context.riv`, `viewmodel_list_trigger.riv`, and
  `transition_index_condition.riv` are parked for M6 layout component paint
  drawing; `complex_ik_dependency.riv` is parked for constraint support, and
  `stateful_source_switch.riv` is parked for nested-artboard support.
- `state_transition_fire_trigger.riv`, `stateful_artboard_swap.riv`,
  `stateful_multi_property.riv`, and `stateful_nested.riv` are parked for
  nested-artboard support.
- `tape.riv` is parked for image support, `target_event.riv` and
  `transition_artboard_condition_test.riv` are parked for nested-artboard
  support, `time_based_interpolation.riv` is parked for M5 data-binding color,
  `transition_duration_bind_list.riv` is parked for M6 layout component paint
  drawing, and `two_bone_ik.riv` is parked for constraint support.
- `trigger_based_listeners.riv` and `transition_self_comparator_test.riv` are
  parked for nested-artboard support, `virtualized_artboard_databound_children.riv`
  is parked for M6 layout component paint drawing, `walle.riv` and
  `viewmodel_based_condition.riv` are parked for image support, and
  `word_joiner_test.riv` is parked for text support; `zombie_skins.riv` is
  parked for nested-artboard support.
- `artboard_list_map_rules.riv`, `artboard_list_overrides.riv`, and
  `component_list_child_origin.riv` are parked for M6 layout component paint
  drawing; `artboard_width_test.riv`, `transition_duration_bind_nested.riv`,
  and `trigger_fires_single_change.riv` are parked for nested-artboard support.
- `advance_blend_mode.riv` remains parked for M2 non-zero sample support (its
  sample-0 Rust diagnostic currently reaches nested artboards); `animated_clipping.riv`
  and `background_measure.riv` are parked for text support, and
  `component_list_virtualized.riv` is parked for M6 layout component paint drawing.
- `bullet_man.riv`, `car_widgets_v01.riv`, and `collapsing_elements.riv` are
  parked for nested-artboard support; `collapse_data_binds.riv` is parked for
  text support.
- `component_stateful.riv`, `component_stateful_vm_instance.riv`,
  `component_stateful_vm_instance_2.riv`, and `computed_values_test.riv` are parked
  for nested-artboard support; `computed_root_transform.riv` is parked for M6
  layout component paint drawing; `cubic_value_test.riv` is parked for M2
  keyframe/interpolator application after its sample-0 transform diff.
- `custom_property_trigger.riv`, `data_binding_images_test.riv`, and
  `data_binding_test_3.riv` are parked for nested-artboard support;
  `data_bind_test_cmdq.riv`, `data_binding_artboards_source_test.riv`, and
  `data_binding_test.riv` are parked for text support.
- `data_binding_test_triggers.riv`, `databind_artboard.riv`,
  `db_health_tracker.riv`, and `death_knight.riv` are parked for nested-artboard
  support; `data_converter_to_number.riv` and `databind_solo_to_enum.riv` are
  parked for text support.
- `double_line.riv` and `ellipsis.riv` are parked for text support;
  `drag_event.riv`, `echo_show_demo.riv`, and `entry.riv` are parked for
  nested-artboard support.
- `feather_render_test.riv` is parked for image support;
  `fit_font_size_test.riv` is parked for text support; `focus_collapsing.riv`
  and `focus_traversal.riv` are parked for nested-artboard support.
- `focusable_element.riv`, `hit_test_nested.riv`, and `hit_test_test.riv` are
  parked for nested-artboard support; `format_number_with_commas.riv` and
  `hello_world.riv` are parked for text support; `formula_random.riv` is
  parked for M5 data-binding transform/formula application.
- `hittest_collapsed_layouts.riv` and `hosted_font_file.riv` are parked for
  text support; `hosted_image_file.riv`, `image_binding_with_listener.riv`,
  and `image_fit_alignment.riv` are parked for image support;
  `hunter_x_demo.riv` is parked for nested-artboard support.
- `image_fit_alignment_2.riv`, `image_fit_alignment_3.riv`, and
  `in_band_asset.riv` are parked for image support; `interactive_scrolling.riv`
  and `interpolate_to_end.riv` are parked for nested-artboard support;
  `interpolation_zero_duration.riv` is parked for M5 zero-duration
  data-binding interpolator transform application.
- `jellyfish_test.riv` is parked for image support; `joel_v3.riv` is parked
  for text support; `joystick_flag_test.riv` is parked for M2 joystick
  application/default state-machine behavior.
- `keyboard_listener.riv` is parked for text support; `library.riv` is parked
  for image support; `library_view_model_test.riv` and
  `library_vmtest_1_host.riv` are parked for nested-artboard support.
- `library_with_text_and_image.riv` is parked for nested-artboard support;
  `list_index_script_access.riv` is parked for text support; `list_items.riv`
  and `list_to_length_test.riv` are parked for M6 layout component paint
  drawing.
- `listener_action_inputs.riv` is parked for scripted transition condition
  support; `listener_view_model.riv` is parked for text support;
  `local_bounds.riv` is parked for image support; and
  `magic_alley_db_reduced_export.riv` is parked for nested-artboard support.
- `modifier_test.riv` and `modifier_to_run.riv` are parked for text support;
  `multitouch.riv`, `multitouch_enter.riv`, and
  `nested_artboard_quantize_and_speed.riv` are parked for nested-artboard
  support; `n_slice_triangle.riv` is parked for n-slice geometry/deformation
  support.
- `nested_event_test.riv`, `nested_events.riv`, `nested_hug.riv`, and
  `nested_needs_advance.riv` are parked for nested-artboard support;
  `new_text.riv` is parked for text support.
- `number_to_list_nested_children.riv` is parked for M6 layout component paint
  drawing; `pause_nested_artboard.riv` is parked for nested-artboard support.
- `oneshotblend.riv` is parked for M2 1D blend/default state-machine
  application at sample `0`; `opaque_hit_test.riv` is parked for M2
  frame-0 state-machine/keyframe color application; `pointer_events.riv`
  is raw-exact at sample `0`, but its manifest includes sample `0.1`, so it
  remains parked until M2 non-zero sample support or an explicit sample-scope
  split.
- `pointer_events_nested_artboards_in_solos.riv`, `pointer_exit.riv`, and
  `recursive_data_bind.riv` are parked for nested-artboard support;
  `rebind_with_nested_viewmodel.riv` is parked for text support.
- `quantize_test.riv` is parked for M2 quantized `LinearAnimation`
  application: C++ applies the frame-0 keyed transform through the default
  scene, moving the ellipse to `x=0`, while static Rust still draws the
  imported transform.
- `relative_data_binding.riv` and `rewards_demo.riv` are parked for
  nested-artboard support; `replace_vm_instance.riv` is parked for text
  support; `reset_phase.riv` is parked for M6 layout component paint drawing;
  and `reuse_path_in_effect.riv` is parked for scripted path-effect support.
- `runtime_nested_inputs.riv`, `runtime_nested_text_runs.riv`,
  `scripted_data_context.riv`, and `scripted_listener_context.riv` are parked
  for nested-artboard support; `saturation.riv` is parked for text support.
- `scripted_property_image.riv` is parked for image support;
  `scroll_snap.riv` and `spotify_kids_app_icon.riv` are parked for text
  support; and `scroll_test.riv`, `scroll_threshold.riv`, and
  `shared_viewmodel_instance.riv` are parked for nested-artboard support.
- `spotify_kids_demo.riv` is parked for image support; `superbowl.riv` and
  `text_input.riv` are parked for nested-artboard support; and
  `test_modifier_run.riv` and `text_follow_path_shape_length.riv` are parked
  for text support.
- `text_listener_simpler.riv`, `text_opacity_modifier.riv`,
  `text_stroke_test.riv`, `text_vertical_trim_test.riv`,
  `transition_actions.riv`, and `vertical_align_ellipsis.riv` are parked for
  text support.
- `advance_blend_mode.riv`, `ai_assitant.riv`, `align_target.riv`,
  `bankcard.riv`, and `bindable_artboard_nesty.riv` are parked for
  nested-artboard support; `bad_skin.riv` is parked for image support.
- Corpus entries tagged `cpp-runner-crash` are unsupported until the C++
  golden runner/importer can survive the FileAssetContents, scripting, and
  data-viz crash paths it currently aborts on.
- `solar-system.riv` is unsupported because Rust import rejects
  `blendModeValue = 5` on Shape object 13.

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

## Log

- 2026-07-02: V2 plan, `/goal` command, and this status file created. No V2 code yet.
- 2026-07-02: [M0] Added `tools/golden-runner` RecordingRenderer/Factory scaffold, smoke binary, and `make golden-runner`; `make golden-compare` still not present.
- 2026-07-02: [M0] Golden runner CLI now imports real `.riv` files, selects
  artboards/state machines, advances sampled timelines, replays pointer input
  scripts, and emits recording streams; `make golden-compare` still not
  present.
- 2026-07-02: [M0] Added `crates/rive-render-api` with C++-mirroring
  renderer/factory/resource traits and a recording serializer whose smoke
  output matches the C++ golden runner stream; `make golden-compare` still not
  present.
- 2026-07-02: [M0] Added `corpus.toml` with 8 seeded C++ unit-test assets,
  `tools/golden-compare`, and `make golden-compare`; exact count is now 0.
- 2026-07-02: [M0] Added `tools/rust-golden-runner` for a narrow static
  solid-shape path and marked `dependency_test` exact; exact count is now 1.
- 2026-07-02: [M0] Expanded `corpus.toml` to all 295
  `tests/unit_tests/assets`; `make golden-compare` passes with exact=1,
  unsupported-feature=37, not-yet=257.
- 2026-07-02: [M0] Added GitHub Actions CI for `make golden-compare` and
  `cargo test --workspace`; M0 is complete and the active milestone moves to
  M1.
- 2026-07-02: [M1] Moved the narrow static solid-shape renderer path from
  `rust-golden-runner` into `rive-runtime`; exact remains 1 and
  `make golden-compare` passes.
- 2026-07-02: [M1] Marked `artboardclipping.riv` exact by porting artboard
  origin transforms and selected-artboard paint allocation; exact count is now
  2.
- 2026-07-02: [M1] Marked `shapetest.riv` exact through the runtime renderer
  path; exact count is now 3.
- 2026-07-02: [M1] Triaged `trim.riv` as the next M1 divergence: C++ emits an
  empty synchronized trim path at sample 0 and allocates selected-artboard
  stroke/fill render paints in draw order, while Rust still emits the untrimmed
  path and swaps the paint IDs.
- 2026-07-02: [M1] Marked `trim.riv` exact by preserving empty TrimPath
  effects and effect-bearing paint allocation order; exact count is now 4.
- 2026-07-02: [M1] Gated `custom_image_name.riv`,
  `library_export_test.riv`, and `nested_artboard_opacity.riv` as verified
  Rust unsupported diagnostics for images/nested artboards; exact remains 4,
  unsupported-feature is now 40, and not-yet is now 251.
- 2026-07-02: [M1] Gated `library_with_image.riv`,
  `double_library_with_image.riv`, `library_export_state_machine_test.riv`,
  and `library_export_animation_test.riv` as verified nested-artboard
  unsupported diagnostics; exact remains 4, unsupported-feature is now 44, and
  not-yet is now 247.
- 2026-07-02: [M1] Marked `long_name.riv` exact at sample `0`; exact count is
  now 5.
- 2026-07-02: [M1] Gated `scale_constraint.riv`,
  `translation_constraint.riv`, `transform_constraint.riv`, and
  `rotation_constraint.riv` as verified constraint unsupported diagnostics;
  exact remains 5, unsupported-feature is now 48, and not-yet is now 242.
- 2026-07-02: [M1] Marked `two_artboards.riv` exact at sample `0`; exact
  count is now 6.
- 2026-07-02: [M1] Gated `distance_constraint.riv` as a verified constraint
  unsupported diagnostic; exact remains 6, unsupported-feature is now 49, and
  not-yet is now 240.
- 2026-07-02: [M1] Marked `circle_clips.riv` exact by porting static
  `ClippingShape` clip proxy drawing and reusing the artboard background path
  across paints; exact count is now 7.
- 2026-07-02: [M1] Gated `clipping_and_draw_order.riv` as a verified image
  unsupported diagnostic; exact remains 7, unsupported-feature is now 50, and
  not-yet is now 238.
- 2026-07-02: [M1] Marked `trim_path.riv` exact by porting static artboard
  clip flags, multi-contour TrimPath extraction, empty-trim paint allocation,
  and numeric-token epsilon comparison; exact count is now 8.
- 2026-07-02: [M1] Marked `draw_rule_cycle.riv` and `test_elastic.riv` exact
  at sample `0`, generalized instance paint preallocation to C++
  `ShapePaintMutator` order, and parked `fill_trim_path.riv` for M2 keyframe
  application; exact count is now 10.
- 2026-07-02: [M1] Marked `blend_test.riv`,
  `multiple_state_machines.riv`, and `stroke_name_test.riv` exact at sample
  `0` by matching C++ static-scene marker selection; exact count is now 13.
- 2026-07-02: [M1] Marked `fix_rectangle.riv` exact at sample `0` by matching
  C++ `ShapePaintPath` clockwise fill-rule defaults for stroked composed
  paths; exact count is now 14.
- 2026-07-02: [M1] Marked `data_bind_solo.riv` and `hit_test_solos.riv`
  exact at sample `0` by applying imported Solo collapse, gated
  `follow_path_solos.riv` as a verified constraint unsupported diagnostic, and
  parked `solo_test.riv` for M2 frame-0 keyframe application; exact count is
  now 16.
- 2026-07-02: [M1] Marked `settler.riv` and `sound2.riv` exact at sample `0`,
  gated follow-path, scroll/translation constraint, and nested-artboard files
  as verified Rust unsupported diagnostics, and parked `solos_collapse_tests`,
  `click_event`, and `sound` for M2 frame-0 keyframe application; exact count
  is now 18, unsupported-feature is now 66, and not-yet is now 211.
- 2026-07-02: [M1] Gated `sorted_listeners.riv` as a verified text
  unsupported diagnostic and `solid_affects_has_changed.riv` as a verified
  nested-artboard unsupported diagnostic; exact remains 18, unsupported-feature
  is now 68, and not-yet is now 209.
- 2026-07-02: [M1] Promoted `script_paths_opacity_test.riv`,
  `script_paths_test.riv`, `scripted_boolean.riv`, `scripted_enum.riv`, and
  `scripted_graph.riv` as sample-0 exact, and gated `scripted_color.riv` as a
  verified data-binding-color unsupported diagnostic; exact count is now 23,
  unsupported-feature is now 69, and not-yet is now 203.
- 2026-07-02: [M1] Promoted `scripted_string.riv`,
  `viewmodel_runtime_file.riv`, and `clear_viewmodel_list.riv` as sample-0
  exact, and gated `databind_external_artboard_main.riv`,
  `databind_external_artboard_child.riv`, and `viewmodel_image_reset.riv` with
  existing nested-artboard/text/image diagnostics; exact count is now 26,
  unsupported-feature is now 72, and not-yet is now 197.
- 2026-07-02: [M1] Promoted `component_list_hit_order.riv`,
  `component_list_2.riv`, `bindable_artboard_child.riv`, and
  `text_input_event.riv` as sample-0 exact, and gated `component_list_1.riv`
  plus `custom_property_enum.riv` with verified constraint/custom-property
  enum diagnostics; exact count is now 30, unsupported-feature is now 74, and
  not-yet is now 191.
- 2026-07-02: [M1] Promoted `solos_with_nested_artboards.riv` as sample-0
  exact, and gated `scripted_transition_condition.riv`,
  `data_converter_interpolator_reset.riv`, `stateful_keyed_trigger.riv`,
  `unbound_stateful_component.riv`, and `scripting_root_viewmodel.riv` with
  verified scripted-transition/data-binding/nested-artboard diagnostics; exact
  count is now 31, unsupported-feature is now 79, and not-yet is now 185.
- 2026-07-02: [M1] Promoted `data_binding_test_2.riv`,
  `timeline_event_test.riv`, and `component_based_conditions.riv` as sample-0
  exact, and gated `zero_width_space_line_break.riv`,
  `bidirectional_precedence.riv`, and `collapsable_data_binding.riv` with
  verified text/data-binding diagnostics; exact count is now 34,
  unsupported-feature is now 82, and not-yet is now 179.
- 2026-07-02: [M1] Promoted `state_machine_triggers.riv` as sample-0 exact,
  and gated `viewmodel_from_context.riv`, `viewmodel_list_trigger.riv`,
  `transition_index_condition.riv`, `complex_ik_dependency.riv`, and
  `stateful_source_switch.riv` with verified layout/constraint/nested-artboard
  diagnostics; exact count is now 35, unsupported-feature is now 87, and
  not-yet is now 173.
- 2026-07-02: [M1] Promoted `state_machine_transition.riv` and
  `stateful_list_props.riv` as sample-0 exact by pruning empty
  `ShapePaintPath` segments during runtime path composition, and gated
  `state_transition_fire_trigger.riv`, `stateful_artboard_swap.riv`,
  `stateful_multi_property.riv`, and `stateful_nested.riv` with verified
  nested-artboard diagnostics; exact count is now 37, unsupported-feature is
  now 91, and not-yet is now 167.
- 2026-07-02: [M1] Gated `tape.riv`, `target_event.riv`,
  `time_based_interpolation.riv`, `transition_artboard_condition_test.riv`,
  `transition_duration_bind_list.riv`, and `two_bone_ik.riv` with verified
  image/nested-artboard/data-binding/layout/constraint diagnostics; exact
  remains 37, unsupported-feature is now 97, and not-yet is now 161.
- 2026-07-02: [M1] Added an import-time image-asset diagnostic and gated
  `trigger_based_listeners.riv`, `virtualized_artboard_databound_children.riv`,
  `walle.riv`, `word_joiner_test.riv`, `viewmodel_based_condition.riv`, and
  `transition_self_comparator_test.riv` with verified nested-artboard, layout,
  image, and text diagnostics; exact remains 37, unsupported-feature is now
  103, and not-yet is now 155.
- 2026-07-02: [M1] Gated `artboard_list_map_rules.riv`,
  `artboard_list_overrides.riv`, `artboard_width_test.riv`,
  `component_list_child_origin.riv`, `transition_duration_bind_nested.riv`, and
  `trigger_fires_single_change.riv` with verified layout/nested-artboard
  diagnostics; exact remains 37, unsupported-feature is now 109, and not-yet
  is now 149.
- 2026-07-02: [M1] Promoted `animation_reset_cases.riv` and
  `component_list_grouped.riv` as sample-0 exact, gated
  `animated_clipping.riv`, `background_measure.riv`, and
  `component_list_virtualized.riv` with verified text/layout diagnostics, and
  left `advance_blend_mode.riv` parked for its non-zero sample; exact count is
  now 39, unsupported-feature is now 112, and not-yet is now 144.
- 2026-07-02: [M1] Gated `component_stateful.riv`,
  `component_stateful_vm_instance.riv`, `component_stateful_vm_instance_2.riv`,
  `computed_root_transform.riv`, and `computed_values_test.riv` with verified
  nested-artboard/layout diagnostics, and parked `cubic_value_test.riv` for M2
  keyframe/interpolator application; exact remains 39, unsupported-feature is
  now 117, and not-yet is now 139.
- 2026-07-02: [M1] Gated `custom_property_trigger.riv`,
  `data_bind_test_cmdq.riv`, `data_binding_artboards_source_test.riv`,
  `data_binding_images_test.riv`, `data_binding_test.riv`, and
  `data_binding_test_3.riv` with verified nested-artboard/text diagnostics;
  exact remains 39, unsupported-feature is now 123, and not-yet is now 133.
- 2026-07-02: [M1] Gated `data_binding_test_triggers.riv`,
  `data_converter_to_number.riv`, `databind_artboard.riv`,
  `databind_solo_to_enum.riv`, `db_health_tracker.riv`, and `death_knight.riv`
  with verified nested-artboard/text diagnostics; exact remains 39,
  unsupported-feature is now 129, and not-yet is now 127.
- 2026-07-02: [M1] Promoted `event_on_listener.riv` as sample-0 exact and
  gated `double_line.riv`, `drag_event.riv`, `echo_show_demo.riv`,
  `ellipsis.riv`, and `entry.riv` with verified text/nested-artboard
  diagnostics; exact count is now 40, unsupported-feature is now 134, and
  not-yet is now 121.
- 2026-07-02: [M1] Promoted `events_on_states.riv` as sample-0 exact, gated
  `feather_render_test.riv`, `fit_font_size_test.riv`,
  `focus_collapsing.riv`, and `focus_traversal.riv` with verified
  image/text/nested-artboard diagnostics, and parked
  `event_trigger_event.riv` for M2 frame-0 color application; exact count is
  now 41, unsupported-feature is now 138, and not-yet is now 116.
- 2026-07-02: [M1] Gated `focusable_element.riv`,
  `format_number_with_commas.riv`, `formula_random.riv`, `hello_world.riv`,
  `hit_test_nested.riv`, and `hit_test_test.riv` with verified
  nested-artboard/text/data-binding diagnostics, and broadened the static
  runner transform data-bind diagnostic to include converter-group-backed
  Shape x/y bindings; exact remains 41, unsupported-feature is now 144, and
  not-yet is now 110.
- 2026-07-02: [M1] Gated `hittest_collapsed_layouts.riv`,
  `hosted_font_file.riv`, `hosted_image_file.riv`, `hunter_x_demo.riv`,
  `image_binding_with_listener.riv`, and `image_fit_alignment.riv` with
  verified text/image/nested-artboard diagnostics; exact remains 41,
  unsupported-feature is now 150, and not-yet is now 104.
- 2026-07-02: [M1] Gated `image_fit_alignment_2.riv`,
  `image_fit_alignment_3.riv`, `in_band_asset.riv`,
  `interactive_scrolling.riv`, and `interpolate_to_end.riv` with verified
  image/nested-artboard diagnostics, and parked
  `interpolation_zero_duration.riv` for M5 zero-duration data-binding
  interpolator transform application; exact remains 41, unsupported-feature is
  now 155, and not-yet is now 99.
- 2026-07-02: [M1] Promoted `joystick_nested_remap.riv` as sample-0 exact,
  gated `jellyfish_test.riv`, `joel_signed.riv`, `joel_v3.riv`, and
  `juice.riv` with verified image/gradient/text diagnostics, and parked
  `joystick_flag_test.riv` for M2 joystick application; exact count is now 42,
  unsupported-feature is now 159, and not-yet is now 94.
- 2026-07-02: [M1] Promoted `keyboard_event_to_script.riv` and
  `library_data_enum_test.riv` as sample-0 exact, and gated
  `keyboard_listener.riv`, `library.riv`, `library_view_model_test.riv`, and
  `library_vmtest_1_host.riv` with verified text/image/nested-artboard
  diagnostics; exact count is now 44, unsupported-feature is now 163, and
  not-yet is now 88.
- 2026-07-02: [M1] Promoted `light_switch.riv` and `list_to_path.riv` as
  sample-0 exact, and gated `library_with_text_and_image.riv`,
  `list_index_script_access.riv`, `list_items.riv`, and
  `list_to_length_test.riv` with verified nested-artboard/text/layout
  diagnostics; exact count is now 46, unsupported-feature is now 167, and
  not-yet is now 82.
- 2026-07-02: [M1] Promoted `lock_icon_demo.riv` and
  `looping_timeline_events.riv` as sample-0 exact, and gated
  `listener_action_inputs.riv`, `listener_view_model.riv`, `local_bounds.riv`,
  and `magic_alley_db_reduced_export.riv` with verified scripted-condition,
  text, image, and nested-artboard diagnostics; exact count is now 48,
  unsupported-feature is now 171, and not-yet is now 76.
- 2026-07-02: [M1] Added an n-slice Rust runner diagnostic and gated
  `modifier_test.riv`, `modifier_to_run.riv`, `multitouch.riv`,
  `multitouch_enter.riv`, `n_slice_triangle.riv`, and
  `nested_artboard_quantize_and_speed.riv` with verified text,
  nested-artboard, and n-slice diagnostics; exact remains 48,
  unsupported-feature is now 177, and not-yet is now 70.
- 2026-07-02: [M1] Promoted `nested_solo.riv` as sample-0 exact, and gated
  `nested_event_test.riv`, `nested_events.riv`, `nested_hug.riv`,
  `nested_needs_advance.riv`, and `new_text.riv` with verified
  nested-artboard/text diagnostics; exact count is now 49,
  unsupported-feature is now 182, and not-yet is now 64.
- 2026-07-02: [M1] Gated `number_to_list_nested_children.riv`,
  `off_road_car.riv`, and `pause_nested_artboard.riv` with verified
  layout/gradient/nested-artboard diagnostics, and parked `oneshotblend.riv`,
  `opaque_hit_test.riv`, and `pointer_events.riv` for M2 state-machine or
  non-zero-sample support; exact remains 49, unsupported-feature is now 185,
  and not-yet is now 61.
- 2026-07-02: [M1] Promoted `rapid_pointer_events.riv` as sample-0 exact,
  gated `pointer_events_nested_artboards_in_solos.riv`, `pointer_exit.riv`,
  `rebind_with_nested_viewmodel.riv`, and `recursive_data_bind.riv` with
  verified nested-artboard/text diagnostics, and parked `quantize_test.riv`
  for M2 quantized animation application; exact count is now 50,
  unsupported-feature is now 189, and not-yet is now 56.
- 2026-07-02: [M1] Added a scripted-path-effects runner diagnostic, promoted
  `remove_from_list.riv` as sample-0 exact, and gated
  `relative_data_binding.riv`, `replace_vm_instance.riv`, `reset_phase.riv`,
  `reuse_path_in_effect.riv`, and `rewards_demo.riv` with verified
  nested-artboard/text/layout/scripted-path diagnostics; exact count is now
  51, unsupported-feature is now 194, and not-yet is now 50.
- 2026-07-02: [M1] Gated `rocket.riv`, `runtime_nested_inputs.riv`,
  `runtime_nested_text_runs.riv`, `saturation.riv`,
  `scripted_data_context.riv`, and `scripted_listener_context.riv` with
  verified gradient/nested-artboard/text diagnostics; exact remains 51,
  unsupported-feature is now 200, and not-yet is now 44.
- 2026-07-02: [M1] Gated `scripted_property_image.riv`,
  `scroll_snap.riv`, `scroll_test.riv`, `scroll_threshold.riv`,
  `shared_viewmodel_instance.riv`, and `spotify_kids_app_icon.riv` with
  verified image/text/nested-artboard diagnostics; exact remains 51,
  unsupported-feature is now 206, and not-yet is now 38.
- 2026-07-02: [M1] Ported DashPath stroke effects and promoted
  `stacked_path_effects.riv` as sample-0 exact; gated
  `spotify_kids_demo.riv`, `superbowl.riv`, `test_modifier_run.riv`,
  `text_follow_path_shape_length.riv`, and `text_input.riv` with verified
  image/nested-artboard/text diagnostics; exact count is now 52,
  unsupported-feature is now 211, and not-yet is now 32.
- 2026-07-02: [M1] Gated `text_listener_simpler.riv`,
  `text_opacity_modifier.riv`, `text_stroke_test.riv`,
  `text_vertical_trim_test.riv`, `transition_actions.riv`, and
  `vertical_align_ellipsis.riv` with verified text diagnostics; exact remains
  52, unsupported-feature is now 217, and not-yet is now 26.
- 2026-07-02: [M1] Gated `advance_blend_mode.riv`,
  `ai_assitant.riv`, `align_target.riv`, `bad_skin.riv`, `bankcard.riv`, and
  `bindable_artboard_nesty.riv` with verified nested-artboard/image
  diagnostics; exact remains 52, unsupported-feature is now 223, and not-yet
  is now 20.
- 2026-07-02: [M1] Gated `bullet_man.riv`, `car_widgets_v01.riv`,
  `collapse_data_binds.riv`, and `collapsing_elements.riv` with verified
  nested-artboard/text diagnostics; rechecked `cubic_value_test.riv` and
  `event_trigger_event.riv` as M2 stream divergences; exact remains 52,
  unsupported-feature is now 227, and not-yet is now 16.
- 2026-07-02: [M1] Gated `zombie_skins.riv` with a verified nested-artboard
  diagnostic; exact remains 52, unsupported-feature is now 228, and not-yet
  is now 15. Next M1 implementation target is gradient rendering.
- 2026-07-02: [M1] Ported static linear/radial gradient shader creation in
  dependency order, promoted `joel_signed.riv` as exact, and reclassified
  `juice.riv`, `off_road_car.riv`, and `rocket.riv` as concrete M1
  divergences after matching shader creation; exact is now 53,
  unsupported-feature is now 224, diverges is now 3, and not-yet remains 15.
- 2026-07-02: [M1] Corrected `juice.riv` and `rocket.riv` from M1 divergences
  to M2 frame-0 animation/keyframe application after inspecting their default
  animation graphs; exact remains 53, unsupported-feature remains 224,
  diverges is now 1, and not-yet is now 17.
- 2026-07-02: [M1] Promoted `off_road_car.riv` as sample-0 exact by caching
  `ClippingShape` render paths per clipping shape and matching C++ `Mat2D`
  inverse/mapPoints/skinning float behavior; exact is now 54,
  unsupported-feature remains 224, diverges is now 0, and not-yet remains 17.
- 2026-07-03: [M1] Added the feature-gated `rive-renderer-ffi` crate with a
  Rust `Factory`/`Renderer` wrapper and C ABI bridge over C++
  `RiveRenderFactory`/`RiveRenderer`; default workspace tests stay independent
  of native renderer artifacts, and the bridge currently syntax-checks against
  a `RenderContextNULL` smoke backend. Exact remains 54; next M1 work is the
  real Metal/window or offscreen-pixel demo target.
- 2026-07-03: [M1] Made `rive-renderer-ffi --features native` link and run on
  this machine by compiling the needed C++ renderer sources when
  `librive_pls_renderer.a` is absent, added a native draw-count unit test, and
  added `ffi_null_draw` as a real `.riv` import/draw smoke (`dependency_test`
  draws 3 calls). Exact remains 54; full Metal/offscreen pixels remain blocked
  on Apple's Metal Toolchain for the C++ renderer archive build.
- 2026-07-03: [M1] Installed the Apple Metal Toolchain, built the C++
  `librive_pls_renderer.a` and dependency archives, and taught
  `rive-renderer-ffi` to link the prebuilt archive set with matching
  canvas/text/layout/decoder feature defines so the null backend's vtable
  matches the archive ABI. `rive-renderer-ffi --features native`,
  `ffi_null_draw`, `make golden-compare`, and `cargo test --workspace` pass;
  exact remains 54. Remaining M1 FFI demo work is the actual Metal
  offscreen/window pixel target.
- 2026-07-03: [M1] Ported the C++ `TestingWindowMetalTexture` offscreen target
  pattern into `rive-renderer-ffi`: macOS native mode now has a Metal context,
  BGRA8 render target texture, external-command-buffer flush, and RGBA pixel
  readback from Rust. `ffi_metal_draw` imports `dependency_test.riv`, draws 3
  calls at `800x800`, reads `640000` nonzero pixels
  (`checksum=9119d6210ebbef10`), and `ffi_null_draw` still passes. Verified
  `rive-renderer-ffi --features native`, `make golden-compare`
  (`exact=54`, `diverges=0`, `unsupported-feature=224`, `not-yet=17`), and
  `cargo test --workspace`; M1 is complete and the active milestone moves to
  M2.
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
