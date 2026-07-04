# V2 Log Archive

Historical log entries moved out of `docs/v2-status.md` to keep the
per-session working file small. Completed milestones and rolled-off active
milestone entries are kept verbatim here. Newest milestone last.

## M0 + M1 (completed 2026-07-03)


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

## M2 active log rolloff (archived 2026-07-03)

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

- 2026-07-03: [M2] Widened `circle_clips.riv` from sample `0` to samples `0`
  and `0.25`, keeping its animated clipping playback stream exact. Exact
  segments are now 96 across 70 exact files; focused golden compare reports
  `exact=1`, `exact-segments=2`, `diverges=0`, `unsupported-feature=0`,
  `not-yet=0`.

- 2026-07-03: [M2] Moved state-machine transition interpolators into
  `crates/rive-runtime/src/state_machine.rs` and removed the duplicate
  cubic/elastic helper copy from `lib.rs`, reusing the animation interpolator
  math instead. Exact segments remain 96 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=96`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Moved state-machine blend-state import/runtime model data
  (`RuntimeBlendState1D`, `RuntimeBlendStateDirect`, and direct blend source
  metadata) into `crates/rive-runtime/src/state_machine.rs`, leaving only the
  live artboard-applying blend instances in `lib.rs`. Exact segments remain
  96 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=96`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Moved live state-machine blend-state instances
  (`BlendState1DInstance` and `BlendStateDirectInstance` advance/mix/apply
  runtime) into `crates/rive-runtime/src/state_machine.rs` beside the imported
  blend-state model, leaving `lib.rs` with layer orchestration. Exact segments
  remain 96 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=96`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Moved `StateMachineLayerInstance` and transition animation
  reset/apply runtime into `crates/rive-runtime/src/state_machine.rs`, leaving
  `StateMachineInstance` in `lib.rs` to orchestrate data binding and layers
  through crate-visible accessors. Exact segments remain 96 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=96`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.

- 2026-07-03: [M2] Moved imported `RuntimeStateMachineLayer` and
  `RuntimeLayerState` model types into
  `crates/rive-runtime/src/state_machine.rs`, preserving their public
  crate-root exports while keeping construction in the existing import
  builder. Exact segments remain 96 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=96`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Moved `RuntimeStateTransition`, transition timing,
  exit-time allowance, and transition fire/listener action dispatch into
  `crates/rive-runtime/src/state_machine.rs` beside the layer runtime that
  consumes it; transition conditions remain in `lib.rs` with their component
  and data-binding comparand helpers. Exact segments remain 96 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=96`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.

- 2026-07-03: [M2] Tripwire check after repeated state-machine
  modularization commits pivoted the queue back to metric-moving sample
  widening. Widened `long_name.riv` from sample `0` to samples `0` and
  `0.25`, keeping its simple animated stream exact. Exact segments are now
  97 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=97`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `fix_rectangle.riv` from sample `0` to samples
  `0` and `0.25`, keeping its animated path/color stream exact. Exact
  segments are now 98 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=98`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `draw_rule_cycle.riv` from sample `0` to samples
  `0` and `0.25`, keeping its draw-rule/keyframe stream exact. Exact
  segments are now 99 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=99`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `trim_path.riv` from sample `0` to samples `0`
  and `0.25`, keeping its animated TrimPath stream exact. Exact segments are
  now 100 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=100`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `nested_solo.riv` from sample `0` to samples `0`
  and `0.25`, keeping its Solo/state-machine playback stream exact. Exact
  segments are now 101 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=101`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `fill_trim_path.riv` from sample `0` to samples
  `0` and `0.25`, keeping its animated Fill/TrimPath stream exact. Exact
  segments are now 102 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=102`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Focused `[0, 0.25]` promotion probe for
  `stacked_path_effects.riv` found a narrow animated stacked DashPath/TrimPath
  divergence at `0.25`; the file stays exact only at sample `0` until that M2
  path-effect composition gap is ported.

- 2026-07-03: [M2] Focused `[0, 0.25]` promotion probe for `juice.riv` found
  a multi-sample gradient lifetime divergence: Rust emits new linear-gradient
  shader IDs before the second sample while C++ reuses the original shaders,
  so the file stays exact only at sample `0` until gradient reuse is ported.

- 2026-07-03: [M2] Added a per-run gradient shader cache to the Rust render
  resource cache so unchanged LinearGradient/RadialGradient paints reuse the
  same shader objects across samples, matching C++ dirty-update lifetime.
  Widened `juice.riv` from sample `0` to samples `0` and `0.25`; exact
  segments are now 103 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=103`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `scripted_enum.riv` from sample `0` to samples
  `0` and `0.25`, keeping its enum view-model/state-machine playback stream
  exact. Exact segments are now 104 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=104`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `library_data_enum_test.riv` from sample `0` to
  samples `0` and `0.25`, keeping its library enum view-model/data-bind
  playback stream exact. Exact segments are now 105 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=105`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `viewmodel_runtime_file.riv` from sample `0` to
  samples `0` and `0.25`, keeping its mixed view-model/state-machine playback
  stream exact. Exact segments are now 106 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=106`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `joystick_nested_remap.riv` from sample `0` to
  samples `0` and `0.25`, keeping its joystick/nested-remap playback stream
  exact. Exact segments are now 107 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=107`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `scripted_graph.riv` from sample `0` to samples
  `0` and `0.25`, keeping its view-model list/state-machine playback stream
  exact. Exact segments are now 108 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=108`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `rapid_pointer_events.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive listener/view-model state
  playback stream exact before M3 scripted input. Exact segments are now 109
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=109`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `text_input_event.riv` from sample `0` to samples
  `0` and `0.25`, keeping its passive keyboard/text-listener playback stream
  exact before M3/M6 scripted input and text work. Exact segments are now 110
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=110`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `hit_test_solos.riv` from sample `0` to samples
  `0` and `0.25`, keeping its passive Solo/listener playback stream exact
  before M3 hit-test input. Exact segments are now 111 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=111`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `list_to_path.riv` from sample `0` to samples `0`
  and `0.25`, keeping its passive list-path/view-model playback stream exact
  before M5 external data-binding work. Exact segments are now 112 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=112`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `solos_with_nested_artboards.riv` from sample `0`
  to samples `0` and `0.25`, keeping its passive Solo/nested-artboard
  playback stream exact before M4 nested advancement work. Exact segments are
  now 113 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=113`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `click_event.riv` from sample `0` to samples `0`
  and `0.25`, keeping its passive click-listener/event playback stream exact
  before M3 scripted input. Exact segments are now 114 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=114`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `component_list_hit_order.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive component-list/listener
  playback stream exact before M3/M4 scripted input and component-list work.
  Exact segments are now 115 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=115`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `event_trigger_event.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive trigger-event/view-model
  playback stream exact before M3 scripted input and M5 external binding.
  Exact segments are now 116 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=116`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `opaque_hit_test.riv` from sample `0` to samples
  `0` and `0.25`, keeping its passive nested-artboard/listener playback
  stream exact before M3 scripted hit-test input and M4 nested advancement.
  Exact segments are now 117 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=117`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `keyboard_event_to_script.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive scripted-drawable/state-machine
  playback stream exact before M3 keyboard input and M6 scripting work. Exact
  segments are now 118 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=118`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `clear_viewmodel_list.riv` from sample `0` to
  samples `0` and `0.25`, keeping its passive view-model/list state-machine
  playback stream exact before M5 external data-binding work. Exact segments
  are now 119 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=119`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `component_based_conditions.riv` from sample `0`
  to samples `0` and `0.25`, keeping component-comparator state-machine
  playback exact before M5 external data-binding mutation work. Exact segments
  are now 120 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=120`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `component_list_2.riv` from sample `0` to samples
  `0` and `0.25`, keeping passive component-list/listener playback exact
  before M3 scripted input and M4/M6 list/layout work. Exact segments are now
  121 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=121`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `component_list_grouped.riv` from sample `0` to
  samples `0` and `0.25`, keeping passive grouped component-list playback
  exact before M3 scripted input and M4/M6 list/layout work. Exact segments
  are now 122 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=122`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `data_bind_solo.riv` from sample `0` to samples
  `0` and `0.25`, keeping passive data-bind/Solo playback exact before M5
  external view-model mutation work. Exact segments are now 123 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=123`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `data_binding_test_2.riv` from sample `0` to
  samples `0` and `0.25`, keeping passive data-converter/state-machine
  playback exact before M5 external binding mutation work. Exact segments are
  now 124 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=124`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `lock_icon_demo.riv` from sample `0` to samples
  `0` and `0.25`, keeping passive lock-icon TrimPath/state-machine playback
  exact before M3 scripted input and later interaction work. Exact segments
  are now 125 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=125`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `state_machine_transition.riv` from sample `0` to
  samples `0` and `0.25`, keeping passive listener-trigger/bool
  state-machine transition playback exact before M3 scripted input. Exact
  segments are now 126 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=126`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

- 2026-07-03: [M2] Widened `solos_collapse_tests.riv` from sample `0` to
  samples `0` and `0.25`, keeping animated Solo active-child/collapse
  playback exact before M3 constraints/input work. Exact segments are now 127
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=127`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.

- 2026-07-03: [M2] Widened `rocket.riv` from sample `0` to samples `0` and
  `0.25`, keeping its rounded-path/draw-rule animated playback stream exact.
  Exact segments are now 128 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=128`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

### Rolloff 2026-07-03 exact-segments 129-170

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
- 2026-07-03: [M2] Widened `dependency_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping its base vector playback
  stream exact across the wider sample set. Exact segments are now 152 across
  70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=152`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `draw_rule_cycle.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping draw-rule ordering exact
  across the wider animated sample set. Exact segments are now 153 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=153`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `event_on_listener.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping listener-driven event
  playback exact across the wider animated sample set. Exact segments are now
  154 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=154`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `event_trigger_event.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping event-trigger playback
  exact across the wider animated sample set. Exact segments are now 155
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=155`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `events_on_states.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping state-entry/end event
  playback exact across the wider animated sample set. Exact segments are now
  156 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=156`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `fill_trim_path.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping trim/fill path playback
  exact across the wider animated sample set. Exact segments are now 157
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=157`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `fix_rectangle.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping animated rectangle/path
  playback exact across the wider sample set. Exact segments are now 158
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=158`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `hit_test_solos.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping solo/listener playback
  exact across the wider sample set. Exact segments are now 159 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=159`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `joel_signed.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping its larger animated
  skin/constraint/vector playback stream exact across the wider sample set.
  Exact segments are now 160 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=160`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `joystick_flag_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping basic joystick playback
  exact across the wider sample set. Exact segments are now 161 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=161`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `joystick_nested_remap.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping nested-remap joystick
  playback exact across the wider sample set. Exact segments are now 162
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=162`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `juice.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping gradient/vector animation playback
  exact across the wider sample set. Exact segments are now 163 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=163`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `keyboard_event_to_script.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping its scripted-drawable
  playback stream exact without opening M6 scripting scope. Exact segments are
  now 164 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=164`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `library_data_enum_test.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping enum/data-converter
  playback exact without opening M5 mutation scope. Exact segments are now 165
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=165`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `light_switch.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping state-machine bool/light switch
  playback exact across the wider sample set. Exact segments are now 166
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=166`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `list_to_path.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping list-path/data-context playback
  exact across the wider sample set. Exact segments are now 167 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=167`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `lock_icon_demo.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping skinned lock-icon/trim-path
  playback exact across the wider sample set. Exact segments are now 168
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=168`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `long_name.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping its rectangle animation playback
  exact across the wider sample set. Exact segments are now 169 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=169`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `looping_timeline_events.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping looping callback-event
  playback exact across the wider sample set. Exact segments are now 170
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=170`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `multiple_state_machines.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping default state-machine
  selection/playback exact across the wider sample set. Exact segments are now
  171 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=171`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `nested_solo.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping Solo collapse/state-machine
  playback exact across the wider sample set. Exact segments are now 172
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=172`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `off_road_car.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping its animated skinned vector/path
  playback exact across the wider sample set. Exact segments are now 173
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=173`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `oneshotblend.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping one-shot 1D blend-state playback
  exact across the wider sample set. Exact segments are now 174 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=174`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `opaque_hit_test.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping nested-bool/draw-rule playback
  exact across the wider sample set. Exact segments are now 175 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=175`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `pointer_events.riv` from samples `0` and `0.1`
  to samples `0`, `0.1`, and `0.25`, keeping listener/bool pointer-event
  playback exact at the next M2 sample. Exact segments are now 176 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=176`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `quantize_test.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping quantized keyframe playback
  exact across the wider sample set. Exact segments are now 177 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=177`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `rapid_pointer_events.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive
  listener/data-bind state-machine playback exact before M3 scripted pointer
  input work. Exact segments are now 178 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=178`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `remove_from_list.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive
  text/list/scripted-drawable playback exact before M4/M5/M6 list,
  data-binding, and scripting work. Exact segments are now 179 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=179`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `rocket.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping animated vector/gradient playback
  exact across the wider sample set. Exact segments are now 180 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=180`, `diverges=0`, `unsupported-feature=225`, `not-yet=0`,
  and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `script_paths_opacity_test.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive
  scripted-drawable opacity/keyed-double playback exact before M6 scripting
  work. Exact segments are now 181 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=181`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `script_paths_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive
  scripted-drawable/keyed-double playback exact before M6 scripting work.
  Exact segments are now 182 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=182`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `scripted_boolean.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive view-model bool
  state-machine playback exact before M5/M6 mutation and scripting work.
  Exact segments are now 183 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=183`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `scripted_enum.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping passive enum/view-model
  state-machine playback exact before M5/M6 mutation and scripting work.
  Exact segments are now 184 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=184`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `scripted_graph.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping passive list/number view-model
  state-machine playback exact before M5/M6 mutation and scripting work.
  Exact segments are now 185 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=185`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `scripted_string.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive string
  view-model state-machine playback exact before M5/M6 mutation and scripting
  work. Exact segments are now 186 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=186`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `settler.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping animated rectangle/vector
  state-machine playback exact across the wider sample set. Exact segments
  are now 187 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=187`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `solo_test.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping Solo active-child keyed-ID
  playback exact across the wider sample set. Exact segments are now 188
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=188`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `solos_collapse_tests.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping Solo collapse with
  clipping and passive rotation-constraint content exact across the wider
  sample set. Exact segments are now 189 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=189`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `solos_with_nested_artboards.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping the already-exact
  passive Solo/nested-artboard state-machine case exact without opening M4
  nested-artboard runtime scope. Exact segments are now 190 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=190`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `sound.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping passive render/state-machine
  playback exact while leaving audio event behavior in M6 scope. Exact
  segments are now 191 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=191`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `sound2.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping the passive audio/open-url/nested
  render path exact while leaving audio and nested-artboard behavior in M6/M4
  scope. Exact segments are now 192 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=192`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `stacked_path_effects.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping stacked TrimPath/DashPath
  playback exact across the wider sample set. Exact segments are now 193
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=193`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `state_machine_transition.riv` from samples `0`
  and `0.25` to samples `0`, `0.25`, and `0.5`, keeping listener-trigger,
  listener-bool, and color state-transition playback exact across the wider
  sample set. Exact segments are now 194 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=194`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `state_machine_triggers.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping trigger-condition
  state-machine playback exact across the wider sample set. Exact segments
  are now 195 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=195`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `stateful_list_props.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping the passive text/layout,
  view-model, and list-property render path exact while leaving those runtime
  behaviors in M6/M5/M4 scope. Exact segments are now 196 across 70 exact
  files; `make golden-compare` reports `exact=70`,
  `exact-segments=196`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `stroke_name_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping stroke-trigger
  state-machine playback exact across the wider sample set. Exact segments
  are now 197 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=197`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `test_elastic.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping ElasticInterpolator keyed-double
  playback exact across the wider sample set. Exact segments are now 198
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=198`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `text_input_event.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping the passive
  keyboard/text-input listener and view-model render path exact while leaving
  interactive text input behavior in later M3/M5/M6 scope. Exact segments are
  now 199 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=199`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `timeline_event_test.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping passive
  callback-event animation/state-machine playback exact without opening M3
  event-dispatch scripting. Exact segments are now 200 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=200`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `trim_path.riv` from samples `0` and `0.25` to
  samples `0`, `0.25`, and `0.5`, keeping animated TrimPath vector playback
  exact across the wider sample set. Exact segments are now 201 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=201`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `trim_path_linear.riv` from samples `0` and
  `0.25` to samples `0`, `0.25`, and `0.5`, keeping the linear TrimPath
  vector playback case exact across the wider sample set. Exact segments are
  now 202 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=202`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `two_artboards.riv` from samples `0` and `0.25`
  to samples `0`, `0.25`, and `0.5`, keeping selected-artboard draw/playback
  exact across the wider sample set. Exact segments are now 203 across 70
  exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=203`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
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
- 2026-07-03: [M2] Widened `rapid_pointer_events.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive pointer-event listener/view-model state-machine playback exact
  across the wider sample set while leaving scripted pointer dispatch in M3
  scope. Exact segments are now 244 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=244`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `remove_from_list.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  scripted/list/view-model playback exact across the wider sample set while
  leaving list mutation, scripting, and layout-component paint behavior in
  later M4/M6 scope. Exact segments are now 245 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=245`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
