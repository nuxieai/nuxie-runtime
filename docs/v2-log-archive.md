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
- 2026-07-03: [M2] Widened `rocket.riv` from samples `0`, `0.25`, and `0.5`
  to samples `0`, `0.25`, `0.5`, and `0.75`, keeping the richer
  vector/gradient/clipping state-machine playback stream exact across the
  wider sample set. Exact segments are now 246 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=246`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `script_paths_opacity_test.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive scripted-drawable opacity playback exact across the wider sample
  set while leaving active scripting behavior in M6 scope. Exact segments are
  now 247 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=247`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `script_paths_test.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive scripted-drawable playback exact across the wider sample set while
  leaving active scripting behavior in M6 scope. Exact segments are now 248
  across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=248`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `scripted_boolean.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  view-model bool state-machine playback exact before M5/M6 mutation and
  scripting work. Exact segments are now 249 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=249`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `scripted_enum.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  enum/view-model state-machine playback exact before M5/M6 mutation and
  scripting work. Exact segments are now 250 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=250`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `scripted_graph.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  list/view-model state-machine playback exact before M4/M5 mutation work.
  Exact segments are now 251 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=251`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `scripted_string.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive
  string/view-model state-machine playback exact before M5/M6 mutation and
  scripting work. Exact segments are now 252 across 70 exact files;
  `make golden-compare` reports `exact=70`, `exact-segments=252`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `settler.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping CubicEase keyed
  double animation playback exact across the wider sample set. Exact segments
  are now 253 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=253`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `solo_test.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping Solo
  active-child state-machine playback exact across the wider sample set. Exact
  segments are now 254 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=254`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `solos_collapse_tests.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping Solo
  collapse and clipping playback exact across the wider sample set. Exact
  segments are now 255 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=255`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `solos_with_nested_artboards.riv` from samples
  `0`, `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`,
  keeping passive Solo/nested-artboard playback exact before M4 nested runtime
  advancement. Exact segments are now 256 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=256`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `sound.riv` from samples `0`, `0.25`, and `0.5`
  to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive audio-event
  metadata and listener playback exact without opening audio output behavior.
  Exact segments are now 257 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=257`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `sound2.riv` from samples `0`, `0.25`, and `0.5`
  to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive audio,
  open-url, and nested-artboard metadata playback exact before M4/M6 runtime
  behavior. Exact segments are now 258 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=258`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `stacked_path_effects.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  stacked TrimPath and DashPath playback exact across the wider sample set.
  Exact segments are now 259 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=259`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `state_machine_transition.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive transition and listener playback exact across the wider sample set.
  Exact segments are now 260 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=260`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `state_machine_triggers.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive trigger-condition state-machine playback exact across the wider
  sample set. Exact segments are now 261 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=261`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `stateful_list_props.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive stateful list/view-model playback exact across the wider sample
  set. Exact segments are now 262 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=262`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `stroke_name_test.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping animated
  stroke/fill name playback exact across the wider sample set. Exact segments
  are now 263 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=263`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `test_elastic.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  ElasticInterpolator keyed animation playback exact across the wider sample
  set. Exact segments are now 264 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=264`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `text_input_event.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping passive text
  input listener and view-model bool playback exact while leaving active
  scripted keyboard/text input behavior in later milestones. Exact segments
  are now 265 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=265`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `timeline_event_test.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive timeline callback-event playback exact across the wider sample set.
  Exact segments are now 266 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=266`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `trim_path.riv` from samples `0`, `0.25`, and
  `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping TrimPath
  animation playback exact across the wider sample set. Exact segments are
  now 267 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=267`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `trim_path_linear.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping linear
  TrimPath animation playback exact across the wider sample set. Exact
  segments are now 268 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=268`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `two_artboards.riv` from samples `0`, `0.25`,
  and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping selected
  artboard animation playback exact across the wider sample set. Exact
  segments are now 269 across 70 exact files; `make golden-compare` reports
  `exact=70`, `exact-segments=269`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `viewmodel_runtime_file.riv` from samples `0`,
  `0.25`, and `0.5` to samples `0`, `0.25`, `0.5`, and `0.75`, keeping
  passive view-model metadata playback exact across the wider sample set.
  Exact segments are now 270 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=270`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `animation_reset_cases.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping reset/transition/blend animation playback exact across the
  fifth sample. Exact segments are now 271 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=271`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `bindable_artboard_child.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive bindable-artboard/view-model state-machine playback
  exact across the fifth sample. Exact segments are now 272 across 70 exact
  files; `make golden-compare` reports `exact=70`, `exact-segments=272`,
  `diverges=0`, `unsupported-feature=225`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `blend_test.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping direct/1D blend-state animation playback exact across the fifth
  sample. Exact segments are now 273 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=273`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `circle_clips.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping animated clipping-circle playback exact across the fifth sample.
  Exact segments are now 274 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=274`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `clear_viewmodel_list.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive view-model-list/scripted-drawable playback exact
  across the fifth sample while leaving list mutation, scripting, and
  layout-component paint behavior in later milestones. Exact segments are now
  275 across 70 exact files; `make golden-compare` reports `exact=70`,
  `exact-segments=275`, `diverges=0`, `unsupported-feature=225`,
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-03: [M2] Widened `click_event.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive event/listener bool state-machine playback exact across the
  fifth sample while leaving scripted pointer/event dispatch in M3 scope.
  Exact segments are now 276 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=276`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `clip_tests.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping animated clipping-shape playback exact across the fifth sample.
  Exact segments are now 277 across 70 exact files; `make golden-compare`
  reports `exact=70`, `exact-segments=277`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `component_based_conditions.riv` from samples `0`,
  `0.25`, `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and
  `1.0`, keeping passive component-comparator/data-bind condition playback
  exact across the fifth sample while leaving external view-model mutation in
  M5 scope. Exact segments are now 278 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=278`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-03: [M2] Widened `component_list_2.riv` from samples `0`, `0.25`,
  `0.5`, and `0.75` to samples `0`, `0.25`, `0.5`, `0.75`, and `1.0`,
  keeping passive component-list/list-index state-machine playback exact
  across the fifth sample while leaving active list/layout mutation in later
  milestones. Exact segments are now 279 across 70 exact files; `make
  golden-compare` reports `exact=70`, `exact-segments=279`, `diverges=0`,
  `unsupported-feature=225`, `not-yet=0`, and `cargo test --workspace`
  passes.

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

## M2 + M3 (completed 2026-07-04)

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
- 2026-07-04: [M3] Ported the direct-rectangle click phase slice from C++
  `ListenerGroup`, fixed `ListenerViewModelChange` trigger actions to
  invalidate the bindable trigger target-to-source data-bind path, and added
  `tests/input_scripts/bindable_artboard_child_click.txt` for
  `bindable_artboard_child.riv`. Exact segments are now 365 across 85 exact
  files; `make golden-compare` reports `exact=85`, `exact-segments=365`,
  `diverges=0`, `unsupported-feature=210`, `not-yet=0`, and
  `cargo test --workspace` passes.
- 2026-07-04: [M3] Widened all 12 scripted direct-pointer corpus entries
  from samples through `1.0` to samples through `1.25`, keeping direct
  rectangle down/up, click, event, Solo, and listener-owned view-model trigger
  paths exact after the interaction has more time to settle. Exact segments
  are now 377 across 85 exact files; `make golden-compare` reports
  `exact=85`, `exact-segments=377`, `diverges=0`,
  `unsupported-feature=210`, and `not-yet=0`. C++ probes found no visible
  render delta yet for `lock_icon_demo.riv` or `joel_signed.riv`; keep them in
  the unscripted candidate list until a render-affecting coordinate or input
  sequence is identified.
- 2026-07-04: [M3] Widened the same 12 scripted direct-pointer entries from
  samples through `1.25` to samples through `1.5`, adding another post-click
  checkpoint without broadening runtime scope. Exact segments are now 389
  across 85 exact files; `make golden-compare` reports `exact=85`,
  `exact-segments=389`, `diverges=0`, `unsupported-feature=210`, and
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Ported the direct-rectangle enter/exit hover slice from
  C++ `ListenerGroup`, including the `StateMachineListenerSingle`
  `listenerTypeValue = 0` default, and added
  `tests/input_scripts/sound_enter.txt` for `sound.riv`. The script starts
  outside the rectangle, moves inside, and keeps the number-driven hover
  animation exact through sample `1.5`. Exact segments are now 392 across 85
  exact files; `make golden-compare` reports `exact=85`,
  `exact-segments=392`, `diverges=0`, `unsupported-feature=210`, and
  `not-yet=0`, and `cargo test --workspace` passes.
- 2026-07-04: [M3] Extended `sound.riv`'s hover script to move back outside
  the direct rectangle and sample the exit path at `0.35`, keeping both
  direct enter and exit listener-number changes exact through sample `1.5`.
  Exact segments are now 393 across 85 exact files; `make golden-compare`
  reports `exact=85`, `exact-segments=393`, `diverges=0`,
  `unsupported-feature=210`, and `not-yet=0`, and `cargo test --workspace`
  passes.
- 2026-07-04: [M3] Closed the scripted-pointer milestone and opened M4.
  The remaining unscripted exact listener candidates
  (`component_list_2.riv`, `component_list_follow_path.riv`,
  `component_list_grouped.riv`, `component_list_hit_order.riv`,
  `joel_signed.riv`, `lock_icon_demo.riv`,
  `solos_with_nested_artboards.riv`, `stateful_list_props.riv`, and
  `text_input_event.riv`) produced no C++ render delta on a bounded coarse
  click/hover probe after filtering synthetic input markers, or belong to
  nested/list/text/keyboard domains. M4 should start with
  `library_export_test.riv` or `nested_artboard_opacity.riv`, the two smallest
  `milestone = "M4"` parked files. `make golden-compare` reports `exact=85`,
  `exact-segments=393`, `diverges=0`, `unsupported-feature=210`, and
  `not-yet=0`, and `cargo test --workspace` passes.

## M4 (completed 2026-07-04)

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

## M5 (completed 2026-07-04)

- 2026-07-04: [M5] Closed M5 queue after direct probes showed the final four
  M5 entries (`scripted_data_context.riv`, `shared_viewmodel_instance.riv`,
  `stateful_source_switch.riv`, and `transition_duration_bind_nested.riv`)
  now reach nested child `TextValueRun`; all four moved to M6 `text`. `make
  golden-compare` reports `exact=128`, `exact-segments=449`, `diverges=0`,
  `unsupported-feature=167`, `not-yet=0`, and parked
  `M6=124 gated=7 harness=36`; manifest query confirms M5=0, and `cargo
  test --workspace` passes.
- 2026-07-04: [M5] Retagged relative data binding to text: the runner now
  admits nested child `Shape.x/y` no-converter binds, and
  `relative_data_binding.riv` moved from M5 to M6 after the same probe reached
  nested child `TextValueRun`. `make golden-compare` reports `exact=128`,
  `exact-segments=449`, `diverges=0`, `unsupported-feature=167`,
  `not-yet=0`, and parked `M5=4 M6=120 gated=7 harness=36`; `cargo
  test --workspace` passes.
- 2026-07-04: [M5] Opened nested child custom string binds: the runner now
  admits nested child `CustomPropertyString.propertyValue` source-to-target
  binds with no converter or `DataConverterToString`, and current C++
  `ParametricPathBase` Rectangle width/height keys 20/21. Direct C++/Rust
  stream comparison promotes `library_view_model_test.riv` to exact. `make
  golden-compare` reports `exact=128`, `exact-segments=449`, `diverges=0`,
  `unsupported-feature=167`, `not-yet=0`, and parked `M5=5 M6=119 gated=7
  harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Opened zero-duration interpolator transform binds: the
  obsolete Rust-runner gate for near-zero-duration `DataConverterInterpolator`
  Shape x/y targets was removed after direct C++/Rust stream comparison showed
  `interpolation_zero_duration.riv` is exact. `make golden-compare` reports
  `exact=127`, `exact-segments=448`, `diverges=0`,
  `unsupported-feature=168`, `not-yet=0`, and parked `M5=6 M6=119 gated=7
  harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported nested child opacity/rectangle binds: the runner now
  admits nested child `Node.opacity` and `Rectangle.width/height` no-converter
  binds, nested artboard advance applies child artboard data binds, and
  authored-transparent Backboard/background draws are skipped to match C++.
  `hide_test.riv` is promoted to exact, while `interpolate_to_end.riv` moved to
  M6 after the same slice reached nested child `TextValueRun`. `make
  golden-compare` reports `exact=126`, `exact-segments=447`, `diverges=0`,
  `unsupported-feature=169`, `not-yet=0`, and parked `M5=7 M6=119 gated=7
  harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported artboard formula/group transform binds: artboard
  property bindings now run `DataConverterGroup` and `DataConverterFormula`
  through shared converter state, reset source-change random caches when
  source values mutate, and use the C++ `RandomProvider` fallback sequence for
  unseeded formula randoms. The runner now admits grouped Shape x/y binds
  while keeping the zero-duration interpolator transform gate for
  `interpolation_zero_duration.riv`, and `formula_random.riv` is promoted to
  exact after direct C++/Rust stream comparison. `make golden-compare` reports
  `exact=125`, `exact-segments=446`, `diverges=0`, `unsupported-feature=170`,
  `not-yet=0`, and parked `M5=9 M6=118 gated=7 harness=36`;
  `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported nested host artboard binding: `NestedArtboard.artboardId` source-to-target artboard values now rebuild or clear the runtime child instance from shared graph context, draw skips the static nested fallback when a host is data-bound to `-1`, and the runner narrows the nested-host gate to converted or target-to-source host swaps. `recursive_data_bind.riv` is promoted to exact after direct C++/Rust stream comparison, while `databind_artboard.riv` moves to M6 after the same bind now reaches `text`. `make golden-compare` reports `exact=124`, `exact-segments=445`, `diverges=0`, `unsupported-feature=171`, `not-yet=0`, and parked `M5=10 M6=118 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported artboard source-to-target interpolator bindings: artboard property bindings now keep stateful `DataConverterInterpolator` converter state, advance it with scene elapsed time, and apply converted number/color values to target properties. The Rust runner admits source-to-target `SolidColor.colorValue` interpolator binds, and `data_converter_interpolator_reset.riv` plus `time_based_interpolation.riv` are promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=123`, `exact-segments=444`, `diverges=0`, `unsupported-feature=172`, `not-yet=0`, and parked `M5=12 M6=117 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported custom-property enum target-to-source binding: artboard `DataBindContext` custom-property bindings now capture `CustomPropertyEnum.propertyValue` as `RuntimeDataBindGraphValue::Enum`, the runner admits no-converter target-to-source enum binds, and `custom_property_enum.riv` is promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=121`, `exact-segments=442`, `diverges=0`, `unsupported-feature=174`, `not-yet=0`, and parked `M5=14 M6=117 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Retagged text-target nested child binds: nested child data-bind diagnostics now report `text` when the unsupported target is `Text`, `TextValueRun`, or `TextStylePaint`, and `component_stateful.riv` plus `component_stateful_vm_instance_2.riv` moved from M5 to M6 after direct probes showed `TextValueRun` as the first blocker. `make golden-compare` reports `exact=120`, `exact-segments=441`, `diverges=0`, `unsupported-feature=175`, `not-yet=0`, and parked `M5=15 M6=117 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Opened direct no-converter color binds: artboard `DataBindContext` source-to-target bindings now admit `FieldKind::Color` and apply `SolidColor.colorValue` through the runtime color setter, mirroring C++ `src/data_bind/context/context_value_color.cpp`. `collapsable_data_binding.riv` and `scripted_color.riv` are promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=120`, `exact-segments=441`, `diverges=0`, `unsupported-feature=175`, `not-yet=0`, and parked `M5=17 M6=115 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Opened no-converter transform binds: Rust now admits direct Shape x/y and nested child RootBone x/y DataBindContext number targets through the existing artboard property-binding path. `bidirectional_precedence.riv` is promoted to exact after direct C++/Rust stream comparison, and `ai_assitant.riv` is retagged from M5 to `gated` after the RootBone bind probe exposes the existing `feather` renderer diagnostic. `make golden-compare` reports `exact=118`, `exact-segments=439`, `diverges=0`, `unsupported-feature=177`, `not-yet=0`, and parked `M5=19 M6=115 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported stateful nested child number binding: artboard DataBindContext number sources now apply to double/uint targets, nested child artboards refresh Ellipse width/height from host stateful `ViewModelInstanceNumber` values after parent binds run, and `component_stateful_vm_instance.riv` is promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=117`, `exact-segments=438`, `diverges=0`, `unsupported-feature=178`, `not-yet=0`, and parked `M5=21 M6=115 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Opened live data-bound nested host pause: `NestedArtboard.isPaused` source-to-target binding now runs through the existing artboard nested-host binding path, and `pause_nested_artboard.riv` is promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=116`, `exact-segments=437`, `diverges=0`, `unsupported-feature=179`, `not-yet=0`, and parked `M5=22 M6=115 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Ported custom-property trigger keyed-callback target-to-source binding: `CustomPropertyTrigger.fire` increments `propertyValue`, artboard custom-property data binds now read trigger counts, and `custom_property_trigger.riv` is promoted to exact after direct C++/Rust stream comparison. `make golden-compare` reports `exact=115`, `exact-segments=436`, `diverges=0`, `unsupported-feature=180`, `not-yet=0`, and parked `M5=23 M6=115 gated=6 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M5] Opened M5 after draining the M4 queue: all remaining `milestone = "M4"` entries were probed with `rust-golden-runner` and moved to their first verified later diagnostic (`data-binding-nested-child`, `data-binding-nested-host`, `data-binding-custom-property-trigger`, `nested-artboard-layout`, `text`, `images`, `focus-data`, or `layout-component-paint`). `make golden-compare` reports `exact=114`, `exact-segments=435`, `diverges=0`, `unsupported-feature=181`, `not-yet=0`, and parked `M5=24 M6=115 gated=6 harness=36`; `cargo test --workspace` passes.

## M7 Next Archive (completed 2026-07-09)


1. Active `not-yet` and `milestone = "M6"` queues are empty.
   `rewards_demo.riv` is exact-status under
   `verification = "tolerant(0.0005)"`; the tolerance covers residual
   HarfRust/Skrifa text-outline coordinate drift only.
2. Initial M7 public Rust API crate exists at `crates/rive`: `File::import`,
   artboard listing/selection, artboard instantiation, one-shot advance/draw
   through the renderer traits, and raw runtime/graph escape hatches. First C
   ABI facade exists at `crates/rive-capi` with import/free and artboard
   metadata functions. `make perf-compare`, `make perf-corpus`, and
   `make perf-hot-loop` build release C++/Rust runners by default, and
   `perf-hot-loop` scores runner-emitted whole-repeat `total_ms` rather than
   wall-clock process time, stream-serialization time, or per-frame phase timer
   overhead. Both runners have null-renderer benchmark backends, so M7 perf
   checks exclude golden recording output. Both runners also support
   `--benchmark-repeat N` for long single-sample profiling runs. A release
   `ai_assitant.riv` profile found fixed schema-name property
   lookup in the paint/path hot paths; caching fixed paint keys previously
   reduced Rust direct `ai_assitant` 100-segment repeat time from about
   1019 ms to about 255 ms. Follow-up path-geometry key caching,
   repeat-aware `perf-compare`, removal of `artboard_data_bind.rs`
   hot-loop graph/binding clones, shallow sharing of immutable
   animation/state-machine definition vectors, an epoch-keyed retained
   prepared draw-command frame, epoch-keyed retained draw `RenderPath`
   handles, cached fixed layout/schema property keys, and cached fixed
   data-bind property keys now give focused
   10-iteration verification with `make perf-hot-loop PERF_CORPUS_LIMIT=5
   PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999` at aggregate
   Rust/C++=3.096 over 5 exact entries / 10 segments (`ai_assitant`=3.347,
   `align_target`=1.947, `animated_clipping`=2.711). This repeat=1 focused
   ratio is noisy and strict `PERF_MAX_RATIO=2.0` still fails by inspection.
   M7 perf is now explicitly defined as steady-state per-frame runtime cost;
   direct `ai_assitant` with `--benchmark-repeat 100` now reports
   Rust/C++=34.736 on the current 10-iteration run
   (cpp median=0.543 ms, rust median=18.878 ms), confirming retained
   frame/path preparation and cached keys are real clean-frame wins but still
   far from the strict target. Generated schema kind/property switch tables now
   remove the remaining linear schema/type lookup from the hot
   `RuntimeFile::data_bind_path_for_referencer_object`,
   `InstanceObjectArena::set_property_value` / `property_kind`, and layout/draw
   property helper paths; focused 10-iteration verification now reports
   aggregate Rust/C++=2.543 over the same 5 exact entries / 10 segments
   (`ai_assitant`=2.611, `align_target`=1.831, `animated_clipping`=2.460).
   Direct `ai_assitant --benchmark-repeat 100` improves to Rust/C++=17.233
   (cpp median=0.625 ms, rust median=10.766 ms). A fresh release sample then
   split Taffy layout bounds behind a `layout_epoch`, mirroring C++
   `markLayoutNodeDirty` without invalidating layout for paint/color and
   non-text string updates; text-shape string/style changes and fractional
   layout sizing still invalidate layout like C++. Focused 10-iteration
   verification after the text/fractional safety pass reports aggregate
   Rust/C++=2.699 over the same 5 exact entries / 10 segments
   (`ai_assitant`=2.785, `align_target`=2.399,
   `animated_clipping`=2.406). Direct `ai_assitant --benchmark-repeat 100`
   now reports Rust/C++=13.850 (cpp median=0.591 ms, rust median=8.183 ms);
   C++ median variance makes the ratio noisy, but Rust steady-state time
   improved. Retained gradient preparation in `RuntimeRenderPathCache` now
   caches graph-static gradient mutator buckets and dependency-order vectors
   instead of rebuilding them every paint-prep pass. Focused 10-iteration
   verification reports aggregate Rust/C++=2.647 over the same 5 exact entries
   / 10 segments (`ai_assitant`=2.906, `align_target`=1.832,
   `animated_clipping`=2.400). Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.398 ms, rust median=7.700 ms, Rust/C++=19.356; the
   ratio remains C++-median-sensitive, but Rust steady-state time improved.
   Retained render-paint draw configuration in `RuntimeRenderPaintCache` now
   records the last persistent paint type/stroke/blend/shader/feather config,
   skips redundant draw-time paint setters, and invalidates that config when
   gradient preparation mutates a retained paint. Focused 10-iteration
   verification reports aggregate Rust/C++=2.518 over the same 5 exact entries
   / 10 segments (`ai_assitant`=2.583, `align_target`=1.864,
   `animated_clipping`=2.422). Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.393 ms, rust median=7.341 ms, Rust/C++=18.668.
   A path-specific retained draw-path epoch now separates `RenderPath` rebuild
   invalidation from broad prepared-frame/paint invalidation:
   `RuntimeRenderPathCache::draw_path` uses `ArtboardInstance::path_epoch`,
   bumped by path/vertices/world-transform/layout/NSlicer dirt, collapse, and
   C++ `StrokeEffect`-style TrimPath/DashPath/Dash/Feather path-affecting
   property changes, including Feather `inner`/`spaceValue` because they change
   the cached inner-feather command stream. Paint-only changes no longer rebuild
   retained draw paths, while animated trim/dash/effect paths still invalidate
   correctly. Focused 10-iteration verification reports aggregate
   Rust/C++=2.405 over the same 5 exact entries / 10 segments
   (`advance_blend_mode`=4.554, `ai_assitant`=2.533,
   `align_target`=1.663, `animated_clipping`=2.266,
   `animation_reset_cases`=3.966). Direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.363 ms, rust
   median=7.695 ms, Rust/C++=21.222.
   A 2026-07-08 scout implementation of a Rust-only `Shape` paint
   path-command cache was intentionally not landed. While present it kept
   focused tests, `make golden-compare`, and `cargo test --workspace` green,
   but the fenced release hot-loop did not show a completion-grade win:
   focused 5-entry aggregate moved to Rust/C++=2.588, and direct
   `ai_assitant --benchmark-repeat 100` reported cpp median=0.555 ms, rust
   median=10.197 ms, Rust/C++=18.375. The useful finding is the layer
   boundary: caching cloned `Vec<RuntimePathCommand>` above
   `RuntimeShapePaintCommand` is not the C++ optimization. The next landing
   slice should either make steady frames skip prepare via audited
   idempotent dirt raisers, or port actual `RawPath`/`PathComposer`
   retention behind C++ dirt gates.
   A follow-up scout that retained artboard/background/layout/clip
   `RenderPath` handles behind the existing layout/path epochs was also
   intentionally not landed. It kept focused tests and `make golden-compare`
   green, but the fenced release/null-renderer perf gate moved the focused
   aggregate to Rust/C++=2.705 and then 3.338; direct
   `ai_assitant --benchmark-repeat 100` was only neutral at Rust/C++=19.424.
   Treat this as too shallow a layer: clip/layout/background path rebuild
   gating can wait until the lower-level `ShapePaintPath`/`PathComposer`
   retention has landed or a profile shows it on the hot path.
   A second lower-level scout that converted `RuntimeShapePaintCommand`
   path/effect/inner-feather payloads to shared `Arc<[RuntimePathCommand]>`
   slices and cached shape paint path-command buffers by
   `(graph, shape, path kind, path_epoch, layout_epoch)` was also backed out.
   It preserved `make golden-compare` at exact=263 / exact-segments=584 /
   diverges=0 and kept the focused path/probe tests green, but the fenced
   release/null-renderer aggregate stayed worse than the current baseline:
   Rust/C++=2.627 and 2.619. Direct `ai_assitant --benchmark-repeat 100`
   improved only to Rust/C++=18.598. The next attempt should stop clean
   frames from entering prepare at all via audited C++ dirt gates, or port
   actual `PathComposer`/raw-path retention, not wrap prepared command vectors.
   Retained path-geometry command frames now live on
   `RuntimeRenderPathCache` by `(graph_global_id, path_local)` and
   `(path_epoch, layout_epoch)`, so prepared-frame rebuilds reuse C++
   `ShapePaintPath` / `RawPath`-shaped runtime geometry command streams for
   clean paths instead of rerunning `runtime_path_geometry` and
   `path_commands` for every paint path. Shape paint paths still transform,
   NSlice, reverse, and prune per composed path. Collapse checks now use a
   component-count cycle guard instead of allocating a `BTreeSet`, and
   RawPath-style empty-segment pruning compacts in place. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Same-turn fenced repeat-aware hot-loop improves
   from aggregate Rust/C++=3.809 to 3.616. Strict <=2.0 remains open. Next:
   profile remaining `advance_blend_mode` / `animation_reset_cases` fixed
   overhead and `ai_assitant` advance/data-bind after this path-retention
   cleanup; likely next targets are prepared-frame clean-skip/idempotent dirt
   or sampled data-bind/context hotspots, not shallow command-vector caches.
   Nested-artboard layout bounds are now retained on `ArtboardInstance` by
   `(graph_global_id, layout_epoch)`, matching the C++ `markLayoutNodeDirty`
   / `Artboard::markLayoutDirty` boundary for layout recomputation during
   nested advance. Focused release/null-renderer verification reports
   aggregate Rust/C++=2.329 over the same 5 exact entries / 10 segments
   (`advance_blend_mode`=5.649, `ai_assitant`=2.221,
   `align_target`=1.888, `animated_clipping`=2.461,
   `animation_reset_cases`=4.264). Two direct
   `ai_assitant --benchmark-repeat 100` checks report about Rust/C++=19.5-20.0
   (rerun cpp median=0.595 ms, rust median=11.919 ms, Rust/C++=20.018), so the
   strict <=2.0 target remains open and long-repeat Rust median is still noisy.
   `RuntimeRenderPaintCache` now also records a paint-preparation key
   `(graph_global_id, cache_epoch)` and skips repeated non-dependency-order
   paint preparation when no Rust property setter or component dirt raiser
   changed the instance since the last prepare, matching C++'s clean-frame
   `updateComponents` early-out at Rust's conservative cache epoch boundary.
   Focused release/null-renderer runs over the same 5 exact entries / 10
   segments reported aggregate Rust/C++=2.493, 1.832, and 2.166; direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.582-0.603 ms,
   rust median=5.149-5.885 ms, Rust/C++=8.852-9.756. This is a real
   steady-state Rust win, but strict <=2.0 is still not reliable on the focused
   corpus. A follow-up macOS `sample` profile of
   `ai_assitant --benchmark-repeat 100000` showed the remaining Rust time
   dominated by advance/data-bind, especially owned view-model nested artboard
   context-chain rebinding and property-path allocation. A narrow allocation
   cleanup now avoids the extra collected `Vec` while resolving context source
   paths and avoids staging owned-view-model artboard binding updates in a
   temporary vector. Direct `ai_assitant --benchmark-repeat 100` reports rust
   median=4.553-4.764 ms (Rust/C++=7.731-9.399), and the Rust-only
   repeat=100000 run moves from elapsed=4437.5 / advance=3476.3 ms to
   elapsed=3840.8 / advance=2936.9 ms. Focused release/null-renderer runs were
   still noisy (aggregate Rust/C++=2.517 and 2.776), so strict <=2.0 remains
   open. Rust nested owned-view-model binding now passes borrowed context-path
   slices instead of cloning a `Vec<Vec<usize>>` chain on every nested host,
   matching C++ `DataContext` parent-chain lookup more closely without adding
   new skip semantics. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0. Direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.603 ms, rust
   median=4.348 ms, Rust/C++=7.210; a fresh baseline worktree for the prior
   commit ran Rust-only repeat=100000 at elapsed=4235.4 / advance=3275.3 ms,
   while this slice runs elapsed=4109.3 / advance=3120.9 ms. Focused
   release/null-renderer is still not completion-grade but moves to aggregate
   Rust/C++=2.321. The next M7 target should stop doing context-chain allocation
   cleanup and port actual C++ data-bind dirt retention: `DataBind::addDirt`,
   `DataBindContainer` dirty queues, and push-driven target-to-source updates.
   A follow-up scout that added naive `target_dirty` bits directly to
   artboard property/image bindings was intentionally not landed: it kept
   focused probes and `make golden-compare` green, but repeat-heavy
   `ai_assitant` regressed to Rust/C++=10.962 and 15.381, Rust-only
   repeat=100000 regressed to elapsed=4766.0 / advance=3385.2 ms, and the
   focused 5-entry ratio moved to Rust/C++=2.614. The next attempt should port
   the actual C++ container lists and enrollment semantics, not add per-binding
   dirty booleans around the current scans.
   Artboard source-to-target property/image binds now have container-owned
   dirty target queues indexed by source path, seeded for initial apply and
   enrolled through `set_artboard_data_bind_value_for_path`, formula token /
   operation converter updates, and stateful converter advance. This mirrors
   the C++ `DataBindContainer` dirty-list shape for the source-to-target
   subset without yet moving polling target-to-source binds onto push queues.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, and `cargo test --workspace` passes. A same-session
   throwaway worktree at `988fc29` measured Rust-only repeat=100000 at
   elapsed=3080.3 / advance=2392.7 ms and focused hot-loop aggregate
   Rust/C++=2.723; this slice measures Rust-only repeat=100000 at
   elapsed=2480.2 / advance=1859.1 ms and focused hot-loop aggregate
   Rust/C++=2.371 / 2.599. Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.666 ms, rust median=4.851 ms, Rust/C++=7.279.
   Artboard target-to-source binds now have container-owned source queues:
   generated property setters enroll push-capable custom-property and direct
   numeric source binds, source-to-target applies suppress self-notification by
   data-bind index like C++ `DataBind::suppressDirt`, computed layout/solo/shape
   sources stay on polling fallback, and converter-backed custom sources stay
   on a conservative persisting lane until every converter dirty edge is modeled
   explicitly. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, and `cargo test --workspace` passes.
   Focused release/null-renderer hot-loop runs report aggregate Rust/C++=2.784
   and 2.500; direct repeat-heavy `ai_assitant --benchmark-repeat 100` reports
   cpp median=0.569 ms, rust median=4.019 ms, Rust/C++=7.060. Strict <=2.0
   remains open. Next: profile remaining advance/data-bind time, then replace
   the converter-backed custom persisting fallback with explicit C++
   converter-parent dirty edges before widening this queue pattern further.
   A narrower follow-up landed only the audited OperationViewModel-number
   converter-parent dirty edge for artboard source path changes. Converter-backed
   custom-property sources intentionally remain on the conservative persisting
   lane: a broader RangeMapper/converter-property scout was backed out after it
   drove wrong `db_health_tracker` clip positions, which confirms global
   DataBindContext converter-property writes are not the safe fallback-removal
   path. Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace` passes, and fenced hot-loop reports
   aggregate Rust/C++=2.592. Next: enumerate and port concrete C++
   converter-parent dirty edges one converter family at a time before removing
   the persisting fallback.
   Converter-backed custom-property sources now narrow that fallback by
   converter family instead of treating every converter as persisting:
   `PassThrough`, `BooleanNegate`, `TriggerIncrement`, `ToNumber`,
   `ListToLength`, `StringRemoveZeros`, `Formula`, and groups containing only
   push-safe children leave the conservative polling lane. Families with
   unmodeled converter-owned dirt edges remain persisting: `NumberToList`,
   `ToString`, operation-view-model/system operation, `Rounder`, `RangeMapper`,
   `StringTrim`, `StringPad`, `Interpolator`, and unsupported converters.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace` passes, and fenced hot-loop improves
   the current focused aggregate to Rust/C++=2.409. Strict <=2.0 remains open.
   Next: port concrete C++ converter-property dirty setters family-by-family,
   then shrink this predicate again.
   `DataConverterOperationValue.operationValueChanged()` is now the next
   landed concrete family: artboard `OperationValue` converters leave the
   persisting lane because Rust already updates them through
   `set_artboard_operation_value`, resets formula randoms, and enqueues
   dependent property/custom parents. System-operation subclasses remain
   conservative because their bind-target path is not modeled by that exact
   updater. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace` passes, and
   fenced hot-loop improves the focused aggregate to Rust/C++=2.201. Strict
   <=2.0 remains open.
   `DataConverterToString::{decimals,colorFormat}Changed()` is now modeled as
   a family-specific converter-property dirty edge: imported ToString
   converter-property binds are queued by source path, seeded for initial
   application, update dependent copied converters by target converter id, and
   enqueue their property/custom parents without broad DataBindContext
   converter-property writes. `ToString` now leaves the converter-backed custom
   persisting lane. Remaining conservative families are `NumberToList`,
   operation-view-model/system operation, `Rounder`, `RangeMapper`,
   `StringTrim`, `StringPad`, `Interpolator`, and unsupported converters. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0,
   `cargo test --workspace` passes, and fenced hot-loop is noisy but roughly
   neutral at aggregate Rust/C++=2.238 then 2.195. Strict <=2.0 remains open.
   `DataConverterStringTrim::trimTypeChanged()` is now modeled through the same
   family-specific converter-property dirty lane: imported StringTrim `trimType`
   binds are queued by source path, seeded for initial application, update
   dependent copied converters by target converter id, and enqueue their
   property/custom parents without broad DataBindContext converter-property
   writes. `StringTrim` now leaves the converter-backed custom persisting lane.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `Rounder`, `RangeMapper`, `StringPad`, `Interpolator`, and
   unsupported converters. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass, and fenced
   hot-loop is noisy but roughly neutral at aggregate Rust/C++=2.173 then
   2.239. Strict <=2.0 remains open.
   `DataConverterStringPad::{length,text,padType}Changed()` is now modeled
   through the same family-specific converter-property dirty lane: imported
   StringPad binds are queued by source path, seeded for initial application,
   update dependent copied converters by target converter id, and enqueue their
   property/custom parents without broad DataBindContext converter-property
   writes. `StringPad` now leaves the converter-backed custom persisting lane.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `Rounder`, `RangeMapper`, `Interpolator`, and unsupported
   converters. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass, and fenced
   hot-loop is noisy but roughly neutral at aggregate Rust/C++=2.120 then
   2.186. Strict <=2.0 remains open.
   `DataConverterRounder.decimalsChanged()` is generated but empty in C++, and
   the handwritten Rounder class does not override it. Rust therefore removes
   Rounder custom sources from the conservative polling lane without adding a
   converter-property updater: there is no C++ `DataConverter::markConverterDirty`
   edge to model for this family. The status review keeps the scout and perf
   methodology discoveries in force: broad converter-property writes remain
   rejected after the `db_health_tracker` RangeMapper scout, shallow cached
   command/path wrappers remain rejected by fenced perf, and M7 decisions now
   use release/null-renderer hot-loop whole-repeat `total_ms` scoring.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `RangeMapper`, `Interpolator`, and unsupported converters.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0,
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass, and fenced hot-loop reports aggregate
   Rust/C++=2.310 then 2.181. Strict <=2.0 remains open.
   A 2026-07-08 family-specific `RangeMapper` scout was intentionally not
   landed. It added artboard converter-property bindings only for the four C++
   dirty callbacks (`minInputChanged()`, `maxInputChanged()`,
   `minOutputChanged()`, `maxOutputChanged()`), kept `flags`,
   `interpolationType`, and `interpolatorId` out of the lane because their
   generated callbacks are empty, and moved RangeMapper custom sources out of
   the persisting fallback. `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime queues`, and
   `cargo test -p rive-runtime range_mapper` passed, but full
   `make golden-compare` failed `db_health_tracker` at line 3390 with the same
   clip-path x-position drift as the earlier broad RangeMapper scout
   (Rust first point x=48.2119293 vs C++ x=64.2483139). The code was backed out.
   Treat RangeMapper as requiring deeper C++ DataBind/DataConverter ownership
   and ordering analysis before another fallback-removal attempt; do not retry
   the StringPad-style copied-converter updater for this family.
   `DataConverterInterpolator.durationChanged()` is now modeled through the
   same family-specific converter-property dirty lane: C++ only marks dirty for
   `durationChanged()`, while generated `interpolationTypeChanged()` and
   `interpolatorIdChanged()` are empty. Imported Interpolator `duration` binds
   are queued by source path, seeded for initial application, update dependent
   copied converters by target converter id, and enqueue their property/custom
   parents without broad DataBindContext converter-property writes; the existing
   stateful-converter advance queue continues to cover the in-flight
   `InterpolatorAdvancer::advance()` dirty edge. `Interpolator` now leaves the
   converter-backed custom persisting lane.
   `DataConverterNumberToList.viewModelIdChanged()` is now modeled through a
   family-specific converter-property dirty lane: C++ clears cached
   `m_listItems` and marks the converter dirty; Rust has no persistent
   `ViewModelInstanceListItem` cache in this layer, so copied NumberToList
   converters store `view_model_id` plus the file's view-model count and
   recompute list size from the current id each conversion. Imported
   NumberToList `viewModelId` binds are queued by source path, seeded for
   initial application, update dependent copied converters by global id, and
   enqueue their concrete property/custom/list parents without broad
   DataBindContext converter-property writes. `NumberToList` now leaves the
   converter-backed custom persisting lane.
   Operation-view-model and system-operation custom sources now leave the
   conservative persisting lane. The status-doc scout review keeps the
   RangeMapper and perf-methodology fences in force: no broad DataBindContext
   converter-property writes, no StringPad-style RangeMapper retry, and no
   shallow command/path-wrapper caching without release/null-renderer hot-loop
   evidence. `DataConverterOperationViewModel` has no C++ converter-property
   dirty callback to model here; Rust relies on the existing source-path
   dependent refresh edges. `DataConverterSystemDegsToRads` and
   `DataConverterSystemNormalizer` inherit `DataConverterOperationValue`, so
   their `operationValue` binds now use the same explicit updater by converter
   global id and enqueue concrete parents. Remaining conservative families are
   `RangeMapper` and unsupported converters. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass; fenced hot-loop
   reports aggregate Rust/C++=2.111. Strict <=2.0 remains open. Next: profile
   remaining advance/data-bind time or audit RangeMapper C++ ownership/update
   order before another fallback-removal attempt.
   A follow-up release `ai_assitant --benchmark-repeat 1000000` sample found
   the remaining Rust time still concentrated in owned view-model/nested
   artboard data-context propagation: `bind_owned_view_model_artboard_context_chain`,
   `sync_nested_child_artboard_data_contexts`, and
   `RuntimeFile::data_bind_path_for_referencer_object`. Rust now mirrors C++'s
   pointer-walking context propagation more closely by no longer cloning nested
   child property/image binding vectors during every sync pass, and by using
   fixed cached generated-property keys for the sampled nested-host
   `ViewModelInstance*` lookups instead of the generic name dispatcher.
   Long-repeat Rust-only `ai_assitant` improves from elapsed=2095.6 /
   advance=1602.2 ms to elapsed=1734.9 / advance=1233.3 ms for 100000
   segments. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop is still noisy/neutral at aggregate
   Rust/C++=2.119, so strict <=2.0 remains open. Next: port the C++ retained
   `DataBindPath`/data-context lookup shape for nested hosts so Rust stops
   resolving `data_bind_path_for_referencer_object` inside the steady frame.
   Nested host `DataBindPath` resolution is now retained on
   `RuntimeNestedArtboardInstance`, including dynamic `artboardId` swaps,
   mirroring C++ `NestedArtboard::dataBindPath()` plus lazy
   `DataBindPath::resolvedPath()` retention. The steady owned-view-model nested
   context path now consumes the retained path slice instead of calling
   `RuntimeFile::data_bind_path_for_referencer_object`. This is immutable
   import/build data retention, not a new skip/cache invalidation rule. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Long-repeat Rust-only `ai_assitant` improves from
   elapsed=1734.9 / advance=1233.3 ms to elapsed=1408.5 / advance=902.0 ms for
   100000 segments. Fenced release/null-renderer hot-loop reports aggregate
   Rust/C++=2.338 then 2.430, so strict <=2.0 remains open. Next: profile
   remaining release/null-renderer advance/data-bind time after the retained
   path change while keeping the scout fences in force: no broad
   DataBindContext converter-property writes, no StringPad-style RangeMapper
   retry, and no shallow command/path-wrapper caching without fenced evidence.
   A fresh release sample after the retained-path slice found the remaining
   Rust time still concentrated in owned view-model nested artboard
   data-context propagation, with allocations in context-source path lookup and
   smaller retained animation/state-machine name clone/drop traffic. Rust now
   walks numeric owned-view-model context-source paths directly through active
   child property lists, retaining the existing name-based fallback; nested
   host context-chain prepends use stack storage for the common shallow case;
   artboard property/image/custom binding paths are cloned only after equality
   proves a changed value; and retained linear-animation/state-machine names
   share `Arc<str>` definitions. This follows C++ `DataContext` pointer walks
   and immutable definition sharing without adding new invalidation or skip
   caching. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct
   Rust-only `ai_assitant --benchmark-repeat 100000` improves from
   elapsed=1408.5 / advance=902.0 ms to elapsed=1225.7 / advance=706.0 ms.
   Single-file release/null-renderer `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.429 ms, rust median=1.928 ms, Rust/C++=4.496, and the
   focused 5-entry hot-loop reports aggregate Rust/C++=2.363. Strict <=2.0
   remains open. Next: profile the remaining
   `bind_owned_view_model_artboard_context_chain` /
   `collect_nested_artboard_context_source_values` time and continue only
   audited C++ retention/dirt slices; keep the scout fences in force: no broad
   converter-property writes, no StringPad-style RangeMapper retry, and no
   shallow command/path-wrapper caching without release/null-renderer evidence.
   Nested artboard context-source propagation now uses a single accumulator
   while walking descendants instead of allocating a descendant-value `Vec` at
   each host, cloning that vector into the parent, and then replaying the owned
   values into the child. Nested host loops also walk the retained host map
   directly instead of snapshotting keys into a temporary vector. This mirrors
   C++'s retained object traversal shape and does not add skip/cache
   invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11839.2 /
   advance=6853.7 ms to elapsed=11624.2 / advance=6594.8 ms. Focused
   release/null-renderer hot-loop reports aggregate Rust/C++=2.281 over the
   5-entry / 10-segment corpus, while single-file repeat=100 reports
   cpp median=0.373 ms, rust median=1.944 ms, Rust/C++=5.207. Strict <=2.0
   remains open. Next: profile and port the remaining child-context
   construction in `bind_owned_view_model_artboard_context_chain` rather than
   broadening to fenced-off converter/property caches.
   Nested owned-view-model child-context paths now use borrowed/inline stack
   storage for the common numeric `DataBindPath` case instead of allocating a
   `Vec<usize>` before prepending the child to the parent context chain. This
   keeps C++'s `DataContext::getViewModelInstance` shape: check the current
   context root, walk the numeric tail, then fall through to parent contexts;
   it does not add a new skip/cache invalidation rule. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Focused release/null-renderer hot-loop improves
   the 5-entry / 10-segment aggregate from Rust/C++=2.281 to 2.169
   (`ai_assitant`=1.921). Direct Rust-only `ai_assitant
   --benchmark-repeat 1000000` is noisy but roughly neutral/slightly improved
   in one same-session run, elapsed=11715.7 / advance=6692.8 ms before and
   elapsed=11580.4 / advance=6636.0 ms after. Strict <=2.0 remains open.
   Next: profile remaining `bind_owned_view_model_artboard_context_chain` and
   `collect_nested_artboard_context_source_values` time, keeping the scout
   fences in force.
   Source-producing artboard data-bind paths are now shared as immutable
   `Arc<[u32]>` slices for custom-property, layout-computed, solo-source, and
   nested context-source values, so context-source propagation clones a retained
   path handle instead of allocating a fresh path `Vec`. This is import/build
   data sharing, not a skip/cache invalidation rule. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Focused release/null-renderer hot-loop is a tiny/noisy aggregate improvement
   from Rust/C++=2.169 to 2.164 over the 5-entry / 10-segment corpus
   (`ai_assitant`=1.936). Direct Rust-only `ai_assitant
   --benchmark-repeat 1000000` is noisy/slightly worse at elapsed=11884.7 /
   advance=6753.7 ms, and single-file repeat=100 JSON at
   `/tmp/rive-ai-shared-paths-perf.json` reports cpp median=0.371 ms, rust
   median=1.988 ms, Rust/C++=5.363. Strict <=2.0 remains open. Next: profile
   remaining advance/data-bind time before landing more context allocation
   cleanup; if no clear C++ retention/dirt slice appears, move to a
   higher-leverage audited dirt/retention target.
   Nested-host source locals for child data-context sync are now retained on
   `RuntimeNestedArtboardInstance` by child binding path and rebuilt during
   dynamic `artboardId` swaps. `sync_nested_child_artboard_data_contexts`
   consumes the retained path-to-local map and falls back to the old slot walk
   for unresolved paths; that loop also walks the retained host map directly
   instead of snapshotting keys into a temporary vector. This mirrors C++
   `DataContext`/view-model instance pointer walks without adding a new
   skip/cache invalidation rule. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11884.7 /
   advance=6753.7 ms to elapsed=11658.6 / advance=6683.3 ms. Single-file
   repeat=100 JSON at `/tmp/rive-ai-retained-source-locals-perf.json` reports
   cpp median=0.389 ms, rust median=1.898 ms, Rust/C++=4.880. Focused
   release/null-renderer hot-loop is improved but still noisy at aggregate
   Rust/C++=2.024 then 2.253, so strict <=2.0 remains open. Next: profile the
   remaining advance/data-bind time again before choosing between another
   audited retained data-context lookup and a higher-leverage dirt/retention
   target.
   Nested-host root `ViewModelInstance` locals are now retained on
   `RuntimeNestedArtboardInstance` by `viewModelId`, rebuilt with dynamic
   `artboardId` swaps, and reused by child data-context sync before walking
   property values. Successful fallback source-local resolutions are also
   retained by binding path even when the current sync pass does not materialize
   a value. This mirrors C++ `DataContext` root pointers plus
   `DataBindPath::resolvedPath()` retention without adding a negative cache or
   invented skip invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11658.6 /
   advance=6683.3 ms to elapsed=9791.6 / advance=4650.2 ms. Single-file
   repeat=100 JSON at `/tmp/rive-ai-root-locals-perf-final.json` reports
   cpp median=0.463 ms, rust median=1.717 ms, Rust/C++=3.710. A 3M sample
   shows `stateful_nested_host_value_local_for_slots` dropping from 775 to 48
   samples; the new top is draw/prepare plus schema `definition_by_name`,
   `bind_owned_view_model_artboard_context_chain`, and BTree range lookups.
   Focused release/null-renderer hot-loop reports aggregate Rust/C++=2.136
   (`ai_assitant`=1.852), so strict <=2.0 remains open. Next: profile/port the
   remaining C++-audited retained/dirt targets in
   `bind_owned_view_model_artboard_context_chain` / context-source BTree
   lookups or address remaining schema lookup in sampled draw/prepare without
   violating the scout fences.
   Draw-time drawable classification now avoids schema reflection for the two
   sampled hot checks: render-opacity filtering uses the fixed `Shape` /
   `TextInputDrawable` class surface, and nested-artboard dispatch uses the
   three concrete nested host classes, matching C++'s concrete `is<T>()` /
   type-key shape instead of calling `definition_by_name(...).is_a(...)` in
   the frame loop. This removes a sampled draw/prepare schema lookup without
   adding a skip cache or changing dirty invalidation. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Single-file repeat=100 JSON at
   `/tmp/rive-ai-draw-kind-perf.json` reports cpp median=0.465 ms,
   rust median=1.675 ms, Rust/C++=3.603. Focused release/null-renderer
   hot-loop reports aggregate Rust/C++=2.243 then 2.133, so strict <=2.0
   remains open. Next: re-profile after this schema-classification cleanup and
   pick the next C++-audited retained/dirt or hot BTree target from the new
   sample.
   Fixed draw/image/mesh/nested property keys now also route through the
   cached runtime draw-key helper instead of calling the generic
   `property_key_for_name` dispatcher for literal schema pairs in the frame
   loop. This covers `DrawRules.drawTargetId`, `DrawTarget.placementValue`,
   `Drawable.blendModeValue`, `Image.assetId/origin/fit/alignment`,
   `Vertex.x/y`, `NestedArtboard.artboardId`, `NestedArtboardLeaf`
   fit/alignment, and `Artboard.opacity`. It is C++-shaped generated-key
   retention, not a new skip/cache invalidation rule. Full `make
   golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, focused draw tests, `cargo fmt --all -- --check`,
   and `git diff --check` pass. Single-file repeat=100 JSON at
   `/tmp/rive-ai-draw-property-keys-perf.json` reports cpp median=0.531 ms,
   rust median=1.586 ms, Rust/C++=2.985. Focused release/null-renderer
   hot-loop reports aggregate Rust/C++=2.187 then 2.122, so strict <=2.0
   remains open. Next: re-profile after this fixed-key cleanup and choose
   between the sampled hot BTree/draw-order path and remaining audited
   data-context retention work.
   Sorted drawable order is now retained in `RuntimeRenderPathCache` by
   `(graph_global_id, draw_order_epoch)`, and `draw_order_epoch` is bumped by
   the C++ DrawOrder dirt raisers for `DrawRules.drawTargetId` and
   `DrawTarget.placementValue` through `ComponentDirt::DRAW_ORDER`. This keeps
   prepared command rebuilds from reconstructing draw-target BTree groupings
   when only paint/data-bind/cache epoch changes. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; focused draw-order and
   draw tests, `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Direct `ai_assitant --benchmark-repeat 100` JSON
   at `/tmp/rive-ai-sorted-draw-order-perf.json` reports cpp median=0.522 ms,
   rust median=1.592 ms, Rust/C++=3.051; focused hot-loop reports aggregate
   Rust/C++=2.136 then 2.107, so strict <=2.0 remains open. Next: re-profile;
   likely remaining targets are draw command/prepare retention below sorted
   order and data-context BTree/range work. Keep the scout/perf fences in
   force: no broad converter-property writes, no StringPad-style RangeMapper
   retry, and no shallow command/path-wrapper caching without
   release/null-renderer evidence.
   Nested host traversal now retains `nested_artboard_locals` on
   `ArtboardInstance`, replacing repeated BTree key collection and range cursor
   walks in nested advance, owned view-model context binding, nested
   context-source propagation, and nested child data-context sync. The retained
   ordered host list mirrors the current nested map keys and is updated only
   when dynamic `artboardId` swaps create or remove a child instance. This
   follows C++ retained object traversal and does not add new dirty or skip
   semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused nested/data-bind tests, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop reports aggregate Rust/C++=2.017, and a
   closeout rerun reports aggregate Rust/C++=2.093
   (`advance_blend_mode`=6.239, `ai_assitant`=1.820,
   `align_target`=1.750, `animated_clipping`=2.337,
   `animation_reset_cases`=4.105; rerun `ai_assitant`=1.877). A Rust-only
   `ai_assitant --benchmark-repeat 3000000` sample at
   `/tmp/rive-ai-retained-host-locals.sample.txt` reports elapsed=24583.5 ms,
   advance=12061.8 ms, prepare=5582.8 ms, draw=6771.0 ms, and no longer shows
   `find_leaf_edges_spanning_range` or `BTreeMap::` in the sampled nested host
   traversal. Strict <=2.0 remains open. Next: profile the remaining
   advance/data-bind time, especially
   `bind_owned_view_model_artboard_context_chain` and
   `advance_artboard_data_binds_with_root_transform`, plus draw
   command/prepare retention below sorted order under the scout/perf fences.
   Prepared shape-paint commands now retain their deterministic draw-path
   cache slot indices instead of rebuilding the per-command path-slot vector
   during every draw replay. Dynamic text shape-paint commands still assign
   slots when the transient text commands are generated. This removes the
   sampled `runtime_cached_path_slot_index` / `RawVec::grow_one` draw replay
   site without changing draw invalidation or skip semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused draw/path tests, `cargo test --workspace`, `cargo fmt
   --all -- --check`, and `git diff --check` pass. Fenced release/null-renderer
   hot-loop reports aggregate Rust/C++=2.136 with `ai_assitant`=1.972. A
   Rust-only `ai_assitant --benchmark-repeat 3000000` sample at
   `/tmp/rive-ai-draw-slot.sample.txt` reports elapsed=23857.3 ms,
   advance=12268.1 ms, prepare=5482.9 ms, draw=5939.5 ms, and no longer shows
   `runtime_cached_path_slot_index` in the sampled draw replay. Strict <=2.0
   remains open. Next: profile and port remaining data-bind/context-chain
   hotspots first, especially `advance_artboard_data_binds_with_root_transform`
   and `bind_owned_view_model_artboard_context_chain`; draw-side leftovers are
   now lower-level `runtime_configure_paint_with_cache` and
   `RuntimeRenderPathCache::draw_path`.
   Nested-host data-bind application now reuses retained binding entries
   instead of cloning `artboard_nested_host_bindings` every advance, and nested
   child data-context sync now drops same-value child updates before allocating
   update work. This is a direct no-op removal around C++'s retained
   `DataBind`/`DataContext` references, not a new dirty gate. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   focused nested/data-bind tests, `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced release/null-renderer
   hot-loop reports aggregate Rust/C++=2.105 with `ai_assitant`=1.939. Strict
   <=2.0 remains open. Next: profile and port the larger
   `bind_owned_view_model_artboard_context_chain` path-resolution hotspot.
   A follow-up scratch-reuse scout for name-based owned view-model context
   source paths was intentionally not landed. It reused a caller-owned
   `Vec<usize>` while resolving `bind_owned_view_model_artboard_context_chain`
   and kept focused nested/data-bind tests plus `make golden-compare` green
   at exact=263 / exact-segments=584 / diverges=0, but fenced
   release/null-renderer hot-loop rejected it: aggregate Rust/C++ worsened to
   2.250 and then 2.478. The useful finding is the layer boundary: C++
   `DataBindContext::bindFromContext` resolves and retains a concrete
   `ViewModelInstanceValue` source via `DataContext`; Rust's artboard owned
   context path still recomputes the context-chain source lookup every frame.
   Next: port a C++-aligned retained source binding/rebind path for artboard
   owned-context data binds, or first write that design if the invalidation
   surface is too large for one safe slice.
   Artboard owned-context data binds now retain their resolved source
   property path on property/image/custom bindings behind an
   `(owned view-model index, context-chain)` key. This mirrors the C++
   `DataBindContext::bindFromContext` source-retention boundary without
   adding a value-read skip: each frame still reads the current source value
   and routes changes through the existing data-bind value/target queues.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused nested/data-bind tests, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop reports noisy aggregate Rust/C++=2.154
   then 2.438, but the targeted direct `ai_assitant --benchmark-repeat 100`
   JSON at `/tmp/rive-ai-retained-owned-source-paths-perf.json` reports
   cpp median=0.442 ms, rust median=1.420 ms, Rust/C++=3.212. Strict <=2.0
   remains open. Next: profile the remaining `ai_assitant` advance/data-bind
   time after retained source paths; likely targets are C++-aligned rebind
   gating for owned context chains or remaining nested context-source
   propagation, not scratch-only allocation helpers.
   Owned-context artboard rebinds are now gated by the root owned
   view-model's mutation generation plus the retained context-chain identity.
   `RuntimeOwnedViewModelInstance` bumps the generation for public owned-value
   mutations and view-model relinks; `RuntimeArtboardOwnedContextKey` includes
   that generation; and each artboard skips only its own bind/apply and nested
   animation-context rebind when its key is clean while still descending into
   children so local dynamic `artboardId` swaps can invalidate themselves.
   This keeps the status-doc scout/perf fences in force: it ports the C++
   `DataBindContext::bindFromContext` rebind boundary, not the rejected
   scratch-path reuse or shallow path-command cache layers. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, and `cargo test --workspace` passes. Fenced
   release/null-renderer hot-loop reports aggregate Rust/C++=2.073 then
   2.274 over the 5-entry / 10-segment focused corpus (`ai_assitant`=1.875
   then 2.165). Single-file repeat=100 JSON at
   `/tmp/rive-ai-owned-context-generation-perf.json` reports cpp median=0.420
   ms, rust median=1.411 ms, Rust/C++=3.357. A Rust-only 3M
   `ai_assitant` repeat improves from the prior retained-source-path baseline
   elapsed=23231.2 / advance=11731.1 ms to elapsed=20152.4 /
   advance=8773.6 ms. Strict <=2.0 remains open. Next: profile the focused
   outliers (`animated_clipping`, the small-file fixed overhead in
   `advance_blend_mode` / `animation_reset_cases`, and `ai_assitant` if it
   stays >2 on rerun) under the same scout/perf fences: no broad
   DataBindContext converter-property writes, no StringPad-style RangeMapper
   retry, no scratch-only owned-context path reuse, and no shallow
   command/path-wrapper caching without release/null-renderer evidence.
   Text shape-paint commands are now retained in `RuntimeRenderPathCache` by
   graph/text plus `path_epoch`, `layout_epoch`, and the conservative instance
   cache epoch. The profile target was `animated_clipping`: a live macOS sample
   showed `runtime_draw_command` spending almost all time in
   `runtime_text_shape_paint_commands` / `StaticTextSlice::render_data` /
   HarfRust shaping, while C++ `Text::buildRenderStyles()` retains
   `m_drawCommands` and `Text::draw()` replays them until `markShapeDirty`.
   Rust-only `animated_clipping --benchmark-repeat 3000000` improves from
   elapsed=147254.8 / draw=146414.0 ms to elapsed=4008.4 / draw=3348.1 ms.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat=1 hot-loop remains noisy above
   target at aggregate Rust/C++=2.395 then 2.223; direct repeat=100
   `animated_clipping` JSON at
   `/tmp/rive-animated-clipping-text-cache-perf.json` reports cpp median=0.100
   ms, rust median=0.397 ms, Rust/C++=3.954. Strict <=2.0 remains open. Next:
   profile the small-file fixed overhead in `advance_blend_mode` /
   `animation_reset_cases`, and consider a repeat-aware corpus aggregation
   harness slice before using repeat-heavy evidence as the main M7 score.
   `perf-compare` corpus mode now supports repeat-aware steady-state scoring:
   when `--benchmark-repeat N` is used, the selected exact files are expanded
   after `--corpus-limit` into one runner target per sample segment, preserving
   the golden runners' single-sample repeat invariant while reporting a corpus
   aggregate over the same file x sample segments. The first focused command,
   `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
   PERF_MAX_RATIO=999 PERF_BENCHMARK_REPEAT=100`, reports entries=10 /
   segments=10 and aggregate Rust/C++=3.711 (`advance_blend_mode@0`=9.385,
   `advance_blend_mode@0.25`=8.619, `ai_assitant@0`=3.681,
   `align_target@0`=2.146, `animated_clipping@0`=2.857,
   `animation_reset_cases` samples around 4.0). Strict <=2.0 remains open.
   Prepared draw-command frames are now keyed by
   `(graph_global_id, prepared_epoch)` instead of the broad instance
   `cache_epoch`. `prepared_epoch` is bumped by path/layout/draw-order/render
   opacity/image/nested-artboard identity and draw-affecting properties, while
   nested input proxy values, data-bind/view-model metadata, and
   nested-artboard animation knobs keep only the broad cache epoch. This ports
   the C++ `Artboard::updateComponents` / `ComponentDirt` retention boundary
   without adding a new unaudited skip layer. Rust-only long-repeat
   `advance_blend_mode --benchmark-repeat 1000000` improves from
   elapsed=1382.1 / prepare=695.0 ms to elapsed=1269.1 / prepare=609.9 ms;
   `animation_reset_cases` is roughly neutral at elapsed=516.3 ms. Focused
   repeat-aware hot-loop is noisy/neutral at aggregate Rust/C++=3.897 then
   3.852, and a fresh sample at
   `/tmp/rive-advance-blend-prepared-epoch.sample.txt` still shows nested
   prepared-frame rebuilds dominated by `runtime_shape_paint_path_commands`,
   `path_commands`, `runtime_path_geometry`, and allocation. Strict <=2.0
   remains open. Next: port lower-level C++ `RawPath`/`PathComposer` /
   `ShapePaintPath` retention or another sampled audited data-bind/context
   target; do not retry shallow command-vector/path-wrapper caches without
   fenced release/null-renderer evidence.
   Runtime state-machine definitions are now retained behind
   `Arc<Vec<RuntimeStateMachine>>`, so `advance_state_machine_instance` clones
   only the outer definition handle instead of cloning/dropping the
   `RuntimeStateMachine` definition every advance. This mirrors C++
   `StateMachineInstance` holding a stable `StateMachine` pointer and stays
   within the scout/perf fences: immutable definition retention, not a new
   skip/cache invalidation rule. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop
   reports aggregate Rust/C++=3.613 over the 5-entry / 10-segment corpus
   (`ai_assitant`=3.299), so strict <=2.0 remains open. Next: profile the
   remaining fixed overhead / advance-data-bind time under the same fences.
   A follow-up `ai_assitant --benchmark-repeat 3000000` sample found current
   Rust time split between advance/data-bind and draw/prepare, with
   dependency-ordered gradient paint preparation spending visible time in
   per-frame `BTreeMap`/`BTreeSet` insert/drop. Dependency-ordered paint prep
   now uses small vectors for nested-host command lookup and gradient paint /
   host de-dupe, preserving the old duplicate rules (`collect` last-wins for
   prepared commands, `or_insert` first-wins for layout-discovered commands)
   while matching C++'s retained object/vector traversal shape. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   focused draw probes, `cargo test --workspace`, `cargo fmt --all -- --check`,
   and `git diff --check` pass. Fenced repeat-aware hot-loop improves from
   aggregate Rust/C++=3.632 to 3.447 and 3.327 on rerun, but strict <=2.0
   remains open. Next: re-profile `ai_assitant` and the fixed-overhead files;
   likely remaining targets are advance/data-bind context lookup and lower
   draw/prepare retention, not shallow command/path-wrapper caches.
   A follow-up `ai_assitant --benchmark-repeat 30000000` sample found
   remaining Rust time split across paint/prepare, data-bind, and
   layout-adjusted world-transform lookup. Runtime components now retain the
   static layout-topology facts needed by
   `runtime_component_world_transform_with_bounds`, and
   `ArtboardInstance::component` / `component_mut` use each dense slot's
   retained component index instead of a frame-loop `BTreeMap` lookup. This is
   C++-shaped retained object/index traversal and does not add new skip/cache
   invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test -p rive-runtime`,
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop baseline was
   aggregate Rust/C++=3.515 with Rust median sum 2.429 ms; after the slice,
   reruns report aggregate Rust/C++=3.326 and 3.392 with the better Rust
   median sum at 2.342 ms. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-layout-topology.json` reports cpp median=0.427 ms, rust
   median=1.196 ms, Rust/C++=2.800. Strict <=2.0 remains open. Next:
   re-profile `ai_assitant` plus the fixed-overhead files; likely targets
   remain dependency-ordered paint/prepare work and
   `advance_artboard_data_binds_with_root_transform`, not broad
   converter-property writes or shallow command/path-wrapper caches.
   A fresh `ai_assitant --benchmark-repeat 30000000` sample found the current
   Rust time still split across dependency-ordered paint preparation, draw
   replay, and artboard data-bind/context propagation. Dependency-ordered
   paint preparation now uses the existing retained preparation frame even
   when nested layout gradients force dependency ordering: the cache key
   includes the root `cache_epoch` plus nested command identity and child
   `cache_epoch` values, so clean nested paint frames skip the prep pass while
   child animation/data-bind changes still invalidate it. Artboard data-bind
   source queues also stop cloning target-source vectors during enqueue and
   recycle their update-index buffers across frames, matching C++ retained
   `DataBindContainer` dirty-list storage without adding new skip semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop baseline was
   aggregate Rust/C++=3.330; after the slice reruns report aggregate
   Rust/C++=3.110 and 3.067. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-dependency-prep-skip.json` reports cpp median=0.376 ms,
   rust median=1.031 ms, Rust/C++=2.747. Strict <=2.0 remains open. A fresh
   release/null-renderer sample after this dependency-prep skip found
   `runtime_draw_command`, `advance_artboard_data_binds_with_root_transform`,
   and `runtime_configure_paint_with_cache` as the leading Rust hot sites.
   Draw-time render-paint config now carries the artboard `cache_epoch`, so
   clean frames skip recomputing stroke/blend/shader/feather configuration
   while gradient preparation can still invalidate by removing the cached
   config when it mutates retained paint. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   repeat-aware hot-loop improves from aggregate Rust/C++=3.260 to 2.903 and
   3.105 on rerun; direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-paint-config-epoch.json` reports cpp median=0.566 ms,
   rust median=1.404 ms, Rust/C++=2.483. Strict <=2.0 remains open. Next:
   re-profile; likely remaining targets are draw replay / lower
   `RuntimeRenderPathCache::draw_path` map lookups and
   `advance_artboard_data_binds_with_root_transform`, not broad
   converter-property writes or shallow path-command caches. The next profile
   found `runtime_draw_command`, data-bind context propagation, retained path
   lookups, and state-machine advance as the remaining split. Two same-turn
   scouts were backed out: source-queue vector take/recycle worsened fenced
   hot-loop aggregate to Rust/C++=3.119 and 3.077, and carrying a borrowed
   retained `RenderPaint` through draw gave direct `ai_assitant` Rust/C++=2.658
   versus the prior 2.483. Prepared shape-paint commands now retain
   `paint_global_id`, matching C++'s retained `ShapePaint` object identity and
   removing a draw-time local-to-global map lookup. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Fenced hot-loop is noisy: aggregate Rust/C++=3.038 then 2.889; direct
   repeat=100 `ai_assitant` JSON at
   `target/perf-ai-shape-paint-global-id.json` reports cpp median=0.604 ms,
   rust median=1.495 ms, Rust/C++=2.477. Strict <=2.0 remains open. Next:
   re-profile; likely targets remain data-bind context/source-local lookup and
   lower `RuntimeRenderPathCache::draw_path` map lookup, not source-queue
   vector swaps or borrowed retained-paint threading.
   Nested child data-context sync now retains resolved source locals by child
   property/image binding index on `RuntimeNestedArtboardInstance`, seeded from
   the existing path map and rebuilt with dynamic `artboardId` swaps. This ports
   the C++ `DataContext`/`DataBind` retained-source shape without adding a new
   skip gate: each frame still reads the current source value, while the steady
   path avoids a per-binding path-map lookup before the fallback slot walk. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop is noisy at aggregate
   Rust/C++=2.972 then 3.167, but Rust median sum was 3.331 then 2.724 ms and
   `ai_assitant` Rust median improved to 1.414 then 1.140 ms. Direct repeat=100
   `ai_assitant` JSON at `target/perf-ai-binding-source-local-slots.json`
   reports cpp median=0.581 ms, rust median=1.462 ms, Rust/C++=2.516. A fresh
   sample shows `stateful_nested_host_binding_value_for` and
   `stateful_nested_host_value_local_for_slots` lower, with remaining time split
   across draw replay/path-cache `BTreeMap` lookups, data-bind source queues,
   converter advance, and state-machine advance. Strict <=2.0 remains open.
   Next: profile the remaining draw `BTreeMap::get` under
   `runtime_draw_command` / `RuntimeRenderPathCache::draw_path` and the
   remaining data-bind queue drains; keep the source-queue vector-swap and
   borrowed retained-paint threading scouts rejected.
   Retained render paints now live in dense global-id slots
   (`RuntimeRenderPaints`) instead of a `BTreeMap`, so persistent draw and
   gradient-prep paint access mirrors C++ retained `ShapePaint::renderPaint()`
   pointer lookup while preserving the old factory allocation side effects for
   golden ordering. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   repeat-aware hot-loop reports aggregate Rust/C++=3.000 with
   `ai_assitant`=2.799. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-render-paint-slots-rerun.json` reports cpp median=0.576 ms,
   rust median=1.437 ms, Rust/C++=2.495, with Rust draw median down to
   0.289 ms from the previous artifact's 0.312 ms. Strict <=2.0 remains open.
   Next: profile the remaining lower `RuntimeRenderPathCache::draw_path`
   lookup and data-bind queue drains; keep source-queue vector swaps and
   borrowed retained-paint threading rejected.
   A status-doc review of the scout/perf discoveries keeps the fences binding:
   no broad converter-property writes, no RangeMapper retry without deeper C++
   ownership/order analysis, no scratch-only context-path reuse, no shallow
   command/path wrappers, no source-queue vector swaps, and no borrowed
   retained-paint threading without release/null-renderer evidence.
   `ai_assitant` profiling after state-machine pending-action retention still
   showed `advance_nested_artboards_collect_events` beside data-bind,
   state-machine, and draw replay. C++ `NestedArtboard::advanceComponent`
   advances nested animations without a caller-owned per-child event vector, and
   `StateMachineInstance` owns reported event queues. Rust nested artboard
   advance now makes event collection optional: no-observer paths pass `None`
   through nested animation advance and avoid allocating ignored event vectors or
   cloning reported events. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop is
   noisy by ratio but Rust median sum improves from the pre-slice 3.488 ms to
   2.476 ms and 2.488 ms on rerun; aggregate reports Rust/C++=3.041 and 3.073
   because C++ median sum also dropped. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-nested-event-option.json` reports cpp median=0.413 ms, rust
   median=1.041 ms, Rust/C++=2.521. Strict <=2.0 remains open.
   A follow-up profile after the optional nested-event slice showed the hot
   split across `advance_artboard_data_binds_with_root_transform`,
   `runtime_draw_command`, nested-event collection, state-machine advance, and
   nested data-context lookup. C++ `DataBindContainer` owns persistent and dirty
   vectors and uses membership state to avoid duplicate dirty-list enrollment.
   Rust now uses a per-binding `custom_property_update_flags` bitmap when
   merging dirty and persisting custom-property source updates, and skips both
   custom-property and numeric source update-index construction for empty lanes.
   This keeps the source-queue vector-swap scout rejected while porting the
   C++ queue-membership shape. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop is
   noisy: the baseline after nested-event collection was aggregate Rust/C++=2.877
   with Rust median sum 2.467 ms; the post-slice rerun reports aggregate
   Rust/C++=3.007 with Rust median sum 2.319 ms, while the first post-slice run
   regressed to Rust median sum 2.667 ms. Direct repeat=100 `ai_assitant` JSON
   at `target/perf-ai-data-bind-queue-flags.json` reports cpp median=0.379 ms,
   rust median=0.989 ms, Rust/C++=2.613. Strict <=2.0 remains open. Next:
   profile remaining `runtime_draw_command` / `RuntimeRenderPathCache::draw_path`,
   state-machine fixed overhead, and remaining nested data-bind context-path
   work under these fences.
   Solo target binding apply now walks the retained binding list without
   cloning each `RuntimeArtboardSoloBindingInstance` before calling the Solo
   update path, matching C++ `DataBindContainer` retained-list traversal plus
   `context_value_{number,string,enum}` direct `Solo::updateByIndex` /
   `Solo::updateByName` dispatch. Full `make golden-compare` remains exact=263
   / exact-segments=584 / diverges=0, and `cargo test -p rive-runtime --quiet`
   passes. Fenced repeat-aware hot-loop is noisy: aggregate Rust/C++=2.939
   with Rust median sum 2.315 ms. The targeted single-file repeat=100 JSON at
   `target/perf-ai-solo-binding-no-clone.json` reports cpp median=0.391 ms,
   rust median=0.939 ms, Rust/C++=2.403, improving the prior `ai_assitant`
   Rust median from 0.989 ms. Strict <=2.0 remains open. Next: profile the
   remaining draw-path lookup, state-machine fixed overhead, and nested
   data-bind context-path work.
   A same-session dense paint-configuration sidecar scout was intentionally not
   landed. It replaced the draw-time `BTreeMap<u32,
   RuntimeCachedRenderPaintConfiguration>` with global-id slots, mirroring the
   retained render-paint slot shape, but fenced release/null-renderer evidence
   rejected it: focused hot-loop moved to aggregate Rust/C++=3.001 and direct
   repeat=100 `ai_assitant` worsened from rust median=0.939 ms to 0.997 ms
   (`target/perf-ai-dense-paint-config.json`). The sampled BTree hit is not
   enough by itself; keep profiling draw replay and prefer a deeper C++
   retained-path / object-identity slice over another sidecar container swap.
   The release Rust profile now matches the build-profile parity scout's
   shipping-runtime recommendation: root `Cargo.toml` sets `lto = "fat"`,
   `codegen-units = 1`, and `panic = "abort"` for `[profile.release]`, after a
   `catch_unwind`/`resume_unwind` search found no C ABI unwind reliance. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, `git diff --check`,
   and `cargo build --release -p rive-capi` pass. Fenced repeat-aware hot-loop
   with the LTO profile reports noisy median aggregates Rust/C++=3.371 then
   2.760; direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-release-profile.json` reports cpp median=0.423 ms,
   rust median=1.352 ms, Rust/C++=3.195, while min timings are cpp=0.388 ms
   and rust=0.981 ms (~2.53x). Strict <=2.0 remains open. Next: implement the
   scout's min-based/deliberate perf gate (`--aggregate=min` and image-bearing
   focused corpus) before using focused perf numbers to choose another runtime
   slice.
   The min-based/deliberate gate tooling is now landed: `perf-compare` accepts
   `--aggregate median|min`, thresholds the selected statistic, preserves both
   median and min sums in JSON, and supports `--corpus-ids` so focused perf is
   not alphabetical truncation. `make perf-hot-loop` defaults to
   `PERF_AGGREGATE=min` and the deliberate focused corpus
   `advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo`,
   with default `PERF_ITERATIONS=10` and `PERF_BENCHMARK_REPEAT=100` so bare
   `make perf-hot-loop` runs the adopted fence rather than a smoke-only path.
   Fenced release/null-renderer smoke with the defaults reports aggregate min
   Rust/C++=4.758 over 11 file/sample entries; the newly visible image path
   dominates (`spotify_kids_demo@0` min Rust/C++=10.413), followed
   by the known tiny-file fixed overhead outliers. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Follow-up image micro-slices showed the shallow cache and mesh-index
   precompute layers are not the C++ optimization. Retained clipping-shape
   path geometry, the N-slicer fast-miss, retained draw-command object kinds,
   retained path-composer local-index lookups, dense draw-path retained slots,
   and graph-scoped dense path-geometry command slots now remove sampled
   draw-replay rebuild/discovery/string-dispatch/vector-scan/BTreeMap lookup
   paths while preserving the full ratchet. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A same-session
   fenced run before the local-index slice reported aggregate min Rust/C++=3.399
   but C++ min-sum=1.024 ms, outside the sanity band; the post-slot perf runs
   were deferred because verification-time load remained above the fence,
   latest 25.13/42.76/45.72. Strict <=2.0 remains open. Next runtime target
   should be a low-load release sample and then actual image/`PathComposer`/
   raw-path retention or deeper draw-replay fixed-overhead work, under the
   existing scout fences.
   Layout-adjusted draw world transforms are now cached in
   `RuntimeRenderPathCache` dense local slots behind the existing
   `(cache_epoch, layout_epoch)` dirt boundary. Shape path prep, clipping prep,
   gradient paint prep, image draw, mesh-image draw, and nested-artboard host
   draw now route through this cache when a layout-bounds frame exists, while
   no-layout calls keep using the retained component transform directly. This
   targets scout item 17's draw-replay world-transform recompute bucket without
   adding a new image draw-state cache, mesh-index precompute, or geometry
   float-math rewrite. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Perf was not
   rerun because load was 24.90/37.27/29.64, outside the acceptance fence.
   Strict <=2.0 remains open. Next: run a clean low-load `make perf-hot-loop`,
   then profile/port the remaining image/`PathComposer`/raw-path retention or
   deeper draw-replay fixed-overhead work under the scout fences.
   Mesh render buffers are now retained in dense graph-local slots on
   `RuntimeRenderPaintCache`. C++ `MeshDrawable` owns its vertex/UV/index render
   buffers directly; Rust previously kept `RuntimeMeshRenderBuffers` behind a
   draw-time `BTreeMap` keyed by mesh local id. `RuntimeMeshRenderBufferSlots`
   now keeps the same preallocated buffers in local-id slots while preserving
   the existing mesh discovery, source-buffer allocation, vertex-byte reuse, and
   weighted mesh math. This is not the rejected image mesh-index precompute
   scout: image-to-mesh discovery still happens through the existing graph
   lookup. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Two post-slice
   release/null-renderer samples with `make perf-hot-loop PERF_MAX_RATIO=999`
   report aggregate min Rust/C++=3.219 then 3.176, but C++ min-sums=0.992 ms
   and 1.053 ms are outside the 0.70-0.95 ms sanity band despite low 1-minute
   post-run load. Strict <=2.0 remains open. Next: run a clean
   low-load/sanity-band `make perf-hot-loop`, then continue actual
   image/`PathComposer`/raw-path retention or deeper draw-replay fixed-overhead
   work under the scout fences.
   Image layout local transforms are now retained in `RuntimeRenderPathCache`
   behind the existing `(cache_epoch, layout_epoch)` dirt boundary plus image
   and layout dimensions. This ports the C++ `Image::updateImageScale` shape:
   clean frames reuse the computed image scale/offset local transform, then
   multiply it by the cached parent layout world transform. Mesh-image draw now
   also routes through this retained image world transform, matching C++
   `Mesh::draw`'s use of the parent image `worldTransform()`. The slice does
   not cache blend/opacity/draw state and does not repeat the rejected shallow
   image draw-state cache. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A post-slice
   `make perf-hot-loop PERF_MAX_RATIO=999` reports aggregate min Rust/C++=3.225,
   but C++ min-sum=1.043 ms is outside the 0.70-0.95 ms sanity band. Strict
   <=2.0 remains open. Next: run a clean low-load/sanity-band
   `make perf-hot-loop`, then continue actual `PathComposer`/raw-path retention
   or deeper draw-replay fixed-overhead work under the scout fences.
   Draw `RenderPath` cache entries now retain the raw path payload used to
   rebuild the renderer path, mirroring C++ `ShapePaintPath::m_rawPath` /
   `ShapePaintPath::renderPath`. On a path-epoch miss Rust rebuilds the cached
   `RawPath` in place with `rewind()` capacity reuse, then calls
   `RenderPath::add_raw_path`; clean frames keep reusing the retained
   `RenderPath` as before. This does not reintroduce the rejected shared
   shape path-command buffer/cache layer and does not change path geometry
   math, fill-rule handling, or the existing `path_epoch` invalidation
   boundary. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`, the focused
   draw-path reuse test, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A same-session
   release/null-renderer sample before
   the final capacity-reuse polish reported aggregate min Rust/C++=3.219, but
   C++ min-sum=1.037 ms is outside the 0.70-0.95 ms sanity band and movement
   versus 3.225 is below the noise floor; no decision-grade post-polish perf
   sample was taken because load rose to 21.26/15.84/13.92. Strict <=2.0
   remains open. Next: rerun a clean low-load/sanity-band `make perf-hot-loop`,
   then profile/deepen draw-replay fixed overhead or clean-prepare-skip work
   rather than extending raw-path wrappers.
3. The former `nested-stateful-view-model-property`,
   `nested-layout-clip-data-bind`, `nested-node-transform-data-bind`,
   `nested-text-outline-contour-order`, `layout-component-paint`, and
   `nested-feather-gradient-space` unsupported queues are empty.
4. Remaining non-exact entries are intentionally parked as `gated` or
   `harness`. Gated diagnostics include `scripted-data-context`
   (`scripted_data_context.riv`), `scripted-transition-condition` (2 gated),
   `scripted-path-effects` (1 gated), and `text-polygon-sibling`
   (`bankcard.riv`). Keep these parked queues as explicit unsupported/gated
   work until an M7 or scripting/harness slice can either promote a file or
   replace the guard with a sharper diagnostic.
   The old `text-input` manifest queue is empty.
5. M5 is closed for the current corpus: `grep -B6 'milestone = "M5"'
   corpus.toml` is empty. Do not reopen data-binding work unless a newly added
   corpus entry exposes a pre-text/pre-layout data-binding diagnostic.
6. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M6 unless a related refactor needs a cheap draw-regression check.

7. Threads are now policy (see `/goal` "Threads" section): the main loop
   stays the single writer here; use read-only scout threads to triage the
   remaining M6 queues in parallel. Start the first lane thread in a NEW
   worktree for the C++ golden-runner crash repair (`milestone =
   "harness"`, 36 files; FileAssetContents/scripting/data-viz crash paths
   in `tools/golden-runner` only), merging back into this branch once the
   full ratchet passes. Recovered files enter as `not-yet` — denominator
   growth, zero conflict with M6 runtime work.

8. Harness lane MERGED (e5941e7): the C++ golden-runner now survives 34 of
   36 `milestone = "harness"` files (FileAssetContents stripping for the
   non-scripting librive build, flush + `_Exit(0)` before teardown, ABI
   define alignment). MAIN-LOOP FOLLOW-UP is partially complete: 10 recovered
   entries were promoted exact after the image scripting property-value
   ordering fix; continue flipping the remaining recovered files from
   `milestone = "harness"` only after assigning each to exact/not-yet/gated
   with a verified compare result.
   Residuals (2): `data_viz_demo` and `data_binding_artboards_test` crash
   only because the runner binds a blank default view-model instance;
   binding named instance 0 (like the reference unit tests) recovers both
   but perturbs 66 currently-exact entries — treat as a coordinated
   convention-change decision, not a harness fix. Keep them
   `milestone = "harness"` until decided.

9. REVISED (see Decisions 2026-07-07): do not adopt the global named
   view-model instance 0 binding convention yet. The coordinated runner
   experiment recovered `scripted_color.riv` after binding the selected
   artboard's own owned view-model context, but still left 48 exact entries
   divergent because serialized list data makes C++ `ArtboardComponentList`
   instantiate and draw item artboards while Rust still has only partial
   component-list runtime support. Keep `data_viz_demo` and
   `data_binding_artboards_test` in `milestone = "harness"` under the
   current blank-default runner convention. Reopen the convention only after
   Rust supports `ArtboardComponentList` item artboard instancing, draw,
   layout, and view-model data-context binding well enough for the affected
   exact corpus to reverify green in the same commit.

10. SCOUT RESULT (read-only pre-classification of the 34 recovered harness
   files; streams/diffs in the session scratchpad — trust but re-verify on
   promotion): (a) promoted exact in the main loop:
   audio_script, multi_listeners, script_dependency_test,
   script_dependency_test2, script_dependency_test_using_library(+_v2),
   script_namespace_test, script_string_converter_test,
   scripted_listener_action, image_scripting_property_value. The latter
   required matching the non-scripting C++ golden runner's import stack:
   `ScriptAsset` does not displace a pending image `FileAssetImporter`, so
   the second image decodes after the source render-paint allocation.
   (b) gated-scripting (21): all remaining script*/viewmodel*/gamepad/
   data_bind_artboard_input/path_effect_with_feathers/group_effect/
   replace_view_model files — blocked on the Luau VM; note
   path_effect_with_feathers is ScriptedPathEffect content, NOT M6 feather
   work. (c) HARNESS-BLOCKED runtime candidates (3): relative_data_bind_path
   (nested-child data bind into NestedArtboard),
   scripted_data_converter_bound_input (data bind target Shape.x through
   static-text subset), databind_viewmodel (DataConverterToString value
   mismatch feeding a Text run — Rust data_bind_graph ToString produces a
   different string than C++). They remain `milestone = "harness"` until the
   C++ runner path is recovered and each file is reverified.
   PROCESS FIX REQUIRED before flipping the 18 stream-subset scripting
   files: the Rust runner silently drops ScriptedDrawable draws (known-
   ignored list in text.rs), so they would land as `diverges` and invite
   wrong work — add a loud `unsupported: scripting` diagnostic for
   ScriptedDrawable-bearing files first, then flip them straight to
   `milestone = "gated"`. Unsupported is never silent.

11. PERF METHODOLOGY FENCE (measurement gate before optimization). Earlier
   debug-vs-debug and recording-serializer perf numbers are void. The release
   C++/Rust runner builds, null-renderer benchmark mode, whole-repeat
   `total_ms` scoring, and perf JSON artifact path have landed; keep using
   them for all M7 decisions. Per-frame phase timings remain diagnostics only.
   Required order for any new optimization slice:
   (a) Release-vs-release perf builds: `cargo build --release` for the
       Rust runner and a release C++ runner + release reference libraries;
       correctness ratchet stays on debug. Re-baseline all ratios and
       discard debug-era perf conclusions and priorities.
   (b) Null-renderer benchmark mode on BOTH runners (same trait calls,
       output discarded) so the measured cost is pure runtime
       advance/prepare/draw-path work, not stream serialization.
       Re-baseline again.
   (c) Only then resume optimization slices, each one: flamegraph
       attribution (samply/Instruments) -> read the C++ source at the same
       hot site -> PORT the C++ optimization if one exists (keyframe
       cursors, ComponentDirt gating, RawPath rewind/reuse, paint/path
       caching) -> invent a novel optimization only when C++ has none
       there.
   (d) Statistical floor: >=10 iterations with median + spread, a pinned
       perf corpus spanning tiny/medium/heavy files, and a per-commit perf
       JSON artifact so trends are data, not "noisy but typical" recall.
   Fidelity rules while optimizing: no tolerance widening for perf; no
   float-math restructuring in geometry paths (the fused scaleAndAdd
   lesson — no reassociation/fast-math; SIMD only if the ratchet stays
   strictly green); no invalidation/skip logic that does not mirror an
   audited C++ dirt gate — invented caching is how original-author
   decisions get silently broken on unsampled timelines.

12. SCOUT REPORT — C++ animation-apply audit (port-ready, cited against
    reference @7c778d13). Headline: C++ has NO keyframe cursor and NO
    value-unchanged skip in the animation layer — do not invent them.
    KeyedProperty::apply is a stateless binary search over CACHED
    per-keyframe seconds (keyed_property.cpp:20-52) with an O(1)
    past-last-frame fast path (:28-32); the unchanged-value short-circuit
    lives in generated property setters (node_base.hpp:53-62), which
    Rust's changed-bool setters already mirror. Port slices, ranked:
    (1) STOP PER-FRAME DEEP CLONES — likely the dominant cost of the
        21.9x: crates/rive-runtime/src/artboard.rs:510 clones the entire
        RuntimeLinearAnimation (all keyed objects/keyframes incl. string
        byte Vecs) on EVERY apply, and artboard.rs:594 clones the whole
        RuntimeStateMachine on every advance. C++ applies from a shared
        immutable definition (LinearAnimation::apply(...) const,
        linear_animation.cpp:71-85) with mutation confined to the
        instance. Restructure to shared immutable definitions
        (Arc/index-based split borrows), apply by &ref.
    (2) Cache keyframe seconds at build (KeyFrame::computeSeconds,
        keyframe.cpp:10, called once at keyed_property_importer.cpp:15);
        Rust recomputes frame/fps with a zero-branch on every comparison
        of every search (animation.rs:1102-1107 + 5 sibling impls).
    (3) Precompute cubic solver state at build: 11-entry bezier-x table
        (cubic_interpolator_solver.cpp:28-95, built once at
        cubic_interpolator.cpp:5-11) — Rust rebuilds it inside every
        get_t call (animation.rs:145-156); also cache CubicValue
        coefficients behind a from/to guard (cubic_value_interpolator
        .cpp:26-35 vs animation.rs:128-139).
    (4) Kill steady-state allocs in advance plumbing: persistent
        reported-events buffers (state_machine_instance.hpp:336, drained
        :2293-2317), blend instance lists built once with reserve
        (blend_state_instance.hpp:51-71), pooled AnimationReset
        (animation_reset_factory.cpp:226-235) — vs Rust fresh Vecs per
        advance (artboard.rs:552-560, :601-604, :617-645).
    Also: interpolator pointers resolve once at onAddedDirty, validation
    is hoisted to init (invalid keyed objects erased), advanceAndApply
    caps at 5 passes breaking when Components dirt clears
    (state_machine_instance.cpp:2589-2616).

13. SCOUT REPORT — C++ draw-retention audit (port-ready, cited against
    reference @7c778d13). Governing principle: C++ computes NOTHING during
    draw() — all geometry/paint work happens in updateComponents gated by
    dirt (clean frame: first-branch return, artboard.cpp:1186-1189), and
    draw() replays retained RenderPath/RenderPaint handles. Confirmed Rust
    per-frame rebuilds: sorted drawable order w/ BTreeMaps+clones
    (draw.rs:224-299), vertex->command re-derivation per paint
    (draw.rs:2836-2951), unconditional runtime_rebuild_path on every cache
    access (draw.rs:5028-5041), layout bounds re-derived (draw.rs:996).
    Ranked port slices:
    (1) ShapePaintPath retention: retained RawPath + retained RenderPath +
        one dirty bool (shape_paint_path.hpp:78-84, .cpp:13-76); rebuild
        becomes a no-op on clean frames. Largest draw-phase win.
    (2) PathComposer gated by Path|NSlicer dirt (path_composer.cpp:40-117)
        plus dirt plumbing Path::markPathDirty/onDirty/Shape::pathChanged
        (path.cpp:327-348, shape.cpp:99-108); note plain transform changes
        do NOT rebuild vertex paths — WorldTransform only couples to path
        rebuild when a deformer exists (path.cpp:358-359).
    (3) Path::m_rawPath retention with rewind() capacity reuse
        (path.cpp:350-380; raw_path.cpp:446-451 rewind keeps capacity;
        addPath bulk memcpy+SIMD transform :255-279); zero-opacity deferral
        via canDeferPathUpdate + m_deferredPathDirt (path.cpp:111-126,
        :344-347).
    (4) RenderPaint mutate-in-place for instance lifetime: solid color
        writes mutate immediately w/o dirt (solid_color.cpp:24-54), stroke
        props via Paint dirt (stroke.cpp:37-53), gradients rebuild only on
        Paint|Stops|(WorldTransform iff world-space) into retained
        m_colorStorage with only the shader rcp swapped
        (linear_gradient.cpp:86-201).
    (5) Retained sorted drawable list (intrusive, resorted only on
        DrawOrder dirt, artboard.cpp:569-660,1142-1145) + retained clip
        paths (clipping_shape.cpp:151-173).
    CROSS-CUTTING PREREQUISITE for 1-3: per-component dirt bitset with the
    updateComponents early-out (artboard.cpp:1184-1223) so clean frames
    skip the entire prepare phase. Pairs with the animation-apply slices; do
    the deep-clone removal first, then this prerequisite, then slices by rank.

14. SCOUT REPORT — C++ dirt-gating audit (port-ready, cited against
    reference @7c778d13). Confirms Rust already mirrors the
    updateComponents loop skeleton (add_dirt / update_components_with_hook
    / dirt_depth vs artboard.cpp:1184-1223) — the gap is that per-frame
    work is not BEHIND the gates. Core primitives to port exactly:
    Component::addDirt early-out when bits already set (component.cpp:
    34-38, the single most important line: repeated writes collapse to one
    bit test); dirt cleared BEFORE update() runs (artboard.cpp:1209);
    DirtDepth lowered by upstream re-dirt triggers inner-loop break +
    re-sweep (artboard.cpp:978-990, 1215-1218); advanceAndApply settles
    with up to 5 updatePass loops breaking when Components dirt clears
    (state_machine_instance.cpp:2589-2615). Clean-frame contract: SM
    layers still APPLY keyframes every frame, but generated setters'
    equality early-outs mean steady values raise zero dirt, so
    updateComponents returns at its first branch and NO component is
    visited — draw() never checks dirt, it reads coherent caches.
    Ranked slices:
    (1) Idempotent property writes + the *Changed() dirt-raiser table
        (node.cpp:9-10, transform_component.cpp:54-61,119-121,
        world_transform_component.cpp:10-28, parametric_path.cpp:63-66,
        path_vertex.cpp:21-30, stroke.cpp:37-41,
        linear_gradient.cpp:203-215). Turns steady-value animation frames
        into zero-dirt frames.
    (2) Geometry behind Path dirt (= item 11 slices 1-3), incl. the
        invisible-shape deferral bonus: canDeferPathUpdate +
        m_deferredPathDirt (shape.cpp:35-52, path.cpp:344-347,361-365,
        path_composer.cpp:29-38,44-48) — opacity-0 shapes never build
        geometry.
    (3) Sorted draw list behind DrawOrder dirt only (raisers:
        draw_rules.cpp:40, draw_target.cpp:31); clipping ops behind
        Clipping dirt (artboard.cpp:1146-1149).
    (4) Render paints behind Paint|Stops|RenderOpacity (= item 11 slice
        4).
    (5) Data-bind dirty queues instead of scans (data_bind_container.cpp:
        145-258, data_bind.cpp:487-511, core.cpp:25-46 push observers with
        one-branch no-subscriber fast path, artboard.cpp:1169-1173).
    COMBINED SEQUENCE across the animation/draw/dirt scouts: kill per-frame
    definition clones -> idempotent writes/raiser table -> draw-retention
    prerequisite + retention slices in rank order -> remaining animation/data-bind
    dirt slices as flamegraph data directs. Full ComponentDirt bit inventory with consumers is in the
    scout transcript; component_dirt.hpp:8-81 is the source of truth.

15. SCOUT REPORT — release flamegraph attribution (samply, release build,
    null-renderer hot loop; profiles in session scratchpad). REORDERS the
    dirt-gating combined sequence:
    (0) NEW TOP SLICE — schema reflection in hot paths, ~36% of self time:
        definition_by_name (rive-schema lib.rs:252, LINEAR SCAN + string
        eq, 17.5%), definition_by_type_key (lib.rs:232 linear scan, 8.4%),
        Definition::property_by_key (lib.rs:217, walks ancestors via
        definition_by_name, 5.4%), property_key_for_name (properties.rs:
        200, string->key per property READ, 5.4%). C++ uses compile-time
        property-key constants + switch tables; runtime name/definition
        resolution must not exist in the frame loop. Fix: precompute
        typed accessor/key tables at instance build (fidelity-neutral —
        this is invented Rust structure, not C++ behavior).
    (1) Clone hypothesis CONFIRMED in direction, corrected in site:
        allocator/copy traffic is 25-44% of self time, but ~70-85% of
        clone samples come from ArtboardGraph deep clones in
        artboard_data_bind.rs (~:1617, runtime_graph().cloned() in
        update_*_source_bindings, multiple times per advance);
        artboard.rs:594 is secondary (~5-11%), artboard.rs:510 minor.
        Fix the data-bind clones FIRST, then item 10(1).
    (2) ai_assitant's 16.1% TrimContour::get_segment: re-dashing every
        frame with linear segment scans; C++ caches m_contours + dashed
        result behind dirt (trim_path.cpp, contour_measure.cpp).
    (3) Taffy node tree rebuilt every prepare+draw (60% inclusive on
        blend file) with reflection-heavy style reads; C++ runs layout
        only on markLayoutNodeDirty.
    MEASUREMENT CORRECTIONS: (a) current tree measures 8.44x on
    ai_assitant (not 37.5x — earlier number was different tree state);
    (b) CRITICAL harness hazard: with --benchmark-repeat 4000, C++ drops
    to ~1.5us/segment because dirt-gating makes frames 2..N nearly free —
    the steady-state gap is orders of magnitude larger than the
    single-pass ratio, and the ratio is extremely sensitive to
    amortization. DEFINE the M7 perf target explicitly as STEADY-STATE
    per-frame cost (high repeat count, cold frame excluded or reported
    separately); the retention/dirt slices in items 10-12 are what close
    the steady-state gap. Record the chosen definition as a Decision
    before optimizing further.

16. LANE MERGED (88fe434): scripting spike. `crates/rive-scripting`
    (feature `luau`, default-on, zero deps leaking) proves luaur 0.1.8
    (PINNED =0.1.8, upstream Luau commit 8f33df9): boots, loads real
    Rive-editor Luau BYTECODE directly (ScriptAssets carry bytecode v6 in
    a SignedContentHeader envelope — ported as rive_scripting::envelope;
    the runtime never compiles source), executes corpus scripts
    end-to-end (ArtboardGrid generator->instance with inputs), and
    resolves the corpus require chain via C++-style registration retries
    (mirrors ScriptingContext::performRegistration). mlua fallback NOT
    needed on this evidence. Known gaps recorded in the lane report:
    bytecode loads via one unsafe luau_load seam (upstream ask: safe
    ChunkMode::Binary — file on pjankiewicz/luaur); sandbox parity
    REQUIRED before real integration (C++ init order: open libs -> rive
    globals -> luaL_sandbox -> load; GETIMPORT resolves globals at LOAD
    time — install all globals first); bind Vector via luaur's native
    vector type, not a table. Seam proposal: traits ScriptingVm /
    ScriptInstance / ScriptHost defined IN rive-runtime (keeps luaur out
    of its deps), implemented by rive-scripting, wired in crates/rive
    behind a feature; method dispatch gated by SERIALIZED
    OptionalScriptedMethods bitmask (script_asset.hpp:70-181), input
    writes raise ScriptUpdate dirt. Bindings sizing from a census of all
    57 corpus scripts: ~half of the 18.2k C++ glue needed, in order:
    boot/registration+scripted_object protocol (~2.5k) -> Vector/Mat2D/
    Color/Path/Paint/renderer verbs (~2.5k) -> artboards/animations
    (~1k) -> DataValue+viewmodel properties (~2-3k) -> listener/input
    tail (~1.5k). NOT needed by corpus: lua_gpu (3.7k), lua_promise,
    lua_mat4, lua_buffer_ext, most of lua_image_decode, lua_audio.
    Signature verification (libhydrogen) deferrable — corpus unsigned.

17. LANE MERGED (d8cf8cb): C ABI embed loop + perf JSON.
    crates/rive-capi now covers file->artboard-instance->state-machine->
    inputs->advance->draw via a caller-provided RiveRenderCallbacks
    repr(C) vtable (FFI-renderer pattern, opaque u64 handles, balanced
    release_* calls, nullable callbacks); `make capi-smoke` runs a real C
    embed loop. perf-compare gained --json/--meta (phase-sum metrics,
    benchmark_repeat recorded, meta passed in never computed) +
    `make perf-json` + additive CI jobs (capi-smoke; perf-json artifact,
    continue-on-error). Additive crates/rive API: Factory/Renderer
    re-exports, Artboard::state_machine_name/default_state_machine_index,
    ArtboardInstance::default_state_machine_instance/
    advance_with_state_machine. Follow-ups: (a) once draw-frame retention
    stabilizes, add an additive cache-holding draw so the C ABI reuses
    render handles across frames; (b) pointer events + view-model
    contexts are additive ABI gaps; (c) default-SM selection: capi
    falls back to first (C++ defaultScene) while the golden runner uses
    flagged-or-none — align once embed parity matters.

16. SCOUT REPORT — gate protocol + phase-gap localization (decision-grade,
    45 fenced runs; scripts/JSONs in session scratchpad). FOUR findings
    that redirect M7 perf work:
    (a) ADOPT MIN-BASED AGGREGATION NOW: the median-based aggregate has a
        +-0.42 noise band per run (observed phantom 2.18 reading when
        min-based truth was 2.95); sum of per-target min_ms over 10
        iterations gives +-0.07 with the same central value. Contention
        noise is one-sided; min recovers intrinsic cost, both sides
        treated identically. Improvements < ~0.08 ratio are below
        single-run resolution — don't claim them without 2 runs pre/post.
    (b) PIN THE GATE DEFINITION: total(N) = first-frame + (N-1)*clean-
        frame, and the two have different ratios. Scout-time focused
        aggregate was ~2.98 at repeat=100 but ~3.95 at repeat=1000
        (pure steady state), before the deliberate image-bearing default
        gate made the current standing worse. The adopted M7 gate is
        repeat=100; track improvements at that fixed N.
    (c) THE GAP IS RENDER-SIDE, NOT ADVANCE-SIDE: for text-bearing files
        Rust ADVANCE is already FASTER than C++; the focused-corpus gap
        concentrates in prepare+draw clean-frame replay plus ~0.5-1us/
        frame fixed overhead (epoch checks) that dominates tiny files
        (3.5-6.4x) vs heavy (2.1-2.6x). Recent data-bind/advance slices
        target the smaller half of the remaining gap — shift to draw
        replay cost and tiny-file fixed overhead.
    (d) HEADLINE — IMAGE FILES ARE 10-170x AND INVISIBLE TO THE GATE:
        car_widgets_v01=145-170x, echo_show_demo=81-112x,
        jellyfish_test=61-65x, spotify_kids_demo=11x — draw-dominated,
        LINEAR in repeat count (~4.6ms/frame steady on car_widgets => a
        retained-draw cache is missed/rebuilt every frame on the image
        path). PERF_CORPUS_LIMIT=5 takes the first five ALPHABETICAL
        files, which contain no images. Fix the image draw retention AND
        make the gate corpus deliberate (include >=1 image file, e.g.
        spotify_kids_demo); track repeat=1000 as a secondary diagnostic.
    GATE PROTOCOL (adopt as Decision): ratio-of-sums over per-target
    min_ms, 10 iterations, repeat pinned; acceptance = 3 independent
    invocations ALL <= 2.0 with 1-min load < ~8 and C++ min-sum inside
    its 0.70-0.95ms sanity band. Scout-time standing before the
    image-bearing default gate was 2.98 (band 2.82-3.06) at repeat=100
    and ~3.95 at repeat=1000; see the top-level M7 Perf Fence for the
    current standing. The distance to 2.0 is real, not noise. Tool
    follow-ups landed:
    --aggregate=min flag, deliberate --corpus-ids gate, and make defaults for
    10 iterations / repeat=100 on perf-hot-loop.

17. SCOUT REPORT — fresh flamegraph of TODAY'S tree (samply, release,
    steady + cold regimes profiled separately; C++ steady profile captured
    as fairness baseline; profiles in session scratchpad). Old top sites
    CONFIRMED DEAD in steady: schema scans <1%, ai malloc ~1.5%,
    TrimContour retention works, Taffy zero frames. What remains, ranked
    by payoff:
    (1) CHEAPEST BIG WIN — cold-frame name reflection is ~60% of the gate
        metric's delta (repeat=100 blends heavily with the cold frame):
        authored_transform does SIX name lookups per component on the
        first world-transform pass (artboard.rs:885-890 ->
        objects.rs:298 runtime_property_metadata_by_name linear scan +
        ancestor walk); definition_by_name 19.4% self in the cold region.
        MECHANICAL: per-component cached transform property keys or
        direct storage fields. Also clears the keyframe-apply name
        resolution survivor on animation_reset_cases (5.3% incl) and
        align_target's raw uint_property reads (9.6% incl).
    (2) STEADY #1 (~43% of steady delta) — data-bind cluster runs ~12
        unconditional sub-passes per frame (artboard_data_bind.rs:2887):
        Rust re-reads/compares every source value every frame; C++ only
        ticks converter clocks, with target writes gated by
        ComponentDirt::Bindings via the addDirtyDataBind queue
        (data_bind_container.cpp:38, data_bind.cpp:487/546). STRUCTURAL:
        port the dirty-databind architecture, not another value-compare
        cache.
    (3) STEADY #2 (~31%) — draw replay dispatch: BTreeMap gets inside
        runtime_draw_command are 8.6% self (-> dense Vec-by-slot),
        type_name string dispatch per command (-> precomputed command-
        kind enum), world transforms recomputed in draw (2.4-3.1%
        everywhere -> cache behind dirt like transform_component.cpp).
        Per-file structural item: animated_clipping rebuilds clip-path
        Vec + verbs every frame (50.7% inclusive + ~21% allocator) ->
        port ClippingShape/PathComposer clip RenderPath retention.
        Gradient re-prep per frame explains advance_blend_mode's 4.55
        (prepare 36% inclusive) -> gate stops behind Paint dirt.
    (4) SM FIXED OVERHEAD — REFUTED as a priority: C++ spends 25-30% of
        its own frame there; Rust is ~2x per-unit, the BEST bucket. Do
        not spend slices here.
    HARNESS NOTE: mach_absolute_time reads are up to 23% of tiny-file
    steady frames (4 timed sections/frame), compressing small-file
    ratios — consider timing whole repeat blocks instead of per-frame
    sections. Regime split for honest tracking: cold ai_assitant 2.41x,
    steady ~3.4x — item 16's pinned-N decision applies.

18. SCOUT REPORT — build-profile parity audit (variants built from a
    pinned snapshot in an isolated target dir; raw data + binaries in
    session scratchpad). VERDICT: the gate is currently UNFAIR AGAINST
    RUST. C++ librive.a builds at Rive's shipping config with FULL LTO
    (-flto=full is the default in rive_build_config.lua; archive members
    are LLVM bitcode) while the Rust workspace has NO [profile.release]
    anywhere — bare cargo defaults (lto off, codegen-units=16), which is
    not how a shipping Rust runtime is built. Measured, fidelity-verified
    (full golden-compare, 263 exact, diverges=0 per variant):
    (1) ADOPT: [profile.release] lto = "fat", codegen-units = 1 in the
        root Cargo.toml — aggregate ratio ~2.80 -> ~2.58 (median-agg,
        r=100), lopsided toward align_target (-12%) and animated_clipping
        (-19%). If the 122s build hurts the loop, lto = "thin" (38s) is
        also fidelity-clean; measure once before choosing.
    (2) ADOPT (with one check): panic = "abort" in release — further
        ~2.58 -> ~2.49. Verify no catch_unwind reliance in rive-capi
        consumers first (none observed in the runner path).
    (3) DO NOT ADOPT: -C target-cpu=native — passes fidelity (Rust FP is
        IEEE-strict regardless of ISA) but fails FAIRNESS: C++ builds
        generic arm64. Only symmetrically, only if a machine-tuned gate
        is ever wanted.
    (4) CODE-LEVEL NOTE, FENCED: C++ gets free FMA fusion from clang's
        default -ffp-contract=on; Rust never contracts. Closing this
        means explicit f32::mul_add at hot sites, which CHANGES float
        results — each site requires golden re-verification and may flip
        exact files; treat as last-resort under the geometry-float fence,
        never as a bulk pass.
    C++ side verdict: NOT under-built; nothing to fix there. After
    (1)+(2), remaining ~2.5x is genuine runtime-architecture cost —
    items 16/17 are the map for that. The Cargo.toml change is the
    main loop's slice to land (single-writer rule).
