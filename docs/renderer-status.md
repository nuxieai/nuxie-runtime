# Phase R Status

The execution contract is `docs/renderer-port-map.md`. This file records only
current evidence, open gates, and decisions needed by the next session.

## Metric

Run `make renderer-golden`.

- Rust wgpu: exact=1,409, diverges=0, gated=59, total=1,468.
- Stub baseline: exact=0 for every active entry.
- Exact: `first-light-rectangle-msaa`,
  `first-light-triangle-clockwise-atomic`, `gm-rect-clockwise-atomic`,
  the Dawn-WebGPU-on-Metal MSAA references for `batchedconvexpaths`,
  `batchedtriangulations`, `concavepaths`, `convex_lineonly_ths`,
  `convexpaths`, `oval`, `pathfill`, and the
  `poly_{clockwise,evenOdd,nonZero}` family,
  plus `emptyfeather`, `emptystroke`, `emptystrokefeather`,
  `emptytransparentclear`,
  `feather_corner`, `feather_cusp`, `feather_ellipse`,
  `feather_polyshapes`, `feather_roundcorner`, `feather_shapes`, and
  `feather_strokes`, plus non-mirrored Montserrat and Roboto feather text,
  `gamma_correction_clip`, and `inner_join_geometry`,
  plus `CubicStroke`, `OverStroke`, `bevel180strokes`, `bug339297`,
  `bug5099`, `bug6083`, `bug615686`, and `bug6987`,
  plus `bug7792`, `clippedcubic`, `crbug_996140`, `cubicclosepath`,
  `cubicpath`, and `emptyclear`,
  `gm-batchedconvexpaths-clockwise-atomic`, and
  `gm-path_skbug_11886-clockwise-atomic`,
  `gm-convex_lineonly_ths-clockwise-atomic`, and
  `gm-rotatedcubicpath-clockwise-atomic`,
  `gm-batchedtriangulations-clockwise-atomic`, and
  `gm-zerolinestroke-clockwise-atomic`,
  `gm-CubicStroke-clockwise-atomic`, and
  `gm-zero_control_stroke-clockwise-atomic`, and
  `gm-roundjoinstrokes-clockwise-atomic`, and
  `gm-widebuttcaps-clockwise-atomic`, and
  `gm-emptystroke-clockwise-atomic`,
  `gm-bevel180strokes-clockwise-atomic`, and
  `gm-OverStroke-clockwise-atomic`,
  `gm-strokes3-clockwise-atomic`, and
  `gm-lots_of_tess_spans_stroke-clockwise-atomic`, and
  `gm-emptyfeather-clockwise-atomic`, plus
  `first-light-direct-feather-stroke-clockwise-atomic` and
  `first-light-atlas-feather-stroke-clockwise-atomic`, and
  `gm-feather_strokes-clockwise-atomic`, and
  `gm-feather_shapes-clockwise-atomic`,
  `gm-feather_cusp-clockwise-atomic`,
  `gm-feather_ellipse-clockwise-atomic`, and
  `gm-feather_polyshapes-clockwise-atomic`, and
  `gm-feather_corner-clockwise-atomic`,
  `gm-feather_roundcorner-clockwise-atomic`, and
  `gm-cliprectintersections-{clockwise-atomic,msaa}`,
  `gm-cliprects-clockwise-atomic`,
  `gm-gamma_correction_clip-clockwise-atomic`,
  `gm-strokes_poly-clockwise-atomic`, and
  `gm-parallelclips-clockwise-atomic`, and
  `gm-clippedcubic-clockwise-atomic`,
  `gm-clippedcubic2-clockwise-atomic`,
  `gm-clippedcubic2-msaa`, and `gm-cliprects-msaa`,
  `gm-path_stroke_clip_crbug1070835-clockwise-atomic`,
  `riv-artboardclipping-frame-0-clockwise-atomic`,
  `riv-circle_clips-frame-{0..4}-clockwise-atomic`,
  `riv-clip_tests-frame-{0..4}-clockwise-atomic`, and
  `gm-emptystrokefeather-clockwise-atomic`, plus
  `gm-largeclippedpath_clockwise-clockwise-atomic` and
  `gm-largeclippedpath_clockwise_nested-clockwise-atomic`, and the
  `gm-largeclippedpath_{winding,evenodd}{,_nested}-clockwise-atomic` matrix,
  plus `gm-bug339297_as_clip-msaa` and the six
  `gm-largeclippedpath_{clockwise,evenodd,winding}{,_nested}-msaa` Dawn
  references,
  `gm-negative_interior_triangles-clockwise-atomic`, and
  `gm-negative_interior_triangles_as_clip-clockwise-atomic`, and
  `gm-convexpaths-clockwise-atomic`, `gm-pathfill-clockwise-atomic`,
  `gm-oval-clockwise-atomic`, and
  `gm-mutating_fill_rule-clockwise-atomic`, plus
  `gm-concavepaths-clockwise-atomic` and
  the `gm-poly_{clockwise,evenOdd,nonZero}-clockwise-atomic` family, plus
  `gm-cubicpath-clockwise-atomic` and
  `gm-cubicclosepath-clockwise-atomic`, plus
  `gm-beziers-{clockwise-atomic,msaa}`
  and the `gm-bug{5099,6083,615686,6987,7792}-clockwise-atomic` set, plus
  `gm-bug339297-clockwise-atomic` and
  `gm-bug339297_as_clip-clockwise-atomic`, plus
  `gm-hittest_evenOdd-{clockwise-atomic,msaa}` and
  `gm-hittest_nonZero-{clockwise-atomic,msaa}`, plus
  `gm-image_filter_options-clockwise-atomic`,
  `gm-image_lod-clockwise-atomic`, and
  `gm-image-clockwise-atomic`,
  `gm-image_aa_border-clockwise-atomic`, plus
  `gm-image-msaa`, `gm-image_aa_border-msaa`,
  `gm-image_filter_options-msaa`, and `gm-image_lod-msaa`, and
  `gm-interleavedfillrule-msaa` and the
  `gm-labyrinth_{butt,round,square}-msaa` family, and
  `gm-lots_of_tess_spans_stroke-msaa`, `gm-mandoline-msaa`,
  `gm-mesh_ht_{1,7}-msaa`, `gm-mutating_fill_rule-msaa`,
  `gm-negative_interior_triangles{,_as_clip}-msaa`, and
  `gm-overfill_{blendmodes,opaque}-msaa`, and
  `gm-overfill_transparent-msaa`,
  `gm-overstroke_{blendmodes,opaque,transparent}-msaa`,
  `gm-parallelclips-msaa`, `gm-path_skbug_{11859,11886}-msaa`,
  `gm-path_stroke_clip_crbug1070835-msaa`, `gm-quadcap-msaa`, and
  `gm-rawtext-msaa`, plus `gm-rect-msaa`, `gm-rotatedcubicpath-msaa`,
  `gm-roundjoinstrokes-msaa`, `gm-skbug12244-msaa`, `gm-strokefill-msaa`,
  `gm-strokes3-msaa`, and `gm-strokes_round-msaa`, plus
  `gm-strokes_zoomed-msaa`, `gm-teenyStrokes-msaa`,
  `gm-transparentclear{,_blendmode}-msaa`,
  `gm-trickycubicstrokes{,_feather,_roundcaps}-msaa`,
  `gm-widebuttcaps-msaa`, `gm-zeroPath-msaa`,
  `gm-zero_control_stroke-msaa`, and
  `gm-zerolinestroke-msaa`, and
  `gm-mesh-clockwise-atomic`, and
  `gm-degengrad-clockwise-atomic`,
  `gm-rect_grad-clockwise-atomic`,
  `gm-strokedlines-clockwise-atomic`,
  `gm-verycomplexgrad-clockwise-atomic`, and
  `gm-xfermodes2-clockwise-atomic`, and
  `riv-clipping_and_draw_order-frame-0-clockwise-atomic`, plus
  `riv-tape-frame-0-clockwise-atomic`, and
  `riv-superbowl-frame-0-clockwise-atomic`, and
  `riv-jellyfish_test-frame-0-clockwise-atomic`, plus
  `riv-death_knight-frame-0-clockwise-atomic`,
  `riv-deterministic_mode-frame-0-clockwise-atomic`,
  `riv-interactive_scrolling-frame-0-clockwise-atomic`,
  `riv-rocket-frame-{0..4}-clockwise-atomic`,
  `riv-scroll_test-frame-0-clockwise-atomic`,
  `riv-scroll_threshold-frame-0-clockwise-atomic`, and
  `riv-zombie_skins-frame-0-clockwise-atomic`, plus
  `riv-new_text-frame-0-clockwise-atomic` and
  `riv-ai_assitant-frame-0-clockwise-atomic`, plus
  `riv-db_health_tracker-frame-0-clockwise-atomic` and
  `riv-off_road_car-frame-{0..4}-clockwise-atomic`, plus
  `riv-joel_signed-frame-{0..4}-clockwise-atomic`, plus
  `riv-juice-frame-{0..4}-clockwise-atomic`, plus
  `riv-bad_skin-frame-0-clockwise-atomic`, plus 26 newly promoted GM entries:
  `crbug_996140`, both empty-clear cases, Montserrat and Roboto feather text,
  `inner_join_geometry`, `interleavedfillrule`, all three labyrinth variants,
  `mandoline`, both `mesh_ht` cases, transparent overfill, opaque and
  transparent overstroke, `path_skbug_11859`, `quadcap`, `skbug12244`,
  `strokes_zoomed`, `teenyStrokes`, all three tricky-cubic stroke variants,
  and both transparent-clear blend cases, plus the mirrored Montserrat and
  Roboto feather-text entries, plus
  `gm-overstroke_blendmodes-clockwise-atomic` and
  `gm-zeroPath-clockwise-atomic`, plus
  `gm-overfill_blendmodes-clockwise-atomic` and
  `gm-overfill_opaque-clockwise-atomic`, plus
  `gm-strokes_round-clockwise-atomic` and
  `gm-strokefill-clockwise-atomic`, plus
  `gm-rawtext-clockwise-atomic`, plus the zero-delta
  `first-light-nested-clip-probe-clockwise-atomic` sampled-plane oracle, plus
  `riv-advance_blend_mode-frame-{0,1}-clockwise-atomic`,
  `riv-animated_clipping-frame-0-clockwise-atomic`,
  `riv-animation_reset_cases-frame-{0..4}-clockwise-atomic`, and
  `riv-artboard_list_map_rules-frame-0-clockwise-atomic`, plus
  `riv-artboard_list_overrides-frame-0-clockwise-atomic`,
  `riv-artboard_width_test-frame-0-clockwise-atomic`,
  `riv-background_measure-frame-0-clockwise-atomic`,
  `riv-ball_test-frame-0-clockwise-atomic`,
  `riv-bankcard-frame-0-clockwise-atomic`,
  `riv-bidirectional_precedence-frame-0-clockwise-atomic`, and
  `riv-bindable_artboard_child-frame-{0..7}-clockwise-atomic`, plus
  `riv-blend_test-frame-{0..4}-clockwise-atomic`, plus
  `riv-clear_viewmodel_list-frame-{0..4}-clockwise-atomic` and
  `riv-click_event-frame-{0..7}-clockwise-atomic`, plus
  `riv-collapsable_data_binding-frame-0-clockwise-atomic`,
  `riv-complex_ik_dependency-frame-0-clockwise-atomic`, and
  `riv-component_based_conditions-frame-{0..4}-clockwise-atomic`, plus
  `riv-component_list_1-frame-0-clockwise-atomic`,
  `riv-component_list_2-frame-{0..4}-clockwise-atomic`,
  `riv-component_list_child_origin-frame-0-clockwise-atomic`, and
  `riv-component_list_follow_path-frame-0-clockwise-atomic`, plus
  `riv-component_list_follow_path_distance-frame-0-clockwise-atomic`,
  `riv-component_list_grouped-frame-{0..4}-clockwise-atomic`, and
  `riv-component_list_hit_order-frame-{0..4}-clockwise-atomic`, plus
  `riv-component_list_virtualized-frame-0-clockwise-atomic`, both
  `riv-component_stateful_vm_instance` fixtures,
  `riv-computed_root_transform-frame-0-clockwise-atomic`, and
  `riv-cubic_value_test-frame-{0..4}-clockwise-atomic`, plus
  `riv-custom_image_name-frame-0-clockwise-atomic`,
  `riv-custom_property_enum-frame-0-clockwise-atomic`,
  `riv-custom_property_trigger-frame-0-clockwise-atomic`,
  `riv-data_bind_artboard_input-frame-0-clockwise-atomic`, and
  `riv-data_bind_solo-frame-{0..4}-clockwise-atomic`, plus
  `riv-data_binding_test_2-frame-{0..4}-clockwise-atomic`,
  `riv-data_binding_test_3-frame-0-clockwise-atomic`,
  `riv-data_binding_test_triggers-frame-0-clockwise-atomic`,
  `riv-data_converter_interpolator_reset-frame-0-clockwise-atomic`,
  `riv-databind_artboard-frame-0-clockwise-atomic`, both
  `riv-databind_external_artboard` fixtures, and
  `riv-databind_solo_to_enum-frame-0-clockwise-atomic`, plus
  `riv-dependency_test-frame-{0..4}-clockwise-atomic`,
  `riv-distance_constraint-frame-0-clockwise-atomic`,
  `riv-double_library_with_image-frame-0-clockwise-atomic`, and
  `riv-drag_event-frame-0-clockwise-atomic`, plus
  `riv-draw_index_list-frame-0-clockwise-atomic`,
  `riv-draw_rule_cycle-frame-{0..4}-clockwise-atomic`,
  `riv-entry-frame-0-clockwise-atomic`, and
  `riv-event_on_listener-frame-{0..7}-clockwise-atomic`, plus
  `riv-event_trigger_event-frame-{0..7}-clockwise-atomic` and
  `riv-events_on_states-frame-{0..7}-clockwise-atomic`, plus
  `riv-feather_render_test-frame-0-clockwise-atomic`,
  `riv-fill_trim_path-frame-{0..4}-clockwise-atomic`, and
  `riv-fix_rectangle-frame-{0..4}-clockwise-atomic`, plus
  `riv-focus_collapsing-frame-0-clockwise-atomic`,
  `riv-focusable_element-frame-0-clockwise-atomic`,
  `riv-follow_path-frame-0-clockwise-atomic`, and
  `riv-follow_path_constraint-frame-0-clockwise-atomic`, plus
  `riv-follow_path_path_0_opacity-frame-0-clockwise-atomic`,
  `riv-follow_path_shapes-frame-0-clockwise-atomic`,
  `riv-follow_path_solos-frame-0-clockwise-atomic`,
  `riv-follow_path_with_0_opacity-frame-0-clockwise-atomic`,
  `riv-formula_random-frame-0-clockwise-atomic`,
  `riv-gamepad_test-frame-{0,1}-clockwise-atomic`, and
  `riv-group_effect-frame-0-clockwise-atomic`, plus
  `riv-hide_test-frame-0-clockwise-atomic`,
  `riv-hit_test_nested-frame-0-clockwise-atomic`, and
  `riv-hit_test_solos-frame-{0..7}-clockwise-atomic`, plus
  `riv-hit_test_test-frame-0-clockwise-atomic`,
  `riv-hittest_collapsed_layouts-frame-0-clockwise-atomic`,
  `riv-hosted_font_file-frame-0-clockwise-atomic`,
  `riv-hosted_image_file-frame-0-clockwise-atomic`,
  `riv-image_binding_with_listener-frame-0-clockwise-atomic`,
  `riv-image_fit_alignment{,_2,_3}-frame-0-clockwise-atomic`,
  `riv-image_scripting_property_value-frame-0-clockwise-atomic`, and
  `riv-in_band_asset-frame-0-clockwise-atomic`, plus
  `riv-interpolation_zero_duration-frame-0-clockwise-atomic`,
  `riv-joel_v3-frame-0-clockwise-atomic`,
  `riv-joystick_flag_test-frame-{0..4}-clockwise-atomic`,
  `riv-joystick_nested_remap-frame-{0..4}-clockwise-atomic`, and
  `riv-keyboard_event_to_script-frame-{0..4}-clockwise-atomic`, plus
  `riv-library_data_enum_test-frame-{0..4}-clockwise-atomic`,
  `riv-library_export_animation_test-frame-0-clockwise-atomic`,
  `riv-library_export_state_machine_test-frame-0-clockwise-atomic`,
  `riv-library_export_test-frame-0-clockwise-atomic`,
  `riv-library_view_model_test-frame-0-clockwise-atomic`, and
  `riv-library_vmtest_1_host-frame-0-clockwise-atomic`, plus
  `riv-library_with_image-frame-0-clockwise-atomic`,
  `riv-library_with_text_and_image-frame-0-clockwise-atomic`, and
  `riv-light_switch-frame-{0..7}-clockwise-atomic`, plus
  `riv-list_index_script_access-frame-0-clockwise-atomic`,
  `riv-list_items-frame-0-clockwise-atomic`,
  `riv-list_to_length_test-frame-0-clockwise-atomic`,
  `riv-list_to_path-frame-{0..4}-clockwise-atomic`, and
  `riv-listener_action_inputs-frame-0-clockwise-atomic`, plus
  `riv-lock_icon_demo-frame-{0..4}-clockwise-atomic` and
  `riv-long_name-frame-{0..4}-clockwise-atomic`, plus
  `riv-looping_timeline_events-frame-{0..4}-clockwise-atomic`,
  `riv-magic_alley_db_reduced_export-frame-0-clockwise-atomic`,
  `riv-multiple_state_machines-frame-{0..4}-clockwise-atomic`,
  `riv-multitouch-frame-0-clockwise-atomic`,
  `riv-multitouch_enter-frame-0-clockwise-atomic`,
  `riv-n_slice_triangle-frame-0-clockwise-atomic`, and
  `riv-nested_artboard_opacity-frame-0-clockwise-atomic`, plus
  `riv-nested_artboard_quantize_and_speed-frame-{0..4}-clockwise-atomic`,
  `riv-nested_event_test-frame-0-clockwise-atomic`,
  `riv-nested_events-frame-0-clockwise-atomic`, and
  `riv-nested_needs_advance-frame-0-clockwise-atomic`, plus
  `riv-number_to_list_nested_children-frame-0-clockwise-atomic`,
  `riv-oneshotblend-frame-{0..4}-clockwise-atomic`,
  `riv-opaque_hit_test-frame-{0..7}-clockwise-atomic`,
  `riv-path_effect_with_feathers-frame-0-clockwise-atomic`,
  `riv-pause_nested_artboard-frame-0-clockwise-atomic`,
  `riv-pointer_events-frame-{0..7}-clockwise-atomic`, and
  `riv-pointer_events_nested_artboards_in_solos-frame-{0..7}-clockwise-atomic`,
  plus `riv-quantize_test-frame-{0..4}-clockwise-atomic`,
  `riv-rapid_pointer_events-frame-{0..7}-clockwise-atomic`,
  `riv-rebind_with_nested_viewmodel-frame-0-clockwise-atomic`,
  `riv-recursive_data_bind-frame-0-clockwise-atomic`,
  `riv-relative_data_bind_path-frame-{0,1}-clockwise-atomic`,
  `riv-relative_data_binding-frame-0-clockwise-atomic`,
  `riv-remove_from_list-frame-{0..4}-clockwise-atomic`, and
  `riv-replace_view_model-frame-{0,1}-clockwise-atomic`, plus
  `riv-reset_phase-frame-0-clockwise-atomic`,
  `riv-reuse_path_in_effect-frame-0-clockwise-atomic`,
  `riv-rotation_constraint-frame-0-clockwise-atomic`,
  `riv-runtime_nested_inputs-frame-{0,1}-clockwise-atomic`,
  `riv-scale_constraint-frame-0-clockwise-atomic`,
  `riv-script_affects_has_changed-frame-0-clockwise-atomic`,
  `riv-script_artboard_{opacity_test,origin_test,test}-frame-0-clockwise-atomic`,
  `riv-script_create_text_runs-frame-{0,1}-clockwise-atomic`,
  `riv-script_create_viewmodel_instance-frame-0-clockwise-atomic`,
  `riv-script_dependency_test-frame-0-clockwise-atomic`,
  `riv-script_dependency_test2-frame-0-clockwise-atomic`,
  `riv-script_dependency_test_using_library{,_v2}-frame-0-clockwise-atomic`,
  `riv-script_inputs_test_1-frame-0-clockwise-atomic`,
  `riv-script_layout_test-frame-0-clockwise-atomic`,
  `riv-script_namespace_test-frame-0-clockwise-atomic`,
  `riv-script_path_effects_test-frame-0-clockwise-atomic`, and
  `riv-script_paths_opacity_test-frame-{0..4}-clockwise-atomic`, plus
  `riv-script_paths_test-frame-{0..4}-clockwise-atomic`,
  `riv-script_string_converter_test-frame-0-clockwise-atomic`,
  `riv-scripted_as_path-frame-0-clockwise-atomic`,
  `riv-scripted_boolean-frame-{0..4}-clockwise-atomic`,
  `riv-scripted_color-frame-0-clockwise-atomic`,
  `riv-scripted_data_context-frame-0-clockwise-atomic`,
  `riv-scripted_data_converter_bound_input-frame-{0,1}-clockwise-atomic`,
  `riv-scripted_enum-frame-{0..4}-clockwise-atomic`,
  `riv-scripted_graph-frame-{0..4}-clockwise-atomic`,
  `riv-scripted_listener_action-frame-0-clockwise-atomic`,
  `riv-scripted_listener_context-frame-0-clockwise-atomic`,
  `riv-scripted_memory_leak-frame-0-clockwise-atomic`,
  `riv-scripted_property_image-frame-0-clockwise-atomic`,
  `riv-scripted_string-frame-{0..4}-clockwise-atomic`,
  `riv-scripted_transition_condition-frame-0-clockwise-atomic`,
  `riv-scripted_viewmodel_cache-frame-0-clockwise-atomic`,
  `riv-scripting_linear_animation-frame-{0,1}-clockwise-atomic`,
  `riv-scripting_root_viewmodel-frame-0-clockwise-atomic`,
  `riv-settler-frame-{0..4}-clockwise-atomic`,
  `riv-shapetest-frame-0-clockwise-atomic`,
  `riv-shared_viewmodel_instance-frame-0-clockwise-atomic`,
  `riv-smi_test-frame-0-clockwise-atomic`,
  `riv-solid_affects_has_changed-frame-0-clockwise-atomic`,
  `riv-solo_test-frame-{0..4}-clockwise-atomic`,
  `riv-solos_collapse_tests-frame-{0..4}-clockwise-atomic`,
  `riv-solos_with_nested_artboards-frame-{0..4}-clockwise-atomic`,
  `riv-sorted_listeners-frame-0-clockwise-atomic`,
  `riv-sound-frame-{0..8}-clockwise-atomic`,
  `riv-sound2-frame-{0..4}-clockwise-atomic`,
  `riv-spotify_kids_demo-frame-0-clockwise-atomic`,
  `riv-stacked_path_effects-frame-{0..4}-clockwise-atomic`,
  `riv-state_machine_transition-frame-{0..7}-clockwise-atomic`,
  `riv-state_machine_triggers-frame-{0..7}-clockwise-atomic`,
  `riv-state_transition_fire_trigger-frame-0-clockwise-atomic`,
  `riv-stateful_artboard_swap-frame-0-clockwise-atomic`,
  `riv-stateful_keyed_trigger-frame-0-clockwise-atomic`,
  `riv-stateful_list_props-frame-{0..4}-clockwise-atomic`,
  `riv-stateful_multi_property-frame-0-clockwise-atomic`,
  `riv-stateful_nested-frame-0-clockwise-atomic`,
  `riv-stateful_source_switch-frame-0-clockwise-atomic`,
  `riv-stroke_name_test-frame-{0..4}-clockwise-atomic`,
  `riv-target_event-frame-0-clockwise-atomic`,
  `riv-test_elastic-frame-{0..4}-clockwise-atomic`,
  `riv-text_input_event-frame-{0..4}-clockwise-atomic`,
  `riv-text_opacity_modifier-frame-0-clockwise-atomic`,
  `riv-text_stroke_test-frame-0-clockwise-atomic`,
  `riv-time_based_interpolation-frame-0-clockwise-atomic`,
  `riv-timeline_event_test-frame-{0..4}-clockwise-atomic`,
  `riv-transform_constraint-frame-0-clockwise-atomic`,
  `riv-transition_artboard_condition_test-frame-0-clockwise-atomic`,
  `riv-transition_duration_bind_{list,nested}-frame-0-clockwise-atomic`,
  `riv-transition_index_condition-frame-0-clockwise-atomic`,
  `riv-transition_self_comparator_test-frame-0-clockwise-atomic`,
  `riv-translation_constraint-frame-0-clockwise-atomic`,
  `riv-trigger_based_listeners-frame-0-clockwise-atomic`,
  `riv-trim-frame-0-clockwise-atomic`,
  `riv-trim_path-frame-{0..4}-clockwise-atomic`,
  `riv-trim_path_linear-frame-{0..4}-clockwise-atomic`,
  `riv-two_artboards-frame-{0..4}-clockwise-atomic`,
  `riv-two_bone_ik-frame-0-clockwise-atomic`,
  `riv-unbound_stateful_component-frame-0-clockwise-atomic`,
  `riv-viewmodel_based_condition-frame-0-clockwise-atomic`,
  `riv-viewmodel_from_context-frame-0-clockwise-atomic`,
  `riv-viewmodel_from_instance-frame-{0,1}-clockwise-atomic`,
  `riv-viewmodel_image_reset-frame-0-clockwise-atomic`,
  `riv-viewmodel_instance_to_artboard-frame-0-clockwise-atomic`,
  `riv-viewmodel_list_trigger-frame-0-clockwise-atomic`,
  `riv-viewmodel_runtime_file-frame-{0..4}-clockwise-atomic`,
  `riv-virtualize_blendmode-frame-0-clockwise-atomic`,
  `riv-virtualized_artboard_databound_children-frame-0-clockwise-atomic`,
  `riv-walle-frame-0-clockwise-atomic`,
  `riv-word_joiner_test-frame-0-clockwise-atomic`, and
  `riv-zero_width_space_line_break-frame-0-clockwise-atomic`, plus 583 of the
  624 provenance-bound strict RIV MSAA rows, plus 17 newly exact MSAA
  gradient rows: `gm-{degengrad,rect_grad,verycomplexgrad}`;
  `riv-{bad_skin,bankcard,coin,db_health_tracker,deterministic_mode,new_text}`;
  all five `riv-rocket` frames; `riv-{scroll_test,scroll_threshold,zombie_skins}`;
  and the MSAA image-mesh rows `gm-mesh` and `riv-tape-frame-0`.
  The retained rows are
  queryable by their concrete diagnostics in `corpus-r.toml`.

## Milestones

- [x] R0: Pixel golden harness. Parser/replay, PNG comparator, artifacts,
  manifest ratchet, checked-in references, stub baseline, and CI are landed.
  The oracle contains 108 upstream GM streams, 294 valid `.riv` streams, 735
  legacy native Metal references, and 1,466 clockwise-atomic/MSAA entries. The
  pre-existing `solar-system` import error and 33 direct RenderContext/ORE GM
  source files have named gates.
- [x] R1: wgpu foundation and first light. Device/queue/offscreen readback,
  retained render-api objects, state stack, generated WGSL validation, 4x MSAA
  bootstrap coverage, one GM stream, and one real `.riv` stream are exact.
- [x] R2: Algorithm core.
- [x] R3: Corpus convergence.
- [x] R3.1: Retained gate burn-down.
- [ ] R4: Performance parity.
- [ ] R5: Native fast paths and extensions; demand-gated after R4.

## Next

1. [x] Build the R3 renderer fuzz-replay harness for both C++ and Rust with
   NaN/huge transforms, zero-area paths, absurd stroke widths, deep clip
   stacks, and hostile gradient stops. Rust must not panic, hang, or lose the
   device; behavioral deltas become named findings and a smoke gate enters CI.
2. [x] Probe the first ten gated clockwise-atomic `.riv` entries against their
   pinned Metal references: `advance_blend_mode` frames 0-1, `align_target`,
   `animated_clipping`, `animation_reset_cases` frames 0-4, and
   `artboard_list_map_rules`. Promote unchanged-contract passes and replace
   the first failing `algorithm-core` placeholder with an evidence-backed
   diagnostic.
3. [x] Probe the next ten gated clockwise-atomic `.riv` entries:
   `artboard_list_overrides`, `artboard_width_test`, `audio_script`,
   `background_measure`, `ball_test`, `bidirectional_precedence`, and
   `bindable_artboard_child` frames 0-3. Capture their missing pinned Metal
   references first, then promote unchanged-contract passes and diagnose the
   first failure without widening tolerance.
4. [x] Probe the next ten gated clockwise-atomic `.riv` entries:
   `bindable_artboard_child` frames 4-7,
   `bindable_artboard_nesty` frame 0, and `blend_test` frames 0-4. Capture
   their missing pinned Metal references first, then apply the same unchanged
   contract and diagnostic rules.
5. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
   entries: `clear_viewmodel_list` frames 0-4 and `click_event` frames 0-4.
   Capture missing pinned Metal references, preserve the unchanged `2/32`
   contract, and classify any first failure before promotion.
6. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
   entries: `click_event` frames 5-7, `collapsable_data_binding`,
   `collapse_data_binds`, `collapsing_elements`, `complex_ik_dependency`, and
   `component_based_conditions` frames 0-2. Capture missing pinned Metal
   references and apply the unchanged contract and diagnostic rules.
7. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
   entries: `component_based_conditions` frames 3-4, `component_list_1`,
   `component_list_2` frames 0-4, `component_list_child_origin`, and
   `component_list_follow_path`. Capture missing pinned Metal references and
   apply the unchanged contract and diagnostic rules.
8. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
   entries: `component_list_follow_path_distance`, `component_list_grouped`
   frames 0-4, and `component_list_hit_order` frames 0-3. Capture missing
   pinned Metal references and apply the unchanged contract and diagnostic
   rules.
9. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
   entries: `component_list_hit_order` frame 4, `component_list_virtualized`,
   `component_stateful`, both `component_stateful_vm_instance` fixtures,
   `computed_root_transform`, `computed_values_test`, and `cubic_value_test`
   frames 0-2. Capture missing pinned Metal references and apply the unchanged
   contract and diagnostic rules.
10. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `cubic_value_test` frames 3-4, `custom_image_name`,
    `custom_property_enum`, `custom_property_trigger`,
    `data_bind_artboard_input`, and `data_bind_solo` frames 0-3. Capture
    missing pinned Metal references and apply the unchanged contract and
    diagnostic rules.
11. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `data_bind_solo` frame 4, `data_bind_test_cmdq`,
    `data_binding_artboards_source_test`, `data_binding_artboards_test`,
    `data_binding_images_test`, `data_binding_test`, and `data_binding_test_2`
    frames 0-3. Capture missing pinned Metal references and apply the
    unchanged contract and diagnostic rules.
12. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `data_binding_test_2` frame 4, `data_binding_test_3`,
    `data_binding_test_triggers`, `data_converter_interpolator_reset`,
    `data_converter_to_number`, `databind_artboard`, both
    `databind_external_artboard` fixtures, `databind_solo_to_enum`, and
    `databind_viewmodel`. Capture missing pinned Metal references and apply
    the unchanged contract and diagnostic rules.
13. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `databind_viewmodel` frame 1, `dependency_test` frames 0-4,
    `distance_constraint`, `double_library_with_image`, `double_line`, and
    `drag_event`. Capture missing pinned Metal references and apply the
    unchanged contract and diagnostic rules.
14. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `draw_index_list`, `draw_rule_cycle` frames 0-4, `ellipsis`,
    `entry`, and `event_on_listener` frames 0-1. Capture missing pinned Metal
    references and apply the unchanged contract and diagnostic rules.
15. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `event_on_listener` frames 2-7 and `event_trigger_event` frames
    0-3. Capture missing pinned Metal references and apply the unchanged
    contract and diagnostic rules.
16. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `event_trigger_event` frames 4-7 and `events_on_states` frames
    0-5. Capture missing pinned Metal references and apply the unchanged
    contract and diagnostic rules.
17. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `events_on_states` frames 6-7, `feather_render_test`,
    `fill_trim_path` frames 0-4, `fit_font_size_test`, and `fix_rectangle`
    frame 0. Capture missing pinned Metal references and apply the unchanged
    contract and diagnostic rules.
18. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `fix_rectangle` frames 1-4, `focus_collapsing`,
    `focus_traversal`, `focusable_element`, `follow_path`,
    `follow_path_constraint`, and `follow_path_path`. Capture missing pinned
    Metal references and apply the unchanged contract and diagnostic rules.
19. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `follow_path_path_0_opacity`, `follow_path_shapes`,
    `follow_path_solos`, `follow_path_with_0_opacity`,
    `format_number_with_commas`, `formula_random`, `gamepad_test` frames 0-1,
    `group_effect`, and `hello_world`. Capture missing pinned Metal references
    and apply the unchanged contract and diagnostic rules.
20. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `hide_test`, `hit_test_nested`, and `hit_test_solos` frames 0-7.
    Capture missing pinned Metal references and apply the unchanged contract
    and diagnostic rules.
21. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `hit_test_test`, `hittest_collapsed_layouts`, `hosted_font_file`,
    `hosted_image_file`, `image_binding_with_listener`,
    `image_fit_alignment`, `image_fit_alignment_2`, `image_fit_alignment_3`,
    `image_scripting_property_value`, and `in_band_asset`. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
22. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `interpolate_to_end`, `interpolation_zero_duration`, `joel_v3`,
    `joystick_flag_test` frames 0-4, and `joystick_nested_remap` frames 0-1.
    Capture missing pinned Metal references and apply the unchanged contract
    and diagnostic rules.
23. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `joystick_nested_remap` frames 2-4,
    `keyboard_event_to_script` frames 0-4, `keyboard_listener`, and `library`.
    Capture missing pinned Metal references and apply the unchanged contract
    and diagnostic rules.
24. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `library_data_enum_test` frames 0-4,
    `library_export_animation_test`, `library_export_state_machine_test`,
    `library_export_test`, `library_view_model_test`, and
    `library_vmtest_1_host`. Capture missing pinned Metal references and apply
    the unchanged contract and diagnostic rules.
25. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `library_with_image`, `library_with_text_and_image`, and
    `light_switch` frames 0-7. Capture missing pinned Metal references and
    apply the unchanged contract and diagnostic rules.
26. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `list_index_script_access`, `list_items`, `list_to_length_test`,
    `list_to_path` frames 0-4, `listener_action_inputs`, and
    `listener_view_model`. Capture missing pinned Metal references and apply
    the unchanged contract and diagnostic rules.
27. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `local_bounds`, `lock_icon_demo` frames 0-4, and `long_name`
    frames 0-3. Capture missing pinned Metal references and apply the
    unchanged contract and diagnostic rules.
28. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `long_name` frame 4, `looping_timeline_events` frames 0-4,
    `magic_alley_db_reduced_export`, `modifier_test`, `modifier_to_run`, and
    `multi_listeners`. Capture missing pinned Metal references and apply the
    unchanged contract and diagnostic rules.
29. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `multiple_state_machines` frames 0-4, `multitouch`,
    `multitouch_enter`, `n_slice_triangle`, `nested_artboard_opacity`, and
    `nested_artboard_quantize_and_speed` frame 0. Capture missing pinned
    Metal references and apply the unchanged contract and diagnostic rules.
30. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `nested_artboard_quantize_and_speed` frames 1-4,
    `nested_event_test`, `nested_events`, `nested_hug`,
    `nested_needs_advance`, and `nested_solo` frames 0-1. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
31. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `nested_solo` frames 2-4, `number_to_list_nested_children`,
    `oneshotblend` frames 0-4, and `opaque_hit_test`. Capture missing pinned
    Metal references and apply the unchanged contract and diagnostic rules.
32. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `opaque_hit_test` frames 1-7,
    `path_effect_with_feathers`, `pause_nested_artboard`, and
    `pointer_events` frame 0. Capture missing pinned Metal references and
    apply the unchanged contract and diagnostic rules.
33. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `pointer_events` frames 1-7 and
    `pointer_events_nested_artboards_in_solos` frames 0-2. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
34. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `pointer_events_nested_artboards_in_solos` frames 3-7,
    `pointer_exit`, and `quantize_test` frames 0-3. Capture missing pinned
    Metal references and apply the unchanged contract and diagnostic rules.
35. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `quantize_test` frame 4, `rapid_pointer_events` frames 0-7,
    and `rebind_with_nested_viewmodel`. Capture missing pinned Metal
    references and apply the unchanged contract and diagnostic rules.
36. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `recursive_data_bind`, `relative_data_bind_path` frames 0-1,
    `relative_data_binding`, `remove_from_list` frames 0-4, and
    `replace_view_model` frame 0. Capture missing pinned Metal references
    and apply the unchanged contract and diagnostic rules.
37. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `replace_view_model` frame 1, `replace_vm_instance`,
    `reset_phase`, `reuse_path_in_effect`, `rotation_constraint`,
    `runtime_nested_inputs` frames 0-1, `runtime_nested_text_runs`,
    `saturation`, and `scale_constraint`. Capture missing pinned Metal
    references and apply the unchanged contract and diagnostic rules.
38. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `script_affects_has_changed`, the three `script_artboard`
    tests, `script_create_text_runs` frames 0-1,
    `script_create_viewmodel_instance`, both local `script_dependency`
    tests, and `script_dependency_test_using_library`. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
39. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `script_dependency_test_using_library_v2`,
    `script_inputs_test_1`, `script_layout_test`, `script_namespace_test`,
    `script_path_effects_test`, and `script_paths_opacity_test` frames 0-4.
    Capture missing pinned Metal references and apply the unchanged contract
    and diagnostic rules.
40. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `script_paths_test` frames 0-4,
    `script_string_converter_test`, `scripted_as_path`, and
    `scripted_boolean` frames 0-2. Capture missing pinned Metal references
    and apply the unchanged contract and diagnostic rules.
41. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `scripted_boolean` frames 3-4, `scripted_color`,
    `scripted_data_context`, `scripted_data_converter_bound_input` frames
    0-1, and `scripted_enum` frames 0-3. Capture missing pinned Metal
    references and apply the unchanged contract and diagnostic rules.
42. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `scripted_enum` frame 4, `scripted_graph` frames 0-4,
    `scripted_listener_action`, `scripted_listener_context`,
    `scripted_memory_leak`, and `scripted_property_image`. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
43. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `scripted_string` frames 0-4,
    `scripted_transition_condition`, `scripted_viewmodel_cache`,
    `scripting_linear_animation` frames 0-1, and
    `scripting_root_viewmodel`. Capture missing pinned Metal references and
    apply the unchanged contract and diagnostic rules.
44. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `scroll_snap`, `settler` frames 0-4, `shapetest`,
    `shared_viewmodel_instance`, `smi_test`, and
    `solid_affects_has_changed`. Capture missing pinned Metal references and
    apply the unchanged contract and diagnostic rules.
45. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `solo_test` frames 0-4 and `solos_collapse_tests` frames 0-4.
    Capture missing pinned Metal references and apply the unchanged contract
    and diagnostic rules.
46. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `solos_with_nested_artboards` frames 0-4, `sorted_listeners`,
    and `sound` frames 0-3. Capture missing pinned Metal references and apply
    the unchanged contract and diagnostic rules.
47. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `sound` frames 4-8 and `sound2` frames 0-4. Capture missing
    pinned Metal references and apply the unchanged contract and diagnostic
    rules.
48. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `spotify_kids_app_icon`, `spotify_kids_demo`,
    `stacked_path_effects` frames 0-4, and `state_machine_transition` frames
    0-2. Capture missing pinned Metal references and apply the unchanged
    contract and diagnostic rules.
49. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `state_machine_transition` frames 3-7 and
    `state_machine_triggers` frames 0-4. Capture missing pinned Metal
    references and apply the unchanged contract and diagnostic rules.
50. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `state_machine_triggers` frames 5-7,
    `state_transition_fire_trigger`, `stateful_artboard_swap`,
    `stateful_keyed_trigger`, and `stateful_list_props` frames 0-3.
51. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `stateful_list_props` frame 4, `stateful_multi_property`,
    `stateful_nested`, `stateful_source_switch`, `stroke_name_test` frames
    0-4, and `target_event`.
52. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `test_elastic` frames 0-4, `test_modifier_run`,
    `text_follow_path_shape_length`, `text_input`, and `text_input_event`
    frames 0-1.
53. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `text_input_event` frames 2-4, `text_listener_simpler`,
    `text_opacity_modifier`, `text_stroke_test`, `text_vertical_trim_test`,
    `time_based_interpolation`, and `timeline_event_test` frames 0-1.
54. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `timeline_event_test` frames 2-4, `transform_constraint`,
    `transition_actions`, `transition_artboard_condition_test`, both
    `transition_duration_bind` fixtures, `transition_index_condition`, and
    `transition_self_comparator_test`.
55. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `translation_constraint`, `trigger_based_listeners`,
    `trigger_fires_single_change`, `trim`, `trim_path` frames 0-4, and
    `trim_path_linear` frame 0.
56. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `trim_path_linear` frames 1-4, `two_artboards` frames 0-4, and
    `two_bone_ik`.
57. [x] Probe the next ten `algorithm-core` gated clockwise-atomic `.riv`
    entries: `unbound_stateful_component`, `vertical_align_ellipsis`,
    `viewmodel_access`, `viewmodel_based_condition`,
    `viewmodel_from_context`, `viewmodel_from_instance` frames 0-1,
    `viewmodel_image_reset`, `viewmodel_instance_to_artboard`, and
    `viewmodel_list_trigger`.
58. [x] Probe the final ten unprobed `algorithm-core` gated
    clockwise-atomic `.riv` entries: `viewmodel_runtime_file` frames 0-4,
    `virtualize_blendmode`, `virtualized_artboard_databound_children`,
    `walle`, `word_joiner_test`, and `zero_width_space_line_break`.
59. [x] Build the pinned C++ Dawn full-stream and representative atomic
    coverage/color-plane oracle for `spotify_kids_app_icon`. The strict replay
    and 24-batch C++ schedule are provenance-bound; C++ Dawn and Rust match the
    final clip and pass `2/32` against one another, allocate no packed color
    plane, and retain 48,759/48,790-pixel native-Metal residuals with more than
    99.9% mask overlap. Sol approved the separate
    `metal-webgpu-fixed-function-color-output` gate without changing the
    reference or contract.
60. [x] Build the reusable C++ Dawn WebGPU-on-Metal MSAA reference runner and
    provenance-bound PNG capture path, then probe the first solid-path upstream
    GMs in disjoint Terra batches. Native C++ Metal has no MSAA flush and the
    corpus currently has zero MSAA reference PNGs, so references must live
    under a distinct Dawn-WebGPU-on-Metal identity rather than the native Metal
    root. The strict generator now accepts 39 of the first 40 candidate streams;
    `degengrad` remains the one gradient-resource compiler gap.
61. [x] Isolate the shared Dawn-versus-wgpu MSAA fill-rule divergence across
    `poly_clockwise`, `poly_evenOdd`, and `poly_nonZero`. Start with the exact
    draw schedule and stencil/coverage intermediate rather than widening the
    unchanged `2/32` contract; use the result to determine whether
    `concavepaths` and `pathfill` share the same failure class. All five shared
    the missing C++ midpoint-fan fill schedule and intersection-board depth
    group state;
    the translated stencil/depth subpasses make all five exact at zero pixels
    beyond the unchanged threshold.
62. [x] Add and capture the next ten strict source-order C++ Dawn MSAA cases,
    then probe them under the unchanged `2/32` contract: `CubicStroke`,
    `OverStroke`, `bevel180strokes`, `beziers`, `bug339297`,
    `bug339297_as_clip`, `bug5099`, `bug6083`, `bug615686`, and `bug6987`.
    The expanded 20-case registry recaptures the original ten byte-identically.
    Eight new cases pass and are promoted. `beziers` retains 5,385 pixels/max
    152 under both grouped and serialized execution, proving a cubic-stroke
    raster gap rather than a scheduling gap. `bug339297_as_clip` reaches the
    existing explicit non-atlas MSAA path-clip boundary.
63. [x] Add and capture the next ten uncaptured strict source-order C++ Dawn
    MSAA cases, then probe them under the unchanged `2/32` contract:
    `bug7792`, `clippedcubic`, `clippedcubic2`, `cliprectintersections`,
    `cliprects`, `crbug_996140`, `cubicclosepath`, `cubicpath`,
    `dstreadshuffle`, and `emptyclear`. The strict compiler rejects the
    intervening `degengrad` gradient-resource stream, so it remains outside
    this path-only capture wave. The original 20 PNGs recapture
    byte-identically. Six cases pass and are promoted. `clippedcubic2`,
    `cliprectintersections`, and `cliprects` expose the direct MSAA path's
    generated `noclipdistance` vertex variant; `dstreadshuffle` exposes the
    direct fixed-color path's missing destination-read advanced blend path.
    The registry now explicitly permits a no-draw replay only when the strict
    stream counts declare no `drawPath`, allowing `emptyclear` to capture
    without weakening accidental-empty replay detection.
64. [x] Port C++'s generated clip-distance MSAA direct-path vertex variant and
    select it for `PaintData::HAS_CLIP_RECT`, with a focused GPU regression.
    Reprobe `clippedcubic2`, `cliprectintersections`, and `cliprects` against
    their captured C++ Dawn references before changing their gates. The seven
    direct draw states now select byte-exact upstream clipped/unclipped vertex
    variants behind `CLIP_DISTANCES`; unsupported adapters fail closed.
    `clippedcubic2` and `cliprects` pass at zero pixels beyond `2/32` and are
    promoted. `cliprectintersections` retains 240 pixels/max 55 in sparse
    one-pixel edge/corner components and stays gated as
    `msaa-clip-intersection-edge-coverage` without a tolerance change.
65. [x] Port direct MSAA destination-read advanced blending for solid path
    draws, then reprobe the captured `dstreadshuffle` reference. Preserve the
    existing atlas destination-copy path and unchanged `2/32` contract. The
    direct path now uses upstream's generated advanced/HSL fragment variants,
    destination binding 13, and per-draw resolve/copy/reload barriers with
    bounded pixel-copy regions. Focused GPU regressions cover destination
    reads, fixed-to-advanced and consecutive advanced/HSL ordering, analytic
    strokes, fill-rule color passes, and empty copy bounds. All renderer tests
    pass. With C++'s dither enabled, `dstreadshuffle`
    improves from 24,130 pixels/max 49 to 2,231/max 43; alpha is exact and a
    full-frame copy is byte-identical to the bounded copy. It remains gated as
    `dawn-wgpu-msaa-advanced-blend-intermediate-precision` under unchanged
    `2/32` rather than fitting a tolerance to the residual.
66. [x] Add and capture the next ten strict source-order C++ Dawn MSAA cases,
    then probe them under the unchanged `2/32` contract: `emptyfeather`,
    `emptystroke`, `emptystrokefeather`, `emptytransparentclear`,
    `feather_corner`, `feather_cusp`, `feather_ellipse`,
    `feather_polyshapes`, `feather_roundcorner`, and `feather_shapes`. The
    strict compiler accepts all ten. All prior 30 PNGs recapture
    byte-identically. Seven rows pass unchanged `2/32` and are promoted.
    `emptystrokefeather` remains gated as
    `msaa-empty-feather-stroke-inner-coverage` at 1,728 pixels/max 174: its 36
    equal-size components cover only degenerate cap centers, with exact alpha
    and matching feather halos. `feather_cusp` remains at 435/max 4 and
    `feather_shapes` at 180/max 4 under
    `msaa-atlas-feather-large-radius-coverage-precision`: their exact-alpha
    residuals are isolated to sparse, mostly single-pixel larger-radius atlas
    feather contours. Sol approved the first gate and required the less-causal
    precision name for the latter two. No tolerance changed.
67. [x] Port C++'s degenerate feather-stroke cap inner coverage through the
    MSAA atlas path, then reprobe `emptystrokefeather` against its pinned C++
    Dawn reference. Preserve the already-matching halo, all seven newly exact
    siblings, and the unchanged `2/32` contract. The focused oracle must prove
    the 36 cap-center components gain inner stroke coverage without changing
    nondegenerate feather strokes. The atlas branch was discarding the
    intersection-board draw group and uploading `z_index=1`; its `Less` depth
    test therefore rejected cap coverage over earlier marker draws while the
    untouched halo still passed. Carrying the scheduled index makes the paired
    C++/Rust marker-cap oracle byte-exact and promotes the GM at zero pixels
    beyond delta 2. An enabled Rust GPU regression keeps the marker-overlap
    depth behavior in the default suite. The seven exact siblings remain
    green; the two unrelated large-radius gates are unchanged at 435/max-4
    and 180/max-4.
68. [x] Isolate the shared C++/Rust atlas-feather larger-radius coverage
    precision boundary in `feather_cusp` and `feather_shapes`. Start with a
    single residual contour and compare tessellation inputs, R16 mask samples,
    atlas placement, and final sampling under the unchanged `2/32` contract;
    port a proven semantic mismatch or retain the existing backend-precision
    gate with a bounded oracle. Preserve the now-exact empty-stroke depth path
    and all seven exact siblings from the same source-order wave. C++ computes
    atlas bounds from the softened fill path while Rust used the original
    controls, and C++ relaxes Wang parametric precision for large feather
    radii while Rust retained the normal precision. Rust now matches both
    rules and uses fused placement translation arithmetic. Paired high-radius
    oracles pin runtime-derived exact placement, exact signed zero/nonzero mask
    topology, bounded tessellation/R16 precision, and final C++ Dawn/Rust
    pixels under `2/32`.
    `feather_cusp` improves from 435/max 4 to zero pixels beyond delta 2;
    `feather_shapes` improves from 180/max 4 to 11/max 3. Both are promoted,
    while the empty-stroke depth path and seven siblings remain green.
69. [x] Add and capture the next ten accepted strict source-order C++ Dawn
    MSAA cases after `feather_shapes`, beginning with `feather_strokes` and
    continuing through the feather-text, gamma-clip, hit-test, and image
    candidates. Record strict-generator rejections as named harness gaps,
    prove all prior 40 captures remain byte-identical, and probe accepted
    references under the unchanged `2/32` contract before changing gates. The
    strict registry now contains 50 cases. `image`, `image_aa_border`,
    `image_filter_options`, and `image_lod` retain the explicit
    `strict-replay-decode-image` harness gate, so `inner_join_geometry` and
    `interleavedfeather` complete the wave. Jobs 1 and 4 produce all 150
    artifacts byte-identically, and the
    prior 40 PNGs remain byte-identical. `feather_strokes`, non-mirrored
    Montserrat and Roboto feather text, `gamma_correction_clip`, and
    `inner_join_geometry` pass unchanged `2/32` and are promoted. Both
    mirrored text streams render blank only after their `scaleX=-1` transform
    and retain `dawn-wgpu-msaa-reflected-feather-atlas-transform`. Both
    32k-draw hit-test streams independently fail Rust readback mapping and
    retain `rust-wgpu-msaa-large-draw-readback-map`. `interleavedfeather`
    retains `dawn-wgpu-msaa-interleaved-feather-color-precision` at 145
    pixels/max 85 across 140 tiny components with alpha within one. The
    generator now splits those oversized hit-test replays into strictly
    validated 128-path helpers, and the oracle uses the runtime's supported
    no-LTO build so the 44 MB registry compiles repeatably.
70. [x] Port reflected atlas-feather transform parity, beginning with the
    `scaleX=-1` Montserrat stream. Compare C++ and Rust atlas path bounds,
    placement, contour orientation, and final sampling before changing either
    mirrored text gate. Preserve the five exact siblings from queue item 69
    and the unchanged `2/32` contract. Bounds, atlas placement, and final
    sampling are determinant-neutral in both implementations; the first
    mismatch was contour orientation. C++ reverses clockwise feather-atlas
    fills under a negative transform determinant, while the Rust MSAA atlas
    path always emitted the forward contour. A focused selector regression
    now pins reverse clockwise fills, forward non-zero fills, and forward
    strokes under reflection, and the production path uses that direction.
    The Dawn reprobes pass unchanged `2/32`: mirrored Montserrat has 13
    byte-inexact pixels/max 68 and mirrored Roboto has 0/max 2. All five exact
    siblings remain green, promoting both gates and moving the ratchet to
    exact=717/diverges=0/gated=751.
71. [x] Port large-draw MSAA submission/readback parity for
    `hittest_evenOdd` and `hittest_nonZero`. Reproduce the map failure around
    their 32k-path command streams, compare C++ logical-flush resource limits
    with Rust batching, submission, and readback lifetimes, and add a bounded
    stress oracle at the first failing draw count. The Rust path must complete
    without device loss or map failure before either gate changes. Preserve
    the 717 exact entries and unchanged per-entry contracts. The first failing
    Rust frame is exactly 2,044 direct draws: the per-draw tessellation
    textures and bind resources exhaust one Metal command buffer long before
    C++ reaches its shared logical-flush limits. Large clip-independent,
    source-over MSAA schedules now preserve intersection-board order while
    submitting at most 1,024 draws per encoder and waiting for each encoder's
    resources to retire; clipped and destination-read schedules keep their
    existing single-submit path until their cross-flush state can be replayed.
    An enabled GPU regression renders the exact
    2,044-draw boundary. Both complete 32k streams pass their pinned C++ Dawn
    references with zero pixels beyond delta 2 and max delta 1, advancing the
    ratchet to exact=719/diverges=0/gated=749 with no contract change.
72. [x] Port strict replay image decoding through the provenance-bound C++
    Dawn MSAA reference path for `image`, `image_aa_border`,
    `image_filter_options`, and `image_lod`. Preserve encoded payload identity,
    decoded dimensions, sampler/mipmap behavior, and the fail-closed strict
    compiler checks; serial and four-job capture must leave the existing 50
    cases byte-identical. Capture the four valid Dawn references and probe
    Rust under the unchanged `2/32` contracts before changing any
    `strict-replay-decode-image` gate. Preserve the 719 exact entries. The
    strict compiler now validates and emits the exact encoded bytes, decoded
    dimensions, image-resource identity, sampler fields/key, blend mode, and
    opacity for all four streams. The 54-case serial and four-job captures
    are byte-identical, as are all 100 retained PNG/RGBA artifacts from the
    prior 50 cases. All four Rust probes reach the same explicit
    `images in msaa mode` rejection before rendering, so their new Dawn
    references are installed but the rows remain gated under the narrower
    `rust-wgpu-msaa-image-rect` diagnostic. The ratchet remains
    exact=719/diverges=0/gated=749 with no tolerance change.
73. [x] Port C++ rectangular-image execution in Rust MSAA mode, beginning with
    `image_filter_options` and then `image`, `image_aa_border`, and
    `image_lod`. The upstream trace corrected the initial task wording:
    `gpu::ImageRectDraw` is atomic-only; C++ MSAA scales a unit-rectangle
    `PathDraw` and evaluates an image paint through the generated path shader.
    Rust now follows that route with C++-encoded image `PaintData`, inverse
    paint coordinates, constant mip LOD and bias, the existing generated MSAA
    image-paint shader branch, and exact filter/wrap sampler conversion. A
    focused GPU regression proves the MSAA result byte-identical to Rust's
    atomic path, and a pure test pins the C++ LOD formula. All four Dawn probes
    have zero pixels beyond delta 2 (`image_filter_options` max 1; the other
    three max 2), so they are promoted without changing any contract. The
    renderer suite passes 210 enabled tests, and the full ratchet advances to
    exact=723/diverges=0/gated=745 while preserving all 719 prior exact rows.
74. [x] Capture and probe the next ten strict source-order C++ Dawn MSAA
    entries after `interleavedfeather`: `interleavedfillrule`, the three
    `labyrinth_*` streams, and the six `largeclippedpath_*` variants. Preserve
    the existing 54-case registry byte-for-byte in serial and four-job modes,
    record strict-generator rejections as named harness gaps, and install only
    provenance-complete references. Probe accepted rows under their unchanged
    contracts before changing any `algorithm-core` gate, and preserve the 723
    exact entries. All ten strict streams compile without a harness gap. The
    64-case serial and four-job captures are byte-identical across all 192
    PNG/RGBA/provenance artifacts, and all 54 retained reference PNGs remain
    byte-identical. `interleavedfillrule` and all three `labyrinth_*` rows pass
    unchanged `2/32` with zero over-threshold pixels and max delta 0/1, while
    all six `largeclippedpath_*` rows reach the same explicit
    `path clips on non-atlas msaa draws` rejection. The former promote and the
    latter narrow to `non-atlas-msaa-path-clip`, advancing the ratchet to
    exact=727/diverges=0/gated=741 without changing a tolerance. The full
    workspace, normal 584-segment floor, and scripted 35-segment floor pass.
75. [x] Port C++ non-atlas MSAA path-clip execution through Rust's direct path
    pipeline. Begin with `gm-bug339297_as_clip-msaa` and the six pinned
    `gm-largeclippedpath_*` Dawn references, preserving the existing atlas
    stencil path, clip-stack ID reuse, nested non-zero/even-odd/clockwise
    semantics, clip rectangles, and unclipped draws. Replace the focused
    explicit-boundary regression with GPU coverage for direct clipped draws,
    probe all seven rows under their unchanged contracts, and preserve the 727
    exact entries. Rust now mirrors C++ `gpu.cpp`'s active-stencil-clip states:
    analytic draws compare equal to `0x80`; borrowed coverage and nested/even-
    odd stencil passes compare less-or-equal; forward/cleanup passes include
    the clip bit in their compare mask while preserving fill-rule bits. Direct
    path pipelines select independent path-clip and clip-rectangle variants,
    while the existing atlas path and clip-stack reuse/reset scheduling remain
    unchanged. Focused GPU tests prove both a triangular direct clip and its
    intersection with a clip rectangle. All seven Dawn probes pass unchanged
    `2/32` contracts with zero over-threshold pixels and max delta 1, advancing
    the full ratchet to exact=734/diverges=0/gated=734. The renderer suite,
    workspace, normal 584-segment floor, and scripted 35-segment floor pass.
76. [x] Capture and probe the next ten gated strict source-order C++ Dawn MSAA
    entries after `largeclippedpath_*`: `lots_of_tess_spans_stroke`,
    `mandoline`, `mesh`, `mesh_ht_{1,7}`, `mutating_fill_rule`,
    `negative_interior_triangles{,_as_clip}`, `overfill_blendmodes`, and
    `overfill_opaque`. Extend the 64-case registry without changing retained
    artifacts, require byte-identical serial/four-job capture, probe accepted
    rows under their unchanged `2/32` contracts, narrow every rejection to an
    observed diagnostic, and preserve all 734 exact entries. Strict generation
    accepts nine rows and rejects `mesh` at its first `makeRenderBuffer`, now
    named `strict-replay-render-buffer`. A fresh serial capture and the
    four-job capture match across all 219 artifacts, and all 64 retained PNGs
    remain byte-identical. All nine accepted Rust probes have zero
    over-threshold pixels and max delta at most 1, advancing the ratchet to
    exact=743/diverges=0/gated=725 without changing a tolerance. The workspace,
    211 enabled renderer tests, normal 584-segment floor, scripted 35-segment
    floor, and full renderer corpus pass.
77. [x] Capture and probe the next ten gated strict source-order C++ Dawn MSAA
    entries after `overfill_opaque`: `overfill_transparent`,
    `overstroke_{blendmodes,opaque,transparent}`, `parallelclips`,
    `path_skbug_{11859,11886}`, `path_stroke_clip_crbug1070835`, `quadcap`,
    and `rawtext`. Extend the accepted registry without changing retained
    artifacts, require byte-identical serial/four-job capture, probe accepted
    rows under unchanged contracts, and narrow every strict-generator or Rust
    rejection to its observed boundary while preserving all 743 exact rows.
    All ten strict streams compile. Serial and four-job captures match across
    all 249 PNG/RGBA/provenance artifacts, and all 73 retained PNGs remain
    byte-identical. Eight Rust probes pass unchanged `2/32` contracts with max
    delta at most 2. Transparent and advanced-blend overstroke expose repeated
    self-overdraw (36,855/max 104 and 30,357/max 90): C++ selects dedicated
    `msaaStrokes` state with depth writes enabled, while Rust's generic
    analytic pipeline disables depth writes. They narrow to
    `msaa-stroke-depth-write`; the ratchet advances to
    exact=751/diverges=0/gated=717 without changing a tolerance. The workspace,
    36 oracle-format tests, 11 capture tests, normal 584-segment floor,
    scripted 35-segment floor, and full renderer corpus pass.
78. [x] Port C++ `gpu.cpp::get_depth_state(msaaStrokes)` as a dedicated Rust
    MSAA stroke pipeline with depth writes enabled, preserving generic
    analytic-fill state. Add a focused overlapping-stroke GPU regression,
    then promote `gm-overstroke_{blendmodes,transparent}-msaa` under their
    unchanged `2/32` contracts while preserving all 751 exact rows. The
    direct-path branch was stroke-only in Rust, so it is now named `Stroke`
    and uses C++'s `Less` depth compare with writes enabled; all midpoint-fan
    fill pipelines retain their existing depth/stencil states. The focused
    transparent duplicate-contour regression failed from accumulated alpha
    before the change and passes afterward. Both probes pass unchanged
    contracts (`blendmodes` byte-exact; `transparent` zero over-threshold
    pixels/max delta 2), and the full corpus advances to
    exact=753/diverges=0/gated=715 while preserving all prior exact rows. The
    212-test renderer suite, workspace, normal 584-segment floor, scripted
    35-segment floor, and full renderer corpus pass.
79. [x] Capture and probe the next ten gated strict source-order C++ Dawn MSAA
    entries after `rawtext`: `rect`, `rect_grad`, `rotatedcubicpath`,
    `roundjoinstrokes`, `skbug12244`, `strokedlines`, `strokefill`, `strokes3`,
    `strokes_poly`, and `strokes_round`. Extend the 83-case registry without
    changing retained artifacts, require byte-identical serial/four-job
    capture, replace the stale native-Metal `rect` harness reference only with
    complete Dawn provenance, probe accepted rows under unchanged contracts,
    and narrow every rejection to an observed diagnostic while preserving all
    753 exact rows. `rect_grad` and `strokedlines` fail closed on
    `makeLinearGradient` and narrow to `strict-replay-gradient-paint`; the
    other eight streams extend the registry from 83 to 91 cases. Serial and
    four-job captures are byte-identical across all 273 artifacts, and all 83
    retained PNGs are unchanged. Seven probes pass their unchanged contracts:
    `rect`, `rotatedcubicpath`, `skbug12244`, `strokefill`, and `strokes3`
    have zero over-threshold pixels/max delta 1; `roundjoinstrokes` has two/max
    48; `strokes_round` has ten/max 3. `strokes_poly` retains `2/32` and narrows
    to `dawn-wgpu-msaa-stroke-edge-coverage`: its 60 over-threshold pixels/max
    58 split into 59 components, none larger than two pixels. The ratchet is
    exact=760/diverges=0/gated=708 with no tolerance change.
80. [x] Port C++ `render_context.cpp::LogicalFlush::{pushDraws,rewind}` resource
    rollover into Rust for clip-dependent and destination-read schedules.
    Translate the complete path, contour, tessellation, and signed draw-pass
    budgets; preserve clip-stack, destination-copy, and intersection-board
    ordering across submissions; and add focused boundary tests for every
    rollover cause. This closes the remaining R3 resource-budget finding from
    `docs/renderer-wgpu-adversarial-review.md` while the independent Dawn
    reference campaign continues outside the main implementation queue. Rust
    now rolls at C++'s 30,719 path IDs, 65,535 contours, 4,194,279 combined
    tessellation vertices, and 32,767 signed reordered passes; its standalone
    image draws share the atomic path-index budget. Late gradient-height,
    clockwise feather-atlas, and clockwise coverage-storage allocations are
    admitted transactionally against C++ and device limits. MSAA and atomic
    flush boundaries regenerate active clips and preserve destination-read,
    depth/stencil, and intersection-board order. Exact-boundary, natural
    allocation, clip-replay, and forced-versus-uninterrupted pixel tests pass;
    Sol accepted the implementation after the dynamic-allocation and
    run-level tessellation review findings were closed. The full renderer
    corpus remains exact=760/diverges=0/gated=708, all 223 enabled renderer
    tests and the workspace pass, and the five-case fuzz-replay gate is green.
81. [x] Integrate the independently reviewed strict Dawn reference campaign,
    then probe its eleven new MSAA rows under the unchanged `2/32` contract:
    `strokes_zoomed`, `teenyStrokes`, `transparentclear`,
    `transparentclear_blendmode`, `trickycubicstrokes`,
    `trickycubicstrokes_feather`, `trickycubicstrokes_roundcaps`,
    `widebuttcaps`, `zeroPath`, `zero_control_stroke`, and
    `zerolinestroke`. Promote passes and narrow failures to observed
    diagnostics without changing references or tolerances. Sol accepted all
    102 production-validator provenance bindings and the serial/four-job
    byte-identity evidence. Nine new rows pass unchanged contracts:
    `trickycubicstrokes_roundcaps` has three over-threshold pixels/max 62 and
    the other eight have zero/max at most 2. `trickycubicstrokes` remains
    gated at 279/max 217, with 276 pixels confined to its degenerate closed
    butt/miter cubic; `widebuttcaps` remains gated at 5,750/max 254, confined
    to the four cubic cells while all twelve polyline siblings agree. Both
    have exact alpha and share the narrower
    `msaa-degenerate-cubic-butt-miter-topology` diagnostic. The ratchet
    advances to exact=769/diverges=0/gated=699 without changing a tolerance.
82. [x] Port C++'s MSAA degenerate-cubic butt/miter stroke topology using the
    two pinned Dawn rows as the final-pixel oracle. First isolate the prepared
    tessellation for `trickycubicstrokes` path 20 and the four
    `widebuttcaps` cubic draws; then translate the shared C++ branch and
    promote both rows under unchanged `2/32` contracts. Exact C++/Rust CPU
    span, physical tessellation-texture, and five-selector final 4x MSAA pixel
    oracles pin the implementation. Rust now preserves C++'s paired cubic-root
    split, internal neutral join tangents, original terminal cap tangent, and
    fused interpolation. The remaining final-pixel delta exposed a separate
    pipeline mismatch: C++ culls counterclockwise MSAA stroke faces, while
    Rust had culling disabled. Enabling back-face culling makes both full Dawn
    probes byte-exact and advances the ratchet to
    exact=771/diverges=0/gated=697 without changing a tolerance.
83. [x] Extend strict Dawn reference generation for `.riv` frame selection,
    beginning with the 624 rows classified
    `strict-replay-riv-frame-selection`. Once supported, run all newly eligible
    Dawn captures as one continuous background campaign while renderer
    implementation continues; validate provenance and deterministic recapture,
    then promote passing rows in batches under their unchanged contracts. The
    strict compiler accepts 584 rows and keeps 40 behind exact gradient or
    render-buffer gaps. Two four-worker 686-case campaigns agree across all
    2,058 artifacts, every retained GM PNG is unchanged, and 581 RIV rows pass
    unchanged `2/32` contracts. The ratchet advances to
    exact=1,352/diverges=0/gated=116.
84. [x] Port non-atlas MSAA path clipping for
    `riv-clipping_and_draw_order-frame-0-msaa`, then reprobe the missing
    clipped facial draws in `riv-spotify_kids_demo-frame-0-msaa`. Both strict
    Dawn references are pinned; the first reaches the explicit
    `path clips on non-atlas msaa draws` rejection and the second differs at
    3,788 pixels/max delta 230. Preserve the unchanged `2/32` contracts and
    the existing direct-path clip behavior. MSAA image rectangles now use the
    scheduled path-clip stack, and nested clip-reset rectangles carry their
    actual draw-group depth instead of a hardcoded depth that failed over prior
    content. The latter restores all 3,690 missing Spotify facial pixels. The
    two rows now narrow independently: `clipping_and_draw_order` is
    8,905/max 18 entirely inside its profiled JPEG and joins the existing
    `platform-image-decode-color-profile` gate; Spotify is 98/max 41 on the
    mirrored foot/leg contour edges and moves to
    `msaa-overlapping-contour-edge-coverage`. The ratchet remains
    exact=1,352/diverges=0/gated=116 without changing a tolerance.
85. [x] Close `msaa-overlapping-contour-edge-coverage` in
    `riv-spotify_kids_demo-frame-0-msaa`. Isolate the two mirrored foot fills
    and round strokes against their pinned Dawn pixels, capture exact C++/Rust
    tessellation inputs for the boundary samples, then port the first proven
    topology or pipeline mismatch and promote under the unchanged `2/32`
    contract. C++ schedules opaque, unclipped MSAA paths ahead of overlapping
    advanced-blend subpasses, disables fixed-function blending for opaque
    paint, and starts later clipped work in a fresh-depth submission after the
    destination-read draw. Porting those three contracts makes Spotify
    byte-exact and advances the ratchet to
    exact=1,353/diverges=0/gated=115.
86. [x] Run the formal R3 exit audit. Prove every one of the 115 retained
    gates has a specific feature, backend/compiler boundary, or harness
    diagnostic; eliminate any generic placeholder that still masks runnable
    work. If the `#R-3` exit criteria hold, mark R3 complete and make the R4
    same-backend benchmark harness the next executable queue item. The checked-
    in strict Dawn inventory accounts for all 43 final placeholders: 41 are
    `strict-replay-gradient-paint` and two are
    `strict-replay-render-buffer`. The complete 115-gate taxonomy now has zero
    generic or empty diagnostics, so R3 closes. See
    `docs/renderer-r3-exit-audit.md`.
87. [x] Open R3.1 by closing `incompatible-clip-rectangles` in
    `riv-bullet_man-frame-0-clockwise-atomic`. Isolate the first incompatible
    rectangle pair and port C++'s transformed clip-stack behavior instead of
    flattening non-axis-aligned rectangles into one scissor. Reprobe the pinned
    native Metal row under its unchanged `2/32` contract and preserve all exact
    clip fixtures. The first real pair is an identity 500x500 clip followed by
    a transformed 50x50 rectangle. Rust now retains the optimized outer clip
    rect and pushes the incompatible inner rectangle through the ordinary clip
    stack, matching C++. The focused regression, all 231 enabled renderer
    tests, and the full corpus pass; Bullet Man is byte-exact and advances the
    ratchet to exact=1,354/diverges=0/gated=114.
88. [x] Close `msaa-cubic-stroke-raster-parity` in `gm-beziers-msaa` using its
    pinned C++ Dawn reference and a single-stroke tessellation/raster oracle.
    Historical replay against unchanged stream/reference hashes proves the
    gate was stale and misclassified: the row moves from 5,385 pixels/max 152
    immediately before `90c8fd52` to 8 pixels/max 3 immediately after C++'s
    dedicated MSAA stroke depth state landed. Its focused duplicate-contour GPU
    regression pins the actual self-overdraw boundary. The current probe passes
    the unchanged `2/32` contract and advances the ratchet to
    exact=1,355/diverges=0/gated=113.
89. [x] Close `msaa-clip-intersection-edge-coverage` in
    `gm-cliprectintersections-msaa` from its pinned C++ Dawn reference without
    broadening the 240-pixel residual into a tolerance. Historical replay with
    unchanged asset hashes proves the row moves from 240 pixels/max 55 before
    `90c8fd52` to byte-exact/max 1 after C++'s dedicated MSAA stroke depth
    state. The sparse edge components were translucent stroke self-overdraw,
    not clip-intersection rasterization. The current probe passes the unchanged
    `2/32` contract and advances the ratchet to
    exact=1,356/diverges=0/gated=112.
90. [x] Adjudicate the seven
    `native-clockwise-atomic-advanced-feather-parity` rows with full-stream C++
    Dawn clockwise-atomic references. Port any same-backend Rust defect; only
    reclassify a row after the full stream passes its unchanged contract. The
    minimum strict-gradient replay work needed for this oracle is an allowed
    R3.1 prerequisite. Fresh native-Metal references from upstream
    `7c778d13` now exist for all seven rows and every Rust stream reaches
    pixels. The capture sweep exposed and
    closed three admission mismatches before pixel adjudication: off-frame
    path draws are culled before clip/resource allocation, clockwise coverage
    uses C++'s stroke/feather-expanded pixel bounds, and singular nested clips
    become empty instead of unsupported. Current differing-pixel/max-delta
    probes are: `bankcard` 1,485,510/20, `car_widgets_v01` 875,754/249, `coin`
    48/58, `data_viz_demo` 169,028/253, `echo_show_demo` 217,492/171,
    `hunter_x_demo` 659,956/255, and `rewards_demo` 311,799/192. Coin's first
    excess appears on its second clipped, non-feathered ring; the final 48
    outliers form 13 one-pixel-wide path/clip-edge components, largest 12.
    Reclassified it to the existing Metal/WebGPU subpixel-edge boundary with
    no tolerance change. `bankcard` then exposed mixed atomic draw-type
    reordering: Rust hoisted all atlas blits ahead of ordinary paths instead of
    preserving C++ batch order. Interleaving them reduces the row from
    1,485,510 pixels/max delta 20 to a passing 22/max 18 under the unchanged
    `2/32` contract. Bankcard is promoted and five substantive rows remain.
    A later command-prefix bisect of Rewards found another shared atomic-batch
    defect: each draw resolves coverage left by the preceding draw, but Rust
    selected HSL-enabled feather and atlas shader variants from only the
    current draw. Selecting the feature from the combined logical batch removes
    the full-frame color loss and reduces Rewards from 357,444 pixels/max 53 to
    1,677/max 33. Hunter was subsequently reclassified to the reviewed
    subpixel-edge boundary, while Echo narrowed to a separate actionable
    clip-edge/composite diagnostic. Rewards command 16 then exposed a
    1,024-unit packed winding contribution that collided with the clockwise
    atomic coverage prefix. Unclipped positive interior weights now replay as
    unit-weight instanced draws, with adjacent unit-weight triangles retained
    as one batch. The command-16 C++ blit is 0/max 1 and the full frame improves
    from 1,677/max 33 to 1,575/max 33. A fresh command-21 WebGPU-on-Metal
    artifact comparison then proved all 254 visible isolated differences are
    exactly on its sparse clip-plane edge differences, while the unclipped
    control passes. Rewards is reclassified to the reviewed subpixel-edge
    boundary. The final corrected-dither campaign closes the remaining three
    classifications. Full C++ Dawn/Rust wgpu Data Viz passes `2/32` at
    22/max 3 and moves to the reviewed Metal/WebGPU edge boundary. Car Widgets
    (10,872/max 13) and Echo Show (96/max 3) retain matching raster/clip facts
    but cross a specialized-clockwise/general-atomic RGBA8 resolve/reload
    boundary that C++ avoids by retaining one packed color plane. Both move to
    the executable `rust-wgpu-atomic-color-plane-lifetime-parity` finding.
91. [x] Complete strict gradient-paint and render-buffer replay, capture the 46
    newly comparable rows, and promote or enqueue every result. R4 runner
    wiring resumes only after the R3.1 exit criteria hold. Strict linear and
    radial gradient reconstruction is complete: the compiler validates
    canonical stops and shader references, emits exact C++ resources, and
    changes each paint shader binding exactly when the stream does. The
    render-buffer compiler now validates exact type, flags, byte size,
    one-time mapping, initialization, mesh roles/capacities, and sampler state;
    it also retains prior uploads for later RIV frames. The regenerated
    inventory has all 46 rows capture-ready, preserves five gated rows with
    valid strict provenance, and leaves only the synthetic first-light harness
    row unsupported. A continuous 732-case Dawn campaign preserved all 686
    prior PNGs byte-for-byte and added the 46 references. The Rust probe
    promotes `riv-interactive_scrolling-frame-0-msaa` byte-exact, narrows 37
    rows to `rust-wgpu-msaa-gradient-path`, three to
    `rust-wgpu-msaa-image-mesh`, and five to
    `rust-wgpu-msaa-feather-gradient-advanced-blend`.
92. [x] Port C++ MSAA gradient-painted path preparation, starting with the
    gradient-only `gm-rect_grad-msaa` oracle, then sweep all 37
    `rust-wgpu-msaa-gradient-path` rows under their unchanged contracts. Rust
    now renders the shared gradient ramp before direct MSAA paths, binds it in
    the per-flush group, emits gradient fill/stroke paint and auxiliary data,
    and includes shader paints in destination-read accounting. Seventeen rows
    promote under unchanged `2/32`; all 20 residuals were probed and split
    into the concrete queues below.
93. [x] Port C++ MSAA image-mesh draws for `gm-mesh-msaa`,
    `riv-jellyfish_test-frame-0-msaa`, and `riv-tape-frame-0-msaa`; preserve
    typed-buffer, sampler, clipping, blend, and draw-order semantics. The
    generated C++ WGSL path promotes `gm-mesh-msaa` and `riv-tape-frame-0-msaa`
    under unchanged contracts. Same-backend C++ Dawn prefix captures prove
    all 19 Jellyfish meshes remain within delta 2; its three later translucent
    image rectangles cumulatively introduce 3,691/max 3, 8,548/max 4, then
    11,988/max 5 pixels. That row is retained under the concrete
    `dawn-wgpu-msaa-image-rect-dither-accumulation` precision diagnostic.
94. [x] Port feathered MSAA gradient strokes, starting with
    `riv-ai_assitant-frame-0-msaa`, then close the five
    `rust-wgpu-msaa-feather-gradient-advanced-blend` rows. Reuse the existing
    ramp, destination-copy, and atlas-composite machinery without changing
    tolerances. AI Assistant is exact; Data Viz, Echo Show, and Rewards pass
    their unchanged contracts. Car Widgets now passes at 14 pixels/max delta
    8 after matching C++'s absolute-value feather setter. Hunter X now passes
    with zero pixels beyond delta 2/max delta 1 after matching C++ WebGPU's
    2048-pixel feather-atlas rollover and resolved-color MSAA reload between
    logical flushes. The ratchet advances to exact=1,402/diverges=0/gated=66.
95. [x] Fix repeated path-clipped MSAA strokes in
    `gm-strokedlines-msaa`. Strict replay now preserves gradient snapshots
    across all 15 sequential clip stacks; the corrected reference compares at
    zero pixels beyond delta 2/max delta 1 under the unchanged contract.
96. [x] Close gradient destination-read compositing in
    `gm-xfermodes2-msaa`. Rust now preserves MSAA depth across destination
    reads and schedules C++-equivalent independent fill subpasses without
    reordering multipass draws. The target compares at zero pixels beyond
    delta 2/max delta 2; mesh and Spotify regressions remain exact.
97. [x] Port the MSAA form of incompatible transformed clip rectangles for
    `riv-bullet_man-frame-0-msaa`, preserving the already exact
    clockwise-atomic implementation. Mesh clips now enter the MSAA schedule;
    Bullet Man compares at zero pixels beyond delta 2/max delta 1.
98. [x] Attribute and close the clipped/stroked gradient residuals in
    `riv-death_knight-frame-0-msaa`, all five `riv-juice` frames, and all
    five `riv-off_road_car` frames. All 11 pass their strict references under
    unchanged contracts after the combined clipping, gradient, and MSAA
    scheduling fixes.
99. [x] Adjudicate the five `riv-joel_signed` MSAA edge residuals. Enabling
    C++-equivalent dithering for fixed MSAA paths closes the shared residual;
    all five frames pass their unchanged contracts.
100. [x] Promote three independently repeated MSAA singleton captures:
    `gm-dstreadshuffle-msaa`, `riv-jellyfish_test-frame-0-msaa`, and
    `gm-strokes_poly-msaa`. Sol reproduced four fresh Rust wgpu/Metal rounds
    for every row; each row was byte-stable across those rounds and passed its
    unchanged `2/32` Dawn reference contract at 0/max 1, 0/max 1, and
    12/max 46 respectively. No reference or tolerance changed; the ratchet is
    exact=1,408/diverges=0/gated=60.
101. [x] Promote `first-light-rectangle-msaa` from its exact C++ Dawn
    reference under the unchanged `2/0` contract. The header-only stream now
    has a strict first-light replay profile and a case-local provenance
    identity. Existing captures retain their immutable legacy registry
    identities, so adding one case requires no bulk provenance rewrite or
    filesystem transaction. Focused C++ Dawn/Rust wgpu comparison is byte-exact
    at 0/max 0; the ratchet advances to exact=1,409/diverges=0/gated=59.
102. [x] Capture a full same-backend C++ Dawn/Rust wgpu Car Widgets frame after
    the advanced-flush dither fix. If it passes, reclassify only the residual
    native Metal/WebGPU pixels; otherwise isolate the first failing prefix.
    The 3,288-command frame is 10,872/max 13. Prefix bisection isolates
    command 435 ColorDodge and command 2,830 Multiply amplifiers immediately
    after specialized clockwise work. Their shared coverage values are exact
    or off by one and the latter clip plane agrees at 253,163/253,171 words;
    the bad pixels lie inside the shared masks. The row moves to
    `rust-wgpu-atomic-color-plane-lifetime-parity`.
103. [x] Capture Data Viz commands 16, 17, and 22 with the corrected dither
    contract, then compare the first still-failing production schedule,
    tessellation span, payload, and coverage field against an independent C++
    oracle. Do not substitute or reorder production inputs. The complete
    production frame passes the unchanged same-backend contract at 22/max 3,
    superseding a prefix-only failure hunt; the native residual moves to
    `metal-webgpu-subpixel-edge-coverage`.
104. [x] Extend the exact Echo C++ Dawn/Rust wgpu prefix comparison through the
    full command stream. Commands 39 and 104 now pass at 0/max 1 after enabling
    C++'s flush-wide atomic dither feature; only a later failing prefix may
    justify more production code. The 511-command frame is 96/max 3. Command
    462 passes and command 463 is the first failure; C++ and Rust touch the
    exact same 230,896 coverage words, leaving only the color-plane lifetime
    difference across strategy partitions. Echo moves to
    `rust-wgpu-atomic-color-plane-lifetime-parity`.
105. [x] Close Rewards' remaining command-21 clipped residual with a fresh
    cross-language payload/plane oracle. An empty-directory recapture from the
    pinned C++ Dawn executable reproduced all nine artifact hashes. CPU spans
    and preparation agree, the unclipped control passes, and coverage differs
    at only six words. The clip plane differs at 802 sparse edge words, 797 of
    which are C++ partial coverage versus Rust full coverage; every one of the
    254 over-threshold isolated pixels lies on those words. The full native
    residual is 1,575/max 33 across 1,517 tiny components, largest six pixels.
    Rewards therefore moves to `metal-webgpu-subpixel-edge-coverage` without a
    status, reference, or tolerance change. The rejected mixed artifact
    harness remains out of production; see
    `docs/renderer-rewards-command21-audit.md`.
106. [x] Start R4 by wiring the live release C++ and Rust renderer runners to
    the checked-in `rive-renderer-perf-runner-v1` protocol. The first vertical
    slice is one manifest scene in both modes with seven outer samples,
    submit-to-GPU-complete timing, identical adapter identity, and matching
    logical-flush/draw counters. The counters must expose atomic strategy
    partitions so R4's first batching task also owns the two-row retained
    color-plane lifetime finding. Both runners now use WebGPU over Metal on
    the same Apple M5 Max adapter and wait for GPU completion after every
    frame. The required CubicStroke slice passes all structural fences in both
    modes across seven alternating samples. The fixed 16-variant report also
    completes without a structural mismatch.
107. [x] Test collapsing per-draw clockwise-atomic render-pass boundaries. The first
    fixed report measures 26.37x aggregate Rust/C++ time: clockwise atomic
    grows from 7.70x at one draw to 93.80x at 20 draws, while MSAA grows from
    5.30x to 14.28x. Rust currently opens separate borrowed and main render
    passes for nearly every clockwise draw even when attachments and phase are
    shared. A controlled old-Rust/current-Rust A/B over all 16 variants rejects
    the merge: aggregate Rust time regresses by 23.95%, every clockwise scene
    slows by 10.5%-35.2%, and MSAA remains flat. The full pixel corpus stays
    green, but no production code is retained from the experiment.
108. [x] Test packing non-shared clockwise tessellation spans into shared
    vertical textures. Broad packing exposed real clip, interior-triangle, and
    advanced-blend ordering constraints, so the measured candidate was narrowed
    to source-over, triangle-free draws without clip updates. It preserved the
    complete pixel corpus at exact=1,409/diverges=0/gated=59, but the alternating
    old-Rust/current-Rust report rejected it at a 1.2225x aggregate regression.
    Every targeted clockwise scene slowed: `OverStroke` 1.31x,
    `batchedconvexpaths` 1.43x, and `bevel180strokes` 1.17x. No production code
    is retained from the experiment.
109. [x] Attribute the fixed R4 gap before attempting another optimization.
    Paired Time Profiler and Metal System Trace captures cover the one-draw
    `bug5099` and 20-draw `bevel180strokes` controls in both live runners. Rust
    grows from 3.167 ms to 63.695 ms while C++ grows from 0.260 ms to 0.942 ms.
    The Rust 20-draw path spends 2,798/3,526 renderer samples in
    `create_buffer_init` and emits 19.98 pending-write submissions/frame at
    59.722 ms of encoder time, versus one total command buffer/frame in C++.
    Rust GPU execution is 6.474 ms/frame, proving pending-write processing is
    the first dominant site. Item 110 narrows its resource cause; see
    `docs/renderer-r4-profile-attribution.md`.
110. [x] Test C++'s flush-wide resource-preparation order in Rust's
    tessellation path. Preparing all per-draw resources before encoding left
    19.99 pending-write submissions/frame. A stronger candidate packed all
    spans into one buffer and shared one uniform/path/contour bind group, but a
    short Metal trace still measured 19.85 submissions/frame, each immediately
    before a distinct tessellation texture's first pass. The packed candidate
    changed the 20-draw median from 69.644 ms to 70.463 ms and the one-draw
    control from 3.331 ms to 5.016 ms. Both candidates were removed.
111. [x] Test C++'s persistent tessellation-resource lifetime with bounded,
    completed-frame texture reuse. An exact-size pool retained at most 256
    textures/64 MiB and recycled only after GPU completion, but the trace
    remained at 19.88 pending-write submissions and 53.996 ms of pending-write
    encoder time per frame. It regressed `bug5099` from 4.010 ms to 4.301 ms
    and `bevel180strokes` from 71.605 ms to 78.398 ms. The pool was removed;
    texture creation was not the pending-write submission cause.
112. [x] Coalesce independent generic-atomic intersection-board groups under
    the existing 1,024 authored-draw Metal safety fence. Logical-flush
    boundaries still submit, and a single oversized group remains intact.
    The 110-frame trace falls from 19.88 to exactly 1.00 `PendingWrites` per
    frame while preserving all twenty tessellation and atomic passes. The
    fixed 16-variant old-Rust/current-Rust report improves from 162.237 ms to
    138.841 ms aggregate (0.8558x); every affected clockwise scene is flat or
    faster and MSAA stays within noise. The renderer corpus remains
    exact=1,409/diverges=0/gated=59 and both V2 floors stay green.
113. [x] Port C++ WebGPU's persistent three-buffer upload-ring lifetime for
    tessellation spans, uniforms, paths, and contours. Each slot now owns one
    alignment-correct union-usage arena, grows only when needed, consolidates
    overflow pages on the next submission, and writes each populated page once
    before submit. Three fixed 16-variant alternating reports improve aggregate
    time to 0.9605x, 0.9826x, and 0.9797x; the last measures the exact final
    binary. Its lone minimum outlier is falsified by a targeted A-B-B-A where
    both candidate medians beat their bracket baselines.
    A fenced A-B-B-A trace finds `PendingWrites` neutral at 1.006x while paired
    frame medians improve in both brackets. The earlier 1.2565x trace result
    was invalidated by uncontrolled machine load and non-interleaved captures.
    The renderer corpus remains exact=1,409/diverges=0/gated=59, both V2 floors
    pass at 584 and 35 exact segments, and the full workspace suite is green.
114. [x] Attribute the remaining single `PendingWrites` command buffer under
    the controlled R4 measurement fence. Feature-gated telemetry shows only
    1,040 written upload bytes for one draw and 20,496 for twenty draws after
    warmup. The baseline profile instead places 3,438/4,030 per-frame samples
    in initialized-buffer creation and 2,323 in zero/copy work, versus two in
    `Queue::write_buffer`; the generic-atomic clip and coverage planes alone
    recreate and zero roughly 8 MiB/frame at 1,024 square. Porting C++'s
    persistent three-buffer atomic-backing lifetime drops two repeated fixed
    report aggregates to 0.2913x and 0.2908x while untouched MSAA controls
    remain within 3.6% and 2.3%. In load-recorded A-B-B-A Metal captures,
    `PendingWrites` changes from 27.532/28.067 ms in the bracket baselines to
    2.899/2.897 ms in the candidates with 75%-82% host idle. The post-change
    profile has 27 initialized-buffer samples out of 185, down from 3,438 out
    of 4,030. Renderer exact=1,409/diverges=0/gated=59, V2 floors 584/35, and
    the full workspace suite remain green.
115. [x] Attribute the now-dominant generic-atomic command-encoding work.
    Feature-gated timers show dummy texture, sampler, and sampler bind-group
    creation consume only about 66 microseconds of the 5.23 ms twenty-draw
    frame. Metal traces instead show 102 Rust encoders/frame versus 24 in C++:
    Rust repeated the full atomic backing/path/resolve lifecycle for each
    intersection-board group. The port now prepares all groups flush-wide,
    preserves every explicit group barrier, and resolves once under the
    unchanged 1,024-draw and logical-flush fences. Two repeated reports improve
    aggregate time to 0.7978x and 0.8164x. A load-matched A-B-B-A reduces
    encoder rows from 11,221 to 4,951 and traced frame medians from
    7.740/7.822 ms to 3.939/3.300 ms. Renderer exact=1,409/diverges=0/gated=59.
    The normal and scripted V2 floors pass at 584 and 35 exact segments, and
    the full workspace suite is green.
116. [x] Reprofile the exact item-115 binary and port the largest measured
    C++-aligned site. Twenty-draw MSAA puts 35/119 renderer samples in
    `PathPipeline::prepare`, including 24 in five per-draw initialized-buffer
    allocations. Path data now uses exact slices of the guarded frame upload
    arena, while the null texture and sampler bindings live on the pipeline.
    Two fixed reports improve to 0.9089x and 0.9124x; a load-matched Metal
    A-B-B-A improves 4.766/4.748 ms to 2.845/2.852 ms and removes 2,199 encoder
    rows. The C++/Rust aggregate falls from 5.0537x to 4.5986x and the worst
    scene from 11.99x to 8.75x. Renderer exact=1,409/diverges=0/gated=59, V2
    floors 584/35, and the full workspace suite remain green.
117. [x] Port C++'s flush-wide tessellation texture/pass from the exact
    item-116 binary. Direct MSAA paths now share one packed tessellation
    texture, pass, and flush resource group per logical flush while retaining
    per-draw image bindings and the existing clip, advanced-blend, and
    submission schedule. A sparse-contour regression found by
    `gm-emptystroke-msaa` preserves authored empty-contour ID slots instead of
    compacting them into the next path. Independent Metal exports reproduce
    2,641 to 551 total encoder rows and 2,200 to 110 tessellation rows over 110
    frames, exactly 20 passes/frame to one. Light directional reports improve
    to 0.9236x and 0.9140x; the same-tier C++ Dawn/Rust wgpu aggregate falls
    from 4.5986x to 4.1442x. Renderer exact=1,409/diverges=0/gated=59.
    The V2 floors pass at 584/35 exact segments and the workspace suite is
    green.
118. [x] Build `perf-counter-compare` before another timing-led optimization.
    Extend both fenced runners with deterministic per-stream counters for
    flush composition, tessellation spans/patches, uploaded bytes, texture and
    bind-group work, and encoder/pass counts; diff and rank the fixed corpus by
    excess ratio. In parallel, inventory every C++ renderer pooling, ring,
    budget, reuse, and coalescing mechanism and map each excess counter to its
    reference source site. The live 16-variant report validates nonzero work
    and structural parity on both backends. Encoders/submissions are exact at
    16/16; Rust has 230 excess bind-group sets, 63 excess passes, 62 excess GPU
    draws, and 116,808 excess uploaded bytes. The source-mapped inventory is
    `docs/renderer-r4-mechanism-inventory.md`.
119. [x] Port C++ WebGPU's `needsNewBindings` rule for direct MSAA paths.
    C++ binds stable per-flush/per-draw state once per pass and rebinds only
    after a pass restart or for an image draw. Rust previously bound the same
    flush, dummy-image, and sampler groups for every direct path, producing
    the report's top row: 63 sets versus five for
    `gm-bevel180strokes-msaa`. Carry binding state across compatible direct
    draws, rebind image state when required, and invalidate it on foreign
    pipeline layouts. Accept with the exact counter delta, unchanged pixels
    and V2 floors, plus one directional timing snapshot. Rust bind-group sets
    fall from 554 to 413 over the fixed matrix; `gm-bevel180strokes-msaa`
    falls from 63 to six against C++ Dawn's five. The light aggregate snapshot
    moves from 3.224x to 2.033x. Renderer exact=1,409/diverges=0/gated=59,
    V2 floors 584/35, and the workspace suite remain green.
120. [x] Share plain-stroke midpoint tessellation across the generic-atomic
    flush. Attribution split the 54,696-byte target into 3,432 bytes of static
    frame data and 51,264 bytes of tessellation-arena writes: Rust rebuilt the
    same packed texture lifetime for each of twenty non-feather strokes, while
    C++ lays out the flush once. Extending the existing shared midpoint layout
    to plain strokes reduces `gm-bevel180strokes-clockwise-atomic` from 42 to
    23 passes (matching C++), 54,696 to 13,224 uploaded bytes, and 41 to 22 GPU
    draws. The fixed-matrix Rust totals move from 154 to 116 passes, 273,640 to
    172,008 uploaded bytes, and 220 to 187 GPU draws. A first broad candidate
    failed seven advanced-blend rows; loaded-destination-color runs now retain
    their prior per-draw lifetime. The corrected renderer corpus is
    exact=1,409/diverges=0/gated=59, V2 floors are 584/35, and the workspace
    suite passes. The four affected Rust directional frames sum to 4.124 ms
    from 6.937 ms; no load-controlled timing claim is attached.
121. [x] Attribute the new mode-paired top excess on
    `gm-batchedtriangulations`: MSAA emits 14 GPU draws versus four in C++
    Dawn, while clockwise atomic emits 14 passes and 13 draws versus five and
    five. C++ sorts the four disjoint interior fills by draw type and merges
    contiguous ranges. Rust now packs compatible plain outer curves and
    triangles once per flush and retains C++'s required role barrier. Atomic
    passes move 14->5 (C++ 5), draws 13->4 (C++ 5; its extra draw explicitly
    initializes what Rust clears as an attachment), and path patches are exact
    at 56. Fixed-matrix Rust passes move 116->107 and draws 187->178; ranked
    excess rows move 81->72. Renderer exact=1,409/diverges=0/gated=59, V2
    floors 584/35, and the workspace suite pass. The target's light snapshot
    is Rust/C++=1.382x; counters and output are the acceptance evidence.
122. [x] Port C++'s compact midpoint layout and subpass-major merge for
    `gm-batchedtriangulations-msaa`. Compatible disjoint opaque nonzero fills
    now share one flush-wide padding envelope and contiguous instance ranges;
    the encoder emits each of the three fill subpasses once over that combined
    range. The target moves from 14 to five GPU draws, 114 to 105 instances,
    and 32 to 23 tessellation spans, with spans and path patches (81) now exact
    against C++ Dawn. Fixed-matrix Rust draws move 178->169, instances
    6,362->6,353, spans 1,677->1,668, uploaded bytes 168,936->168,424, and
    ranked excess rows 72->71. The target's one-frame snapshot is 4.904x and
    the matrix sum is 2.172x; both are directional context, while exact work
    reduction and unchanged output are the acceptance evidence. Renderer
    exact=1,409/diverges=0/gated=59, V2 floors 584/35, the renderer feature
    suite (265 passed/38 ignored), and the full workspace suite pass. Sol's
    independent review found one missing merged-pixel regression; the test now
    compares scheduled and serialized output byte-for-byte, and the review has
    no open findings.
123. [x] Resolve ordinary fixed MSAA directly into the final attachment when
    the run owns the target clear. The multisample attachment now receives the
    frame clear and resolves into the final view; preserve-target chunks retain
    fallback composition, and advanced destination-read MSAA is unchanged.
    Fixed-matrix passes move 107->91, exactly matching C++ Dawn, and ranked
    excess rows move 71->47. `gm-batchedtriangulations-msaa` is exact at two
    passes, four draws, and 104 instances. Empty-clear, logical-flush,
    advanced-blend, mixed atomic/fallback, and translucent split-submission
    regressions pass. Renderer exact=1,409/diverges=0/gated=59, V2 floors are
    584/35, the renderer feature suite passes 267/38, and the workspace passes.
124. [x] Compact shared plain-stroke midpoint tessellation to C++'s flush-wide
    padding layout. `gm-bevel180strokes` emits 120 Rust spans versus 63 in C++
    Dawn in both modes. Rust relocates twenty standalone stroke layouts but
    retains each layout's three padding spans; C++ writes three padding spans
    once around the logical-flush geometry. Remove only those local zero-ID
    padding spans, retain one flush-wide leading/interior/final envelope, and
    preserve contour IDs and geometry order. The target is 120->63 spans in
    both modes; MSAA instances should move 160->103 exactly, while atomic moves
    161->104 versus C++'s 105 because C++ emits an explicit initialize draw.
    Both targets are met. Fixed-matrix Rust spans move 1,668->1,542, instances
    6,345->6,219, uploaded bytes 168,424->160,744, and ranked excess rows
    47->38. The focused regression, renderer feature/workspace suites, full
    renderer corpus, and V2 floors pass. The light timing snapshot overlapped
    the pixel sweep and is retained only as contaminated directional context.
125. [x] Condense compatible direct-stroke ranges in both renderer modes.
    `gm-OverStroke` contains twelve opaque, unclipped, solid SrcOver strokes.
    C++ condenses them to seven low-level stroke batches; Rust issues twelve.
    Merge only strokes in the same logical flush, scheduled draw group,
    prepass polarity, pipeline, shared tessellation resource, and contiguous
    base-instance range. Clip/clip-rect, transparent, image, gradient, feather,
    advanced-blend, binding, pipeline, schedule, or range changes end a batch.
    Target MSAA draws 13->8 exactly and atomic 14->9 (C++ reports ten because
    its explicit initialize operation is a draw; Rust's attachment clear is
    not). Both targets are met. Per-group compact rows preserve contiguous
    ranges when the full flush exceeds one row; opacity, clip, gradient, image,
    feather, pipeline, schedule, flush, and overlap boundaries retain separate
    draws. Fixed-matrix draws move 161->151, instances 6,219->6,191, spans
    1,542->1,514, uploads 160,744->159,208, and ranked excess rows 38->35.
    Production counters, grouped-versus-unbatched pixels, synthetic boundary
    cases, and Sol review are green.
126. [x] Match C++'s global interior preparation for large single-contour
    fills. The +200 path patches in each clockwise-atomic `bug339297` row are
    fill geometry, not a duplicated stroke pass. Rust's local single-contour
    ear triangulator bypasses C++'s `GrInnerFanTriangulator`; route every
    eligible contour through the ported global triangulator, prune authored
    zero-length fill lines with C++ numeric equality, and match C++ WebGPU's
    physical counterclockwise-face cull convention. Pin exact path-patch
    targets 623->423 and 631->431, direct preparation records/texels, and
    same-tier C++ Dawn pixels before replacing the two cross-backend primary
    references. Timing is one directional snapshot only. Both target counts
    are exact, the direct contour/triangle/texel oracle matches record for
    record, and same-tier final pixels are within 2/32 at zero pixels beyond
    delta 2/max-1. Restoring real C++ topology moves the fixed matrix from
    4,666 to 4,266 path patches and 6,191 to 5,987 instances while passes,
    spans, and uploads rise to 94, 1,708, and 179,824; this is a parity
    correction, not a blanket work reduction. Renderer
    exact=1,409/diverges=0/gated=59, renderer features pass 275/39, V2 floors
    remain 584/35, and the
    workspace passes. No A-B-B-A campaign was run. Sol found no correctness
    issue.
127. [x] Match C++'s one logical-flush midpoint-padding envelope for
    homogeneous plain fills. C++ allocates tessellation vertices globally per
    `LogicalFlush` and emits one leading padding span, an optional inter-type
    alignment span, and one final sentinel. Rust already shared the texture
    but retained path-local padding for ten translucent SrcOver fills.
    Generalize the shared midpoint layout independently of draw batching while
    retaining clip, clip-rect, gradient, image, feather, blend, fill-rule,
    row-width, and flush fences. `gm-batchedconvexpaths` atomic spans move
    101->78 and MSAA spans 105->78, both exact; atomic instances move 244->221
    and uploads 9,752->8,216 bytes, while MSAA instances move 318->291 exactly
    and uploads 10,400->8,608 bytes. Draw calls and patches remain unchanged.
    The fixed matrix moves 1,708->1,658 spans, 5,987->5,937 instances, and
    179,824->176,496 upload bytes; ranked positive rows fall 39->35. Both
    target images are byte-identical before and after. The 2.114x one-frame
    matrix snapshot is directional context only; exact counters and unchanged
    pixels accept the slice without A-B-B-A. Renderer
    exact=1,409/diverges=0/gated=59, the renderer perf-counter feature suite
    passes 276/39, normal/scripted V2 floors remain 584/35, and the workspace
    passes. The residual 600 atomic and 992 MSAA upload bytes are separate
    alignment or buffer-layout work, not part of this claim.
128. [x] Match C++'s line counting, lazy stencil lifetime, and multi-row
    logical-flush tessellation layout for `gm-bug339297_as_clip-msaa`. Rust
    treated transformed lines as cubics, eagerly cleared stale stencil before
    unclipped content, and stopped shared midpoint padding at one texture row.
    C++ gives each line one segment, retains stale stencil until an unrelated
    clip replaces it, and allocates clip plus content tessellation in one
    logical range across rows. Rust now does the same. Bind sets, draws,
    instances, spans, and patches reach exact C++ Dawn values at
    5/8/848/18/830; uploads move 3,704->3,120 against 2,816 and remain in the
    shared upload cluster. The reusable layout also removes all 17 excess
    `OverStroke` MSAA spans and 16 excess instances. Fixed-matrix spans move
    1,658->1,634, instances 5,937->5,885, uploads 176,496->174,888 bytes,
    and ranked positive rows 35->26. The 1.999x one-frame snapshot is
    directional context only; exact work and strict Dawn pixels accept the
    slice without A-B-B-A. The full corpus then exposed a clip-reentry case in
    `spotify_kids_demo`: two paths had identical geometry but distinct C++
    RawPath mutation IDs. Rust now carries the same globally unique mutation
    snapshot and reuses resident stencil only for the same unchanged path;
    the focused Dawn prefix is byte-exact while the item-128 counters remain
    exact. The final full corpus remains exact=1,409/diverges=0/gated=59.
129. [x] Triage the entire post-item-128 counter tail before another renderer
    edit. `docs/renderer-r4-counter-tail-audit.md` classifies all 26 rows: zero
    accounting-only Decisions, 26 shared-cause rows, and zero singletons.
    They collapse to `BUG-MIX`, `OVER-AENV`, `UPLOAD-DUP`, and `OVER-PATCH`.
    All final-pixel references already exist; only the final patch cluster
    needs a preparation-stage oracle. Attribution can proceed in parallel,
    while implementation and acceptance remain serial. A clean rebuild of
    both counter runners confirms the fixed report has exactly 26 ranked
    excess rows.
130. [x] Close `BUG-MIX`, the ten-row atomic `bug339297` cluster. Rust now
    packs midpoint-fan and outer-curve work into one C++-ordered logical
    tessellation address space, rebuilds row-wrapped forward/reflected spans,
    and shares the resulting texture without collapsing clip-update barriers.
    Normalized `(passes, draws, instances, spans, patches)` reach exact tuples
    `(6,5,542,117,423)` and `(8,7,555,121,431)` for the unclipped and clipped
    variants. Both upload totals fall below C++, all ten rows disappear, and
    the ranked tail moves 26->16. A Sol review caught the empty shared-triangle
    case before commit; the buffer-selection branch now distinguishes an empty
    shared set from per-draw buffers and has a focused regression. A second
    review caught C++'s unsigned reflected-row wrap; the relocation helper now
    preserves it with a zero-relocation byte-for-byte regression. Sol's final
    review passes. The light timing snapshot remains directional context only;
    no A-B-B-A was run. Final verification passes renderer
    exact=1,409/diverges=0/gated=59, normal/scripted V2 floors at 584/35 exact
    segments, the full workspace, formatting, and diff hygiene.
131. [x] Close `OVER-AENV`. Atomic direct strokes now use one logical-flush
    midpoint address space while `draw_group_starts` remain semantic execution
    barriers. `gm-OverStroke-clockwise-atomic` moves spans `506->489`,
    instances `1005->988`, and uploads `43,496->42,472`; the exact 1,024-byte
    reduction is sixteen removed 64-byte padding spans. Rust settles one span
    and one instance below the raw C++ atomic counters: C++ atomic/MSAA
    themselves report 490/489 spans for the same uploaded bytes, and C++
    counts an initialize draw that Rust performs as a clear. No synthetic work
    is added. Both positive rows disappear and the report moves 16->14. A
    focused relocation regression preserves distinct reflected mappings, the
    renderer perf-counter suite passes 285/39, and the light timing snapshot is
    directional context only. Sol's read-only review passes with no findings
    across eligibility, group barriers, relocation, wrapping, bounds, and
    regression coverage. Final verification passes renderer
    exact=1,409/diverges=0/gated=59, normal/scripted V2 floors at 584/35 exact
    segments, the full workspace, formatting, and diff hygiene.
132. [x] Reuse one aligned uniform/path/contour upload set across tessellation,
    MSAA drawing, general atomic drawing, and specialized clockwise-atomic
    drawing. Multiple atomic tessellation textures now upload only distinct
    span buffers, while image-only runs bind a real dummy contour. The MSAA
    report removes eight rows and moves `14->6`; atomic removes the other three
    upload rows and moves `6->3`. All sixteen fixed-matrix Rust upload totals
    are at or below C++ Dawn, with aggregate bytes 148,680 versus 156,832.
    Counted arena writes still include alignment. The final clockwise coverage
    prefix is set before the one shared uniform upload, and all 285 renderer
    tests pass after that ordering regression was caught locally. Sol's
    read-only review passes with no findings across GPU lifetime, submission
    order, binding ranges, repeated textures, and dummy/image-only paths.
    Final verification passes renderer exact=1,409/diverges=0/gated=59,
    normal/scripted V2 floors at 584/35 exact segments, the full workspace,
    formatting, and diff hygiene.
133. [x] Close `UPLOAD-LAYOUT` by evidence from item 132. Both
    `batchedconvexpaths` upload rows disappear under shared typed-resource
    reuse, proving no separate payload-layout residual remains. Per-class
    telemetry would no longer answer an open row and is not added.
134. [ ] Close `OVER-PATCH` from a per-draw C++/Rust `RIVEATS` preparation
    oracle. Locate the first OverStroke draw whose cumulative count diverges,
    then correct only the shared stroke-preparation source that emits patch
    498. Targets are 497 patches in both modes and 986 MSAA instances.

## R2 Completion Record

1. Finish feather coverage in source dependency order. Ordered atomic/fallback
   partitioning, direct/atlas threshold routing, atlas-stroke tessellation
   inputs, and the R16 mask are exact against C++ WebGPU. Direct severe-cusp
   topology, tessellation, normalized atomic coverage, and same-backend final
   pixels are now exact or within the standard `2/32` contract. Porting C++'s
   fixed-color generic-atomic face selection and clockwise `DrawContents`
   encoding removes the max-255 cusp-tip lobe. Fresh native Metal output has
   9,480 pixels beyond delta 2/max delta 11, matching the bounded cross-backend
   feather-overlap family, so Sol approved promotion under a 16,384-pixel
   allowance. Double-sided tessellation now wraps paired
   forward/mirrored spans across texture rows, making all 42 isolated
   `feather_polyshapes` cells exact. C++'s axis-aligned clip-rect fast path and
   arbitrary clip stacks/IDs are ported. The clipping sweep now leaves three
   explicit buckets: large/negative interior triangulation and clip-content
   bounds (`largeclippedpath_*`, `negative_interior_triangles_as_clip`),
   clipped fallback draws (`animated_clipping` and gradient large paths), and
   image support (`clipping_and_draw_order`). C++'s global inner-fan
   triangulator is now ported with intersection simplification, monotone
   decomposition, weighted faces, and grout. Direct WebGPU preparation oracles
   match the 100-contour grid (7,500 triangle vertices) and the exact
   flower+oval clip (2 contours, 108 triangle vertices) record-for-record,
   including every tessellation texel. The dedicated clockwise-atomic
   path/interior main, borrowed, outer-clip, and nested-clip shaders are now
   generated from upstream. The isolated family implements the global
   borrowed-to-main barrier, tiled visible-bounds allocations, a sampled
   WebGPU clip plane, and fixed-function `plus`/`min` clip attachments. The
   full large-path clockwise/winding/even-odd matrix is promoted under the
   forced-clockwise oracle. Negative-determinant interior preparation now uses
   C++'s physical forward-then-reverse tessellation layout, reducing the
   unclipped GM from 16,845 pixels to a bounded 1,040 edge pixels and promoting
   it. Counterclockwise culling on clip path/interior passes reduces the
   positive nested-clip draw from 15,408 pixels to 23. The mirrored nested
   inverse clip is now closed. Test-only snapshots prove both
   determinants produce the same borrowed word (`0x13f800`), main word
   (`0x140000`), and white clip-attachment pixel at corresponding interior
   points. The missing output was the clipped full-rectangle content:
   midpoint-fan double-sided preparation always used reverse-then-forward and
   omitted C++'s negative-determinant coverage flag. Porting determinant-aware
   forward-then-reverse layout reduces the GM from 166,809 pixels/max 208 to 46
   pixels beyond delta 2/max 7 and promotes it. A ten-entry basic-fill sweep
   then promoted `convexpaths` after porting missing forward-span row wrapping.
   `pathfill` is also promoted after connected-component analysis proved its
   253 hard-edge pixels are sparse one-pixel placement differences with 99.5%
   support overlap. `oval` then exposed two stale midpoint-fan boundaries:
   small compound fills were rejected into fallback, and midpoint-fan and
   outer-curve patches shared one atomic cull state. Admitting compound fills,
   splitting the atomic path pipelines by patch class, and applying C++'s
   counterclockwise-face cull to the CWA main path restores every oval, hole,
   and overlap. `mutating_fill_rule` is promoted after its remaining 45 pixels
   prove to be four one-pixel edge components with identical foreground
   support. Topologically complex fills are now isolated into the true
   clockwise-atomic pipeline, restoring self-intersections, repeated vertices,
   and compound clockwise contours without regressing large interior
   triangulation. `concavepaths` falls from 4,052 structural pixels to 9 edge
   pixels and `poly_clockwise` becomes pixel-exact. The remaining
   `poly_evenOdd`/`poly_nonZero` pair exposed a floating-point parity bug in
   dominant-winding selection: Rust summed per-contour areas, while C++
   accumulates the raw path in stream order before halving. Porting that exact
   accumulator restores both files to 2 edge pixels. The tied
   `cubicpath`/`cubicclosepath` gap was paint API parity: C++ stores the
   absolute value of stroke thickness, while Rust retained the GM's `-1` and
   rejected all twelve frame strokes. Porting the setter makes both files
   pixel-exact and closes the ten-entry basic-fill sweep. A read-only
   ten-entry fill/clip rescout then found five byte/threshold-exact bug GMs and
   `beziers` at 17 isolated delta-4 edge pixels; all six are promoted under
   their existing 32-pixel contract. The shared
   `bug339297`/`bug339297_as_clip` pair has identical binary support and
   identical black/white counts across Metal and wgpu; only two full-width AA
   scanlines differ under million-scale coordinate cancellation. Both are
   promoted with a documented 1,280-pixel backend allowance. The
   `hittest_evenOdd`/`hittest_nonZero` pair exposed unbounded invented-wgpu
   resource use: 32,580 tiny draws each allocated a 2,048-wide tessellation
   texture, bind group, and render pass in one command buffer, ending in an
   async-map failure. Homogeneous midpoint fills now shelf-pack into one
   tessellation texture, share one flush bind group and render pass, and use
   the translated intersection board to separate AA-overlapping draws into
   ordered atomic groups. Submitting and polling after each group bounds the
   command-buffer lifetime. Clockwise and clip-update batches retain their
   prior texture dimensions and pass topology. Both hit-test GMs now complete
   and are promoted at 382 pixels beyond delta 2/max delta 7 under a 512-pixel
   backend allowance. The per-group wait is a correctness-first R2 choice and
   remains an explicit R4 performance measurement target.
   The first image vertical slice now ports PNG decode to premultiplied RGBA,
   C++'s `ImageRectDraw` vertices/uniforms, the generated fixed-color atomic
   image shaders, authored sampler modes, and C++ WebGPU's generated-shader
   mipmap pass. `image_filter_options` is exact at the standard threshold;
   `image_lod` falls from 60,631 divergent pixels without mips to 276 sparse
   backend-filter pixels and is promoted under a 512-pixel allowance. The
   encoded-image dispatch now also decodes JPEG, restoring both the circularly
   clipped and later unclipped image in `clipping_and_draw_order`; the clip
   boundary and authored draw order match C++ Metal. Embedded PNG ICC profiles
   are now transformed to sRGB before premultiplication, matching ImageIO's
   decode order and sharply reducing the shared LG UltraFine-profile delta.
   `image`, `image_aa_border`, and `mesh` are promoted under measured
   Metal-vs-wgpu decoder/filter allowances. C++'s
   `ImageMeshDraw` is now ported with retained position/UV/index buffers,
   immutable unmap snapshots, generated atomic mesh shaders, clip IDs, and
   authored opacity/samplers. The non-fixed atomic color path now adds C++'s
   tiled color storage plane, destination-copy initialization, monotonic image
   z indices, generated advanced-blend shaders, and coalesced resolve. GPU
   regressions pin screen, darken, exclusion, and luminosity. `tape` and all
   three focused image GMs are promoted. Requesting the selected adapter's
   actual 2D texture limit instead of the 2,048 downlevel bucket unblocks both
   `superbowl` and the 2,080-square `jellyfish_test`. A draw-prefix and no-mip
   sub-oracle disproved the original mipmap attribution: every rendered image
   selects level zero, disabling generated mips is byte-identical, and the
   pre-image radial-gradient background alone carried 866,438 divergent
   pixels. C++'s simple/complex gradient-ramp layout, generated color-ramp
   pass, opacity modulation, and inverse paint transforms are now ported.
   Five focused gradient GMs and `jellyfish_test` are promoted. A mechanical
   sweep of the remaining gradient-bearing `.riv` corpus captured 30 fresh
   C++ references and promoted 11 entries without changing their 32-pixel
   budgets. Eight entries stop on native clockwise-atomic advanced-feather
   parity or incompatible clip diagnostics. `bad_skin` is now promoted after
   ordering generic-atomic outer
   and interior passes and proving all 69 isolated draws are bounded. The
   matching WebGPU MSAA final blit is now byte-exact across all 4,096 oracle
   pixels for the solid, unclipped, source-over slice. Parent-tight clip bounds
   are a later performance refinement, not a correctness gate. MSAA rectangle
   clip-distance is now byte-exact against C++ WebGPU. The unchanged outer
   non-zero path-clip slice is also exact: three stencil-update batches feed a
   fixed-function clipped atlas draw, including the combined path-plus-rect
   state. Changing unrelated outer clips is now exact as well: a generated
   MSAA stencil reset clears the previous clip before the next three-pass
   update, while unchanged clips reuse their stencil state and unclipped draws
   ignore retained stencil. Nested non-zero atlas clips are now exact too:
   `msaaMidpointFanPathsStencil` accumulates the inner winding and an
   intersecting `clipReset` rewrites the parent clip bit. Even-odd and
   clockwise transitions are exact too: outer even-odd uses stencil/cover,
   outer clockwise preserves the clip bit during cleanup, nested even-odd
   writes parity, and nested clockwise selects the `0xc0` reset mask. Filled
   C++ Dawn fixtures behaviorally distinguish every special mode with holes
   and opposite-winding contours. Destination-copy shader blending is now
   byte-exact for solid feather-atlas draws. The translated intersection board
   now schedules disjoint MSAA draws with the C++ layer reservations. A forced
   C++ clockwise-atomic sweep of the 38 remaining gated GMs promoted 24 under
   their unchanged contracts. Two more clear-state GMs became exact after
   frame attachments adopted C++'s integer-premultiplied clear color. C++'s
   determinant-aware contour direction now closes the mirrored
   Montserrat/Roboto feather-text pair. Two gated clockwise-atomic GM
   divergences remain.
   `interleavedfeather` remains parked as a named native-Metal-versus-WebGPU
   atomic intermediate-precision discontinuity. Its exact-source draws 13-14
   now have a dedicated C++ Dawn WebGPU suboracle: dimensions, f32 path bits,
   transforms, paints, and initialize/fill/stroke/resolve schedule are pinned;
   normalized raw coverage is exact; and only two packed color words
   differ at max byte delta one, producing exactly the same two resolved
   pixels at max channel delta seven. The oracle exposed and fixed generic
   feathered-clockwise preparation retaining the nonzero flag, plus the
   advanced feather-fill pipeline culling the wrong face. This closes the Rust
   defect without justifying a native-Metal corpus tolerance. A pinned
   independent full-stream C++ WebGPU-on-Metal lane now replays all 451 draws
   and passes the existing `2/32` contract at 6 pixels over delta 2. C++ Dawn
   and Rust differ at 84 byte-inexact pixels/max 26, while native Metal differs
   from both WebGPU paths almost identically: 18,492 and 18,495 pixels over
   delta 2/max 78. Sol accepted algorithm parity and required the corpus gate
   be renamed, not widened or promoted.
   C++'s empty-segment outcome is now matched in
   stroke/feather preparation for coincident cubics, closing `zeroPath`.
   `dstreadshuffle` is parked under the same intermediate-color precision
   boundary after an independent full-stream C++ Dawn WebGPU-on-Metal lane
   replayed all 97 draws. The untouched stream remains an intentionally failing
   configured gate at roughly 22.84k pixels over delta 2/max 61. A separately
   pinned control changes only the 97 paint blend-mode setters to SrcOver;
   three Rust samples pass the unchanged `2/32` contract at 11, 13, and 13
   pixels over delta 2/max 4. Exact generated-line comparison proves geometry,
   transforms, colors, ordering, dimensions, and opaque clear are unchanged.
   Sol approved removing the algorithm attribution while keeping the entry
   gated with its native reference and tolerance unchanged. A fresh forced
   reference also promotes `overfill_blendmodes` unchanged. `overfill_opaque`
   is now promoted under a bounded 48-pixel cubic-edge allowance: its two translated
   colored draws each contribute the same 20-pixel residual, while binary
   foreground support is exact. The `strokes_round` draw-38 CPU
   `TessVertexSpan` range now matches C++ all 11 records/176 words exactly
   after restoring five-segment miter/bevel joins, preserving raw line
   tangents, and writing flush padding before geometry. Native Metal
   comparison is clean at zero pixels beyond delta 2, so the entry is promoted
   under its unchanged `2/32` contract.
   `strokefill` is also promoted as a bounded native-Metal-versus-wgpu edge
   case. Its 14 isolated draws contribute no structural jump: each has at most
   30 pixels beyond delta 2, the full frame has 109 pixels/max delta 48 across
   19 components with largest area 15, and thresholded support IoU remains
   above 99.985%. It keeps channel delta 2 with a 128-pixel allowance.
   `rawtext` is promoted after a production-ring C++ oracle proved its first
   compound fill exact before rasterization: all 438 CPU span records, the
   `1+318` patch range, 36 contour records, and the complete 2,048x2
   tessellation texture match Rust byte-for-byte. The source fixes restore
   C++ flush-padding order, line/cubic tangent provenance, fused SIMD line
   conversion, and unsigned reflected-row wrapping. Fresh native Metal and
   the legacy reference agree at the standard threshold; the full Rust frame
   has 263 pixels/max delta 80 split across 76 components with largest area
   10, while its two isolated draws contribute 190 and 73 pixels. Foreground
   support IoU remains above 99.822%, so the entry is promoted at unchanged
   channel delta 2 with a bounded 288-pixel backend allowance.
   The remaining logical-flush sort-key fields are
   deferred until pass-level batching is an explicit R4 task: whole-draw Rust
   execution already preserves their only current correctness dependency.
2. Expand corpus entries only as focused pixel replay proves each feature.
   Do not tune broad tolerances around missing algorithm work.

3. [x] Complete the mid-R2 adversarial review of the invented wgpu
   resource/binding plumbing. `docs/renderer-wgpu-adversarial-review.md`
   records the binding, buffer, synchronization, pipeline-cache, and replay
   findings. Texture extents and atomic path-ID exhaustion are hardened; full
   logical-flush rollover and hostile resource/numeric streams are named R3
   work. The R3 semantic-trap and fuzz-replay entry gates remain open.

## Decisions

- 2026-07-16: R4 item 119 accepts objective renderer work reductions without
  an A-B-B-A campaign. Direct MSAA path bindings remain live across compatible
  path/fill/clip pipelines, image draws update only image state, and foreign
  layouts invalidate the cache. Exact counters and correctness floors are the
  primary evidence; the one-frame aggregate is a directional smoke check.

- 2026-07-16: R4 item 118 makes deterministic backend work the discovery
  oracle. A feature-gated C++ Dawn proc table and Rust wgpu wrappers count
  encoders, passes, bind groups, uploads, submissions, draws, tessellation
  spans, and path patches after warmup. The ordinary timing runners remain
  uninstrumented. The first fixed report finds exact one-encoder/one-submit
  frames but 230 excess Rust bind-group sets and names C++'s
  `needsNewBindings` rule as item 119. Single-frame timing is directional only;
  no threshold or load claim is attached.

- 2026-07-16: R4 item 116 ports C++'s flush-lifetime MSAA path resources after
  paired profiles localize 24/35 `PathPipeline::prepare` samples to per-draw
  initialized buffers. Exact arena slices plus pipeline-lifetime null/sampler
  bindings improve repeated fixed reports to 0.9089x/0.9124x. A load-matched
  Metal A-B-B-A removes twenty security-clear encoders per frame and improves
  both candidate brackets. The full C++/Rust gap is now 4.5986x. Item 117 owns
  the still-measured twenty-to-one tessellation-pass mismatch; item 108 is
  historical evidence only because the upload and submission lifetimes have
  since changed.

- 2026-07-16: R4 item 115 keeps intersection-board barriers but changes their
  ownership from complete per-group atomic lifecycles to one C++-aligned
  flush-wide lifetime. The unsafe no-barrier control was fast but failed 17
  corpus scenes; the accepted implementation passes all 1,468 rows and reduces
  the twenty-draw control from 20 preparations/40 atomic passes to one
  preparation/21 passes. Two repeated reports and a load-matched Metal A-B-B-A
  accept the change. The remaining C++ report is 5.0537x aggregate, so item 116
  reprofiles both modes rather than extrapolating another fix from pass counts.

- 2026-07-15: R4 items 111-112 correct item 110's provisional texture
  attribution. A bounded completed-frame texture pool leaves 19.88
  `PendingWrites` events/frame and slows both controls, so texture first use is
  not the cause. wgpu-core instead records initialized-buffer copies from
  mapped-at-creation buffers in `Queue::pending_writes`; Rust's per-group
  `submit_and_wait` flushed those copies after every independent atomic group.
  Coalescing only those groups under the existing 1,024-draw safety fence is
  accepted at 0.8558x aggregate and 1.00 event/frame. Logical-flush boundaries
  and oversized-group atomicity remain unchanged. Item 113 ports C++ WebGPU's
  persistent three-buffer upload rings against the remaining 39.760 ms/frame.

- 2026-07-15: R4's paired Time Profiler and Metal System Trace attribution
  names wgpu pending-write processing as the first hot site. At 20 draws it
  creates 19.98 submissions and 59.722 ms of encoder time per frame; C++ maps
  persistent resources and submits one command buffer. GPU union is 6.474 ms
  of the 63.695 ms Rust frame, so native fast paths and wait tuning are not
  first. Item 110 subsequently distinguishes first-use texture work from the
  initial initialized-buffer ordering hypothesis.

- 2026-07-15: R4 optimization acceptance uses an alternating old-Rust versus
  current-Rust A/B when the C++ control drifts under machine load. Controlled
  experiments rejected both merging specialized clockwise borrowed/main
  passes (1.2395x aggregate regression) and vertically packing simple
  clockwise tessellation textures (1.2225x regression). Both candidates kept
  the pixel corpus green and were removed. Stop inferring the hot path from
  structural multiplicity alone; paired CPU/GPU attribution is required before
  the next optimization. No pixel, corpus, reference, or tolerance changed.

- 2026-07-15: R4 compares Rust wgpu against C++ Dawn WebGPU-on-Metal, not the
  native C++ Metal renderer. Upstream native Metal explicitly rejects MSAA,
  so it cannot provide the required two-mode control. Pin the C++ runner to
  runtime `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2` and Dawn
  `211333b2e3e429c3508f25c81c547f602adf448c`; both runners select the same
  Apple M5 Max and report submit-to-GPU-complete medians only after per-frame
  completion. The initial fixed report is a baseline, not an acceptance
  threshold: aggregate ratio 26.37x, worst ratio 93.80x, with exact logical
  flush, authored draw, and atomic-strategy counts on every variant.

- 2026-07-15: Close R3.1 with 57 reviewed backend/precision rows and one
  two-row executable Rust finding. Data Viz's full same-backend frame passes
  its unchanged contract; Car Widgets and Echo Show have matching raster/clip
  evidence but Rust resolves and reloads RGBA8 when execution crosses atomic
  strategies. Name that finding
  `rust-wgpu-atomic-color-plane-lifetime-parity`, keep both rows gated without
  changing references or tolerances, and make retained color plus structural
  strategy counters the first R4 batching/flush responsibility. This satisfies
  R3.1's written exit condition without pretending the two rows are platform
  limitations.

- 2026-07-14: Pre-stage R4 as a report/protocol scaffold only; live runner
  wiring remains blocked on R3. The fixed 8-scene x 2-mode manifest pins
  1024x1024 frame 0, high-performance adapter selection, seven outer samples,
  ten warmup frames, 100 measured frames, submit-to-GPU-complete timing, and
  per-frame completion. Baseline and candidate must report the same concrete
  backend/device/driver identity and identical logical-flush/draw counts.
  Subprocess and threshold-failing CLI integration tests fail closed around
  those fences. Sol rejected the first scaffold contract and accepted this
  hardened version after all four findings were covered.

- 2026-07-14: Split Phase R into independent tracks: the main queue ports
  renderer/runtime state, strict Dawn reference capture runs as an oracle-side
  campaign, the semantic-trap audit and fuzz campaign report independently,
  and the R4 report scaffold can advance without live-runner wiring. The first
  expanded wave accepts eight of ten source-order streams, produces 273
  byte-identical serial/four-job artifacts, and preserves all 83 prior PNGs.
  Seven rows promote unchanged; `strokes_poly` remains gated as bounded
  Dawn-versus-wgpu stroke-edge coverage, while both gradient streams retain a
  strict replay-harness gate. The renderer metric is
  exact=760/diverges=0/gated=708. Queue item 80 returns the main loop to C++
  logical-flush rollover.

- 2026-07-14: Translate C++ `gpu.cpp::get_depth_state(msaaStrokes)` directly:
  Rust's stroke-only direct-path pipeline now compares depth with `Less` and
  writes depth, while the independent MSAA fill pipelines remain unchanged.
  A red/green duplicate-contour GPU test pins the self-overdraw behavior.
  `gm-overstroke_blendmodes-msaa` is byte-exact and
  `gm-overstroke_transparent-msaa` has zero pixels beyond delta 2, so both
  promote under unchanged `2/32` contracts. The renderer ratchet is
  exact=753/diverges=0/gated=715. Queue item 79 continues strict source-order
  Dawn capture through the remaining concentrated stroke rows.

- 2026-07-14: Extend the strict C++ Dawn MSAA registry from 73 to 83 cases.
  Serial and four-job captures match across all 249 PNG/RGBA/provenance
  artifacts, and all 73 retained PNGs are byte-identical. Eight candidates
  pass Rust under unchanged `2/32` contracts. The two overstroke failures are
  not blend-specific: opaque is exact, while transparent source-over and
  advanced blend show the same repeated contour self-overdraw. C++ routes
  strokes through `DrawType::msaaStrokes`, whose `gpu.cpp::get_depth_state`
  enables depth writes; Rust currently routes strokes through the generic
  analytic path pipeline with depth writes disabled. Gate both failures as
  `msaa-stroke-depth-write` and make that direct C++ state translation queue
  item 78. The ratchet advances to exact=751/diverges=0/gated=717 with no
  tolerance change.

- 2026-07-14: Extend the strict C++ Dawn MSAA registry from 64 to 73 cases.
  `mesh` fails closed at its first `makeRenderBuffer` and remains outside the
  registry under `strict-replay-render-buffer`; the other nine candidates
  compile. The accepted serial and four-job captures match across all 219
  PNG/RGBA/provenance artifacts, and all 64 retained PNGs are byte-identical.
  An initial serial wave produced one transient `interleavedfeather` artifact;
  four isolated reruns and a fresh full serial wave matched the four-job and
  retained bytes, so only the matching complete wave is accepted. All nine
  new Rust probes pass `2/32` with max delta at most 1. The ratchet advances to
  exact=743/diverges=0/gated=725 with no tolerance change; queue item 77 names
  the next ten strict source-order cases.

- 2026-07-14: Extend the strict C++ Dawn MSAA registry from 54 to 64 cases
  with `interleavedfillrule`, the three labyrinth streams, and all six
  `largeclippedpath_*` variants. Every stream passes the strict compiler; jobs
  1 and 4 produce all 192 artifacts byte-for-byte, and the prior 54 reference
  PNGs are unchanged. Four direct-path rows pass the existing `2/32` contract
  with max delta at most 1 and promote. The six clipped rows all stop at
  Rust's existing non-atlas MSAA path-clip boundary, so their generic gates
  narrow without promotion or tolerance changes. The ratchet is
  exact=727/diverges=0/gated=741; queue item 75 targets that shared seven-row
  renderer boundary.

- 2026-07-14: Extend the provenance-bound C++ Dawn MSAA registry from 50 to
  54 cases with strict `decodeImage` and `drawImage` replay. Generation rejects
  malformed or duplicate image resources, undeclared draws, invalid sampler
  fields, inconsistent sampler keys, invalid blend modes, and decoded
  dimension drift while embedding the original encoded payload bytes. Serial
  and four-job capture produce byte-identical rendered artifacts, and all 100
  prior PNG/RGBA files remain byte-identical to the retained 50-case baseline.
  Rust rejects each new reference through the same explicit
  `images in msaa mode` boundary, so the stale compiler gate narrows to
  `rust-wgpu-msaa-image-rect`; no row promotes and no tolerance changes. Queue
  item 73 targets the corresponding C++ `gpu::ImageRectDraw` path.

- 2026-07-14: Bound large clip-independent source-over MSAA schedules to 1,024
  direct draws per wgpu submission and synchronously retire each encoder. The first
  reproducible Metal failure is 2,044 draws, while C++ packs paths into shared
  logical-flush resources and rolls over only at its path, contour,
  tessellation, or signed draw-pass limits. The twofold headroom is therefore
  an explicit invented-wgpu resource-lifetime guard, not a translated C++
  path limit. Splits follow intersection-board order and only occur when no
  stencil clip or destination-read state must survive; clipped and advanced-
  blend schedules remain unchanged until their state can be replayed across
  logical flushes. The exact-boundary GPU oracle and both full 32k C++ Dawn
  comparisons pass.

- 2026-07-14: Extend the provenance-bound C++ Dawn MSAA registry from 40 to
  50 cases. The two 32k-path hit-test streams exposed pathological monolithic
  compiler functions, so strict generation now validates and emits bounded
  128-path helpers; the diagnostic executable uses the runtime's supported
  no-LTO mode while linked renderer code retains release optimization. Serial
  and four-job captures are byte-identical across all 150 artifacts, and the
  original 40 PNGs recapture byte-identically, proving no oracle pixel drift.
  Five cases pass unchanged `2/32`, advancing the ratchet to
  exact=715/diverges=0/gated=753. Four image candidates retain the named
  `strict-replay-decode-image` compiler gate. The five nonpassing rows receive separate
  reflected-transform, large-draw-readback, and sparse interleaved-color
  diagnostics; no tolerance changed.

- 2026-07-13: Extend the strict C++ Dawn WebGPU-on-Metal MSAA registry from 30
  to 40 cases. All prior PNGs recapture byte-identically. `emptyfeather`,
  `emptystroke`, `emptytransparentclear`, `feather_corner`,
  `feather_ellipse`, `feather_polyshapes`, and `feather_roundcorner` pass the
  unchanged `2/32` contract and advance the ratchet to
  exact=707/diverges=0/gated=761. `emptystrokefeather` retains the localized
  `msaa-empty-feather-stroke-inner-coverage` gate: Rust and C++ both emit the
  synthetic cap topology, but only C++ applies inner coverage to the 36 cap
  centers. The RGB-only max-delta-4 residuals in `feather_cusp` and
  `feather_shapes` retain
  `msaa-atlas-feather-large-radius-coverage-precision`; Sol rejected a
  compiler-specific name because intermediate atlas inputs are not yet exact
  enough to attribute the residual to Dawn/Tint versus wgpu/Naga. No reference
  or tolerance changed.
- 2026-07-13: Port C++ WebGPU's direct MSAA destination-read advanced blend
  path at the pipeline and render-pass boundaries. Color-writing direct path
  states now compile upstream's generated advanced and HSL fragment variants,
  bind the destination texture at binding 13, and place resolve/copy/reload
  barriers before each advanced draw group using clamped draw pixel bounds.
  The existing atlas path is unchanged. A focused Multiply-over-green GPU
  regression proves the shader reads destination pixels. Enabling C++'s
  interleaved-gradient-noise dither reduces the captured `dstreadshuffle`
  residual from 24,130 pixels/max 49 to 2,231/max 43. Its alpha plane is exact,
  and full-frame versus bounded destination copies produce byte-identical
  output, excluding copy coverage as the cause. Because the direct generated
  WGSL is byte-exact with upstream and the remaining delta is RGB-only across
  the Dawn/Tint and wgpu/Naga compiler stacks, narrow the gate to
  `dawn-wgpu-msaa-advanced-blend-intermediate-precision`; do not change the
  reference or `2/32` contract. The ratchet remains
  exact=700/diverges=0/gated=768. Sol found no production defect and approved
  the gate evidence, but required durable segmentation coverage before
  landing; the added fixed-to-advanced, consecutive advanced/HSL, analytic
  stroke, fill-rule, translucent-alpha, and empty-bounds GPU regression passes.
- 2026-07-13: Port C++ WebGPU's direct MSAA clip-distance shader selection at
  the pipeline boundary. Rust's generated clipped and unclipped vertex modules
  are byte-exact with upstream; all seven direct stroke/fill states now compile
  both variants when the adapter exposes `CLIP_DISTANCES`, and preparation
  fails closed otherwise. The focused GPU clip-plane regression passes.
  `clippedcubic2` and `cliprects` pass their unchanged Dawn `2/32` contracts at
  zero over-threshold pixels. `cliprectintersections` is no longer missing clip
  planes but retains 240 pixels/max 55 in sparse one-pixel intersection-edge
  components, so its gate narrows to `msaa-clip-intersection-edge-coverage`.
  The ratchet advances to exact=700/diverges=0/gated=768. No tolerance changed.
- 2026-07-13: The third ten strict C++ Dawn WebGPU-on-Metal MSAA references
  extend the provenance registry to 30 cases; the prior 20 PNGs recapture
  byte-identically. `bug7792`, `clippedcubic`, `crbug_996140`,
  `cubicclosepath`, `cubicpath`, and `emptyclear` pass unchanged `2/32`
  contracts and advance the ratchet to exact=698/diverges=0/gated=770.
  `clippedcubic2`, `cliprectintersections`, and `cliprects` stay gated as
  `msaa-direct-path-clip-rect-distance`: the direct pipeline loads the
  generated `noclipdistance` shader and their heatmaps show unclipped path
  regions. `dstreadshuffle` stays gated as
  `msaa-direct-path-advanced-blend`: all 97 destination-reading solid draws
  currently execute through the fixed-color direct path. The capture registry
  records whether draw batches are expected, permitting strict clear-only
  streams while still rejecting accidental empty replays. No tolerance
  changed.
- 2026-07-13: The second ten strict C++ Dawn WebGPU-on-Metal MSAA references
  extend the provenance-bound registry without changing the original ten
  pixels. `CubicStroke`, `OverStroke`, `bevel180strokes`, `bug339297`,
  `bug5099`, `bug6083`, `bug615686`, and `bug6987` pass the unchanged `2/32`
  contract and advance the ratchet to exact=692/diverges=0/gated=776.
  `beziers` stays gated as `msaa-cubic-stroke-raster-parity`: grouped and
  serialized Rust renders are byte-identical at 5,385 pixels/max 152 against
  C++ Dawn. `bug339297_as_clip` stays gated as
  `non-atlas-msaa-path-clip`, matching the renderer's explicit unsupported
  ingress. No tolerance changed.
- 2026-07-13: Port C++'s MSAA midpoint-fan fill execution instead of routing
  complex fills through the bootstrap CPU triangulator. C++ uses borrowed,
  forward, and cleanup passes for nonzero/clockwise fills and stencil/cover
  passes for even-odd fills, with each draw assigned its intersection-board
  depth group.
  Case-specific C++ guards pin the `8/9/10` and `11/12` schedules and inner-fan
  range. The three `poly_*` cases plus `concavepaths` and `pathfill` now pass at
  zero pixels beyond the existing `2/32` threshold; no reference or tolerance
  changed. The full ratchet advances to exact=684/diverges=0/gated=784.
- 2026-07-13: The first ten strict C++ Dawn WebGPU-on-Metal MSAA references
  are provenance-bound to the embedded replay-registry digest, C++ runtime,
  Dawn revision, stream digest, adapter, and final artifacts. Parallel and
  serial capture are byte-identical. Five entries pass the unchanged `2/32`
  contract and are promoted; `concavepaths` and `pathfill` retain named
  nonconvex-fill gates, while the three `poly_*` entries retain a shared
  fill-rule gate. The generic corpus runner now accepts explicit bounded
  `--jobs`; Sol review caught and drove fixes for image retention, fail-fast,
  deterministic diagnostics, output collisions, and worker-panic handling.
  The default remains one GPU process. A representative 40-entry run improved
  from 40.12s at one job to 15.78s at four jobs (2.54x), and the complete
  four-job ratchet finished in 4m42.87s at exact=679/diverges=0/gated=789.
- 2026-07-13: Use C++ Dawn WebGPU-on-Metal as the MSAA corpus oracle, with a
  distinct reference root and explicit producer/runtime/Dawn/adapter/artifact
  provenance. Native C++ Metal intentionally has no MSAA flush, and all 733
  MSAA rows currently point at nonexistent native-Metal PNGs; default-mode or
  Rust-generated images are not valid substitutes. The C++ runner emits the
  existing exact `RIVEABL` format, and the Rust pixel toolchain validates and
  converts that payload to PNG. This wires the C++ backend anticipated by the
  2026-07-11 reference identity decision without changing any tolerance.
- 2026-07-13: Keep `spotify_kids_app_icon` gated under the new
  `metal-webgpu-fixed-function-color-output` diagnostic. The pinned full-stream
  C++ Dawn oracle executes one 24-batch fixed-function atomic flush while Rust
  partitions the same stream into two runs. Their final clip backing is exact,
  both keep padded coverage rows untouched, neither exposes packed color
  storage, and they pass the unchanged `2/32` contract against each other.
  Against the native Metal reference they retain nearly identical residuals
  (48,759 C++ and 48,790 Rust pixels over delta 2, max 26; above 99.9% mask
  overlap). Raw coverage words are not compared across the different schedules
  because they retain schedule-local path IDs. Independent Sol review approved
  this boundary classification, rejected a causal hardware-blend claim, and
  required exact replay/schedule/absent-color provenance plus the mask-overlap
  assertion. The native reference and tolerance remain unchanged, and the
  exactly two packed-color `metal-webgpu-atomic-intermediate-precision` gates
  remain unchanged.
- 2026-07-13: Close the sampled clockwise-atomic clip-plane finding with a
  production-path readout rather than private Metal instrumentation. A large,
  pixel-aligned compound outer clip forces the global clockwise scheduler; an
  asymmetric nested clip followed by opaque white content records the exact
  `OutermostClip`, `NestedClip`, `ClippedContent` sequence. Rust proves its
  complete captured clip texture equals the probe output, and the 640x640
  output matches the pinned native Metal reference at zero delta. Sol rejected
  the initial small generic-atomic fixture, then approved this routed oracle.
- 2026-07-13: Pin the complete renderer shader lineage before closing R3's
  semantic-trap audit. Clean regeneration now requires runtime `7c778d13`,
  Naga 30.0.0, glslang 16.2.0, SPIRV-Tools 2026.1, ply 3.11, clean tracked and
  untracked shader inputs, and a fixed Python hash seed for upstream's
  otherwise nondeterministic WGSL identifier minifier. CI regenerates and
  asserts 60 raw Rust WGSL modules plus 50 canonical minified C++
  compiler-input headers by exact digest. Sol approved the architecture,
  source/tool fence, CI installation, and evidence wording. The remaining
  audit work is limited to the sampled clip-plane and decoded-image byte
  oracles documented in `docs/renderer-gpu-semantic-trap-audit.md`.
- 2026-07-13: Complete R2 after the exit-contract audit clarified the
  algorithm milestone boundary. All 108 upstream clockwise-atomic GMs are
  accounted for: 106 pass their committed contracts and the remaining two,
  `dstreadshuffle` and `interleavedfeather`, are independently reviewed
  `metal-webgpu-atomic-intermediate-precision` gates backed by SHA- and
  provenance-bound C++ Dawn WebGPU-on-Metal evidence. Zero `algorithm-core`
  gates remain. Both native references and tolerances stay unchanged. The
  invented-wgpu adversarial review, workspace suite, renderer ratchet
  (154/0/1,313), full V2 floor (263 files/584 segments), and scripted V2 floor
  (27/35) are green with no `.riv` regression. R3 starts with the GPU
  semantic-trap audit and renderer fuzz-replay harness.
- 2026-07-13: Reclassify `dstreadshuffle` from `algorithm-core` to
  `metal-webgpu-atomic-intermediate-precision` after pinned untouched and
  SrcOver-control C++ Dawn WebGPU-on-Metal lanes. The strict compiler validates
  stream SHA-256, opaque clear, 97 draws, 96 transforms, 97 saves/restores, 193
  path declarations, 97 paints, and every path/paint snapshot independently of
  the Rust parser. The untouched configured comparison intentionally retains
  and fails the existing `2/32` contract at 22,841-22,851 over-threshold pixels
  across repeated samples/max 61. The control changes exactly 97 blend setters
  and no other generated replay line; three samples pass at 11, 13, and 13
  pixels over delta 2/max 4. Artifact provenance pins the runtime, Dawn,
  adapter, driver, stream, artifact digest, and control override. Sol approved
  the narrower attribution, not promotion or a fitted tolerance. Status,
  native reference, and `2/32` contract remain unchanged.
- 2026-07-13: Reclassify `interleavedfeather` from `algorithm-core` to
  `metal-webgpu-atomic-intermediate-precision` after a pinned full-stream C++
  Dawn WebGPU-on-Metal oracle. A strict stream compiler validates the SHA-256,
  header, 451 draws, 900 transforms, 301 saves/restores, and exact path/paint
  snapshots without using the Rust parser. Artifact provenance records the
  C++ runtime, Dawn, adapter, and driver. Rust passes the entry's pre-existing
  `2/32` contract against C++ Dawn at 6 pixels over delta 2; there are 84
  byte-inexact pixels/max 26. Fresh native Metal differs from C++ Dawn and Rust
  nearly identically at 18,492 and 18,495 pixels over delta 2/max 78. Sol
  approved removing the algorithm attribution while keeping the entry gated,
  its native reference unchanged, and its tolerance unchanged.
  `dstreadshuffle` is next.
- 2026-07-13: Keep both remaining clockwise-atomic GMs gated while accepting
  the isolated `interleavedfeather` ColorBurn pair as Dawn-versus-wgpu
  quantization. The source-generated C++ fixture pins all input f32 bits and
  the four production batches. Sol rejected an initial semantic normalization:
  3,813 opposite-sign coverage words exposed that Rust encoded clockwise as
  nonzero and culled the wrong advanced feather face. Correcting generic
  feathered-clockwise preparation and the two advanced feather-fill pipelines
  makes normalized raw coverage exact. The packed color plane differs at
  exactly two words/max byte delta one, and the resolved frame differs at
  exactly those two coordinates/max
  channel delta seven. No corpus status or tolerance changes in this slice.
  Full-stream C++ WebGPU references for `interleavedfeather` and
  `dstreadshuffle` are the next independent gates.
- 2026-07-13: Promote `feather_cusp` under a bounded 16,384-pixel
  Metal-versus-WebGPU allowance. C++ and Rust match the severe direct cusp's
  complete tessellation inputs and every non-clear atomic coverage word; after
  matching C++'s fixed-color generic-atomic face and clockwise paint encoding,
  the same-backend final blit passes `2/32`. The full native Metal GM retains
  9,480 pixels beyond delta 2/max delta 11, in line with promoted overlapping
  feather families. Sol approved that cross-backend classification after
  catching and requiring fixes for advanced-blend culling and clipped authored
  fill-rule preservation; both now have focused regressions and C++ oracles.
- 2026-07-13: The mid-R2 wgpu resource-seam review is complete. Generated WGSL
  bindings, retained resource lifetimes, queue/readback ordering, and
  factory-lifetime pipeline ownership have no observed correctness mismatch.
  Render-target and decoded-image extents are now validated before wgpu, and
  generic disjoint atomic groups split before their 16-bit path IDs overflow.
  Clip-dependent logical-flush rollover remains a named R3 parity task because
  C++ budgets paths, contours, and tessellation resources together. Buffer
  rings, per-draw dummy resources, submit/wait cadence, and cross-factory
  pipeline caches remain measurement-led R4 work.
- 2026-07-13: Promoted `rawtext` after closing its complete pre-raster path.
  A deterministic stream-derived C++ production oracle pins provenance and
  matches Rust across all 438 CPU tessellation-span records (7,008 words),
  the `1+318` patch range, 36 contour records, and every texel of the 2,048x2
  RGBA32Uint tessellation texture. It exposed four shared preparation gaps:
  flush padding order, line-versus-cubic tangent provenance, C++'s fused SIMD
  line conversion, and unsigned reflected-row wrapping. After porting them,
  the final 263-pixel/max-80 residual is distributed across 76 components
  with largest area 10; isolated draws account for 190 and 73 pixels, and
  thresholded support IoU stays at or above 99.822%. Fresh forced-clockwise
  Metal differs from the legacy reference by zero pixels beyond delta 2. The
  unchanged channel delta 2 and bounded 288-pixel allowance advance the
  ratchet to exact=153/diverges=0/gated=1,314. Renderer golden, the full
  workspace suite, and both V2 golden floors pass.
- 2026-07-13: Promoted `strokefill` under a bounded 128-pixel
  native-Metal-versus-wgpu allowance at unchanged channel delta 2. Prefix
  replay shows no structural jump across its 14 mixed fill/stroke draws; every
  isolated draw is at or below 30 threshold pixels, while the full frame has
  109 pixels/max delta 48 split across 19 components with largest area 15.
  Foreground-support IoU stays above 99.985% at darkness thresholds from 1 to
  192, and the last four authored shapes are byte/threshold exact. A fresh
  forced-clockwise Metal reference agrees with the legacy native reference at
  zero pixels beyond delta 2. The ratchet advances to
  exact=152/diverges=0/gated=1,315; renderer golden, both V2 golden floors,
  and the full workspace tests pass. `rawtext` is the next unresolved GM.
- 2026-07-13: Promoted `strokes_round` without changing tolerance. A new
  production-ring `RIVEATS` oracle pins `firstSpan=0`, `spanCount=11`, the
  64-byte ABI, stream provenance, and exact C++/Rust equality across all 176
  words. It exposed three Rust departures: non-round joins had an invented
  one-segment shortcut instead of C++'s fixed five segments, line tangents
  came from the cubicized one-third control handle instead of the raw line,
  and tail padding followed geometry instead of preceding it at flush scope.
  Porting all three makes the CPU span oracle exact; the post-tessellation
  oracle is exact outside a bounded `0.00035`-radian backend angle delta. Fresh
  native comparison has zero pixels beyond delta 2 and max delta 2 under the
  unchanged `2/32` contract. The ratchet advances to
  exact=151/diverges=0/gated=1,316.
- 2026-07-13: Kept `strokes_round` gated after a 100-draw prefix sweep and
  isolated-draw oracle. Every isolated draw stays at max delta 1 except draw
  38, whose only five threshold violations are contiguous at the smooth
  start/close seam `(25,68..72)`; C++ leaves those pixels white while Rust
  renders the stroke. Restoring C++'s five direct-miter join segments had no
  pixel effect and was reverted. Sol rejected a shared 48-pixel allowance
  because the foreground-support disagreement still admits a tessellation
  input mismatch; the next proof is a record-exact C++/Rust pre-raster oracle
  for draw 38. Separately, fresh `overfill_opaque` prefixes are exact through
  the first colored draw, then add two identical 20-pixel/max-16 cubic-edge
  residuals under a 60-pixel translation. C++ repeats byte-exactly and the
  final C++/Rust foreground support is identical. A bounded 48-pixel
  Metal-vs-wgpu allowance promotes that entry and raises the ratchet to 150.
  Renderer verification is 150/0/1,317; V2 remains 263 files/584 segments,
  scripted V2 remains 27/35, and `cargo test --workspace` is green.
- 2026-07-13: Classified `dstreadshuffle` as the same native-Metal-versus-WebGPU
  atomic intermediate-color precision boundary as `interleavedfeather`.
  Prefixes through Lighten pass; ColorDodge first reaches 198 pixels/max-7,
  ColorBurn reaches 2,042/max-11, isolated ColorDodge passes at 1/max-3, and
  isolated ColorBurn differs at 768/max-9. A full SrcOver control reduces the
  complete board from 23,086 pixels/max-146 to 490/max-24, leaving only sparse
  geometry edges. No tolerance or shader fork is justified. A fresh forced
  native reference promotes `overfill_blendmodes` unchanged at 7/max-3, raising
  the ratchet to 149 exact entries. `strokes_round` is next at 34/max-83.
- 2026-07-13: Matched C++ `RawPath::pruneEmptySegments` behavior in
  stroke/feather preparation for cubic strokes whose four points coincide.
  Rust had retained these curves with zero
  tangents, dropping the round and square cap geometry in `zeroPath`; treating
  them as empty contours emits C++'s opposed synthetic cap joins. A unit test
  pins both cap types. Fresh native Metal comparison falls from 1,490
  pixels/max-128 to 26 sparse edge pixels/max-55 under the unchanged 2/32
  contract, promoting `zeroPath` and raising the ratchet to 148 exact entries.
  `dstreadshuffle` is next.
- 2026-07-13: Parked `interleavedfeather` after isolating its first meaningful
  failure to draws 13-14. Each draw alone is within one channel value of C++,
  but their ColorBurn pair differs at 97 pixels/max-18 and the complete GM at
  18,487 pixels/max-78; replacing ColorBurn with SrcOver makes the pair exact.
  A generated f16 color-plane storage/arithmetic experiment worsened the pair
  and full frame and was reverted. The remaining gap is a named native Metal
  versus WebGPU atomic intermediate-precision discontinuity, not a tolerance
  candidate. A fresh forced C++ reference proves
  `overstroke_blendmodes` passes unchanged at 1 pixel/max-3, promoting it and
  raising the ratchet to 147 exact entries. `zeroPath` is next.
- 2026-07-13: Mirrored clockwise feather fills now use C++'s contour-direction
  contract: direct fills write forward-then-reverse tessellation and atlas
  fills write single-sided descending spans, with matching contour anchors and
  negative-coverage flags. Structural row-wrap tests and fresh native Metal
  references pin the layout. Montserrat drops from 753,955 differing pixels to
  14 and Roboto from 744,884 to zero under the unchanged 2/32 contract, raising
  the ratchet from 144 to 146 exact entries. `interleavedfeather` is next; the
  already-isolated `feather_cusp` residual remains named rather than tolerated.
- 2026-07-13: Reclassified all 38 gated clockwise-atomic GMs against freshly
  forced C++ Metal output. Twenty-four already pass their existing contracts
  and are promoted with mode-correct references; no tolerance changed. The
  two 64x64 transparent-clear fixtures then isolated a real all-pixel defect:
  Rust supplied straight RGB to the frame attachment clear while C++ stores
  integer-premultiplied RGBA. Premultiplying each channel before the wgpu clear
  makes both outputs exact and is pinned by a scalar regression. The renderer
  ratchet rises from 118 to 144 exact entries with zero divergence. The
  remaining 12 GMs are now an evidence-backed queue rather than generic
  `algorithm-core` guesses. A read-only logical-sort audit also found that
  `drawType`, texture, scissor, and contents keys currently affect batching,
  while whole-draw execution conservatively preserves subpass correctness;
  full key parity is therefore R4 work unless a pixel counterexample appears.
  V2 remains 263 files/584 segments, scripted V2 remains 27 files/35 segments,
  and `cargo test --workspace` passes.
- 2026-07-13: Integrated the translated intersection board into MSAA fallback
  scheduling. Rust now reserves C++'s `max(prepassCount, subpassCount)` layers:
  three for fast fills, two for even-odd fills, and one for strokes, atlas
  draws, nested clip updates, and clip resets. Advanced destination-copy
  frames conservatively collapse to one layer per draw, unknown bounds block
  reordering, and the board resets before its signed group index can overflow.
  A full-image regression proves the board order `[0,2],[1]` is byte-identical
  to serialized source order while preserving the authored overlap. The C++
  Dawn oracle independently pins nine tagged MSAA batches: opaque draw 0
  reserves groups 1-3, disjoint translucent draw 2 occupies groups 1-3, and
  overlapping translucent draw 1 begins at group 4. Sol rejected two weaker
  fixtures whose ordering could be explained without all three board layers;
  the final type-10/type-8 boundary distinguishes a short reservation, and
  Sol's closure review reports no findings. `gm-batchedconvexpaths-msaa` now
  executes but remains gated because native Metal has no valid MSAA reference.
  The ratchet remains exact=118/diverges=0/gated=1,349; normal V2 remains 263
  files/584 segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Intra-group draw-type, texture, scissor, and
  subpass sorting remain separate `render_context.cpp` work.
- 2026-07-13: Added generic-atomic advanced blending for feathered fills and
  strokes. The generated non-fixed atomic path and atlas-blit shaders now have
  standard and HSL specializations; shared-flush, per-draw, and atlas paths
  select them, while destination initialization, storage color, and coalesced
  resolve remain unchanged. A C++ Dawn fixture pins the exact
  initialize, `midpointFanCenterAAPatches`, resolve schedule with
  `ENABLE_ADVANCED_BLEND | ENABLE_FEATHER | ENABLE_DITHER`. Dawn and wgpu stay
  within a tested backend envelope of eight pixels at channel delta 1, and the
  ordinary suite exercises all 15 advanced blend modes on direct paths. Sol
  found that atlas-required feathers still selected the fixed-color blit and
  could lose intermediate draws during advanced resolve. The accepted fix adds
  non-fixed standard/HSL atlas pipelines; a two-atlas-draw regression preserves
  both contributions in standard and HSL modes, all seven formerly unsupported
  `.riv` clockwise-atomic entries replay, and Sol's closure review reports no
  findings. A fresh native Metal bankcard reference still differs in 1,485,513
  pixels (max delta 20), so none are promoted; their corpus gate is narrowed to
  `native-clockwise-atomic-advanced-feather-parity`. This slice adds execution
  support without moving the renderer ratchet: exact=118/diverges=0/gated=1,349;
  normal V2 remains 263 files/584 segments, scripted V2 is 27 files/35 segments,
  and `cargo test --workspace` passes. MSAA board-group scheduling is the next
  measured `render_context.cpp` candidate.
- 2026-07-12: Closed MSAA destination-copy shader blending for solid
  feather-atlas draws. A C++ Dawn fixture pins the unextended WebGPU schedule:
  resolve at `dstBlend`, copy the intersected draw bounds into a single-sample
  sampled texture, then restart MSAA with color/depth/stencil load operations.
  Rust now segments the pass at each advanced atlas draw, preserves the old
  fixed-function path, and selects generated standard or HSL atlas shaders for
  all 15 advanced blend modes. The 64x64 ColorDodge fixture matches all 4,096
  RGBA pixels byte-for-byte; GPU regressions compare all modes to the generated
  atomic shader and preserve a non-rectangular path clip across two destination
  copies. A real `bankcard` MSAA replay also completes. Sol found one silent
  omission for gradient-backed advanced feather paints; the accepted fix
  retains a named `Unsupported` boundary until atlas gradient resources are
  implemented. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349; normal V2 remains 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Remaining `render_context.cpp` behavior and integration of the translated
  intersection board are next.
- 2026-07-12: Closed even-odd and clockwise MSAA path clips for feather-atlas
  draws. Rust now selects C++'s exact outer schedules: non-zero and clockwise
  run borrowed/update/cleanup with clockwise cleanup limited to write mask
  `0x7f`, while even-odd runs parity stencil then cover/reset. Nested clips use
  write mask `0x01` for even-odd or `0x7f` otherwise; the parent intersection
  reset reads `0xc0` for clockwise and `0xff` for non-zero/even-odd. Clip
  tessellation now mirrors C++ contour orientation under negative transforms.
  Two C++ Dawn fixtures pin the exact five/six-batch schedules and all 4,096
  RGBA pixels. Sol found that the first stroked fixtures did not behaviorally
  distinguish the special pipelines; the accepted filled fixtures now expose
  nested even-odd and outer even-odd holes plus outer/nested opposite-winding
  rejection. Sol's closure review reports no remaining findings. Terra's
  bounded README/harness update passed all 16 format tests and was accepted
  after local diff review. The renderer ratchet remains
  exact=118/diverges=0/gated=1,349; normal V2 remains 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Destination-copy shader blending is next.
- 2026-07-12: Closed nested non-zero MSAA path clips for feather-atlas draws.
  The C++ Dawn oracle pins the exact `8,9,10,11,14,4` schedule: outer
  borrowed/update/cleanup, double-sided nested winding stencil, parent-bounds
  intersection reset, and clipped atlas blit. Rust ports the generated path
  and stencil shaders with C++'s depth, cull, reference, compare, and write-mask
  state and matches all 4,096 RGBA pixels byte-for-byte. Active clip stacks now
  retain their C++ clip ID and render only a newly appended suffix; a focused
  regression proves `[A] -> [A,B]` consumes one ID and schedules only nested
  update, intersect reset, and content. Sol found and verified that incremental
  correction, then reported the integrated seam clean. The renderer ratchet
  remains exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584
  segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Even-odd and clockwise MSAA clip transitions
  are next.
- 2026-07-12: Closed changing outer non-zero MSAA path clips for atlas draws.
  Rust now ports C++ `ClipReset::clearPreviousClip` through the generated
  `draw_msaa_stencil` shader, exact `Depth24PlusStencil8` not-equal/zero state,
  clockwise front face with counterclockwise culling, transformed `roundOut()`
  reset bounds, and the six `TriangleVertex` rectangle. Unchanged clips skip
  both ID allocation and redundant stencil updates; an unclipped draw may run
  while the prior stencil remains retained, and the next unrelated clip still
  resets it. The Dawn oracle asserts the exact nine-batch schedule
  `8,9,10,4,14,8,9,10,4`, every base/count range, the 97x48 atlas, 2048x1
  tessellation texture, and exact left/reset-gap/right pixels. Rust matches
  all 4,096 pixels, while the previous unclipped, rectangle-clipped, and
  single-path-clipped frames remain exact. Terra identified reset-adjacent GM
  streams, but Sol correctly classified them as negative future boundaries,
  not positive coverage for this feather-atlas slice. Sol's adversarial review
  also tightened the oracle, reset-pixel test, retained-stencil transition,
  and native bounds parity before reporting no remaining findings. The
  renderer ratchet remains exact=118/diverges=0/gated=1,349; normal V2 is 263
  files/584 segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Nested non-zero atlas clip intersection is
  the next source-order R2 boundary.
- 2026-07-12: Closed the unchanged outer non-zero MSAA path-clip slice for
  atlas draws. The fallback attachment is now depth/stencil, the path pipeline
  implements C++'s borrowed/update/cleanup stencil states and exact inner-fan
  range, and the atlas pipeline selects fixed-function path, rectangle, or
  combined path-plus-rectangle clipping. A new C++ Dawn oracle asserts the
  exact four-batch schedule: three clip-update draws followed by an
  active-clip atlas draw whose shader features remain dither-only. Rust matches
  all 4,096 pixels for unclipped, rectangle-clipped, and path-clipped atlas
  frames. Nested clips, changing outer clips, alternate clip fill rules,
  non-atlas MSAA path clips, and MSAA images remain named `Unsupported`
  boundaries with early-ingress regressions. Terra supplied the bounded C++
  batch inventory; the executable oracle corrected its shader-feature
  inference. Sol then found and closed the alternate-fill and image-mesh
  ingress leaks before reporting the final diff clean. `make renderer-golden`
  remains exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584
  segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Clip reset and the remaining clip-stack
  transitions are the next source-order R2 boundary.
- 2026-07-12: Closed MSAA rectangle clip-distance atlas blits. Device creation
  requests `CLIP_DISTANCES` only when the adapter exposes it; the atlas blit
  pipeline then selects upstream's generated clip-distance vertex permutation
  and uploads the existing C++-shaped clip inverse matrix through
  `PaintAuxData`. Adapters without the feature retain the named `Unsupported`
  result. The C++ Dawn oracle now enables WebGPU clip planes only for a new
  clipped case, asserts the `ENABLE_DITHER | ENABLE_CLIP_RECT` batch, and emits
  a complete 64x64 `RIVEABL` artifact. Rust matches all 4,096 pixels exactly;
  an always-on GPU fence also proves output is confined to the clip rectangle.
  Sol's adversarial review found and closed two portability gaps: the ordinary
  test now preserves the unsupported branch, and unrelated C++ oracle modes no
  longer require clip distances. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Path-clip/stencil is the next R2 boundary, followed by destination-copy
  shader blending.
- 2026-07-12: Closed the matching C++ WebGPU MSAA atlas final-blit oracle.
  MSAA now forces feathered solid fills and strokes through the translated
  atlas tessellation and R16 mask passes, then draws the upstream six-vertex
  atlas rectangle through a dedicated 4x fixed-function pipeline. The original
  blank Rust frame differed at all 4,096 pixels/max delta 80; the final RGBA8
  frame is byte-exact, including upstream dither and premultiplied output.
  Always-on GPU tests cover the canonical stroke, two ordered feathered fills,
  and resource retention across multiple atlas draws. Sol's adversarial review
  found that the first patch silently accepted clip rectangles, path clips,
  and non-source-over blending without their required shader permutations.
  Those cases now return named `Unsupported` errors with regressions instead
  of drawing incorrect pixels; Sol's final review reports no remaining blocker
  for the intentionally narrow slice. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349. Rectangle clip-distance, path-clip/stencil,
  and destination-copy blend variants remained as the next source-order work.
- 2026-07-12: Removed a generic-atomic interior race in the invented wgpu
  batching seam. Shared flush groups had submitted outer-curve patches and
  interior triangles in one render pass; five identical isolated `bad_skin`
  hair renders differed at 114-264 pixels/max delta 255 because attachment
  writes were not ordered with the storage atomics. Triangle-backed draws now
  use the existing ordered outer and interior render passes. A GPU regression
  is red without the split and byte-stable across five repeats with it; the
  isolated hair falls to one pixel/max delta 31 versus Metal.
  A new direct C++ WebGPU preparation oracle reproduces the exact authored
  non-zero hair path, transform, and frame under the clockwise override. It
  captures one contour, 48 `TriangleVertex` records, and the complete 2048x1
  RGBA32Uint tessellation texture. Sol rejected the lane's first authored-
  clockwise capture as a false oracle; the amended non-zero capture passed
  closure review. The current single-contour ear clip differs canonically from
  C++ triangles, while substituting the global inner-fan stream and broad CWA
  routing regressed four exact entries, so that separate port boundary was
  reverted rather than traded against this fix.
  Three full post-fix renders are stable at 2,701 pixels beyond delta 2/max
  delta 159. Across 2,635 residual components, 2,626 are single pixels and
  only four exceed area three; all 69 isolated draws stay at or below 40
  pixels beyond delta 2. Sol accepted a bounded 4,096-pixel composite/backend
  allowance at unchanged channel delta 2. The ratchet advances to
  exact=118/diverges=0/gated=1,349; the matching WebGPU MSAA final-blit oracle
  is next.
- 2026-07-12: Added the combined clockwise-atomic and advanced-blend color
  path. Draw-prefix Metal replay isolated `juice`'s first structural jump to
  draw 15, an overlay compound fill routed through fixed-color CWA shaders;
  the generic advanced shader already matched preceding multiply content.
  Advanced CWA draws are now isolated at run boundaries and render their paint
  and coverage through the existing fixed-function CWA pipeline into a
  transparent intermediate. A one-fragment-per-pixel composite then applies
  the upstream advanced equations against an explicit destination copy. This
  preserves C++'s hardware source-over ordering without cross-fragment storage
  races. Rectangle-clip specialization is supported, while path clips and
  feather retain explicit gates after internal CWA selection. Focused GPU
  regressions pin white overlay over a prior compound fill through a partial
  clip rect, compare all fifteen advanced modes against the generated generic
  atomic shader, and prove advanced CWA path clips return `Unsupported` rather
  than panicking.
  All 18 cumulative `juice` prefixes track Metal; all five byte-identical full
  frames retain 140 edge pixels beyond delta 2/max delta 12 and are promoted
  under a 256-pixel allowance. The ratchet advances to
  exact=117/diverges=0/gated=1,350; `bad_skin` is next.
- 2026-07-12: Preserved C++'s frame-wide clockwise fill override after an
  axis-aligned clip is reduced to paint metadata. Rust had excluded every
  clip-rect draw from the true clockwise pipeline, so `joel_signed` rendered a
  detached opposite-winding leaf as non-zero content. Clip-rect-enabled path,
  interior, sampled-clip path, and sampled-clip interior pipeline variants now
  select upstream specialization ID 1 per draw. A visible-bounds eligibility
  check mirrors C++'s pre-allocation cull for offscreen paths. Exact first-draw
  and partial-clip GPU regressions pin winding and clip-rect behavior;
  `gm-mesh` retains its partial clip at 14 pixels beyond delta 2, and
  `db_health_tracker` no longer reaches coverage allocation for its offscreen
  draw. All five byte-identical Joel references
  retain only 191 mostly isolated edge pixels/max delta 5, so they keep delta 2
  with a bounded 256-pixel allowance. The ratchet advances to
  exact=112/diverges=0/gated=1,355; `juice` is next.
- 2026-07-12: C++ `RiveRenderer::applyClip` generates a new clip ID whenever
  an element is rendered into the clip buffer. Rust had encoded stack depth as
  the ID, so unrelated root clips all reused ID 1 and stale coverage from an
  earlier clip admitted `off_road_car`'s windshield gradient around the front
  grille. Unique per-render clip IDs across paths, images, and meshes remove
  that coherent 2,042-pixel leak; a two-root-clip regression pins the lifecycle.
  All five recorded samples are byte-identical on each backend and now share an
  identical 1,862-pixel residual. Its 55 components are confined to thin ground,
  stripe, and small car edges; the largest occupies a 243x13 strip. Replacing
  the implicated gradient with solid cyan leaves exactly the same residual,
  proving the structural clip error is closed. The family keeps max channel
  delta 2 with a bounded 2,048-pixel backend allowance and advances the ratchet
  to exact=107/diverges=0/gated=1,360.
- 2026-07-12: `db_health_tracker` keeps max channel delta 2 with a bounded
  1,152-pixel allowance. Draw-prefix replay proves its first 430 of 473 draws
  exact, including the chart's 353-stroke batch, clips, cards, and markers.
  Divergence starts in the late solid text-outline fills and accumulates in
  small increments to 1,071 pixels across a 2,073,600-pixel frame; 432
  connected components are present, none larger than 13 pixels, and only 32
  samples exceed delta 32. Flat fills and the two gradient draws add no
  residual. This is repeated native Metal/wgpu glyph/path-edge placement, not
  missing algorithm work. The ratchet advances to
  exact=102/diverges=0/gated=1,365.
- 2026-07-12: `ai_assitant` keeps max channel delta 2 with a bounded 384-pixel
  allowance. Its exact background and 16 repeated rotated stroke pairs rise
  smoothly from 1 to 341 residual pixels with no structural jump; 340 of 341
  connected components are single pixels and foreground-support IoU remains
  99.3-99.8%. Replacing every gradient shader with solid cyan preserves the
  residual class (350 pixels, max delta 15), disproving gradient math. Per-draw
  cutoffs concentrate the accumulation in each intact `feather=12` companion,
  isolating native Metal/wgpu feather-edge placement rather than missing feather
  geometry. The ratchet advances to exact=101/diverges=0/gated=1,366.
- 2026-07-12: `new_text` keeps max channel delta 2 with a bounded 48-pixel
  allowance. Draw-prefix replay attributes the first divergence to its compound
  text path: 44 residual pixels split into 22 components, none larger than four
  pixels, with foreground-support IoU near 99.5%. Replacing the path's gradient
  with solid white preserves the same residual class (40 pixels, max delta 47),
  disproving gradient math and isolating native Metal/wgpu path-edge placement.
  The ratchet advances to exact=100/diverges=0/gated=1,367.
- 2026-07-12: Completed the post-gradient `.riv` sweep. The host port now uses
  C++'s exact `math::EPSILON` (`1/4096`) and forward/backward monotonic stop
  clamps; all five gradient GM oracles remain unchanged. Of 38 gated
  gradient-bearing clockwise entries, 30 render and now have fresh C++ Metal
  references, while eight retain explicit native clockwise-atomic
  advanced-feather parity or clip-rect diagnostics. Eleven pass under the
  existing strict delta-2/32-pixel budget:
  `death_knight`, `deterministic_mode`, `interactive_scrolling`, all five
  `rocket` samples, `scroll_test`, `scroll_threshold`, and `zombie_skins`.
  Larger measured residuals remain gated rather than tolerated. The ratchet
  advances to exact=99/diverges=0/gated=1,368.
- 2026-07-12: The `jellyfish_test` mip gate was false attribution. A bounded
  draw/LOD inventory found all 22 image draws at LOD 0; a no-mip render is
  byte-identical to the corrected nearest-mip render. Prefix replay then proved
  the solid background exact, the first radial gradient added 589,692 divergent
  pixels, and both radial gradients added 866,438 before any image draw. Porting
  `render_context.cpp`'s 512-wide simple/complex `GradientSpan` layout through
  the generated color-ramp WGSL, plus C++ gradient normalization, opacity, and
  inverse paint matrices, makes `degengrad`, `rect_grad`, and `verycomplexgrad`
  exact at delta 2; `strokedlines` and `xfermodes2` retain only 4 and 8 edge
  pixels under their existing 32-pixel budgets. `jellyfish_test` falls from
  604,916/max 139 to 22,363/max 7 and is promoted under a 23,000-pixel strict
  delta-2 image/backend allowance. Matching C++'s nearest mip selection also
  shrinks the stale image allowances: `image`/`image_aa_border`/`image_lod`/
  `mesh` to 32 pixels, `tape` to 64, and `superbowl` to 128. The ratchet advances
  to exact=88/diverges=0/gated=1,379.
- 2026-07-12: Image allocation now requests the selected adapter's supported
  `max_texture_dimension_2d` instead of inheriting wgpu's 2,048 downlevel
  default. A 2,080-pixel decode regression pins the request; `jellyfish_test`
  (2,080x2,080) and `superbowl` (2,914x296) both render instead of panicking.
  Fresh `superbowl` output is visually coincident and promoted at strict delta
  2 with 11,268/max 64 samples under a 12,000-pixel backend allowance.
  `jellyfish_test` is not tolerated: all 23 CoreGraphics decodes match Rust
  premultiplication within one byte with identical alpha, while its 604,916/max
  139 frame delta is isolated to translucent glows and edges. It is reclassified
  to `platform-mipmap-filtering` pending a mip-level oracle. The ratchet advances
  to exact=82/diverges=0/gated=1,385.
- 2026-07-12: PNG decode now honors `iCCP` metadata with a pure-Rust moxcms
  transform from the embedded profile to sRGB before alpha premultiplication,
  matching C++ ImageIO's color-convert-then-premultiply order. On `gm-mesh`
  this reduces the fresh C++ delta from 140,327/max 56 to 17,450/max 34 while
  preserving all twelve transforms, clips, and blend modes. The remaining
  Metal-vs-wgpu decoder/filter samples are bounded at strict delta 2:
  `image` 8,814/max 39 under 9,000, `image_aa_border` 5,745/max 71 under 6,000,
  and `mesh` 17,450/max 34 under 18,000. These are whole-entry backend
  allowances over visually coincident output, not missing-algorithm
  tolerances; the ratchet advances to exact=81/diverges=0/gated=1,386.
- 2026-07-12: Advanced atomic image blending follows C++ WebGPU's non-fixed
  color-output lifecycle rather than per-draw framebuffer copies: request the
  seventh fragment storage binding, copy the current target before each
  advanced atomic run, initialize a tiled `u32` color plane through
  `loadColorFromDstTexture`, assign authored z indices, run the generated
  non-fixed image/path shaders, and coalesced-resolve once. A 1x1 GPU oracle
  pins screen, darken, exclusion, and luminosity over a known destination.
  Fresh C++ `gm-mesh` renders all twelve meshes with matching geometry and
  blend character, but its 319x320 PNG carries a large ICC profile; even the
  srcOver control column differs at 19,752 pixels/max delta 56, while the full
  file differs at 140,327 pixels/max delta 56. The entry is reclassified to
  `platform-image-decode-color-profile`; no tolerance was widened.
- 2026-07-12: `ImageMeshDraw` follows C++'s retained-buffer contract: position
  and UV streams are separate `float2` vertex buffers, indices are `u16`, and
  every unmap snapshots a new submitted wgpu buffer so later mutations cannot
  rewrite queued draws. The generated fixed-color atomic mesh shaders provide
  `srcOver`, clipping, clip-rect, transform, opacity, and sampler parity.
  `tape` retains delta 2 with a bounded 6,400-pixel allowance: 6,162 pixels
  differ versus fresh C++ Metal (max delta 31), all inside the three decoded
  image interiors; foreground-support masks differ at only 89-192 sparse edge
  pixels across 1%-20% thresholds. Advanced image blend modes remain named
  algorithm work and are not covered by this allowance.
- 2026-07-12: Encoded-image dispatch supports both corpus formats: PNG and
  JPEG. `clipping_and_draw_order` was a decode gate, not a clip-buffer failure:
  its embedded bytes begin with JPEG SOI, and the PNG-only decoder returned an
  empty image before either draw reached the renderer. With pure-Rust JPEG
  decode, both 278x278 images, the circular clip boundary, and all authored
  ordering are present. Its bounded 10,000-pixel allowance at delta 2 covers
  the measured 9,494 ImageIO-versus-`jpeg-decoder` color samples (max delta 18),
  all confined to the two image interiors; the pre-fix missing-image result was
  104,981 pixels, over ten times the allowance.
- 2026-07-12: ImageRect uses the upstream generated fixed-color atomic shader,
  not the separate atomic color-buffer variant. PNGs upload as premultiplied
  RGBA with the full C++ mip count, and each remaining mip is generated through
  the upstream WebGPU filtered-blit shaders. `image_lod` retains delta 2 with a
  bounded 512-pixel allowance: 276 pixels differ after mip generation, max 43,
  with all authored images and transforms present. Metal platform decode color
  management and clipped-image atomics remain named gates, not tolerances. Sol
  review also made MSAA/fallback images explicitly unsupported until their
  pipelines exist, and hoisted ImageRect geometry, dummy bindings, and all 18
  sampler permutations so non-image draws do not inherit image resource churn.
- 2026-07-12: Legacy homogeneous midpoint-fill batches may share shelf-packed
  tessellation storage and a render pass. Clockwise and clip-update batches
  preserve the established per-draw resource/pass topology. Intersection-board
  groups are submitted independently to bound backend resource lifetime; R4
  must measure and optimize the wait policy without weakening corpus parity.
- 2026-07-10: Phase R activated by the user; incremental R0-R5 strategy chosen.
- 2026-07-10: Pixel space is canonical top-left RGBA8. The C++ Metal bridge
  readback is vertically flipped during replay; the Rust renderer is not
  distorted to match backend-native texture coordinates.
- 2026-07-10: `nuxie-render-stream` is the renderer isolation boundary. Runtime
  and GM capture both produce the same typed stream; C++ FFI and Rust wgpu
  replay consume it independently.
- 2026-07-11: A renderer reference is identified by stream, frame, and mode.
  C++ Metal is the clockwise-atomic oracle; MSAA rows remain harness-gated
  until a C++ backend with implemented MSAA flush is wired into replay.
- 2026-07-11: C++ Metal and C++ WebGPU intentionally use different atlas
  stroke cull states. Final Metal pixels remain a corpus signal, but atlas-mask
  diagnosis compares Rust wgpu against C++ WebGPU at the intermediate R16 mask.
- 2026-07-12: The two large clockwise clip entries retain max channel delta 2
  with a bounded 640-pixel Metal-vs-wgpu allowance. Their 50%-coverage masks
  are pixel-identical; the 592-593 residual pixels are confined to clip
  boundaries, with no missing or extra binary coverage.
- 2026-07-12: Sol review confirmed that forced clockwise-atomic mode
  intentionally replaces authored nonzero/even-odd fill semantics; preserving
  parity would contradict the C++ oracle. Viewport-bounded nested inverses are
  behaviorally equivalent while the parent clip remains active, so parent
  content/tightened bounds stay a performance task unless pixels prove otherwise.
- 2026-07-12: `negative_interior_triangles` keeps max channel delta 2 with a
  bounded 1,152-pixel allowance. The two isolated determinant draws differ at
  553 and 487 pixels, the combined 1%-coverage support masks differ at only 26
  pixels, and the 1,040 residuals are sparse backend edge coverage rather than
  missing geometry. At this point the mirrored as-clip case remained gated
  because its broad blank region was still an algorithm failure.
- 2026-07-12: `negative_interior_triangles_as_clip` keeps max channel delta 2
  with a bounded 64-pixel allowance. After the mirrored fallback fix, only 46
  pixels exceed delta 2 across 2.56M pixels and max delta is 7; both shapes,
  checkerboard clipping, and corresponding interior support are restored.
- 2026-07-12: `convexpaths` keeps max channel delta 2 with a bounded 64-pixel
  allowance. After the row-wrap fix, only 43 pixels exceed delta 2 across
  1.32M pixels; the remaining max-103 samples are sparse hard-edge backend
  differences, not missing support.
- 2026-07-12: `pathfill` keeps max channel delta 2 with a bounded 256-pixel
  allowance. Its 253 residuals have 99.5% support overlap and split into tiny
  hard-edge components; the largest is 56 pixels inside a 19x13 box. Max-255
  samples are one-pixel binary edge placement, not missing shapes.
- 2026-07-12: `oval` keeps max channel delta 2 with a bounded 128-pixel
  allowance. After the midpoint-fan admission/cull fix, all 109 residuals are
  one-pixel edge components, the largest is 16 pixels, and foreground support
  has 99.9965% IoU with equal expected/actual support counts.
- 2026-07-12: `mutating_fill_rule` keeps max channel delta 2 with a bounded
  64-pixel allowance. All 45 residuals form four one-pixel vertical edge
  components, max delta is 11, and expected/actual foreground support is
  identical (IoU 1.0).
- 2026-07-12: Self-intersecting and compound fills form their own
  clockwise-atomic runs. Endpoint normalization keeps ordinary closed cubics
  on the legacy analytic path; this preserves the promoted large-path corpus
  while matching C++ atomic accumulation for complex topology.
- 2026-07-12: Dominant winding uses C++ `RawPath::computeCoarseArea` stream
  order, including coarse cubic subdivision. This order is observable when
  opposite contours nearly cancel and must not be replaced by independently
  rounded per-contour areas.
- 2026-07-12: Render-paint stroke thickness follows C++ and stores `abs(value)`;
  invalid `NaN` remains invalid. Negative GM inputs therefore become positive
  strokes before draw-time culling.
- 2026-07-12: `beziers` retains the standard max-delta-2/32-pixel contract.
  Its 17 residual pixels are disconnected one-pixel edge samples at max delta
  4; no geometry or connected support is missing.
- 2026-07-12: `bug339297` and `bug339297_as_clip` retain max channel delta 2
  with a 1,280-pixel allowance. Both backend pairs have zero binary-support
  differences and identical black/white pixel counts; all residuals occupy
  the same two antialiased full-width scanlines under million-scale coordinate
  cancellation, so this is a Metal-versus-wgpu precision difference rather
  than missing fill or clip geometry.

## R4 Addendum (coordinator, 2026-07-16 — fold into the item queue as
priority work; rationale recorded here so no session re-derives it)

A. **Same-capability denominator (resolved).** The fenced C++ runner already
   uses Dawn WebGPU over Metal, and the Rust runner uses wgpu WebGPU over Metal;
   both select the same Apple adapter, receive the same render mode, replay the
   same stream, and wait for GPU completion after every frame. Native C++ Metal
   cannot execute the fixed MSAA matrix and is therefore not the denominator.
   The R4 gate is C++ Dawn versus Rust wgpu in the same mode. Native Metal may
   be reported as informational evidence where implemented, but it cannot
   block same-tier optimization or redefine the fixed report.
B. **Counter-parity harness (perf-counter-compare).** Deterministic
   per-stream counters on BOTH renderers — flushes, draws/flush,
   tessellation spans, patch counts, uploaded bytes, texture binds,
   encoder rows — diffed per perf-corpus entry and ranked by excess
   ratio. Zero timing noise; every excess row is a located inefficiency
   with a C++ site to read. Structural-counter checking is currently
   defensive; make it the primary offensive attribution tool. Excess-row
   fixes are bounded, oracle-gated, and delegable to workers.
C. **Perf-mechanism inventory scout (proactive port list).** One scout
   enumerates EVERY pooling/ring/budget/reuse/coalescing mechanism in
   render_context.cpp + backend buffer management (the render-side
   analog of V2's dirt-gating audit). Remaining R4 items become 'port
   the checklist' verified with the proportionate evidence in D, with
   profiling as confirmation rather than discovery.
D. **Evidence proportional to uncertainty.** A deterministic reduction in
   redundant work, such as twenty equivalent passes becoming one, is accepted
   with exact structural counters, unchanged pixels/contracts, and a light
   directional timing snapshot that is context, not a load-unmatched gate.
   Repeated alternating reports and load-matched A-B-B-A are reserved for
   timing-defined candidates, disputed effects, or claims not located by exact
   counters. A noisy directional sample alone does not trigger A-B-B-A for an
   objective work-elimination slice. Attribution of multiple corpus entries
   may run concurrently; timing acceptance stays serial and records load.
E. **Timing-defined acceptance gate (ready, not per-slice ceremony).** The
   final timing gate pins immutable runner artifacts and runs A-B-B-A with a
   same-leg C++ control inside every report. It normalizes each candidate leg
   against that control, fences global control drift plus both normalized
   repeat pairs, and samples host load only before and after synchronous timed
   work. Behavioral tests prove the sampler cannot overlap a runner and that
   a failing runner leaves no live child. Sol approved all five contracts.
   Use this gate for the final timing-defined R4 decision or a genuinely
   disputed timing claim; exact counter-parity slices continue to use D.

## Log

- 2026-07-16: Closed R4 item 130 and pre-attributed the complete remaining
  counter tail. One C++-ordered logical tessellation allocation now carries
  both midpoint-fan and outer-curve atomic work across texture rows while
  semantic clip-update passes remain separate. `gm-bug339297` reaches exact
  normalized `(passes,draws,instances,spans,patches)` at
  `(6,5,542,117,423)` and `gm-bug339297_as_clip` at
  `(8,7,555,121,431)`; all ten `BUG-MIX` rows disappear and no replacement
  upload row appears, moving the ranked report 26->16. Sol caught an empty
  shared-triangle buffer selection panic and a checked reflected-row underflow
  during review; both are corrected with focused regressions. Sol's final
  review passes and the 284-test renderer suite is green. Parallel read-only
  attribution leaves four finite implementation targets:
  `OVER-AENV`, shared typed uploads, two `UPLOAD-LAYOUT` rows, and the
  oracle-first extra stroke patch. Timing is a light directional snapshot
  only; no A-B-B-A campaign was run. Final verification passes renderer
  exact=1,409/diverges=0/gated=59, normal/scripted V2 floors at 584/35 exact
  segments, the full workspace, formatting, and diff hygiene.
- 2026-07-16: Closed R4 items 128-129 and replaced row-at-a-time tail work
  with `docs/renderer-r4-counter-tail-audit.md`. Item 128 ports C++'s
  one-segment line rule, lazy stale-stencil lifetime, and multi-row
  logical-flush midpoint layout. The primary MSAA target reaches exact bind,
  draw, instance, span, and patch counts; the shared layout also closes the
  `OverStroke` MSAA span excess. Ranked rows fall 35->26. A Sol pass then
  classifies every current row into four finite shared-cause clusters, with no
  accounting-only closures and no singleton implementation tasks. A separate
  capture inventory proves all final-pixel references already exist. The
  final-source corpus remains exact=1,409/diverges=0/gated=59, and a clean
  counter-runner rebuild confirms exactly 26 ranked excess rows. In the
  orthogonal tooling lane, the final timing gate now performs per-leg C++
  normalization, separate control/A/B repeat fences, pre/post-only host
  sampling, foreground child collection, immutable runner checks, and
  behavioral failure-path tests. The complete `perf-compare` suite passes and
  Sol's second review reports no findings.
- 2026-07-16: Closed R4 item 127 by sharing one flush-wide midpoint-padding
  envelope across the ten homogeneous translucent fills in
  `gm-batchedconvexpaths`. C++ source inspection shows one logical-flush
  tessellation allocation with one leading padding span and one final
  sentinel; Rust's texture was shared but its padding remained path-local.
  Atomic spans move 101->78 and MSAA 105->78, both exact. Atomic
  instances/uploads move 244->221 and 9,752->8,216 bytes; MSAA moves 318->291
  instances exactly and 10,400->8,608 bytes. Draws and patches are unchanged.
  Fixed-matrix spans move 1,708->1,658, instances 5,987->5,937, uploads
  179,824->176,496 bytes, and ranked positive rows 39->35. Target pixels are
  byte-identical. The 2.114x one-frame snapshot is directional context only;
  exact work counters and unchanged pixels accept the change without A-B-B-A.
  Renderer exact=1,409/diverges=0/gated=59, the renderer perf-counter suite
  passes 276/39, normal/scripted V2 floors remain 584/35, and the workspace
  passes. The remaining target upload-byte differences are recorded as
  separate alignment/layout debt. Item 128 attributes the multi-counter
  `gm-bug339297_as_clip-msaa` row.

- 2026-07-16: Closed R4 item 126 by replacing Rust's local single-contour ear
  preparation with the C++-matched global interior triangulator. Source
  inspection disproved the queued borrowed-stroke diagnosis: the two +200
  rows were fill patches. `gm-bug339297` moves 623->423 path patches and
  `gm-bug339297_as_clip` 631->431, both exact with C++ Dawn. A direct oracle
  matches contours, triangle order, and every tessellation texel; generic Dawn
  readback puts both final frames at zero pixels beyond delta 2/max-1. Their
  primary references move from native Metal to same-tier C++ Dawn and tighten
  from 2/1,280 to 2/32. The fixed matrix restores legitimate interior work:
  path patches move 4,666->4,266 and instances 6,191->5,987, while passes
  move 91->94, spans 1,514->1,708, uploads 159,208->179,824 bytes, and ranked
  positive rows 35->39. The one-frame matrix snapshot is 2.421x directional
  context; no A-B-B-A campaign was run. Renderer
  exact=1,409/diverges=0/gated=59, the renderer feature suite passes 275/39,
  normal/scripted V2 floors
  remain 584/35, Dawn readback and the workspace pass, and Sol found no
  correctness issue. Item 127 attributes the new `batchedconvexpaths` top rows.

- 2026-07-16: Closed R4 item 125 by compacting and merging compatible direct
  strokes within the seven C++-matched intersection-board groups.
  `gm-OverStroke` moves 14->9 atomic draws and 13->8 MSAA draws; MSAA is exact
  and atomic is one below C++ because Rust clears instead of drawing an
  initialize operation. Fixed-matrix Rust draws move 161->151, instances
  6,219->6,191, spans 1,542->1,514, uploads 160,744->159,208 bytes, and ranked
  excess rows 38->35. The production counter test, grouped-versus-unbatched
  pixel regression, synthetic boundary tests, renderer feature suite (273/38),
  workspace suite, normal 584-segment floor, and scripted 35-segment floor
  pass. The renderer corpus remains exact=1,409/diverges=0/gated=59. Sol found
  no implementation defect and its end-to-end coverage request is
  incorporated. The light snapshot is load-unmatched context only. Item 126
  investigates the exactly +200 `bug339297` path-patch rows; the later source
  audit identifies them as a single-contour interior-preparation mismatch,
  not borrowed stroke coverage.

- 2026-07-16: Closed R4 item 124 by replacing per-path plain-stroke midpoint
  padding with C++'s one flush-wide envelope in both modes. On
  `gm-bevel180strokes`, spans fall 120->63 in atomic and MSAA, exact with C++;
  MSAA instances fall 160->103 exactly and atomic 161->104 against C++'s 105
  counted-initialize convention. Fixed-matrix Rust spans move 1,668->1,542,
  instances 6,345->6,219, uploaded bytes 168,424->160,744, and ranked excess
  rows 47->38. The focused test, 268-test renderer feature suite, workspace,
  renderer corpus, and V2 floors 584/35 pass. Sol found no issue in the fences,
  relocation, IDs, flush boundaries, or width limits. The one-frame snapshot
  overlapped the renderer sweep and is recorded only as contaminated
  directional context; exact counters and unchanged output are the acceptance
  evidence. Item 125 owns the newly attributed five-range direct-stroke merge
  on `gm-OverStroke`.

- 2026-07-16: Closed R4 item 123 by resolving clear-owned ordinary MSAA runs
  directly into the final target. Preserve-target chunks keep fallback
  composition, and advanced destination-read MSAA is unchanged. Fixed-matrix
  Rust passes fall 107->91, exactly matching C++ Dawn; draws fall 169->161,
  instances 6,353->6,345, created bind groups 61->53, bind-group sets 302->294,
  texture bindings 98->90, and ranked excess rows 71->47.
  `gm-batchedtriangulations-msaa` is exact at 2 passes/4 draws/104 instances.
  The target/matrix one-frame snapshots are 1.657x/2.839x and remain
  directional only. Sol found no implementation issue and prompted a focused
  translucent split-submission regression. Renderer exact=1,409/diverges=0/
  gated=59, V2 floors 584/35, the renderer feature suite passes 267/38, and
  the workspace suite passes. Item 124 owns the new mode-paired 120-versus-63
  stroke tessellation-span excess.

- 2026-07-16: Closed R4 item 122 with C++'s compact midpoint layout and
  subpass-major MSAA fill merge. `gm-batchedtriangulations-msaa` falls from 14
  to five draws, 114 to 105 instances, and 32 to 23 spans; C++ Dawn reports
  4/104/23, with Rust's one extra draw/instance isolated to the fallback
  composite. Path patches stay exact at 81. Fixed-matrix Rust draws move
  178->169, instances 6,362->6,353, spans 1,677->1,668, uploaded bytes
  168,936->168,424, and ranked excess rows 72->71. The target/matrix timing
  snapshots are 4.904x/2.172x and remain directional only. A Sol review added
  a scheduled-versus-serialized pixel regression and closed with no remaining
  findings. Renderer exact=1,409/diverges=0/gated=59, V2 floors 584/35, and
  both renderer-feature and workspace suites pass. Item 123 owns the measured
  two-pass ordinary-MSAA direct-resolve excess.

- 2026-07-16: Closed R4 item 121 by porting C++'s contiguous outer-curve and
  triangle batching for compatible plain interior fills. On
  `gm-batchedtriangulations-clockwise-atomic`, Rust passes fall 14->5 against
  C++ 5 and draws fall 13->4 against C++ 5; C++'s extra draw is its explicit
  initialize operation, while Rust uses an attachment clear. Path patches are
  exact at 56. A first same-pass candidate made a large interior fill differ
  across repeats by three pixels; preserving C++'s outer-to-interior barrier
  restores determinism. Fixed-matrix passes move 116->107, bind-group sets
  332->302, uploaded bytes 172,008->168,936, draws 187->178, and ranked excess
  rows 81->72. The target's light Rust/C++ snapshot is 1.382x. Renderer
  exact=1,409/diverges=0/gated=59, V2 floors 584/35, and the workspace suite
  pass. Item 122 owns the remaining MSAA 12->3 fill-draw merge.

- 2026-07-16: Closed R4 item 120 by admitting plain strokes to the existing
  flush-wide midpoint tessellation layout. On the target scene, passes fall
  42->23, uploaded bytes 54,696->13,224, and draws 41->22; C++ reports
  23/8,448/23. Across the fixed matrix, Rust passes move 154->116, bind groups
  created 133->67, bind-group sets 413->332, texture bindings 278->113,
  uploaded bytes 273,640->172,008, and GPU draws 220->187. The first broad
  eligibility rule regressed seven advanced-blend rows, proving loaded
  destination color is a semantic boundary; fencing that path restores all
  1,468 corpus outcomes. The four affected Rust directional frames sum to
  4.124 ms from 6.937 ms while their C++ controls remain 2.930 ms from
  3.029 ms. This is a light snapshot, not a cross-window load claim. Renderer
  exact=1,409/diverges=0/gated=59, V2 floors 584/35, and the workspace suite
  pass. Item 121 owns the refreshed `batchedtriangulations` pass/draw excess.

- 2026-07-16: Closed R4 item 119 by porting C++ WebGPU's binding-invalidation
  rule to direct MSAA paths. Seven fixed MSAA variants remove 141 redundant
  bind-group sets in total: `CubicStroke` 12->6, `OverStroke` 39->6,
  `batchedconvexpaths` 33->6, `batchedtriangulations` 15->6,
  `bevel180strokes` 63->6, `bug339297` 9->6, and `bug339297_as_clip` 13->7.
  C++ Dawn remains at 324 aggregate sets while Rust moves 554->413. The light
  aggregate snapshot moves 3.224x->2.033x; it is not threshold evidence.
  Renderer exact=1,409/diverges=0/gated=59, V2 floors 584/35, and the full
  workspace suite pass. Item 120 owns the newly highest-ranked atomic upload
  byte excess.

- 2026-07-16: Closed R4 item 118 with a cross-backend work-counter oracle and
  complete C++ performance-mechanism inventory. `make perf-counter-compare`
  builds counter-enabled runners in an isolated target, validates all 16 fixed
  scene/mode variants, and emits ranked JSON/Markdown artifacts. Aggregate
  structural work is C++/Rust: encoders 16/16, submissions 16/16, passes
  91/154, bind-group sets 324/554, uploaded bytes 156,832/273,640, and GPU
  draws 158/220. The light one-frame sum is 3.224x and remains directional.
  The top deterministic excess is `gm-bevel180strokes-msaa` bind-group sets at
  5/63; item 119 ports C++'s binding invalidation rule.

- 2026-07-16: Closed R4 item 117 with C++'s flush-wide MSAA tessellation
  lifetime. A first full-corpus run exposed sparse authored contour IDs after
  empty contours; preserving those slots restored `gm-emptystroke-msaa` to
  exact and added a focused regression. Independent Metal exports reproduce
  2,641 to 551 total encoder rows and 2,200 to 110 tessellation rows, exactly
  twenty passes/frame to one. Light old/current snapshots improve to
  0.9236x/0.9140x, and the same-tier C++ Dawn/Rust wgpu report is now 4.1442x.
  The measurement rule now makes evidence proportional to uncertainty: this
  objective structural win does not require exhaustive A-B-B-A confirmation.
  Renderer exact=1,409/diverges=0/gated=59, V2 floors 584/35, and the workspace
  suite pass. `make golden-compare` and `make scripted-golden-compare` share the
  `target/debug/rust-golden-runner` output path and therefore run serially.
  Item 118 turns deterministic counter parity and the C++
  performance-mechanism inventory into the next discovery step.

- 2026-07-16: Closed R4 item 115 with feature-gated command-encoding
  attribution and a C++-aligned generic-atomic flush lifetime. Per-batch dummy
  and sampler work was only about 66 microseconds; the actual mismatch was a
  full initialize/path/resolve cycle per intersection-board group. Explicit
  group barriers now survive inside one preparation and final resolve. Two
  fixed reports improve to 0.7978x/0.8164x, and the load-matched A-B-B-A trace
  cuts encoder rows from 11,221 to 4,951 while both candidate frame medians beat
  their bracket baselines. The renderer corpus remains
  exact=1,409/diverges=0/gated=59; the V2 floors remain 584/35 and the workspace
  suite passes. The current C++/Rust report is 5.0537x; item 116 profiles the
  remaining generic tessellation and MSAA cadence before another optimization.

- 2026-07-16: Closed R4 item 114 with load-controlled attribution and a
  persistent atomic-backing implementation. The initial host snapshot showed
  concurrent indexing/build activity and was excluded from acceptance. In the
  accepted A-B-B-A Metal sequence, host idle stayed between 75% and 82% while
  baseline `PendingWrites` measured 27.532/28.067 ms per frame and the two
  candidate captures measured 2.899/2.897 ms. Two fixed alternating reports
  improve aggregate frame time to 0.2913x and 0.2908x; untouched MSAA controls
  stay near 1.0. Feature-gated upload telemetry reports just 1,040 written
  bytes for one draw and 20,496 for twenty, while Time Profiler locates the
  former long pole in per-frame atomic backing allocation and zeroing. Three
  guarded persistent slots now grow on demand and clear only active ranges in
  encoder order. The post-change profile reduces initialized-buffer samples
  from 3,438/4,030 to 27/185. Renderer exact=1,409/diverges=0/gated=59, both V2
  floors pass at 584/35, and the full workspace suite passes. Item 115 starts
  from the newly dominant command-encoding samples.

- 2026-07-15: Accepted R4 item 113 after correcting the performance control.
  A one-off, non-interleaved trace had falsely suggested the unified upload
  arena worsened `PendingWrites` by 25.65%. Full-request A-B-B-A captures show
  the interval is neutral at 1.006x while frame medians improve in both paired
  brackets. Three fixed alternating reports improve aggregate time to 0.9605x,
  0.9826x, and 0.9797x; a targeted A-B-B-A also clears the exact-final-binary
  report's lone minimum outlier. The accepted ring
  rotates three guarded slots, packs exact aligned slices into one union-usage
  arena, and writes each populated page once. Renderer exact=1,409/diverges=0/
  gated=59, V2 floors 584/35, and the workspace suite all pass. Item 114 first
  attributes the remaining pending-write work under the new measurement fence.

- 2026-07-15: Closed R4 items 111-112. The bounded 256-texture/64 MiB pool was
  rejected at 1.0727x for `bug5099` and 1.0949x for `bevel180strokes` with no
  trace improvement. Source inspection then identified per-group queue submits
  as the real pending-write flush. Bounded independent-group coalescing reduces
  `PendingWrites` from 19.88 to 1.00/frame and the fixed 16-variant aggregate
  from 162.237 ms to 138.841 ms (0.8558x), with unchanged structural counters.
  `make renderer-golden` passes at exact=1,409/diverges=0/gated=59/total=1,468;
  the normal and scripted V2 floors pass at 584 and 35 exact segments, and
  `cargo test --workspace` passes. Item 113 ports C++'s three-buffer upload
  rings to attack the remaining pending-write encoder cost.

- 2026-07-15: Closed R4 queue item 110 as a measured rejection. The strongest
  shared-buffer candidate retained 19.85 pending-write submissions/frame and
  changed `bevel180strokes` from 69.644 ms to 70.463 ms; `bug5099` changed from
  3.331 ms to 5.016 ms. The source tree is restored exactly, and queue item 111
  targets persistent per-draw texture reuse after GPU completion.

- 2026-07-15: Closed R4 queue item 109 with paired one-draw/20-draw CPU and
  Metal captures. The measured Rust-only long pole is one wgpu pending-write
  flush per tessellated draw: 20-draw Rust uses 101 Metal command buffers and
  6.474 ms GPU time inside a 63.695 ms frame, while C++ uses one command buffer
  and 0.646 ms GPU time inside a 0.942 ms frame. Queue item 110 now adapts
  C++'s flush-wide upload-before-encode ordering; no renderer code, pixel,
  reference, or tolerance changed.

- 2026-07-15: Closed R4 queue item 108 as a measured rejection. Safe vertical
  tessellation packing preserved exact=1,409/diverges=0/gated=59, but lost
  22.25% aggregate against the immutable old-Rust runner and slowed every
  targeted clockwise scene. The implementation and its tests were removed.
  Queue item 109 now requires paired one-draw/20-draw CPU and GPU attribution
  before choosing another optimization site.

- 2026-07-15: Closed R4 queue item 107 as a measured rejection. The unchanged
  baseline runner at commit `762327cc` beats the borrowed/main pass-merge
  candidate by 23.95% aggregate across the fixed 16 variants; all eight
  clockwise scenes regress and MSAA is flat. The candidate still passed the
  complete 1,468-row pixel corpus, but was removed. Queue item 108 now targets
  the per-draw tessellation texture/pass multiplicity exposed by the same
  benchmark scenes.

- 2026-07-15: Wired release-only live performance runners through the pinned
  protocol and corrected the invalid native-Metal baseline to C++ Dawn on the
  same WebGPU/Metal layer as Rust. Seven alternating CubicStroke samples pass
  adapter and structural parity in both modes. The full 8-scene x 2-mode x
  seven-sample campaign completes all 112 runner invocations with no structural
  mismatch and reports 26.37x aggregate time, directly queuing clockwise
  render-pass batching as item 107.

- 2026-07-14: Added and hardened the R4 renderer performance report scaffold.
  It validates the fixed 16 scene/mode variants, emits deterministic JSON and
  Markdown reports, compares min-of-seven steady-state medians, enforces
  concrete adapter and structural parity, and writes diagnostic artifacts
  before a threshold failure. All 33 focused tests and manifest validation
  pass; no live benchmark runner or performance claim is included yet.

- 2026-07-14: Extended strict Dawn capture from 83 to 91 cases and promoted
  seven MSAA rows without changing tolerances. The 273 serial/four-job
  artifacts match, all 83 retained PNGs are unchanged, both gradient compiler
  gaps and the isolated `strokes_poly` edge residual have named gates, and the
  renderer ratchet reaches exact=760/diverges=0/gated=708. A fresh fuzz
  campaign passes, the semantic-trap audit closes, and queue item 80 ports full
  logical-flush rollover while reference capture continues independently. The
  workspace, 212-test enabled renderer suite, normal 584-segment floor, and
  scripted 35-segment floor all pass.

- 2026-07-14: Ported the dedicated MSAA stroke depth state and promoted both
  depth-write-gated overstroke rows without changing tolerances. A focused
  red/green GPU regression, 212 enabled renderer tests, the workspace, the
  1,468-row renderer corpus, and both V2 floors pass. The renderer metric is
  exact=753/diverges=0/gated=715; queue item 79 captures the next ten strict
  source-order Dawn references.

- 2026-07-14: Captured ten more provenance-bound C++ Dawn MSAA references
  with an 83-case registry. Serial and four-job outputs match across all 249
  artifacts, all 73 retained PNGs are unchanged, and eight probes promote
  under unchanged `2/32`. Transparent and advanced-blend overstroke narrow to
  C++'s missing dedicated MSAA stroke depth-write state. The renderer ratchet
  advances to exact=751/diverges=0/gated=717; workspace, renderer, and both V2
  floors pass. Queue item 78 ports that state.

- 2026-07-10: Repaired the release-rename regression in
  `nuxie-renderer-ffi/build.rs`; native Metal replay builds again.
- 2026-07-10: Landed typed stream parsing/replay, encoded image payloads, pixel
  comparison with side-by-side heatmaps, `corpus-r.toml`, stub-failure ratchet,
  and Phase R CI.
- 2026-07-10: Landed `nuxie-renderer` on wgpu 30 with retained paths/paints,
  state capture, solid polygon rendering, 4x MSAA resolve, and readback. First
  GM and `.riv` fixtures are pixel-exact against C++ Metal.
- 2026-07-11: Completed R0 corpus capture: 108 renderer-interface GMs and 294
  valid `.riv` files produced 731 references and 1,465 mode entries. One known
  invalid `.riv` and 33 direct-context/ORE GM source files remain named-gated.
- 2026-07-11: Began R2 with a reproducible upstream shader pipeline. All 50
  generated WebGPU WGSL modules validate through naga. Ported the `gpu.hpp`
  host upload records, enum encodings, packed tessellation fields, color
  swizzles, and blend IDs with C++ ABI size/offset tests.
- 2026-07-11: Ported the first `draw.cpp` path-preparation slice: transformed
  verb iteration, line/quad/cubic normalization, Wang parametric segment
  counts, closed-contour normalization, and concave triangulation. The MSAA
  bootstrap now uses stencil-then-cover for non-zero and even-odd compound
  fills. The `oval` probe's topology is correct; its remaining 3,136-pixel,
  max-delta-73 difference is confined to flattened cubic edge coverage, so it
  stays gated pending analytic patches.
- 2026-07-11: Ported `gpu.cpp`'s immutable analytic patch-buffer generator,
  including mirrored border diagonals and middle-out fan indices. Its 269
  vertices and 441 indices are invariant-tested and now uploaded once per wgpu
  context for the forthcoming tessellation/draw passes.
- 2026-07-11: Instantiated and executed the upstream `tessellate.glsl` WebGPU
  pipeline through wgpu. A submitted smoke test binds real flush/path/contour
  storage and a `TessVertexSpan`, renders through the canonical 12-index span
  topology, and completes against an `rgba32uint` tessellation target.
- 2026-07-11: Ported fill tessellation layout from `LogicalFlush`: local
  line/quad/cubic normalization, device-space Wang counts, contour records,
  the leading invalid eight-vertex range, and per-path eight-vertex padding.
  The first-light triangle lays out one midpoint-fan patch at base instance 1.
- 2026-07-11: Wired the generated `draw_msaa_path` shaders to the tessellation
  texture and immutable patch buffers. Corrected WebGPU viewport orientation,
  one-polar-endpoint fill counts, per-contour pre-padding, and absolute contour
  starts against C++ source. The first-light triangle now reproduces the known
  MSAA-vs-atomic edge delta exactly (112 pixels, max delta 43); the active
  corpus remains exact=3/diverges=0. Compound fills stay on the prior correct
  stencil fallback until the upstream MSAA stencil/cover pass lands.
- 2026-07-11: Wired the generated clockwise-atomic path/resolve shaders with
  tiled storage buffers and the C++ clear/path ID convention. Threaded render
  mode through `corpus-r` and `renderer-replay` so MSAA and atomic entries no
  longer execute the same backend mode. The atomic triangle passes at 30
  differing edge pixels within its 32-pixel cross-backend budget, moving the
  metric to exact=4 with no divergence.
- 2026-07-11: Threaded clockwise-atomic across ordered solid-fill draws by
  clearing once and resolving each fresh tiled coverage allocation with
  premultiplied SrcOver. The four overlapping translucent draws in `gm:rect`
  pass at 4 differing pixels within budget, moving the metric to exact=5.
- 2026-07-11: Swept the solid-fill GM slice. Clockwise-atomic promoted
  `batchedconvexpaths` (30 pixels, max delta 19) and `path_skbug_11886` (2
  pixels), moving exact to 7. Named probes still outside tolerance:
  `batchedtriangulations` 2,856 pixels, `convex_lineonly_ths` 8,792,
  `rotatedcubicpath` 301. Their MSAA variants also remain gated.
- 2026-07-11: Ported atomic reverse-then-forward tessellation: reflected spans,
  doubled patch allocation, forward-half contour starts, and back-face culling.
  The triangle became pixel-exact; `rotatedcubicpath` dropped to 2 pixels and
  `convex_lineonly_ths` to 14, promoting both and moving exact to 9. The prior
  solid-fill passes improved to 0-2 pixels. `batchedtriangulations` remains a
  named interior-triangulation gap at 2,136 pixels.
- 2026-07-11: Ported clockwise-atomic interior triangulation for large fills:
  the C++ area/verb selector, fixed outer-curve patches, Wang-based cubic
  chopping, excess-segment culling, weighted interior triangles, and generated
  atomic interior shaders. Negating triangulator winding to Rive's coverage
  convention reduced `batchedtriangulations` from 2,136 differing pixels (max
  delta 48) to 17 (max delta 9), promoting it and moving exact to 10.
- 2026-07-11: Began stroke geometry with line-only contours, degenerate-line
  removal, C++ cap emulation, miter/round/bevel join records, polar budgets,
  stroke paint encoding, and a forward-only atomic pipeline state.
  `zerolinestroke` is pixel-exact in clockwise-atomic mode, moving exact to 11;
  its MSAA entry remains gated at 204 differing pixels pending MSAA stroke
  state convergence, and cubic strokes remain explicitly rejected by this
  builder until cusp/chop handling lands.
- 2026-07-11: Extended stroke preparation to analytic cubic and quad records,
  including C++ tangent fallback, Wang parametric counts, tangent-rotation
  polar counts, and original-verb cap/join ownership. `CubicStroke` and
  `zero_control_stroke` both pass clockwise-atomic at 0 differing pixels (max
  delta 1), moving exact to 13. The C++ convex/180-degree detector rejects
  cubics requiring a chop until straddled cusp and inflection chopping lands.
- 2026-07-11: Ported convex/180-degree cubic chop emission, including sorted
  inflection/turnaround roots, internal one-segment joins, and C++-style cusp
  straddles with subpixel pivot cubics. A flat two-cusp structural test passes.
  No corpus entry was promoted in this slice: the replay rebuild was cancelled
  after unrelated system-wide compiler I/O repeatedly exhausted the disk;
  pixel probing remains required before changing the exact count.
- 2026-07-11: Ported C++ empty-stroke cap geometry. Open empty contours use
  their authored cap; closed empty contours map round joins to round caps,
  miter joins to square caps, and bevel joins to no geometry. Round and square
  cases emit the two opposed emulated-cap records expected by the analytic
  stroke pipeline. All 24 `nuxie-renderer` tests pass, including a focused
  record-layout test and the upstream GPU execution smoke test. Focused
  `emptystroke` replay produces the expected shape placement but remains gated
  at 1,320 differing pixels (max delta 81), concentrated on round-cap edge
  coverage. A sibling sweep proves `roundjoinstrokes` pixel-exact at zero
  differing pixels and promotes it, moving exact to 14. `widebuttcaps` remains
  gated at 5,004 differing pixels (max delta 254).
- 2026-07-11: Matched upstream `gpu.cpp`'s counterclockwise-face culling for
  forward stroke midpoint-fan patches by culling wgpu front faces after the
  port's viewport-orientation conversion. This removes the wrong-facing half
  of self-overlapping wide cubic strokes while preserving all prior stroke
  goldens. `widebuttcaps` moves from 5,004 differing pixels to zero and is
  promoted, moving exact to 15. `emptystroke` is unchanged at 1,320 differing
  pixels and remains the next isolated round-cap coverage gap.
- 2026-07-11: Closed `emptystroke` after proving its geometry independently of
  backend AA: binarizing both images at 50% coverage produces zero differing
  pixels, while the strict comparison's 1,320 differences are confined to
  subpixel edges across the GM's many tiny circles. The entry keeps the strict
  max-channel threshold of 2 and receives a bounded 1,400-pixel Metal-vs-wgpu
  allowance under Phase R's per-backend perceptual policy. It is promoted,
  moving exact to 16.
- 2026-07-11: Swept the next stroke stress cases. `bevel180strokes` is exact at
  zero differing pixels. `OverStroke` differs at 103 AA-edge pixels, while a
  50% coverage-mask comparison differs at only two pixels; it receives a
  bounded 128-pixel Metal-vs-wgpu allowance. Both are promoted, moving exact
  to 18. `lots_of_tess_spans_stroke` remains the next real source gap at
  749,360 differing pixels because Rust emits materially fewer concentric
  strokes, indicating missing span range/chunking behavior rather than AA.
- 2026-07-11: Ported C++ `TessellationWriter::pushTessellationSpans` row
  wrapping for forward stroke spans. Logical spans now map across 2,048-wide
  tessellation-texture rows, straddling spans are duplicated at the next row's
  negative edge, and texture height/uniforms grow from actual span rows.
  `lots_of_tess_spans_stroke` now renders all 49 radii and drops from 749,360
  to 375,640 differing pixels; its 25% coverage masks are pixel-identical, so
  the remaining gap is dense-overlap coverage magnitude rather than missing
  geometry. Exact remains 18 pending that separate accumulation slice.
- 2026-07-11: Ported the first `render_context.cpp` logical-flush behavior:
  atomic-eligible frame draws now use global monotonic path/contour IDs,
  shared path/paint/coverage/color buffers, per-path tessellation textures,
  fixed-function intermediate path resolves, and one final resolve. Existing
  fill, interior, and stroke probes remain exact. The dense stress comparison
  remains near 375k pixels because the oracle itself is mode-mismatched:
  `renderer-replay --backend ffi-metal --mode clockwise-atomic` is byte-exact
  with the checked default Metal reference because the FFI branch ignores
  `--mode`. Upstream Metal exposes `ContextOptions.disableFramebufferReads`
  for forcing atomic rendering; wire that through the harness before treating
  this GM as an algorithm verdict. Exact remains 18.
- 2026-07-11: Made native replay mode-correct. The FFI begin-frame API now
  accepts default, 4x MSAA, and clockwise-atomic modes; replay passes `--mode`
  through to C++ `FrameDescriptor.msaaSampleCount` or the
  `disableRasterOrdering + clockwiseFillOverride` pair. Forced C++
  clockwise-atomic differs from the old default Metal stress reference by 466
  pixels, while Rust still differs from the forced oracle by 374,732. A
  focused sweep finds the same subpixel coverage family in `strokes3` (42,778
  pixels), while `strokes_zoomed` and both tricky-cubic stroke GMs are exact.
  The next source gap is therefore thin-stroke coverage, not span placement or
  render mode. Exact remains 18.
- 2026-07-11: Closed the apparent `strokes3` thin-coverage gap by porting
  `RiveRenderer::drawPath` no-op culling. A zero-width stroke at the beginning
  of the stream had poisoned the frame-wide atomic eligibility check and sent
  every later draw through the fallback path. Culling empty paths, non-positive
  or NaN stroke widths, and NaN feather values before batching moves the Rust
  result from 42,778 raw differences at delta 128 to 2,054 at delta 1 against
  the checked-in Metal reference. Those differences are all below the existing
  channel tolerance, so `strokes3` promotes without widening its allowance and
  exact moves to 19. The remaining stroke target is the tessellation-span
  stress case.
- 2026-07-11: Closed the tessellation-span stress case by replacing the
  single-row GPU smoke test with a two-row readback oracle. It proved that
  logical tessellation row 0 was landing in texture row 1 under wgpu. Using a
  negative tessellation inverse viewport, matching the render-target
  orientation, restores every boundary texel. `lots_of_tess_spans_stroke`
  moves from 474,329 raw differences at delta 254 to differences bounded
  entirely by the existing delta-2 backend tolerance, so it promotes without
  an allowance change and exact moves to 20. Stroke geometry is complete; the
  next `draw.cpp` slice is feather geometry.
- 2026-07-11: Ported the first feather edge case by culling fill paths whose
  local control polygon is provably collinear. This covers the move-only,
  move-close, and zero-length-line variants in `emptyfeather` without
  classifying self-intersections or curved paths as empty. The GM's remaining
  144 pixels are confined to the red marker AA edges, so it promotes with the
  same bounded-edge policy used by `OverStroke`; exact moves to 21. Real
  feather convolution remains the next R2 target.
- 2026-07-11: Replaced the analytic pipelines' placeholder feather binding
  with the canonical 512x2 `R16Float` Gaussian lookup texture. The Rust port
  reproduces C++'s seven-sample integral, 32x inverse integral, finite
  float-to-half conversion, and both full table hashes byte-for-byte. The
  texture is retained once per renderer context and shared by MSAA and atomic
  draw bindings. Feather specialization remains disabled until its matching
  `draw.cpp` geometry lands; all 28 renderer tests pass and the corpus remains
  exact=21/diverges=0.
- 2026-07-11: Ported direct clockwise-atomic feathered-fill geometry from
  `draw.cpp`: implicit contour closure, stroke-style cubic chopping, capped
  polar budgets, six-or-more-segment feather joins, real contour midpoints,
  reverse-plus-forward tessellation, center-AA patches, and the canonical
  `paintFeather * 1.5` radius. The same builder records both radii and ordinary
  join flags for future feathered strokes. A binding audit also found and
  fixed the tessellation pass still sampling a 1x1 placeholder instead of the
  inverse Gaussian LUT; this changes `feather_ellipse` from faceted diamonds
  to smooth ellipses and drops its max delta from 230 to 53. Its remaining
  broad differences begin where C++ switches feathers at 32 device pixels to
  the quarter-resolution atlas. Compound feather fills now enter the direct
  path; feathered strokes remain runtime-gated until mixed direct/atlas draw
  partitioning lands. All 30 renderer tests pass and the corpus remains
  exact=21/diverges=0.
- 2026-07-11: Locked the direct-versus-atlas feather boundary to C++'s
  `find_atlas_feather_scale_factor`: a feather routes to the atlas at 32 or
  more device pixels (`paintFeather * 1.5 * matrixMaxScale`), and MSAA can
  force atlas routing regardless of radius. Boundary tests cover identity,
  scaled transforms, equality, and forced routing. Until the atlas pass lands,
  these draws correctly keep the frame out of the direct atomic path.
- 2026-07-11: Instantiated C++'s offscreen feather-mask pass with the generated
  `render_atlas` shaders. Fill masks render center-AA patches into `R16Float`
  with additive blending; stroke masks use border patches with max blending.
  The pass shares canonical path/paint/contour records, tessellation texture,
  patch buffers, feather LUT, and linear samplers. A submitted GPU readback
  test proves a real feathered rectangle leaves zero background and nonzero
  center coverage. Atlas blitting, packing, and frame-order integration remain
  the next checkpoint.
- 2026-07-11: Wired atlas masks through generated
  `atomic_draw_atlas_blit` shaders in monotonic draw order. Atomic bindings now
  carry atlas texture/sampler slot 11, mask rectangles use the canonical
  `TriangleVertex` path-ID encoding, and large fills retain direct fills' shared
  coverage/color buffers. A submitted large-feather oracle caught and locked
  two WebGPU orientation requirements: negative atlas inverse-viewport Y and
  clockwise atlas front faces, so scaled masks are both correctly located and
  positive. `feather_ellipse` now renders all atlas-routed rows instead of
  dropping them; its max delta is 179 pending C++ bounds/padding/packing and
  coverage convergence. All 32 renderer tests and the exact=21/diverges=0
  corpus gate pass.
- 2026-07-11: Replaced temporary full-target per-draw masks with one shared
  shelf-packed atlas. Fill bounds now match C++'s transformed control-point
  bounds plus feather radius and one AA pixel, intersect the viewport, reserve
  two pixels of padding, scissor each region, clear once, and load between
  mask batches. Tight bounds and transformed/scaled cases have CPU tests; the
  submitted mask oracle now uses a real 80-unit feather and requires positive
  half-float coverage at its scaled center. `feather_ellipse` remains max delta
  178, proving allocation was not its remaining coverage mismatch. A guarded
  feathered-stroke probe improved after atlas routing but still exposed direct
  border leakage and missing stroke/miter/cap outset, so runtime stroke enablement
  remains intentionally gated. All 33 renderer tests and exact=21/diverges=0
  corpus checks pass.
- 2026-07-11: Corrected atlas contour directions. C++ renders atlas fills with
  forward tessellation only, while direct atomic fills use reverse-plus-forward;
  the shared Rust builder had doubled both. A dedicated atlas builder and
  topology test now preserve one forward half for additive mask rendering.
  `feather_ellipse` drops from max delta 178 to 51; its `exp(0)` and `exp(1)`
  direct rows are max delta 1, while remaining error concentrates in near-cusp
  direct cells and broad cross-backend atlas filtering (atlas rows max 51, 22,
  33, and 25). `feather_shapes` remains max 116 and names corner/cusp geometry
  as separate work. All 34 renderer tests and exact=21/diverges=0 corpus gates
  pass; neither fixture is promoted by widening around broad residuals.
- 2026-07-11: Completed C++ path pixel-outset parity for feather atlas
  placement, including stroke radius, the 4x miter limit, square-cap `sqrt(2)`
  diagonal, feather radius, transformed axis outsets, and one AA pixel. Fill,
  bevel/butt, miter, and square-cap cases have exact bounds tests, and atlas
  stroke masks now name the canonical 48-index border count instead of a magic
  number. A guarded `feather_strokes` replay proved a single closed line square
  clean, while later cubic paths produce local-origin rays in both direct and
  bounded-atlas routes; the issue is therefore cubic stroke-mask/multi-draw
  bookkeeping, not atlas allocation. Runtime feathered strokes remain gated.
- 2026-07-11: Enabled feathered strokes through wgpu's C++-supported
  `alwaysFeatherToAtlas` policy. Atlas stroke pipelines now match C++ back-face
  culling, and CPU tessellation explicitly collapses exactly co-directional
  cubic joins to one segment, preventing smooth closure wedges from reaching
  the mask. The focused `feather_strokes` replay is structurally correct across
  all seven radii with no local-origin rays. It remains corpus-gated at
  1,550,127 differing pixels/max delta 255 because broad atlas filtering and
  low-radius direct-vs-atlas differences are not tolerance work; a classifier
  probe also shows direct feathered strokes still lose draws during atomic
  resolution. The runtime no longer rejects the feature, while promotion waits
  on coverage convergence.
- 2026-07-11: Added the ordered fallback-run compositor required to replace
  the all-or-nothing atomic frame gate without changing fallback AA. Resolved
  4x fallback textures can now blend into the main single-sample target with
  a full-screen triangle, nearest sampling, and premultiplied SrcOver. A
  submitted GPU readback test composites half-alpha premultiplied red over
  opaque blue and verifies `[128, 0, 127, 255]`, proving the pass blends rather
  than replaces. A rejected one-sample fallback probe regressed ratcheted
  `emptystroke` from 1,320/81 to 1,464/128 and was removed completely. Next,
  render contiguous fallback runs into transparent 4x targets and feed their
  resolves through this compositor between atomic runs.
- 2026-07-11: Wired whole-frame fallback through the ordered compositor as the
  parity proof for future per-run routing. Fallback draws now render over
  transparent into the existing 4x target, resolve into a sampled RGBA8
  texture, and premultiplied-SrcOver composite onto a separately cleared main
  target. The ratcheted `emptystroke` probe returns to zero pixels beyond its
  tolerance/max delta 81, proving the extra resolve/composite pass preserves
  the existing 4x analytic AA. Next, reuse this exact pass for each contiguous
  fallback run instead of only the all-fallback frame.
- 2026-07-11: Extracted the validated atomic frame body into a callable
  `encode_atomic_run(draws, clear_target, encoder)` unit without changing frame
  selection. Path/paint IDs, tessellation textures, feather atlas packing,
  shared coverage buffers, and draw ordering are now scoped to the supplied
  contiguous slice, and target clearing is explicit. This is the mechanical
  prerequisite for alternating atomic and resolved-fallback runs; the next
  slice extracts the matching fallback-run encoder and replaces the global
  `all()` gate with contiguous eligibility ranges.
- 2026-07-11: Replaced the global clockwise-atomic `all()` gate with ordered
  contiguous atomic and fallback runs. Each fallback run renders into a
  transparent 4x target, resolves, and composites between atomic runs; only the
  first run clears the destination. A submitted GPU test proves an
  atomic-background/fallback-middle/atomic-foreground sequence preserves all
  three layers and their draw order. All 38 renderer tests pass and the corpus
  remains exact=21/diverges=0. This routing changes the known `emptystroke`
  residual from 1,320 differing pixels/max delta 81 to 546/max delta 255: fewer
  pixels differ, but supported degenerate strokes now expose the already parked
  direct-stroke atomic resolution gap instead of inheriting whole-frame
  fallback output. Close that gap next; do not widen its corpus tolerance.
- 2026-07-11: Removed the invented always-atlas override for feathered strokes
  and restored C++ `PathDraw::SelectCoverageType` routing: direct coverage below
  the half-scale boundary, atlas coverage at and above it. The atlas stroke
  pipeline now also matches C++ WebGPU's explicit no-cull state. All 38
  renderer tests and exact=21/diverges=0 corpus gates pass; `emptystroke` stays
  unchanged at 546/255, while the focused `feather_strokes` mismatch improves
  from 1,550,127 to 1,523,053 pixels. A mode-correct C++ clockwise-atomic
  comparison and a one-draw reproduction isolate the remaining atlas defect:
  straight stroke edges render, but closed miter/bevel join coverage leaves
  hard corner cutouts even without packing or culling. Direct-only routing was
  rejected because large radii produce long-range join rays. Continue with the
  atlas join tessellation/coverage records; do not replace the atlas threshold.
- 2026-07-11: Added a mode-correct C++ clockwise-atomic first-light golden for
  a low-radius direct feathered stroke. Rust differs at 103 localized AA-edge
  pixels and passes the existing bounded 128-pixel backend allowance used by
  `OverStroke`; there is no shape or coverage-mask mismatch. This closes the
  routing verification finding from the two-axis review and moves the corpus
  to exact=22/diverges=0 without promoting the still-broken atlas stress case.
- 2026-07-11: Re-keyed renderer references by stream, frame, and mode and added
  a manifest validator that rejects cross-mode reference reuse. A hermetic C++
  Metal capture command regenerated all 19 active clockwise-atomic references.
  Upstream Metal explicitly leaves MSAA flush unimplemented, so the three
  previously exact MSAA rows are now harness-gated instead of comparing against
  default-mode images. Two large atomic fixtures need only channel delta 3,
  with 2 and 10 pixels above that threshold inside their existing 32-pixel
  budgets. The corrected ratchet is exact=19/diverges=0/gated=1,447.
- 2026-07-11: Ported C++ `RectanizerSkyline` with its exact placement trace and
  replaced shelf atlas packing. The packed texture uses occupied extent rather
  than vertical capacity, coordinates do not truncate to `i16`, and packing is
  bounded by `max_texture_dimension_2d`. Compact 328-region layouts fit at
  1900x900; oversized layouts fail as `RendererError::AtlasPacking` before
  texture creation. The focused and full renderer suites pass 11 and 69 tests.
- 2026-07-11: Ported `intersection_board.cpp` as a standalone checked module.
  An independent randomized model plus direct C++ contract cases cover strict
  edges, translated tiles, maximal groups, extreme rectangles, eight running
  lanes, overlap bits, and baseline transitions. Bounds/allocation failures are
  explicit; 19 focused and 69 full renderer tests pass. Render-batch integration
  remains a separate R2 slice.
- 2026-07-11: Rejected a no-op atlas culling change after both its regression
  and production behavior passed unchanged on the parent. A one-draw oracle
  confirmed that Metal final pixels cannot isolate WebGPU atlas behavior.
  The next atlas step is a C++ WebGPU R16 mask exporter and Rust mask comparator;
  no atlas coverage code changes until that fail-before oracle exists.
- 2026-07-11: Established and independently accepted the matching-backend
  C++ WebGPU R16 atlas-mask oracle. The fixed stroke produces a complete 48x48
  physical atlas with a production-observed 39x39 content region at (2,2), one
  stroke batch scissored to [0,0,39,39], and a canonical 4,628-byte artifact.
  Rust renders the same production placement and compares the full physical
  payload. The configured comparison now gives a trustworthy fail-before at
  (0,0): C++=0.01171875, Rust=0, support threshold=1/1024. Naga is pinned,
  malformed/tolerance/join sensitivity tests pass, and temporary C++/Dawn
  changes restore byte-for-byte. Diagnose this mask discrepancy next; do not
  change atlas coverage without making the configured oracle pass.
- 2026-07-11: Set each atlas mask pass viewport from the complete packed logical
  extent while retaining the physical texture size and per-batch scissor. The
  fixed oracle improves comparator mismatches 1,448 -> 640, exact differing
  pixels 1,521 -> 643, and mean absolute error 0.05800 -> 0.02841. The first
  mismatch remains (0,0), so patch/contour/tessellation inputs are the next
  boundary; tolerances remain unchanged.
- 2026-07-11: Added an independently accepted C++/Rust atlas-input oracle for
  the production stroke batch range, contour records, and complete live
  RGBA32Uint tessellation texture. The fixed fixture first diverges at the
  batch range: C++ submits basePatch=1/patchCount=5 while Rust submits 1/3.
  With only that field normalized for diagnosis, the contour matches and the
  next failure is tessellation texel (10,0) channel 2. This moves the remaining
  mask defect upstream of atlas rasterization into stroke tessellation; fix the
  patch-count/data generation rather than adjusting mask tolerances.
- 2026-07-11: Closed the fixed atlas-stroke parity chain. Rust now applies
  C++'s effective round join/cap style to every feathered stroke, uses the
  upstream fast-acos round budget, and emits both midpoint-to-outer alignment
  padding and the final shader sentinel in the tessellation texture. The
  C++/Rust batch range, contour record, full RGBA32Uint tessellation texture,
  and final R16 atlas mask all compare exactly. Closed/open, double-sided,
  interior, and row-wrap tests preserve logical patch counts while covering
  the physical padding layout; no tolerance changed.
- 2026-07-11: Extended the paired C++ WebGPU oracle through final RGBA8 MSAA
  atlas blitting. The same submitted frame now exports versioned input,
  physical R16 mask, and 64x64 final-target artifacts; inputs and mask remain
  exact. A draw-schedule assertion prevents comparing this MSAA output to an
  atomic Rust path again. Matching Rust MSAA currently differs across all
  4,096 pixels with max delta 80, a named R2 failure. For the primary path, a
  new mode-correct native Metal clockwise-atomic atlas-feather stream differs
  at only 106 pixels/max delta 1, passes the existing 2/128 backend budget, and
  is promoted. Porting C++'s 125% physical atlas growth and feature-scoped
  default dither drops native `feather_strokes` from 1,411,260 to 229,617
  differing pixels (84%) while moving the ratchet to exact=20/diverges=0. The
  earlier 940/max-delta-3 number mixed C++ MSAA with Rust atomic output and is
  explicitly invalidated.
- 2026-07-11: Made `generate-corpus-r` preserve existing generated entry blocks
  by identity. Status, tolerances, references, and gate diagnostics now survive
  regeneration byte-for-byte; a regression test covers an exact promoted row.
- 2026-07-11: Promoted the full clockwise-atomic `feather_strokes` stress GM
  after a draw/radius bisection proved backend variance rather than missing
  geometry. The seven radius rows increase monotonically from 745/delta-1 to
  126,772/delta-7 as huge feather fields overlap; every isolated largest-radius
  shape stays at max delta 2. Across the full 3.6M-pixel frame, normalized RMSE
  is 0.001408 and 9,577 pixels exceed channel delta 2. The entry therefore keeps
  delta 2 with a bounded 16,384-pixel overlap budget. The ratchet advances to
  exact=21/diverges=0 without changing any renderer behavior.
- 2026-07-11: Ported `RiveRenderPath::makeSoftenedCopyForFeathering` for
  feathered fills, including convex/cusp preparation and uniform tangent-
  rotation chops. A paired C++ WebGPU circle oracle now matches Rust's 34-patch
  topology, contour and packed fields exactly, permits only one ULP across 44
  scalar-versus-SIMD XY values, and matches the R16 atlas mask. The full native
  clockwise-atomic `feather_shapes` GM fell from 1,583,729 pixels/max delta 117
  to 458,194/max delta 11. Five of six isolated largest-radius shapes stay at
  max delta 2; only the self-intersecting cusp reaches delta 3. The 12,427 full-
  frame pixels above delta 2 occur under overlapping huge feather fields and
  pass the existing bounded 16,384-pixel backend budget, advancing the ratchet
  to exact=22/diverges=0.
- 2026-07-11: Audited the remaining feather GMs after fill softening and
  promoted two mode-correct native Metal comparisons. `feather_ellipse` has
  6,476 full-frame pixels above delta 2/max delta 9; each isolated largest-
  radius nondegenerate ellipse stays at max delta 2, while the zero-width
  ellipse is exactly blank in both renderers, so the full overlap keeps a
  bounded 8,192-pixel budget. `emptystrokefeather` has only 74 pixels above
  delta 2/max delta 11 and passes a 128-pixel budget while all degenerate
  strokes remain culled. `feather_cusp` and `feather_polyshapes` still show
  max-delta-255 geometry failures and remain the next implementation boundary;
  `feather_roundcorner` remains clip-gated. The ratchet advances to
  exact=24/diverges=0.
- 2026-07-11: Preserved C++'s GPU contour records for empty fill contours.
  `feather_cusp` begins with duplicate moves; Rust previously skipped the empty
  contour but left the drawable contour tagged as ID 2, making the shader read
  beyond its one-record contour buffer and collapsing the severe cusp. A paired
  C++ WebGPU oracle now covers the exact severe cell (duplicate moves,
  `133.635864/-33.6358566` controls, feather 1, scale 1.46300006): both contour
  records, the 20-patch range, packed topology, and complete tessellation
  texture match, with only bounded scalar/GPU float differences. The full GM
  falls from roughly 1.7M raw mismatches to 13,239 pixels beyond delta 2; the
  severe isolated cell falls 656 -> 558 and restores its body, but retains a
  small max-255 cusp-tip lobe downstream of tessellation. C++ Dawn cannot run
  the specialized clockwise-atomic mode (forcing it crashes), so native Metal
  remains the final-pixel oracle and the lobe stays gated. Exact remains 24;
  continue with `feather_polyshapes` per the divergence budget. The required
  workspace floor also exposed a pre-existing stale render-stream assertion;
  updating its expected `decodeImage` payload to include `data=010203` restores
  the full V2 gate without changing runtime behavior.
- 2026-07-11: Ported C++ `pushDoubleSidedTessellationSpans` row wrapping.
  Rust previously relocated already row-local forward spans and assigned every
  mirrored span to row zero, corrupting direct feather fills once one contour's
  half-tessellation crossed the 2,048-texel boundary. The polygonal shark in
  `feather_polyshapes` exposed the defect while atlas rendering remained exact.
  All 42 cells are now individually exact at max channel delta 2; the composite
  has 11,677 pixels beyond delta 2/max delta 11 only where individually exact
  translucent feathers overlap, and passes the existing bounded 16,384-pixel
  overlap budget. A direct WebGPU input oracle also matches the 786-patch,
  one-contour, four-live-row topology and payload; its 125%-growth fifth row is
  zero. Dawn and wgpu classify 320 otherwise-identical feather-join texels with
  opposite LEFT/RIGHT bits, a backend equivalence guarded narrowly by the
  comparator and superseded by exact isolated native-Metal pixels. The ratchet
  advances to exact=25/diverges=0.
- 2026-07-11: Ported C++ `RiveRenderer::IsAABB`/`clipRectImpl` through the
  shader contract. Clip rectangles now inherit through save/restore, intersect
  in compatible matrix spaces, cull empty clips, set
  `PAINT_FLAG_HAS_CLIP_RECT`, and upload the fragment-to-normalized-rect matrix
  plus inverse-fwidth AA data. `feather_corner` and `feather_roundcorner` now
  render instead of returning `Unsupported("clip paths")`; all 84 isolated
  clipped cells are exact at max channel delta 2. Their overlapping composites
  have 3,367/max12 and 4,495/max11 differences and pass bounded 8,192-pixel
  backend budgets. The ratchet advances to exact=27/diverges=0; non-rectangular
  clip stacks remain explicitly unsupported.
- 2026-07-11: Swept the remaining axis-aligned clip GMs after the clip-rect
  port. `cliprectintersections` (45 draws), `gamma_correction_clip` (2), and
  `strokes_poly` (25) are exact when isolated; `cliprects` has 15/18 exact
  draws and three bounded AA-only cells. Their composites pass focused budgets
  of 1,024, 8, 128, and 2,048 pixels respectively without changing max channel
  delta 2. The ratchet advances to exact=31/diverges=0. `strokes_round` remains
  gated at 34/max83 pending a separate hard-edge diagnosis; cubic clip GMs
  retain their pre-existing geometry failures.
- 2026-07-11: Landed the first arbitrary-path clip tracer bullet. Atomic
  pipelines now enable the generated clipping specialization, bind the packed
  clip storage buffer, encode C++-compatible
  replacement/parent clip IDs, and emit a real `clipUpdate` draw before clipped
  content. A GPU triangle-clip test passes, and the first one-clip
  `parallelclips` cell is structurally correct at 15 pixels beyond delta 2/max
  delta 18 versus native Metal.
- 2026-07-11: Ported arbitrary clip stacks, save/restore stack-height reuse,
  and sequential parent/replacement clip IDs. C++ clockwise-atomic intersects
  nested clips by drawing inverse geometry with fixed-function `min` blending;
  Rust's generated atomic shader writes a packed clip storage buffer directly,
  so it reaches the same intersection by drawing each inner path against its
  parent ID. A two-level GPU intersection test passes. All 49 isolated
  `parallelclips` cells have the same 6-or-15 edge pixels beyond delta 2 as
  their single-clip counterparts, proving nesting adds no divergence; the full
  GM is promoted at 518 pixels/max delta 21 and advances the ratchet to
  exact=32/diverges=0. Continue with update reuse across repeated clipped draws
  and clip-content bounds before treating arbitrary clipping as complete.
- 2026-07-11: Swept every gated clockwise-atomic clipping entry after the
  nested-stack port. Fixed an eligibility/preparation mismatch where a large
  clip passed midpoint-fan validation but panicked when optional interior
  triangulation failed; it now falls back to the validated tessellation and
  has a direct regression test. Promoted 14 entries: `clippedcubic`,
  `clippedcubic2`, `path_stroke_clip_crbug1070835`, `artboardclipping`, all
  five `circle_clips` frames, and all five `clip_tests` frames. The
  `clippedcubic2` reference is structurally identical: 144 pixels differ over
  235,625 pixels, every difference is at most one channel level, and the
  manifest allows zero pixels above that delta. The
  ratchet advances to exact=46/diverges=0/gated=1,421. Large clipped paths,
  negative interior triangles, clipped gradient fallback, and images remain
  named algorithm gates rather than tolerance promotions.
- 2026-07-12: Ported C++ `gr_triangulator.cpp` and
  `GrInnerFanTriangulator` as a stable-index mesh: coincident/intersection
  simplification, winding-preserving edge splits, monotone decomposition,
  weighted face emission, and grout are integrated into multi-contour interior
  tessellation. Two direct C++ WebGPU sub-oracles prove preparation parity:
  the 100-contour grid matches all 7,500 TriangleVertex records, while the exact
  9-cubic flower plus 4-cubic oval matches both contour records, all 108
  TriangleVertex records, and every texel of its 2048x1 RGBA32Uint tessellation
  texture bit-for-bit. A provisional borrowed-coverage hybrid was rejected
  after proving atomics and clockwise-atomic coverage encodings cannot be
  mixed. `make renderer-golden` remains exact=46/diverges=0/gated=1,421; the
  next R2 slice is the dedicated clockwise-atomic shader/scheduling/allocation
  family, not further geometry work on these cases.
- 2026-07-12: Generated the upstream clockwise-atomic path/interior main and
  borrowed-coverage WGSL modules through GLSL -> SPIR-V -> naga and wired them
  as an isolated wgpu pipeline family. Ported C++'s per-path visible-bounds
  allocator (2px padding, 32x32 tiling, monotonic offsets) and global
  borrowed-before-main pass schedule. A 640x640 multi-contour GPU proof renders
  interior and nested-winding pixels correctly; `batchedtriangulations` stays
  within tolerance at 18 pixels, and the renderer ratchet remains
  exact=46/diverges=0/gated=1,421. True clip rendering still requires a
  sampled-input plus fixed-function `plus`/`min` attachment translation;
  storage-buffer PLS writes are not a semantic substitute.
- 2026-07-12: Completed the clockwise-atomic clip plane. Dedicated upstream
  outer/nested clip fragments render to an RGBA8 attachment with `plus`/`min`
  blending, while a checked-in upstream wrapper samples that attachment for
  clipped path and interior draws. Corrected WebGPU borrowed-face culling,
  threaded the real maximum path ID, and ported nested inverse-path creation.
  `largeclippedpath_clockwise_nested` improved from 145,064 differing pixels
  to 593, and both promoted large-clockwise entries have pixel-identical 50%
  coverage masks versus native Metal. The renderer ratchet advances to
  exact=48/diverges=0/gated=1,419; direct C++ preparation oracles, all 122
  active renderer unit tests, both V2 floors (584 and 35 exact segments), and
  the full workspace pass.
- 2026-07-12: Captured fresh forced-CWA C++ references for the winding and
  even-odd large-path variants after Terra reconnaissance and Sol review.
  Winding and clockwise references are byte-identical; even-odd uses different
  authored geometry but the same effective clockwise rule. All four Rust
  comparisons have the already-proven 593 boundary pixels/max 128 and
  pixel-identical 50% coverage masks, so they inherit the bounded 640-pixel,
  delta-2 allowance and advance the ratchet to exact=52/diverges=0/gated=1,415.
  The adjacent negative-interior probe remains a real geometry/coverage gap:
  16,845 pixels unclipped and 181,923 as a clip, both max delta 255.
- 2026-07-12: Ported C++'s `forwardThenReverse` physical tessellation layout
  for negative clockwise coverage and counterclockwise face culling for clip
  path/interior passes. Regression tests pin the C++ contour indices (493
  normal, 17 mirrored) and three formerly missing nested-clip pixels. The
  unclipped negative-interior GM improves 16,845 -> 1,040 pixels and advances
  the ratchet to exact=53/diverges=0/gated=1,414. A Terra oracle lane remains
  isolated and unmerged after Sol review found that its first capture modeled
  an opaque standalone draw instead of the real borrowed/main split; its
  forced-CWA Dawn amendment then failed binding validation. Continue linearly
  with a narrow mirrored inverse-clip oracle rather than merging that lane.
- 2026-07-12: Sol reviewed two read-only Terra scouts against direct probes for
  the mirrored inverse clip. The first found real source differences in parent
  clip bounds and fallback fan direction, but applying tight inverse bounds did
  not move the 166,809-pixel result; the fallback is not active in this GM. The
  second correctly ruled out coverage initialization and front-face mapping,
  but its reported shader mismatch was a temporary `abs` diagnostic and was
  rejected. A determinant-paired preparation probe then matched contour/face
  counts, face orientation, and coverage ranges. The next useful evidence is a
  borrowed/main coverage-buffer capture; all diagnostic code was reverted.
- 2026-07-12: Added opt-in CWA storage-buffer and clip-attachment snapshots at
  the borrowed/main boundary. Positive and mirrored nested clips are identical
  at all captured stages, proving the clip was correct; the following clipped
  rectangle was blank because midpoint-fan double-sided preparation ignored
  C++ `forwardThenReverse` plus `NEGATE_PATH_FILL_COVERAGE_FLAG` semantics.
  Porting that shared direction rule restores the mirrored draw and advances
  the ratchet to exact=54/diverges=0/gated=1,413. A Terra scout confirmed native
  Metal has no executable CWA storage-buffer mode, so implementing an entire
  backend solely for a redundant C++ buffer capture was rejected; native final
   pixels remain the cross-implementation oracle.
- 2026-07-12: A read-only Terra sweep measured ten basic gated CWA fills after
  the mirrored fallback fix. `convexpaths` exposed the highest-priority result:
  a pre-frame panic from packing global tessellation locations into signed
  16-bit row-local fields. Porting C++'s existing forward-span row wrapping
  removes the panic and leaves only 43 pixels beyond delta 2/max 103 across
  1.32M pixels, promoting the entry and advancing the ratchet to
  exact=55/diverges=0/gated=1,412. `pathfill` is the nearest next candidate at
  253 pixels beyond delta 2; the remaining eight have named winding/interior
  geometry gaps from 4,578 to 32,596 pixels.
- 2026-07-12: Promoted `pathfill` after a 50%-support/connected-component audit
  localized all 253 pixels beyond delta 2 to sparse hard edges across its
  compound icon stress set. The ratchet advances to
  exact=56/diverges=0/gated=1,411 without renderer changes.
- 2026-07-12: Promoted `oval` by admitting small compound midpoint fans to the
  atomic path, separating midpoint-fan and outer-curve cull state, and porting
  C++'s counterclockwise-face cull to the clockwise-atomic main path. Two GPU
  regressions cover same-direction cubic union and opposite-direction holes;
  the residual is 109 sparse edge pixels and the ratchet advances to
  exact=57/diverges=0/gated=1,410.
- 2026-07-12: A fresh post-`oval` basic-fill scout measured
  `mutating_fill_rule` at 45 pixels beyond delta 2/max 11. `concavepaths`, the
  three `poly_*` variants, `cubicpath`, and `cubicclosepath` retain structural
  topology/primitive gaps from 4,052 to 16,169 pixels, so
  `mutating_fill_rule` is the next R2 target.
- 2026-07-12: Promoted `mutating_fill_rule` after an independent component and
  support audit localized all 45 residuals to four one-pixel circle edges.
  The ratchet advances to exact=58/diverges=0/gated=1,409; `concavepaths` is
  the next measured structural fill target at 4,052 pixels beyond delta 2.
- 2026-07-12: Routed only topologically complex fills through the true
  clockwise-atomic coverage pipeline. Prefix replay localized the first
  `concavepaths` failure to the self-intersecting bowtie; full CWA replay proved
  the upstream behavior, and run splitting retained the established ordinary
  fill path. `concavepaths` now has 9 pixels beyond delta 2/max 13 and
  `poly_clockwise` is pixel-exact, advancing the ratchet to
  exact=60/diverges=0/gated=1,407.
- 2026-07-12: Ported C++ coarse-area accumulation order after the isolated
  counterclockwise six-point polygon proved that equal opposite contours used
  the wrong floating-point tie-break in Rust. `poly_evenOdd` and `poly_nonZero`
  each fall from 9,121 structural pixels to 2 edge pixels/max 17, advancing
  the ratchet to exact=62/diverges=0/gated=1,405.
- 2026-07-12: Ported `RiveRenderPaint::thickness` absolute-value semantics.
  This restores the twelve one-pixel rectangle frames shared by `cubicpath`
  and `cubicclosepath`; both become pixel-exact, the basic-fill sweep closes,
  and the ratchet advances to exact=64/diverges=0/gated=1,403.
- 2026-07-12: A bounded Terra rescout measured ten gated fill/clip GMs.
  `bug5099`, `bug6083`, `bug615686`, `bug6987`, and `bug7792` have zero pixels
  beyond delta 2, while `beziers` has 17 isolated delta-4 edge pixels within
  its existing budget. Promoting all six advances the ratchet to
  exact=70/diverges=0/gated=1,397; the shared two-row `bug339297` family is
  next.
- 2026-07-12: Audited the `bug339297` pair with independent threshold masks
  and color histograms. Support is pixel-identical in clipped and unclipped
  forms, while 1,280 AA samples differ across two scanlines. The documented
  backend allowance promotes both entries and advances the ratchet to
  exact=72/diverges=0/gated=1,395; the hit-test readback failure is next.
- 2026-07-12: Added JPEG decode alongside PNG and captured a dedicated C++
  clockwise-atomic reference for `clipping_and_draw_order`. Both image draws,
  including the circular clip, are restored; the renderer ratchet advances to
  exact=77/diverges=0/gated=1,390. `ImageMeshDraw` is the next R2 image slice.
- 2026-07-12: Ported C++ `ImageMeshDraw` with snapshotted retained buffers and
  the generated fixed-color atomic mesh shaders. A GPU regression pins indexed
  position/UV sampling, and `tape` matches fresh C++ geometry and support under
  its bounded decoder/filter allowance. The ratchet advances to
  exact=78/diverges=0/gated=1,389; advanced image blending is next.
- 2026-07-12: Ported the C++ WebGPU non-fixed atomic color lifecycle and
  generated advanced image shaders. `gm-mesh` now renders every authored blend
  and is reclassified from algorithm work to its measured ICC decoder gate;
  the ratchet remains exact=78/diverges=0/gated=1,389 without widening a
  tolerance. Color-managed PNG decode is next.
- 2026-07-12: Added embedded ICC-to-sRGB conversion before PNG
  premultiplication and promoted `image`, `image_aa_border`, and `mesh` under
  their measured decoder/filter allowances. The ratchet advances to
  exact=81/diverges=0/gated=1,386; rescouting the larger image/mesh corpus is
  next.
- 2026-07-12: Replaced the self-imposed 2,048 texture cap with the adapter's
  supported limit. `superbowl` is promoted under its measured image-backend
  allowance; `jellyfish_test` now renders but remains gated on a mip-level
  oracle. The ratchet advances to exact=82/diverges=0/gated=1,385.
- 2026-07-12: Draw-prefix replay disproved the `jellyfish_test` mipmap gate and
  isolated the missing radial-gradient background. Ported generated color-ramp
  rendering, gradient paint data/transforms, and nearest mip selection; five
  gradient GMs plus `jellyfish_test` advance the ratchet to
  exact=88/diverges=0/gated=1,379, with stale image allowances tightened.
- 2026-07-12: Swept all 38 remaining gradient-bearing `.riv` entries, captured
  30 runnable C++ references, and promoted 11 under unchanged tolerances. The
  precise gradient epsilon/clamp semantics are pinned; the ratchet advances to
  exact=99/diverges=0/gated=1,368 and `new_text` is the next residual.
- 2026-07-12: Classified `new_text` through draw-prefix, connected-component,
  support-mask, and solid-paint controls. Its 44 sparse compound-text edge
  pixels fit a bounded 48-pixel backend allowance; the ratchet advances to
  exact=100/diverges=0/gated=1,367 and `ai_assitant` is next.
- 2026-07-12: Classified `ai_assitant` through paired-draw prefixes,
  connected-component/support masks, and a full solid-paint control. Its 341
  almost entirely singleton stroke-edge pixels fit a bounded 384-pixel backend
  allowance; the ratchet advances to exact=101/diverges=0/gated=1,366 and
  `db_health_tracker` is next.
- 2026-07-12: Classified `db_health_tracker` through all 473 draw prefixes and
  connected components. Draws 1-430 are exact; its 1,071 residuals accumulate
  only across late text-outline edges and fit a bounded 1,152-pixel backend
  allowance. The ratchet advances to exact=102/diverges=0/gated=1,365 and
  `off_road_car` is next.
- 2026-07-12: Ported C++'s unique clip-generation IDs, removing stale root-clip
  coverage from all five identical `off_road_car` samples. The post-fix 1,862
  thin edge pixels fit a bounded 2,048-pixel backend allowance; the ratchet
  advances to exact=107/diverges=0/gated=1,360 and `joel_signed` is next.
- 2026-07-12: Routed clip-rect compound fills through clip-rect-specialized
  clockwise pipelines, promoted all five `joel_signed` frames, and advanced the
  renderer ratchet to exact=112/diverges=0/gated=1,355; `juice` is next.
- 2026-07-12: Ported shader-based advanced blending into the clockwise-atomic
  fill path, promoted all five `juice` frames, and advanced the renderer
  ratchet to exact=117/diverges=0/gated=1,350; `bad_skin` is next.
- 2026-07-12: Ordered generic-atomic outer/interior passes, added the exact
  `bad_skin` C++ preparation oracle, promoted its stable bounded residual, and
  advanced the renderer ratchet to exact=118/diverges=0/gated=1,349; the
  matching WebGPU MSAA final-blit oracle is next.
- 2026-07-12: Ported top-level MSAA `clipReset` for changing outer non-zero
  atlas clips; the nine-batch C++ frame is pixel-exact, all gates stay green at
  exact=118/diverges=0/gated=1,349, and nested clip intersection is next.
- 2026-07-12: Ported nested non-zero MSAA atlas clipping with exact C++ winding,
  intersection-reset, incremental-stack, and full-frame oracle parity; all
  gates remain green at exact=118/diverges=0/gated=1,349, and alternate clip
  fill rules are next.
- 2026-07-12: Ported alternate even-odd and clockwise MSAA atlas clip fills.
  Filled Dawn fixtures distinguish parity holes and opposite-winding rejection;
  commit `44bf47ea` keeps all gates green at
  exact=118/diverges=0/gated=1,349.
- 2026-07-12: Ported MSAA atlas destination-copy shader blending for solid
  feathered draws, including all 15 advanced modes, repeated bounded copies,
  attachment preservation, path clipping, and an exact C++ Dawn frame. The
  renderer ratchet remains exact=118/diverges=0/gated=1,349.
- 2026-07-13: Ported determinant-aware direct and atlas feather contour
  directions, promoted both mirrored feather-text GMs, and advanced the
  renderer ratchet to exact=146/diverges=0/gated=1,321.
- 2026-07-13: Isolated `interleavedfeather` to a ColorBurn-sensitive atomic
  intermediate-precision discontinuity, rejected and reverted destination
  texture and f16 color-plane experiments, and parked the case pending a C++
  color-plane suboracle or backend-matched reference. Promoted the independently
  verified `overstroke_blendmodes` reference under its unchanged 2/32 contract;
  the ratchet is exact=147/diverges=0/gated=1,320 and `zeroPath` is next.
- 2026-07-13: Pruned fully coincident cubics in stroke/feather preparation,
  matching C++ behavior and restoring `zeroPath` round/square caps.
  Fresh native Metal comparison passes the unchanged 2/32 contract at
  26 pixels/max-55; the ratchet is exact=148/diverges=0/gated=1,319 and
  `dstreadshuffle` is next.
- 2026-07-13: Isolated `dstreadshuffle` to the named intermediate-color
  precision boundary and parked it without tolerance changes. Promoted fresh
  `overfill_blendmodes` output at 7 pixels/max-3 under the unchanged 2/32
  contract; the ratchet is exact=149/diverges=0/gated=1,318 and
  `strokes_round` is next.
- 2026-07-13: Localized `strokes_round` to five unresolved foreground-support
  pixels at draw 38's smooth close seam and kept it gated for a pre-raster
  record oracle after Sol rejected a tolerance promotion. Promoted
  `overfill_opaque` under its independently proven 48-pixel cubic-edge
  allowance; all renderer and V2 gates are green at
  exact=150/diverges=0/gated=1,317.
- 2026-07-13: Built a record-exact C++ CPU tessellation-span oracle for
  `strokes_round` draw 38, ported C++'s five-segment non-round joins, full raw
  line tangents, and padding-before-geometry write order, and matched all 11
  spans/176 words. Fresh native output has zero pixels beyond delta 2; the
  unchanged `2/32` contract promotes the entry and advances the ratchet to
  exact=151/diverges=0/gated=1,316.
- 2026-07-13: Audited all 14 `strokefill` draws independently and promoted the
  edge-only 109-pixel residual under a bounded 128-pixel allowance, advancing
  the renderer ratchet to exact=152/diverges=0/gated=1,315. Renderer golden,
  both V2 golden floors, and the workspace tests pass; `rawtext` is next.
- 2026-07-13: Added a stream-derived C++ production oracle for `rawtext`,
  matched all 438 CPU spans and the complete tessellation texture exactly,
  ported four shared fill-preparation details, and promoted the sparse final
  raster residual under a bounded 288-pixel allowance. The renderer ratchet is
  exact=153/diverges=0/gated=1,314; renderer golden, the full workspace suite,
  and both V2 golden floors pass.
- 2026-07-13: Closed the required mid-R2 wgpu resource-seam audit. Added
  adapter-limit preflight for frame and image textures, bounded disjoint atomic
  batches at 65,535 paths, replaced oversized inseparable-run panics with a
  named error, and recorded the R3/R4 boundaries without changing the renderer
  ratchet.
- 2026-07-13: Closed the direct `feather_cusp` structural mismatch with C++'s
  fixed-color generic-atomic face and clockwise paint encoding. Added exact
  C++ atomic-coverage capture plus same-backend final-blit oracles, preserved
  authored clipped fill rules, and kept advanced feather blending green after
  Sol review. Native Metal comparison is bounded at 9,480 pixels/max delta 11;
  promotion advances the renderer ratchet to
  exact=154/diverges=0/gated=1,313. Renderer golden, both V2 golden floors,
  and the full workspace suite pass.
- 2026-07-13: Added the exact-source C++ Dawn atomic ColorBurn pair oracle for
  `interleavedfeather` draws 13-14, including test-only Rust/C++ color and
  coverage plane readbacks. It exposed and fixed generic feathered-clockwise
  paint preparation and advanced feather-fill face culling. Normalized raw
  coverage is exact; the only final difference is two coupled color
  words/pixels at max
  byte/channel deltas one and seven. Both remaining GMs stay gated pending
  independent full-stream C++ WebGPU references. The renderer ratchet remains
  exact=154/diverges=0/gated=1,313 with no tolerance or corpus edit.
- 2026-07-13: Added the pinned full-stream C++ Dawn WebGPU-on-Metal oracle for
  all 451 `interleavedfeather` draws. Rust passes its existing `2/32` contract
  at 6 over-threshold pixels; three-way native Metal comparison proves the
  remaining corpus gap is backend precision rather than algorithm core. The
  entry remains gated under the named backend boundary, with the renderer
  ratchet unchanged at exact=154/diverges=0/gated=1,313.
- 2026-07-13: Added pinned full-stream untouched and SrcOver-control C++ Dawn
  WebGPU-on-Metal oracles for all 97 `dstreadshuffle` draws. The untouched gate
  remains open at roughly 22.84k pixels over delta 2/max 61; changing only the
  97 paint blend modes to SrcOver passes three samples at 11-13 pixels over
  delta 2/max 4. Sol approved reclassifying the corpus diagnostic to the named
  shader-stack precision boundary while preserving gated status, native
  reference, tolerance, and renderer ratchet.
- 2026-07-13: Closed R2 at 106/108 passing clockwise-atomic upstream GMs plus
  two reviewed backend/compiler precision gates and zero remaining
  `algorithm-core` gates. No corpus tolerance or native reference changed.
  R3's semantic-trap audit and dual-renderer fuzz replay are now the active
  entry work.
- 2026-07-13: Pinned reproducible Rust and C++ shader compiler-input lineages,
  including upstream minifier determinism, exact artifact counts/digests, and
  a macOS CI gate. The ABI test now explicitly covers `ImageRectVertex`.
  Renderer tests pass 193/193 active cases and the corpus ratchet remains
  exact=154/diverges=0/gated=1,313 after Sol approval. The sampled nested clip
  plane and decoded-image bytes are the only open semantic-trap oracles.
- 2026-07-13: Closed the sampled nested clip-plane semantic fork with a
  zero-delta 640x640 native Metal versus Rust production readout and a Rust
  routing test that pins `OutermostClip`, `NestedClip`, and `ClippedContent`.
  The renderer ratchet advances to exact=155/diverges=0/gated=1,313; decoded
  image bytes are the only remaining semantic-trap oracle before fuzz replay.
- 2026-07-13: Closed decoded-image color ingress with a native raw-buffer
  oracle over the production C++ and Rust decode paths. The reachable JPEG
  differs at 35,652 pixels/78,669 channels, with 12,509 source pixels over
  delta 2, max delta 37, and exact alpha, confirming a decoder-level difference
  on the same rendered image. The ICC PNG differs at 4,950 pixels/5,013
  channels, max delta 2, exact alpha, proving color conversion is already
  within the corpus threshold.
  `make renderer-decoder-oracle` pins fixture and runtime provenance plus the
  bounded contracts; no corpus tolerance or reference changed. Dual-renderer
  fuzz replay is now the only remaining R3 entry gate.
- 2026-07-13: Closed the R3 dual-renderer fuzz-replay entry gate with five
  deterministic hostile-stream families, per-child wall deadlines, PNG and
  finite-control-region oracles, named C++/Rust pixel findings, and a macOS CI
  smoke target. The first absurd-stroke replay exposed a Rust debug-overflow
  panic; clamping segment arithmetic before integer conversion fixes it and a
  focused unit test pins the regression. Non-finite transforms and degenerate
  geometry are exact, deep clips stay within 21 pixels/max delta 1, and the
  absurd-stroke and invalid-gradient raster differences remain named
  out-of-contract findings. See `docs/renderer-fuzz-replay.md`.
- 2026-07-13: Added an explicit, fail-closed `--probe-gated ID` corpus mode and
  probed the first ten clockwise-atomic `.riv` entries against freshly pinned
  native Metal references. Nine pass the unchanged `2/32` contract and advance
  the renderer ratchet to exact=164/diverges=0/gated=1,304. `align_target`
  remains gated at 77 pixels/max delta 52: 72 outliers lie on the transformed
  glyph outline, four on a circle edge, and one on a large background edge, so
  its placeholder is replaced by the evidence-backed
  `metal-webgpu-subpixel-edge-coverage` diagnostic without changing tolerance.
- 2026-07-13: Probed the second ten-entry clockwise-atomic `.riv` batch against
  freshly pinned native Metal references. Nine pass the unchanged `2/32`
  contract and advance the renderer ratchet to
  exact=173/diverges=0/gated=1,295. `audio_script` remains gated at 251
  pixels/max delta 36: every outlier is a tiny component on one of seven text
  outline draws, while the eight axis-aligned background/rectangle draws are
  clean, so the existing reviewed `metal-webgpu-subpixel-edge-coverage`
  diagnostic applies without a tolerance change.
- 2026-07-13: Probed the third ten-entry clockwise-atomic `.riv` batch against
  freshly pinned native Metal references. Nine pass the unchanged `2/32`
  contract and advance the renderer ratchet to
  exact=182/diverges=0/gated=1,286. `bindable_artboard_nesty` remains gated at
  79 pixels/max delta 60; every outlier lies on its single transformed white
  glyph-outline draw while the full-canvas background is clean, so the
  existing reviewed `metal-webgpu-subpixel-edge-coverage` diagnostic applies.
- 2026-07-13: Probed the fourth ten-entry clockwise-atomic `.riv` batch against
  freshly pinned native Metal references. All five `clear_viewmodel_list`
  frames are pixel-identical and all five `click_event` frames stay entirely
  within channel delta 2, so the unchanged `2/32` contract promotes all ten
  and advances the renderer ratchet to exact=192/diverges=0/gated=1,276.
- 2026-07-13: Probed the fifth ten-entry clockwise-atomic `.riv` batch against
  freshly pinned native Metal references. Eight pass the unchanged `2/32`
  contract and advance the renderer ratchet to
  exact=200/diverges=0/gated=1,268. `collapse_data_binds` remains gated at 61
  pixels/max delta 31: 59 pixels lie on four glyph outlines and two on
  transformed dark/rectangle edges. `collapsing_elements` remains gated at 37
  pixels/max delta 31: every outlier is a single pixel on the fractional right
  edge of five stripe rectangles and its other 43 draws are clean. Sol
  approved applying the existing `metal-webgpu-subpixel-edge-coverage` gate
  to both without tolerance changes.
- 2026-07-13: Probed the sixth ten-entry clockwise-atomic `.riv` batch against
  freshly pinned native Metal references. All ten pass the unchanged `2/32`
  contract; eight have zero over-threshold pixels and the two remaining
  `component_based_conditions` frames repeat the family's one-pixel/max-19
  result. The renderer ratchet advances to
  exact=210/diverges=0/gated=1,258.
- 2026-07-13: Probed the seventh ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. All ten pass the unchanged
  `2/32` contract: `component_list_grouped` frames consistently use 15
  over-threshold pixels/max delta 44, while the other five entries have zero
  over-threshold pixels. The renderer ratchet advances to
  exact=220/diverges=0/gated=1,248.
- 2026-07-13: Probed the eighth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Eight pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=228/diverges=0/gated=1,240. `component_stateful` remains gated at 35
  pixels/max delta 52, all on its two white glyph-outline draws.
  `computed_values_test` remains gated at 46 pixels/max delta 24, all in tiny
  components on the translated glyph outline while its background and later
  rectangle draws are clean. Sol approved applying the existing
  `metal-webgpu-subpixel-edge-coverage` diagnostic to both without tolerance
  changes.
- 2026-07-13: Probed the ninth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. All ten pass the unchanged
  `2/32` contract: `cubic_value_test` frames 3-4 repeat the family's
  four-pixel/max-44 result, `custom_property_trigger` uses six pixels/max 5,
  and the other seven entries have zero over-threshold pixels. The renderer
  ratchet advances to exact=238/diverges=0/gated=1,230.
- 2026-07-13: Probed the tenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Five pass with zero
  over-threshold pixels and advance the renderer ratchet to
  exact=243/diverges=0/gated=1,225. Four vector failures remain gated under
  `metal-webgpu-subpixel-edge-coverage`: `data_bind_test_cmdq` (792/max 95),
  `data_binding_artboards_source_test` (130/max 55),
  `data_binding_artboards_test` (65/max 66), and `data_binding_test`
  (1,163/max 96). Read-only Terra scouts inventoried the draws; main completed
  the missing coordinate-to-path join and corrected an over-broad glyph-only
  claim before Sol approval. `data_binding_images_test` (5,269/max 22, exact
  alpha) is wholly confined to its sole JPEG draw and remains gated under
  `platform-image-decode-color-profile`; the pinned production decoder oracle
  passes on the current upstream ref. No tolerance changed.
- 2026-07-13: Probed the eleventh ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Eight pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=251/diverges=0/gated=1,217; seven have zero over-threshold pixels and
  `databind_solo_to_enum` uses 28 pixels/max 42. `data_converter_to_number`
  remains gated at 424/max 65 across six non-overlapping glyph bands, and
  `databind_viewmodel` remains gated at 96/max 35 on its sole glyph outline.
  A read-only Terra scout attributed every outlier and Sol approved the
  existing `metal-webgpu-subpixel-edge-coverage` diagnostic for both without
  tolerance changes.
- 2026-07-13: Probed the twelfth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Eight pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=259/diverges=0/gated=1,209: the five `dependency_test` frames and
  `double_library_with_image` have zero over-threshold pixels,
  `distance_constraint` uses two pixels/max 3, and `drag_event` uses four
  pixels/max 58. `databind_viewmodel` frame 1 repeats frame 0's byte-identical
  native reference and identical over-threshold mask, so it inherits the
  reviewed edge-coverage gate. `double_line` remains gated at 145/max 57;
  its 43 tiny components are confined to the sole translated even-odd
  foreground fill. A read-only Terra scout supplied the draw inventory and
  Sol approved `metal-webgpu-subpixel-edge-coverage` without a tolerance
  change.
- 2026-07-13: Probed the thirteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Nine pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=268/diverges=0/gated=1,200. `ellipsis` remains gated at 35/max 33;
  all outliers have exact alpha and lie on the sole translated white text
  path's contour boundaries while the full-canvas background is clean. Main
  independently checked the draw count and alpha plane, and Sol approved the
  existing `metal-webgpu-subpixel-edge-coverage` diagnostic without changing
  the reference or tolerance.
- 2026-07-13: Probed the fourteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. All six remaining
  `event_on_listener` frames repeat the family's one-pixel/max-17 result and
  all four `event_trigger_event` frames have zero over-threshold pixels. The
  unchanged `2/32` contract promotes all ten and advances the renderer ratchet
  to exact=278/diverges=0/gated=1,190.
- 2026-07-13: Probed the fifteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. The remaining four
  `event_trigger_event` frames have zero over-threshold pixels and all six
  `events_on_states` frames repeat the one-pixel/max-17 event-family result.
  The unchanged `2/32` contract promotes all ten and advances the renderer
  ratchet to exact=288/diverges=0/gated=1,180.
- 2026-07-13: Probed the sixteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Nine pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=297/diverges=0/gated=1,171. `fit_font_size_test` remains gated at
  106/max 42: every outlier has exact alpha and lies on draw 5's translated
  right-column text contours, while panel fills and the zero-sized middle
  text path are clean. A read-only Terra scout distinguished this residual
  from the resolved runtime layout-bounds divergence; main checked the draw
  structure and alpha plane, and Sol approved the existing
  `metal-webgpu-subpixel-edge-coverage` diagnostic without changing the
  reference or tolerance.
- 2026-07-13: Probed the seventeenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Eight pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=305/diverges=0/gated=1,163. `focus_traversal` remains gated at
  101/max 100, with all outliers confined to six transformed white glyph
  contours. `follow_path_path` remains gated at 223/max 85 across 87 tiny
  neutral-RGB contour components: 205 text/mark pixels plus 18 logo, badge,
  and vector-edge pixels; alpha is exact and the background is clean. Two
  read-only Terra scouts attributed the independent masks, main verified both
  alpha planes and stream inventories, and Sol approved the existing
  `metal-webgpu-subpixel-edge-coverage` diagnostic for both without changing
  either reference or tolerance.
- 2026-07-13: Probed the eighteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. Eight pass the unchanged
  `2/32` contract and advance the renderer ratchet to
  exact=313/diverges=0/gated=1,155. `format_number_with_commas` remains gated
  at 496/max 27 across five fractionally translated white text-outline draws;
  `hello_world` remains gated at 59/max 47 on the sole translated cyan glyph
  outline. Every outlier in both fixtures has exact alpha, the backgrounds
  are clean, and all over-threshold pixels map to glyph boundaries. Two
  read-only Terra scouts supplied independent path attribution, main checked
  the alpha and stream oracles, and Sol approved
  `metal-webgpu-subpixel-edge-coverage` for both without changing either
  reference or tolerance.
- 2026-07-13: Probed the nineteenth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. `hide_test` is pixel exact;
  `hit_test_nested` passes at 18/max 45; and all eight `hit_test_solos` frames
  pass with the same one-pixel/max-13 result. The unchanged `2/32` contract
  promotes all ten and advances the renderer ratchet to
  exact=323/diverges=0/gated=1,145 without a new diagnostic or tolerance
  change.
- 2026-07-13: Probed the twentieth ten-entry clockwise-atomic `.riv` batch
  against freshly pinned native Metal references. `hit_test_test` passes at
  6/max 22; the remaining nine text, hosted-asset, image-binding, image-fit,
  scripting, and in-band-asset fixtures have no pixels beyond delta 2. The
  unchanged `2/32` contract promotes all ten and advances the renderer ratchet
  to exact=333/diverges=0/gated=1,135 without a new diagnostic or tolerance
  change.
- 2026-07-13: Probed the twenty-first through twenty-third ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Three disjoint Terra
  workers captured and probed all 30 entries while main retained manifest and
  gate ownership. Twenty-seven pass the unchanged `2/32` contract.
  `interpolate_to_end` remains gated at 97/max 33, `keyboard_listener` at
  178/max 58, and `library` at 119/max 59. Their dominant residuals have exact
  alpha and are confined to fractional text/path boundaries; `library` also
  has seven delta-3 image pixels that would pass the allowance independently.
  Main verified the alpha oracles, three read-only Terra scouts attributed the
  failures, and Sol approved `metal-webgpu-subpixel-edge-coverage` for all
  three without changing references or tolerances. The combined wave advances
  the ratchet to exact=360/diverges=0/gated=1,108.
- 2026-07-13: Probed the twenty-fourth through twenty-sixth ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references; three read-only Terra workers probed disjoint
  batches into `/tmp`, with two infrastructure-only retries after workers
  omitted the explicit workspace `cd`. Twenty-eight entries pass the
  unchanged `2/32` contract. `listener_view_model` remains gated at 118/max
  81 across three translated text-outline bands, and `local_bounds` at
  144/max 56 across two text draws plus nine vector-edge pixels. Both have
  exact alpha and no over-threshold interior or image residual. Main verified
  the alpha oracles, Terra attributed both failures, and Sol approved
  `metal-webgpu-subpixel-edge-coverage` without changing references or
  tolerances. The combined wave advances the ratchet to
  exact=388/diverges=0/gated=1,080.
- 2026-07-13: Probed the twenty-seventh through twenty-ninth ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references and three read-only Terra workers probed disjoint
  batches. Twenty-four entries pass the unchanged `2/32` contract. Six
  remain gated: `modifier_test` at 151/max 89, `modifier_to_run` at
  563/max 91, `multi_listeners` at 655/max 60, `nested_hug` at 285/max
  82, and both `nested_solo` frames at 42/max 14. Their exact-alpha
  residuals partition completely to transformed text, vector, or circle
  boundaries with clean interiors/backgrounds. The first `multi_listeners`
  scout used the wrong raw-count method; main reproduced the harness result
  in a fresh replay and a replacement scout matched the repository comparator
  exactly. Main verified all alpha planes, Terra attributed the six failures,
  and Sol approved `metal-webgpu-subpixel-edge-coverage` without changing
  references or tolerances. The wave advances the ratchet to
  exact=412/diverges=0/gated=1,056.
- 2026-07-13: Probed the thirtieth through thirty-second ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references and three read-only Terra workers probed disjoint
  batches. Twenty-seven entries pass the unchanged `2/32` contract. The
  three remaining `nested_solo` frames each reproduce at 42/max 14; their
  references and generated images are byte-identical across frames, their
  alpha planes are exact, and every over-threshold pixel lies on one of three
  transformed circle boundaries with clean interiors/backgrounds. Main
  reproduced the harness counts, Terra attributed the residuals, and Sol
  approved `metal-webgpu-subpixel-edge-coverage` without changing references
  or tolerances. The wave advances the ratchet to
  exact=439/diverges=0/gated=1,029.
- 2026-07-13: Probed the thirty-third through thirty-fifth ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references and three read-only Terra workers probed disjoint
  batches. Twenty-nine entries pass the unchanged `2/32` contract.
  `pointer_exit` remains gated at 44/max 47: its alpha plane is exact and all
  44 over-threshold pixels lie on the blue-dot, panel, or canvas boundaries
  with clean interiors. Main reproduced the harness and alpha results, Terra
  attributed the residual, and Sol approved
  `metal-webgpu-subpixel-edge-coverage` without changing the reference or
  tolerance. The wave advances the ratchet to
  exact=468/diverges=0/gated=1,000.
- 2026-07-13: Probed the thirty-sixth through thirty-eighth ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references and three read-only Terra workers probed disjoint
  batches. Twenty-seven entries pass the unchanged `2/32` contract.
  `replace_vm_instance` remains gated at 71/max 57,
  `runtime_nested_text_runs` at 352/max 91, and `saturation` at 37/max 66.
  All three alpha planes are exact; their over-threshold components are
  confined to text contours or one circle boundary with clean interiors and
  backgrounds. Main reproduced the harness and alpha results, Terra
  attributed the residuals, and Sol approved
  `metal-webgpu-subpixel-edge-coverage` without changing references or
  tolerances. The wave advances the ratchet to
  exact=495/diverges=0/gated=973.
- 2026-07-13: Probed the thirty-ninth through forty-first ten-entry
  clockwise-atomic `.riv` batches as one parallel wave. Main captured all 30
  native Metal references and three read-only Terra workers probed disjoint
  batches. All 30 entries pass the unchanged `2/32` contract, so no
  diagnostic reclassification or tolerance decision was needed. The wave
  advances the ratchet to exact=525/diverges=0/gated=943.
- 2026-07-13: Probed the forty-second through forty-seventh ten-entry
  clockwise-atomic `.riv` batches as one six-worker parallel wave. Main
  captured all 60 native Metal references and six read-only Terra workers
  probed disjoint batches. Fifty-eight entries pass the unchanged `2/32`
  contract. `scroll_snap` remains gated at 96/max 60: all over-threshold
  pixels have exact alpha and lie in 38 tiny components on five fractionally
  translated white text contours, while panel fills, strokes, interiors, and
  background are clean. Terra attributed the residual and Sol approved
  `metal-webgpu-subpixel-edge-coverage`. `spotify_kids_app_icon` remains
  `algorithm-core` at 48,790/max 26: Sol rejected broadening the currently
  two-entry `metal-webgpu-atomic-intermediate-precision` family without a
  pinned C++ Dawn full-stream and representative coverage/color-plane oracle.
  No existing reference or tolerance changed. The wave advances the ratchet to
  exact=583/diverges=0/gated=885.
- 2026-07-13: Probed the final one hundred previously unprobed
  `algorithm-core` clockwise-atomic `.riv` entries as ten read-only Terra
  batches in scheduler-limited parallel tranches. Main captured all 100
  native Metal references. Ninety-one entries pass the unchanged `2/32`
  contract. The nine residuals are `test_modifier_run`,
  `text_follow_path_shape_length`, `text_input`, `text_listener_simpler`,
  `text_vertical_trim_test`, `transition_actions`,
  `trigger_fires_single_change`, `vertical_align_ellipsis`, and
  `viewmodel_access`. Every residual has an exact alpha plane and only tiny
  transformed text/path contour components with clean fills and interiors;
  Terra attributed each and Sol independently approved
  `metal-webgpu-subpixel-edge-coverage`. No tolerance changed. The only
  remaining clockwise-atomic `.riv` `algorithm-core` gate is the previously
  probed `spotify_kids_app_icon` oracle hold. The wave advances the ratchet
  to exact=674/diverges=0/gated=794.
- 2026-07-13: Closed the final clockwise-atomic `.riv` `algorithm-core` hold
  with the pinned Spotify full-stream C++ Dawn oracle. The strict compiler
  validates 14 draws, 6 clips, 15 transforms, 18 balanced saves/restores, 20
  paths, 48 paints, and the exact stream/replay/schedule identities. C++ Dawn
  and Rust agree under `2/32`, share the final clip plane exactly, and isolate
  the same native-Metal residual without a packed atomic color plane. Sol's
  three review findings were fixed before reclassification. The ratchet stays
  exact=674/diverges=0/gated=794; only the named diagnostic changed.
- 2026-07-13: Audited the next parallel MSAA wave before dispatch. The corpus
  has 730 `algorithm-core` MSAA rows and zero MSAA reference PNGs because the
  native Metal capture path correctly rejects that mode. Three read-only Terra
  scouts isolated the reusable C++ Dawn path and found the strict source
  generator accepted 26/40 initial GMs. A bounded worker then fixed GM
  pre-metadata declarations and fill-rule-only path reuse, raising support to
  39/40 while preserving geometry-mutation rejection. `pixel-compare` now
  validates exact `RIVEABL` payloads and converts them to PNG. The first worker
  passed all 33 oracle-format tests; a slower registry lane was stopped rather
  than held on the critical path. Build the provenance-bound runner next, then
  fan out the actual probes.
- 2026-07-13: Completed the provenance-bound C++ Dawn MSAA reference runner
  and first ten-case capture. The strict embedded registry, runtime, Dawn,
  stream, adapter, RIVEABL, PNG-pixel, and artifact identities all validate;
  jobs 1 and 4 recaptures are byte-identical. `batchedconvexpaths`,
  `batchedtriangulations`, `convex_lineonly_ths`, `convexpaths`, and `oval`
  pass the unchanged `2/32` contract and advance the ratchet to
  exact=679/diverges=0/gated=789. `concavepaths`, `pathfill`, and the three
  `poly_*` cases retain named evidence-backed gates with no tolerance change.
  Bounded corpus parallelism passed two Terra implementation rounds and Sol
  adversarial review; a 40-entry benchmark measured 2.54x wall-clock speedup
  at four jobs, while the full four-job ratchet finished green in 4m42.87s.
- 2026-07-13: Ported C++'s fill-rule-specific MSAA midpoint-fan schedule and
  intersection-board depth groups. The three `poly_*` cases, `concavepaths`, and
  `pathfill` now pass their unchanged Dawn `2/32` contracts at zero pixels
  beyond threshold, advancing the ratchet to exact=684/diverges=0/gated=784.
- 2026-07-13: Captured the second ten strict C++ Dawn MSAA references with the
  20-case provenance registry; the original ten PNGs recapture byte-identically.
  Eight new rows pass unchanged `2/32` contracts and move the ratchet to
  exact=692/diverges=0/gated=776. `beziers` is isolated to cubic-stroke raster
  parity rather than scheduling, and `bug339297_as_clip` reaches the named
  non-atlas MSAA path-clip boundary. Queue item 63 names the next ten accepted
  uncaptured source-order streams. The 34 oracle format tests, 11 capture
  coordinator tests, full renderer ratchet, workspace suite, and both V2
  golden floors pass.
- 2026-07-13: Captured the third ten strict C++ Dawn MSAA references with a
  30-case provenance registry; the prior 20 PNGs recapture byte-identically.
  The clear-only `emptyclear` stream drove a fail-closed expected-draw-batch
  registry contract. Six rows pass unchanged `2/32` contracts, advancing the
  ratchet to exact=698/diverges=0/gated=770. Three clip-rectangle failures are
  isolated to the direct path's `noclipdistance` vertex variant, and
  `dstreadshuffle` is isolated to missing direct-path destination-read
  blending. Queue items 64-65 name those implementation slices. All 30
  provenance records, 34 oracle format tests, 11 capture coordinator tests,
  the workspace suite, renderer ratchet, and both V2 golden floors pass.
- 2026-07-13: Ported the generated direct MSAA clip-distance pipeline variants
  and selected them for clip-rect paint data. The focused GPU regression and
  all 197 enabled renderer tests pass. Dawn reprobes promote `clippedcubic2`
  and `cliprects` at zero over-threshold pixels; `cliprectintersections` keeps
  the narrower `msaa-clip-intersection-edge-coverage` gate at 240 pixels/max
  55. The full renderer ratchet is exact=700/diverges=0/gated=768 with no
  tolerance change. Queue item 65 is next.
- 2026-07-13: Ported direct MSAA destination-read advanced blending with the
  generated advanced/HSL shader variants, destination binding 13, and bounded
  per-draw resolve/copy/reload barriers. The focused destination-read GPU
  regressions and all 199 enabled renderer tests pass. Sol found no production
  defect and approved the gate evidence after requiring the second regression
  to cover fixed-to-advanced, consecutive advanced/HSL, analytic stroke,
  fill-rule, translucent-alpha, and empty-bounds behavior. With C++'s dither,
  `dstreadshuffle` improves to 2,231 pixels/max 43; alpha is exact and bounded
  versus full-frame copies are byte-identical. It remains gated under the
  narrower `dawn-wgpu-msaa-advanced-blend-intermediate-precision` diagnostic
  with no reference or tolerance change. The renderer ratchet remains
  exact=700/diverges=0/gated=768. Queue item 66 names the next ten strict
  source-order C++ Dawn MSAA captures.
- 2026-07-13: Captured the fourth ten strict C++ Dawn MSAA references with the
  40-case provenance registry; all prior 30 PNGs recapture byte-identically.
  Seven rows pass unchanged `2/32`, advancing the renderer ratchet to
  exact=707/diverges=0/gated=761. `emptystrokefeather` isolates missing inner
  coverage at 36 degenerate cap centers while preserving exact alpha and the
  C++ feather halo. `feather_cusp` and `feather_shapes` retain sparse
  exact-alpha max-delta-4 larger-radius atlas contour residuals. Sol approved
  the evidence-backed gates, narrowed the precision gate name to avoid an
  unsupported compiler attribution, and found no blocker. No tolerance
  changed. Queue item 67 targets the degenerate feather-stroke cap behavior.
- 2026-07-14: Ported C++'s scheduled MSAA depth index through feather-atlas
  blits instead of resetting every atlas path to group 1. A new isolated C++
  oracle keeps the move-only cap tessellation and R16 mask pinned, then places
  an opaque marker beneath the cap to prove the depth dependency: the old Rust
  path leaves center RGBA `[255,0,0,255]`, while C++ and the fixed Rust path
  are byte-exact at `[255,79,79,255]`; an enabled Rust GPU regression retains
  that marker-overlap behavior in the default suite.
  `gm-emptystrokefeather-msaa` now has zero pixels beyond delta 2, all seven
  exact siblings remain green, and the two larger-radius feather gates retain
  their prior 435/max-4 and 180/max-4 results. The renderer ratchet advances
  to exact=708/diverges=0/gated=760 without changing a reference or tolerance.
  Queue item 68 targets the shared larger-radius atlas-feather precision
  boundary.
- 2026-07-14: Closed the larger-radius atlas-feather gap with three C++ parity
  fixes. Atlas placement now uses the same softened fill path as C++, fused
  multiply-add translation matches C++ float contraction, and feather
  tessellation applies C++'s radius-scaled Wang precision. New paired oracles
  capture runtime-derived exact `RIVEATP` placement, complete `RIVEATI` inputs,
  physical `RIVEMSK` coverage with exact signed support and a calibrated
  `2^-9` value bound, and full `RIVEABL` MSAA output for the strongest cusp
  and shapes-cusp contours. The cusp is now 0/max 2 and shapes is 11/max 3
  under the unchanged `2/32` contract, promoting both entries and advancing
  the ratchet to exact=710/diverges=0/gated=758. The workspace suite, 206
  enabled renderer tests, 35 oracle-format tests, C++ oracle build, normal
  584-segment golden floor, scripted 35-segment floor, and full renderer
  corpus all pass. Queue item 69 names the next strict source-order Dawn MSAA
  capture wave.
- 2026-07-14: Captured the fifth ten strict source-order C++ Dawn MSAA
  references with the 50-case registry. Bounded helper generation and the
  supported no-LTO oracle build make the two 32k-path hit-test streams compile
  repeatably. Jobs 1 and 4 produce all 150 artifacts byte-identically, and the
  prior 40 PNGs remain byte-identical. Five rows pass unchanged `2/32`, moving
  the renderer ratchet to exact=715/diverges=0/gated=753. Reflected feather
  transforms, large-draw readback mapping, sparse interleaved feather color,
  and four `strict-replay-decode-image` rejections have separate named gates.
  No tolerance changed. Queue item 70 targets reflected atlas-feather
  transforms.
- 2026-07-14: Ported C++'s determinant-sensitive atlas contour direction into
  the Rust MSAA feather path. The selector regression pins reflected
  clockwise fills against non-zero fills and strokes; the full Dawn probes
  promote mirrored Montserrat at 13/max 68 and mirrored Roboto at 0/max 2
  under unchanged `2/32`. All five queue-item 69 siblings and the full corpus
  remain green, advancing the ratchet to exact=717/diverges=0/gated=751.
  Queue item 71 targets the two valid 32k-draw streams that currently fail
  Rust readback mapping.
- 2026-07-14: Bounded large clip-independent source-over MSAA schedules to
  1,024 draws per submitted encoder after minimizing the Metal map failure to
  exactly 2,044 direct draws. The enabled boundary stress test and both
  complete 32k hit-test streams now finish without device loss; each C++ Dawn
  probe has zero pixels beyond delta 2 and max delta 1. The full renderer
  corpus advances to exact=719/diverges=0/gated=749, all 717 prior exact
  entries remain green, the 243-test renderer suite and workspace pass, and
  the normal 584-segment
  plus scripted 35-segment V2 floors remain green. Queue item 72 targets the
  four-entry strict image-decode capture gate.
- 2026-07-14: Added strict image-resource replay to the C++ Dawn MSAA oracle
  and captured `image`, `image_aa_border`, `image_filter_options`, and
  `image_lod` deterministically in serial and four-job modes without changing
  any prior rendered artifact. All four Rust probes stop at the shared MSAA
  image-rectangle rejection, so their Dawn references are pinned and the
  gate narrows to `rust-wgpu-msaa-image-rect` without promotion. The renderer
  ratchet remains exact=719/diverges=0/gated=749; the workspace, normal
  584-segment, and scripted 35-segment V2 floors pass. Queue item 73 ports the
  image-rectangle path.
- 2026-07-14: Ported C++'s MSAA rectangular-image path as a unit rectangle with
  image paint, including inverse paint coordinates, constant biased mip LOD,
  real texture/sampler bindings, opacity, blend mode, and edge coverage. The
  focused atomic/MSAA GPU oracle is byte-exact; all four pinned Dawn image
  probes pass unchanged `2/32` with zero over-threshold pixels. The renderer
  ratchet advances to exact=723/diverges=0/gated=745, all 719 prior exact rows
  remain green, the 245-test renderer suite passes, and workspace plus normal
  584-segment and scripted 35-segment V2 floors pass. Queue item 74 names the
  next strict source-order capture batch.
- 2026-07-14: Captured ten more provenance-bound C++ Dawn MSAA references with
  a 64-case registry. Serial and four-job runs match across all 192 artifacts,
  and all 54 retained PNGs are unchanged. `interleavedfillrule` and the three
  labyrinth rows promote under unchanged `2/32`; the six large clipped-path
  rows narrow to the shared `non-atlas-msaa-path-clip` boundary. The renderer
  ratchet advances to exact=727/diverges=0/gated=741 with all 723 prior exact
  rows green; the workspace and normal 584-segment plus scripted 35-segment V2
  floors pass. Queue item 75 ports direct MSAA path clipping.
- 2026-07-14: Captured nine more provenance-bound C++ Dawn MSAA references
  with a 73-case registry; `mesh` remains gated at strict render-buffer replay.
  The accepted serial/four-job waves match across all 219 artifacts, all 64
  retained PNGs are unchanged, and all nine Rust probes promote under the
  unchanged `2/32` contract. The renderer ratchet advances to
  exact=743/diverges=0/gated=725; workspace, renderer, and both V2 floors pass.
  Queue item 77 names the next strict source-order capture batch.
- 2026-07-14: Ported C++ logical-flush resource rollover across hard path,
  contour, tessellation, and signed-pass limits plus late gradient, feather
  atlas, and coverage-storage allocation limits. Active clips regenerate and
  MSAA/atomic ordering survives forced submissions, including destination
  reads and intersection-board schedules. Sol accepted the focused boundary
  and pixel-equivalence evidence. The renderer corpus remains
  exact=760/diverges=0/gated=708; format/check, the five-case dual-renderer
  fuzz gate, all 223 enabled renderer tests, the workspace, normal 584-segment
  floor, and scripted 35-segment floor pass. Queue item 81 integrates and
  probes the independently captured Dawn references.
- 2026-07-14: Integrated the Sol-accepted strict Dawn capture campaign after
  all 102 fixture bindings reached the production provenance validator and
  serial/four-job captures agreed byte-for-byte. Nine of eleven new MSAA rows
  pass unchanged `2/32`; `trickycubicstrokes` and `widebuttcaps` retain the
  shared, localized `msaa-degenerate-cubic-butt-miter-topology` gate with no
  tolerance change. The capture inventory now has zero accepted rows waiting
  for promotion, 638 gated MSAA rows, 7 provenance-bound rows, and 631
  generator-unsupported rows. The renderer ratchet advances to
  exact=769/diverges=0/gated=699; 40 format tests, 11 capture tests, inventory
  drift check, workspace, normal 584-segment floor, and scripted 35-segment
  floor pass. Queue item 82 targets the shared two-row topology gap.
- 2026-07-14: Closed the degenerate-cubic MSAA stroke gap with exact bounded
  C++/Rust CPU-span, tessellation-texture, and final-pixel oracles. Paired-root
  splitting, carried join/cap tangents, fused interpolation, and C++ back-face
  culling make `gm-trickycubicstrokes-msaa` and `gm-widebuttcaps-msaa`
  byte-exact. The corpus advances to exact=771/diverges=0/gated=697, and the
  regenerated capture inventory has 636 gated MSAA rows: 631 are generator
  unsupported, including 624 `.riv` frame-selection rows, and five already
  have strict provenance. Queue item 83 extends `.riv` frame-selection
  generation and then launches continuous Dawn capture.
- 2026-07-14: Added strict retained-declaration RIV frame selection and
  mechanically compiled all 624 gated rows: 584 enter the 686-case capture
  registry, while 38 gradient and two render-buffer rows remain unsupported.
  Two continuous four-worker Dawn campaigns produce all 2,058 artifacts
  byte-identically, and all 102 retained GM PNGs remain byte-identical. The
  production provenance inventory validates 8 still-gated rows and leaves 47
  generator-unsupported rows. Isolated Rust probes promote 581 of 584 RIV
  rows under unchanged `2/32`; `clipping_and_draw_order` reaches the explicit
  non-atlas MSAA path-clip rejection, Spotify Kids Demo misses clipped facial
  draws, and `data_binding_images_test` retains its existing platform image
  color-profile delta. The renderer ratchet advances to
  exact=1,352/diverges=0/gated=116. The 43 oracle-format tests, 12 capture
  tests, full renderer ratchet, workspace suite, normal 584-segment floor, and
  scripted 35-segment floor pass. Queue item 84 ports the clipping boundary.
- 2026-07-14: Ported non-atlas MSAA path clipping through image rectangles and
  fixed nested clip-reset depth to use the scheduled draw group, matching C++
  instead of hardcoding z=1. Focused pixel regressions pin both behaviors, and
  the latter restores all 3,690 previously missing Spotify facial pixels.
  `clipping_and_draw_order` now reaches rendering and narrows to its existing
  platform JPEG decode family at 8,905/max 18; Spotify narrows from
  3,788/max 230 to 98/max 41 on two mirrored foot/leg contour edges. The
  renderer ratchet stays exact=1,352/diverges=0/gated=116 under unchanged
  contracts. All 228 enabled renderer tests, the workspace, the full renderer
  corpus, the normal 584-segment V2 floor, and the scripted 35-segment floor
  pass. Queue item 85 targets the isolated contour-edge coverage residual.
- 2026-07-14: Closed Spotify Kids Demo's final MSAA residual by porting the
  interacting C++ opaque-path contracts: authored-paint opacity
  classification, no fixed-function blend for opaque paint, front-to-back
  scheduling when an opaque path overlaps an earlier unclipped advanced blend,
  and a post-destination-read submission that refreshes depth while preserving
  clip stencil. The focused four-draw prefix and full 369x781 Dawn comparison
  are byte-exact; all 230 enabled renderer tests, the workspace, the full
  renderer corpus, the normal 584-segment V2 floor, and the scripted
  35-segment floor pass. Spotify promotes under the unchanged `2/32` contract
  and advances the ratchet to exact=1,353/diverges=0/gated=115. Queue item 86
  is the bounded R3 exit audit before performance work begins.
- 2026-07-14: Closed R3 with a formal audit of all 115 retained gates. The
  strict Dawn inventory reclassifies the final 43 generic MSAA placeholders as
  41 gradient-paint replay gaps and two render-buffer replay gaps; combined
  with the already named rows, the manifest now has 43
  `strict-replay-gradient-paint`, three `strict-replay-render-buffer`, and zero
  `algorithm-core` gates. Every non-gated row passes, both R3 entry gates are
  closed, and `docs/renderer-r3-exit-audit.md` records the complete taxonomy.
  Queue item 87 starts R4 by wiring live same-backend benchmark runners.
- 2026-07-14: Corrected the R3 stop condition after reviewing what the 115
  named gates actually represent. Forty-seven are harness gaps, 58 are
  reviewed backend/decoder/precision boundaries, and ten remain substantive
  feature or parity debt. Added R3.1 before R4: close the ten, implement the two
  strict-replay capabilities that unlock 46 more rows, and leave only the 59
  reviewed platform limitations parked. The uncommitted R4 candidate-runner
  experiment was discarded cleanly; queue item 87 starts with Bullet Man's
  incompatible transformed clip rectangles.
- 2026-07-14: Closed R3.1 queue item 87 by porting C++'s transformed rectangle
  clip fallback. The optimized outer clip rect remains active while each
  incompatible rectangle enters the ordinary path-clip stack. Captured the
  missing mode-specific native Metal reference and promoted
  `riv-bullet_man-frame-0-clockwise-atomic` byte-exact under its unchanged
  `2/32` contract. All 231 enabled renderer tests and the full 1,468-row corpus
  pass at exact=1,354/diverges=0/gated=114; nine substantive R3.1 rows remain.
- 2026-07-14: Closed R3.1 queue item 88 by bisecting the stale
  `gm-beziers-msaa` gate against unchanged Dawn assets. The row changes from
  5,385 pixels/max 152 before `90c8fd52` to 8 pixels/max 3 after its dedicated
  C++ MSAA stroke depth state, whose duplicate-contour GPU test pins the actual
  self-overdraw behavior. Promoted under the unchanged `2/32` contract; the
  ratchet is exact=1,355/diverges=0/gated=113 with eight substantive rows left.
- 2026-07-14: Closed R3.1 queue item 89 by bisecting the stale
  `gm-cliprectintersections-msaa` gate against unchanged Dawn assets. The row
  moves from 240 pixels/max 55 before `90c8fd52` to byte-exact/max 1 after its
  dedicated MSAA stroke depth state. Promoted under the unchanged `2/32`
  contract with no tolerance change; the ratchet is
  exact=1,356/diverges=0/gated=112 and only seven substantive rows remain.
- 2026-07-14: Began R3.1 queue item 90 by capturing all seven missing native
  Metal clockwise-atomic references from upstream `7c778d13` and converting
  every retained row into a runnable pixel comparison. Ported C++'s
  pre-allocation off-frame draw cull,
  stroke/feather-expanded atomic coverage bounds, nested inverse-clip
  preflight, and empty handling for singular nested clips. `car_widgets_v01`
  and `echo_show_demo` advance from unsupported to measured output; all seven
  now reach pixels. The 235-test renderer suite and full corpus pass at
  exact=1,356/diverges=0/gated=112. `coin` is next at 48 differing pixels/max
  delta 58; no gate or tolerance changed in this slice.
- 2026-07-14: Adjudicated `coin` with eight native-Metal/Rust draw prefixes and
  a full-frame connected-component audit. The first over-budget prefix is the
  second clipped, zero-feather ring at 43 pixels/max 95; later authored layers
  overwrite it down to 48 pixels/max 58. The final outliers form 13
  one-pixel-wide path/clip-edge components, largest 12, with no interior color
  region. Reclassified the row from advanced-feather parity to the existing
  `metal-webgpu-subpixel-edge-coverage` boundary. The corpus remains
  exact=1,356/diverges=0/gated=112 with six substantive R3.1 rows left.
- 2026-07-14: Promoted `bankcard` after draw-prefix replay localized its
  1,485,510-pixel advanced-feather failure to mixed atomic draw ordering.
  Rust emitted every atlas blit before ordinary paths, changing the authored
  `path, path, atlas` sequence into `atlas, path, path`; the later full-frame
  paths therefore erased the feather contribution. Atomic mixed batches now
  preserve authored order while all-path batches retain their grouped pass.
  A focused path-to-atlas regression fails before the fix and passes after it.
  The native-Metal comparison falls to 22 pixels/max delta 18 and passes the
  unchanged `2/32` contract. The ratchet advances to
  exact=1,357/diverges=0/gated=111 with five substantive R3.1 rows left.
- 2026-07-14: Implemented strict gradient-paint reconstruction in the C++ Dawn
  stream compiler. Linear and radial declarations now retain exact endpoint,
  color, and stop literals; duplicate resources and undeclared shader use are
  rejected; paint shader transitions are reproduced without perturbing the
  legacy unshaded registry hash. The generated inventory moves all 43 gradient
  rejections to capture-ready, preserves five gated rows with valid strict
  provenance, and leaves only three render-buffer rows plus synthetic
  first-light unsupported.
  Added a dedicated general-atomic Hunter X replay and the required sampled
  target usage. It executes all 258 C++ Dawn batches without validation
  errors. C++ Dawn differs from native Metal across 874,951 pixels/max 40 and
  from Rust across 874,763/max 40; among Rust's 221 native-Metal outliers,
  Dawn is closer to Metal on 177 and Rust on 44. This does not justify a
  backend-boundary reclassification, so Hunter remains an advanced-feather
  parity gate. Renderer replay now supports a tested `--command-limit` for
  draw-prefix localization.
- 2026-07-14: Completed strict render-buffer and image-mesh reconstruction in
  the C++ Dawn stream compiler. Whole-buffer uploads enforce declaration type,
  exact size/bytes, one-time initialization flags, repeatable dynamic updates,
  and map/unmap lifetime; mesh draws enforce initialized typed capacities,
  canonical samplers, counts, and blend modes. Later RIV frames retain prior
  uploads. All three former render-buffer gates now compile from their real
  streams, moving the inventory to 46 capture-ready rows and one synthetic
  header-only unsupported row. The 46-test oracle format suite, 12 capture
  tests, inventory drift check, and C++ Dawn build pass. Queue item 91 now
  launches the single continuous 46-case capture and adjudication campaign.
- 2026-07-14: Closed queue item 91 with one continuous 732-case Dawn capture.
  All 686 prior PNGs are byte-identical; 46 new references carry the same
  registry provenance. Isolated Rust probes promote
  `riv-interactive_scrolling-frame-0-msaa` byte-exact and replace every old
  strict-replay gate with one of three executable renderer queues: 37 ordinary
  MSAA gradient paths, three MSAA image meshes, and five feathered-gradient
  advanced blends. The ratchet advances to exact=1,358/diverges=0/gated=110
  with no tolerance change; queue item 92 owns the 37-row gradient path.
- 2026-07-14: Ported C++ MSAA gradient-painted direct paths. Rust now renders
  and binds the shared ramp texture, emits gradient fill/stroke paint and
  auxiliary transforms, and accounts for shader destination reads. The
  gradient-only oracle is byte-exact and the complete 37-row sweep promotes
  17 entries under unchanged `2/32`, advancing the ratchet to
  exact=1,375/diverges=0/gated=93. The 20 measured residuals are split into
  repeated path clips, gradient destination reads, feathered gradient strokes,
  transformed clip rectangles, clipped/stroked composites, and a five-frame
  45-pixel edge residual. Queue item 93 ports the three image meshes next.
- 2026-07-14: Closed queue item 93 by porting C++'s generated MSAA image-mesh
  pipeline with typed position/UV/index buffers, sampler state, clip-distance
  and stencil clipping, fixed and advanced blends, and authored depth order.
  `gm-mesh-msaa` and `riv-tape-frame-0-msaa` promote under unchanged `2/32`
  contracts. A disposable five-prefix C++ Dawn oracle proves all 19 Jellyfish
  meshes stay within delta 2; its three later translucent image rectangles
  accumulate 3,691/max 3, 8,548/max 4, and 11,988/max 5. Jellyfish retains the
  concrete dither-accumulation precision gate, and the ratchet advances to
  exact=1,377/diverges=0/gated=91.
- 2026-07-14: Closed queue item 94 after a paired C++ Dawn/Rust prefix oracle
  found Hunter X's first real `2/32` failure at command 50. C++ rejects a
  986x1751 padded feather draw in its occupied 2048x2048 skyline, starts a new
  logical flush, resolves prior MSAA color, and reloads the resolved target
  into all four samples before replaying clips. Rust now applies the same MSAA
  atlas accounting and preserve draw. Command 50 falls from 1,257 pixels/max
  delta 41 to 128/max 1, while both the full 1,486-command diagnostic and the
  untouched Hunter corpus frame pass with zero over-threshold pixels/max 1.
  All five feathered-gradient rows pass unchanged contracts; Sol approved the
  patch with no findings. Full verification passes `make renderer-golden` at
  exact=1,402/diverges=0/gated=66/total=1,468, `make golden-compare` at 263
  exact files/584 exact segments, `make scripted-golden-compare` at 27 exact
  files/35 exact segments, `cargo test --workspace`, all 46 oracle-format
  tests, and the pinned 60-module/50-header shader reproducibility check.
- 2026-07-15: Fixed atomic HSL shader-feature selection from a Rewards
  command-prefix cliff. Command 538, a non-HSL atlas feather, resolves the
  pending full-frame Hue path from command 530; selecting the shader from only
  command 538 discarded the prior destination color. Atomic batches now use
  their combined HSL feature for direct-feather and atlas pipelines. A focused
  GPU regression fails before the fix and passes after it, while the Rewards
  native-Metal probe falls from 357,444 pixels/max delta 53 to 1,677/max 33.
  Sol approved the C++ parity and batching lifecycle with no findings. No
  manifest entry or tolerance changed. The 252 enabled renderer tests pass,
  the full corpus remains exact=1,402/diverges=0/gated=66, and the normal and
  scripted V2 floors remain 584 and 35 exact segments. Queue item 90 next
  adjudicates Rewards' remaining thin-edge residual before moving to the next
  open advanced-feather row.
- 2026-07-15: Integrated the production macOS CoreGraphics JPEG decoder and
  promoted the three previously image-decode-gated rows without changing their
  `2/32` contracts: `riv-clipping_and_draw_order-frame-0-msaa` is zero/max 0,
  `riv-data_binding_images_test-frame-0-clockwise-atomic` is zero pixels over
  threshold/max 2, and `riv-data_binding_images_test-frame-0-msaa` is zero/max
  0. The same-backend Metal checks use the existing committed references, and
  `make renderer-decoder-oracle` reports zero decode delta for the reachable
  JPEG. The renderer ratchet advances to exact=1,405/diverges=0/gated=63.
- 2026-07-15: Promoted exactly three repeated MSAA singleton captures after an
  independent Sol review rendered four fresh Rust wgpu/Metal rounds for each
  manifest-bound stream/frame/mode. `gm-dstreadshuffle-msaa` is 0 pixels over
  delta 2/max delta 1 at 530x690
  (`75b6e8bbfbba1f68f199a3da3b29ae78de42be1936f712d93717d7c45a37a67a`),
  `riv-jellyfish_test-frame-0-msaa` is 0/max 1 at 2080x2080
  (`5f1592aa67826fcd9ee05a8e32885490167566c68899693298b8c341061bb781`),
  and `gm-strokes_poly-msaa` is 12/max 46 at 400x400
  (`f17f02cb1dfdaa3218049d2161793518b3f11c0c953c3debb981615d3923e825`).
  All outputs are valid 8-bit RGBA non-interlaced PNGs and byte-stable across
  the fresh rounds. Existing stream/reference provenance and `2/32` contracts
  remain unchanged. The corpus is exact=1,408/diverges=0/gated=60.
- 2026-07-15: Reclassified `riv-hunter_x_demo-frame-0-clockwise-atomic` after
  paired native-Metal prefix and preparation oracles narrowed its unchanged
  222-pixel/max-delta-18 residual to one-pixel feather edges. The first
  residual is command 1,378, an Overlay atlas draw; C++ and Rust agree on the
  atlas threshold, allocation, batch schedule, and CPU preparation semantics,
  and component analysis found only one alpha outlier beyond delta 2. The row
  therefore moves from the provisional advanced-feather diagnostic to the
  existing `metal-webgpu-subpixel-edge-coverage` boundary without changing
  status, reference, tolerance, or the exact=1,408/diverges=0/gated=60
  ratchet. Four actionable advanced-feather rows remain.
- 2026-07-15: Reclassified `riv-echo_show_demo-frame-0-clockwise-atomic`
  after a 16-prefix native-Metal/Rust campaign and an independent fresh replay
  disproved the advanced-feather-only diagnosis. The first contract failure is
  command 16, a clipped opaque unfeathered NonZero SrcOver path at 34 pixels
  over delta 2/max delta 5; all-SrcOver plus zero-feather controls still fail,
  and later unfeathered Screen draws create the largest cliffs. Because no
  same-backend C++ WebGPU clockwise-atomic artifact exists, clip-edge,
  destination-color accumulation, and resolve scheduling remain live Rust
  defect hypotheses. The row keeps its status, reference, and `2/32` contract
  under the narrower actionable
  `native-clockwise-atomic-clip-edge-and-composite-parity` diagnostic. Counts
  remain exact=1,408/diverges=0/gated=60; three advanced-feather rows remain.
- 2026-07-15: Split the accepted Rewards production fix from its rejected
  command-21 artifact harness. C++ and Rust command-16 preparation agree, but
  submitting a positive interior triangle's full 1,024 winding weight as one
  packed clockwise-atomic delta loses that contribution at the reserved
  coverage prefix. Rust now submits unclipped positive weights as bounded
  unit-weight instances, coalescing ordinary unit-weight interiors into one
  draw while leaving clipped and borrowed triangles unchanged. The configured
  C++ command-16 blit is 0 pixels/max delta 1 and the full native-Metal frame
  improves from 1,677/max 33 to 1,575/max 33. No status, reference, or
  tolerance changed. The renderer remains exact=1,408/diverges=0/gated=60;
  all 256 enabled renderer tests, the full workspace, normal 584-segment V2
  floor, scripted 35-segment floor, and 1,468-row renderer corpus pass.
- 2026-07-15: Closed queue item 105 with a fresh empty-directory Rewards
  command-21 recapture from the pinned C++ Dawn executable. All nine artifact
  hashes reproduced exactly. CPU spans, preparation, and the unclipped control
  agree; the clipped output's 254 over-threshold pixels all lie on 802 sparse
  clip-plane edge words, while normalized coverage differs at only six
  unrelated words. The full native residual is 1,575/max 33 across 1,517 tiny
  components, largest six. Rewards moves to the existing
  `metal-webgpu-subpixel-edge-coverage` boundary without changing status,
  reference, tolerance, or exact=1,408/diverges=0/gated=60. The rejected mixed
  artifact harness remains out of production; evidence is in
  `docs/renderer-rewards-command21-audit.md`.
- 2026-07-15: Revalidated subpixel-edge cohort A, the first 13 lexical
  `metal-webgpu-subpixel-edge-coverage` rows after excluding Rewards. Three
  Rust wgpu/Metal rounds were contract-stable, all 13 native C++/Metal
  controls were independently byte-exact to their committed references, and
  the residuals remain sparse boundary components with no shared falsifiable
  Rust defect. No status, reference, or tolerance changed; the ratchet stays
  exact=1,408/diverges=0/gated=60. Full results are in
  `docs/renderer-r3-exit-audit.md`.
- 2026-07-15: Revalidated subpixel-edge cohort B's next 13 rows. Three Rust
  wgpu/Metal rounds were mask-stable and all 13 native C++/Metal controls were
  independently byte-exact. Twelve rows form an exact-alpha sparse-edge
  cluster; Hunter X remains a distinct advanced-blend boundary with no proved
  Rust defect. No status, reference, tolerance, or ratchet count changed.
  Full evidence is in `docs/renderer-r3-1-subpixel-edge-cohort-b-audit.md`.
- 2026-07-15: Revalidated R3.1 subpixel-edge cohort C's next 13 rows. Three
  fresh serial Rust wgpu/Metal rounds were stable below delta 2; all 13 native
  C++/Metal FFI controls were independently byte-exact to their committed
  references. The sole decoded-image stream also passes the decoder oracle.
  Residuals are RGB-only sparse boundary components with no shared falsifiable
  Rust defect, so no status, reference, tolerance, or ratchet count changed.
  Full evidence is in `docs/renderer-r3-exit-audit.md`.
- 2026-07-15: Revalidated subpixel-edge cohort D's 12 assigned text and
  state-machine rows. Three serial Rust wgpu/Metal rounds remained below the
  threshold between rounds, all native C++/Metal controls were decoded-exact,
  and the exact-alpha residuals are sparse subpixel contours across both two
  clipped and ten unclipped SrcOver-only streams. No shared falsifiable Rust
  defect, tolerance, reference, status, or ratchet change; all 12 retain
  `metal-webgpu-subpixel-edge-coverage`. Evidence is in
  `docs/renderer-r3-exit-audit.md`.
- 2026-07-15: Closed queue item 101 with a strict header-only First Light Dawn
  replay and mixed-schema read-only provenance validation. The 732 existing
  captures keep their pinned legacy registry identity; new captures publish a
  case-local identity, avoiding a bulk migration and its unrelated transaction
  subsystem. The captured C++ Dawn and Rust wgpu 64x64 outputs are byte-exact
  under the unchanged `2/0` contract. The ratchet advances to
  exact=1,409/diverges=0/gated=59; Car Widgets is next.
- 2026-07-15: Matched C++'s flush-wide `ENABLE_DITHER` feature for advanced
  general-atomic draws. Rust previously disabled the branch on non-feather
  paths, interiors, and images even though one advanced draw promotes the
  entire C++ flush to the combined shader feature set. The corrected shared
  pipeline contract moves same-backend Echo command 39 from 36/max 4 to
  0/max 1 and command 104 from 9,577/max 5 to 0/max 1. Against native Metal,
  the full residuals fall from 65,522 to 17,854 pixels for Car Widgets,
  12,118 to 2,054 for Data Viz, and 67,300 to 10,016 for Echo Show; their max
  deltas remain 96, 99, and 54. No reference, tolerance, status, or corpus
  count changes yet. Queue items 102-104 now require full or first-failing
  same-backend evidence before another renderer change.
- 2026-07-15: Closed R3.1 queue items 90 and 102-104 with full-stream C++
  Dawn/Rust wgpu evidence. Data Viz passes `2/32` at 22/max 3 and joins the
  reviewed Metal/WebGPU edge boundary. Echo's 511-command frame is 96/max 3;
  its first failing command has an exact 230,896-word coverage transition.
  Car Widgets is 10,872/max 13; both isolated cliffs retain matching coverage
  or clip masks. Car and Echo therefore share the concrete
  `rust-wgpu-atomic-color-plane-lifetime-parity` finding. No renderer code,
  reference, tolerance, or status changed; the honest ratchet remains
  exact=1,409/diverges=0/gated=59. All R3.1 exit criteria now hold and queue
  item 106 starts R4's live same-backend benchmark runner slice. The closing
  verification passes the full renderer corpus, normal V2 floor at 584 exact
  segments, scripted V2 floor at 35 exact segments, and
  `cargo test --workspace`.
