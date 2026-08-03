# Corpus-density report

## Outcome

The corpus refresh ran first:

```text
rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/
make fixtures
```

V2 densification changed all 226 animated rows that had only `t=0` to retain `t=0`, an authored midpoint, and an authored cycle boundary. The densifier prefers a looping animation on the selected artboard; when a file has no authored loop it uses the finite animation boundary, and it falls back to nested/library artboard animations when the selected artboard has no local timeline. No tolerance was widened and no added sample was removed.

The dense scripted comparison exposed 30 previously hidden post-zero failures: 27 rows now carry `status = "diverges"`, and three rows whose Rust stream cannot complete carry `status = "not-yet"`. The four scripting rows with pre-existing `scripted-status:exact` overrides had those overrides removed so their V25/V30/V32/V38 divergent status is actually enforced.

Of the 31 rows that started `not-yet`, 24 are now enrolled as exact and seven retain a concrete blocker. Final corpus state is 356 entries: 319 exact, 27 diverges, and 10 not-yet. The green scripted run reports 1,051 exact segments and 1,051 exact side-channel segments.

The scorecard `exact-segments` floor remains 673. It was not ratcheted because the corpus is not all green.

## Newly exposed V2 findings

| Register | Corpus entry | Concrete finding |
|---|---|---|
| V11 | `global_variables_test` | Rust misses the second transition; draw structure also diverges at 0.5/1.0. |
| V12 | `db_health_tracker` | Rust consumes a CPU and does not return within 180 seconds after t=0. |
| V13 | `animated_clipping` | Rust emits an empty path where C++ emits the animated compound clip. |
| V14 | `artboard_list_overrides` | Rust clip height is 724; C++ is 1074. |
| V15 | `bad_skin` | Command/side-channel phase differs at t=2 under the existing tolerance. |
| V16 | `bankcard` | Compound path 58 geometry differs at the 2.0-second boundary. |
| V17 | `bullet_man` | Animated stroke endpoints differ materially. |
| V18 | `clipping_and_draw_order` | Rust emits translation (1121,259); C++ emits identity. |
| V19 | `component_list_child_origin` | Post-zero paint/advance command phase differs. |
| V20 | `component_stateful_vm_instance` | First ellipse radius is 30 in Rust and 50 in C++. |
| V21 | `component_stateful_vm_instance_2` | Animated scale is -1.5 in Rust and +1.5 in C++. |
| V22 | `computed_values_test` | Rust clip is 245×250; C++ is 490×362.5. |
| V23 | `death_knight` | Gradient/advance command phase differs after t=0. |
| V24 | `echo_show_demo` | Rust exits with missing render paint global 584. |
| V25 | `group_effect` | Post-zero compound path geometry differs. |
| V26 | `hunter_x_demo` | Gradient 189/advance command phase differs. |
| V27 | `image_fit_alignment_2` | Image vertex-buffer command ordering differs. |
| V28 | `multi_listeners` | C++ reports `main-event-2` at delay 0.183333337; Rust does not. |
| V29 | `new_text` | Rust exits with missing render paint global 71. |
| V30 | `path_effect_with_feathers` | Effected path 5 geometry differs. |
| V31 | `rewards_demo` | Gradient 51/advance command phase differs. |
| V32 | `scripted_as_path` | Rust path 7 is empty; C++ emits the authored closed shape. |
| V33 | `stateful_keyed_trigger` | Rust remains red while C++ applies green. |
| V34 | `stateful_nested` | Nested stateful path 3 geometry differs. |
| V35 | `stateful_source_switch` | Rust size is 100; C++ is 75. |
| V36 | `superbowl` | Rust path 101 is empty where C++ emits compound path 61. |
| V37 | `text_vertical_trim_test` | Rust y is 182.76001; C++ is 177.935791. |
| V38 | `viewmodel_instance_to_artboard` | VM-selected nested artboard path 3 geometry differs. |
| V39 | `virtualize_blendmode` | Paint 17/advance command phase differs at t=2. |
| V40 | `zombie_skins` | Gradient 30/advance command phase differs at t=1. |

Full reproduction and exit gates are in `docs/parity-gap-register.md`.

## Original 31 not-yet entries

| Corpus entry | Outcome | Evidence or blocker |
|---|---|---|
| `semantic_data_binding_action_lt1` | blocked (LT-1) | Rust emits a radial gradient where C++ creates render paint. |
| `semantic_text_inference_lt1` | blocked (LT-1) | Rust runner lacks `layout-component-paint` global 347. |
| `databind_null_artboard_swap` | exact | Exact at t=0. |
| `bidirectional_stateful_property` | exact | Exact at 0/0.5/1 under semantic comparison. |
| `paused_nested_artboard_opacity` | blocked (V41) | Rust alpha `0xf7`; C++ alpha `0xff`. |
| `solo_index_test` | blocked (F6) | Rust semantic root bounds (-44,-44,437,341); C++ (333,263,437,341). |
| `stateful_component_image_test` | blocked (V42) | Rust emits `decodeImage` before the C++ paint position. |
| `component_list_clipped_viewport` | exact | Exact at t=0. |
| `vm_listener_fire_event` | exact | Exact at t=0. |
| `image_computed_transform_bind` | exact | Exact at 0/0.5/1. |
| `animated_cubic_participant` | exact | S4 fixture enrolled at t=0. |
| `animated_participant` | exact | S4 fixture enrolled at t=0. |
| `constrained_participant` | exact | S4 fixture enrolled at t=0. |
| `display_none_participant` | exact | S4 fixture enrolled at t=0. |
| `fixed_participant` | exact | S4 fixture enrolled at t=0. |
| `grid_2x2` | exact | S4 fixture enrolled at t=0. |
| `grid_auto_rows` | exact | S4 fixture enrolled at t=0. |
| `grid_participant` | exact | S4 fixture enrolled at t=0. |
| `grid_track_types` | exact | S4 fixture enrolled at t=0. |
| `group_participant` | exact | S4 fixture enrolled at t=0. |
| `hug_participant` | exact | S4 fixture enrolled at t=0. |
| `list_in_group_joins_layout` | exact | S4 fixture enrolled at t=0. |
| `nested_group_participant` | exact | S4 fixture enrolled at t=0. |
| `solo_participant` | exact | S4 fixture enrolled at t=0. |
| `stack` | exact | S4 fixture enrolled at t=0. |
| `stack_participant` | exact | S4 fixture enrolled at t=0. |
| `styled_flex` | exact | S4 fixture enrolled at t=0. |
| `layout_grid_stack` | exact | Exact at 0/1/2. |
| `data_bind_blob_test` | blocked (V43) | Rust rectangle height 2098.35938; C++ 926.574219. |
| `layout_text_match` | exact | Exact at 0/0.5/1. |
| `artboard_opacity_and_transform_test` | blocked (V44) | Rust runner lacks nested-child binding for data-bind global 29. |

## All 226 densified rows

This appendix records each changed row's retained samples and current corpus outcome. Four rows in this set were already among the original 31 pending entries; their blockers are described above.

| Corpus entry | Samples | Outcome |
|---|---|---|
| `ai_assitant` | `[0.0, 0.5, 1.0]` | exact |
| `align_target` | `[0.0, 0.5, 1.0]` | exact |
| `animated_clipping` | `[0.0, 0.5, 1.0]` | diverges (V13) |
| `artboard_list_map_rules` | `[0.0, 0.5, 1.0]` | exact |
| `artboard_list_overrides` | `[0.0, 0.5, 1.0]` | diverges (V14) |
| `artboard_width_test` | `[0.0, 0.5, 1.0]` | exact |
| `audio_script` | `[0.0, 0.5, 1.0]` | exact |
| `background_measure` | `[0.0, 0.5, 1.0]` | exact |
| `bad_skin` | `[0.0, 2.0, 4.0]` | diverges (V15) |
| `ball_test` | `[0.0, 0.5, 1.0]` | exact |
| `bankcard` | `[0.0, 1.0, 2.0]` | diverges (V16) |
| `bidirectional_binding_source` | `[0.0, 0.008333, 0.016667]` | exact |
| `bidirectional_precedence` | `[0.0, 0.5, 1.0]` | exact |
| `bindable_artboard_nesty` | `[0.0, 0.5, 1.0]` | exact |
| `bindable_focus_tree_swap` | `[0.0, 0.5, 1.0]` | exact |
| `bullet_man` | `[0.0, 0.5, 1.0]` | diverges (V17) |
| `car_widgets_v01` | `[0.0, 0.5, 1.0]` | exact |
| `clipping_and_draw_order` | `[0.0, 0.5, 1.0]` | diverges (V18) |
| `coin` | `[0.0, 0.5, 1.0]` | exact |
| `collapsable_data_binding` | `[0.0, 0.5, 1.0]` | exact |
| `collapse_data_binds` | `[0.0, 0.5, 1.0]` | exact |
| `collapsing_elements` | `[0.0, 2.0, 4.0]` | exact |
| `component_list_1` | `[0.0, 0.5, 1.0]` | exact |
| `component_list_child_origin` | `[0.0, 0.5, 1.0]` | diverges (V19) |
| `component_list_follow_path` | `[0.0, 0.5, 1.0]` | exact |
| `component_list_follow_path_distance` | `[0.0, 0.5, 1.0]` | exact |
| `component_list_virtualized` | `[0.0, 0.5, 1.0]` | exact |
| `component_stateful` | `[0.0, 0.5, 1.0]` | exact |
| `component_stateful_vm_instance` | `[0.0, 0.5, 1.0]` | diverges (V20) |
| `component_stateful_vm_instance_2` | `[0.0, 0.5, 1.0]` | diverges (V21) |
| `computed_root_transform` | `[0.0, 0.5, 1.0]` | exact |
| `computed_values_test` | `[0.0, 0.5, 1.0]` | diverges (V22) |
| `custom_property_enum` | `[0.0, 0.5, 1.0]` | exact |
| `custom_property_trigger` | `[0.0, 0.5, 1.0]` | exact |
| `data_bind_artboard_input` | `[0.0, 0.5, 1.0]` | exact |
| `data_bind_font_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_bind_keyframes_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_bind_test_cmdq` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_artboards_source_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_artboards_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_images_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_test` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_test_3` | `[0.0, 0.5, 1.0]` | exact |
| `data_binding_test_triggers` | `[0.0, 0.5, 1.0]` | exact |
| `data_converter_interpolator_reset` | `[0.0, 0.5, 1.0]` | exact |
| `data_converter_to_number` | `[0.0, 1.0, 2.0]` | exact |
| `data_global_repro` | `[0.0, 0.5, 1.0]` | exact |
| `data_viz_demo` | `[0.0, 1.0, 2.0]` | exact |
| `databind_artboard` | `[0.0, 0.5, 1.0]` | exact |
| `databind_external_artboard_child` | `[0.0, 0.5, 1.0]` | exact |
| `databind_external_artboard_main` | `[0.0, 0.5, 1.0]` | exact |
| `databind_solo_to_enum` | `[0.0, 0.5, 1.0]` | exact |
| `db_health_tracker` | `[0.0, 0.5, 1.0]` | not-yet (V12) |
| `death_knight` | `[0.0, 0.5, 1.0]` | diverges (V23) |
| `deterministic_mode` | `[0.0, 0.5, 1.0]` | exact |
| `distance_constraint` | `[0.0, 0.5, 1.0]` | exact |
| `double_line` | `[0.0, 0.5, 1.0]` | exact |
| `drag_event` | `[0.0, 0.5, 1.0]` | exact |
| `draw_index_list` | `[0.0, 0.5, 1.0]` | exact |
| `echo_show_demo` | `[0.0, 0.5, 1.0]` | not-yet (V24) |
| `ellipsis` | `[0.0, 1.5, 3.0]` | exact |
| `entry` | `[0.0, 0.5, 1.0]` | exact |
| `feather_render_test` | `[0.0, 0.5, 1.0]` | exact |
| `fit_font_size_test` | `[0.0, 0.008333, 0.016667]` | exact |
| `focus_collapsing` | `[0.0, 0.5, 1.0]` | exact |
| `focus_traversal` | `[0.0, 0.5, 1.0]` | exact |
| `focusable_element` | `[0.0, 0.5, 1.0]` | exact |
| `follow_path` | `[0.0, 0.5, 1.0]` | exact |
| `follow_path_constraint` | `[0.0, 0.5, 1.0]` | exact |
| `follow_path_path` | `[0.0, 2.0, 4.0]` | exact |
| `follow_path_path_0_opacity` | `[0.0, 0.5, 1.0]` | exact |
| `follow_path_shapes` | `[0.0, 0.5, 1.0]` | exact |
| `follow_path_solos` | `[0.0, 2.0, 4.0]` | exact |
| `follow_path_with_0_opacity` | `[0.0, 0.5, 1.0]` | exact |
| `format_number_with_commas` | `[0.0, 0.5, 1.0]` | exact |
| `formula_random` | `[0.0, 0.5, 1.0]` | exact |
| `global_variables_test` | `[0.0, 0.5, 1.0]` | diverges (V11) |
| `global_viewmodels_test` | `[0.0, 0.5, 1.0]` | exact |
| `group_effect` | `[0.0, 0.5, 1.0]` | diverges (V25) |
| `hello_world` | `[0.0, 0.5, 1.0]` | exact |
| `hide_test` | `[0.0, 0.5, 1.0]` | exact |
| `hit_test_nested` | `[0.0, 0.5, 1.0]` | exact |
| `hit_test_test` | `[0.0, 0.5, 1.0]` | exact |
| `hittest_collapsed_layouts` | `[0.0, 0.5, 1.0]` | exact |
| `hosted_font_file` | `[0.0, 0.5, 1.0]` | exact |
| `hosted_image_file` | `[0.0, 0.5, 1.0]` | exact |
| `hunter_x_demo` | `[0.0, 0.5, 1.0]` | diverges (V26) |
| `image_binding_with_listener` | `[0.0, 0.5, 1.0]` | exact |
| `image_fit_alignment` | `[0.0, 0.5, 1.0]` | exact |
| `image_fit_alignment_2` | `[0.0, 0.5, 1.0]` | diverges (V27) |
| `image_fit_alignment_3` | `[0.0, 0.5, 1.0]` | exact |
| `image_fit_alignment_updated_test` | `[0.0, 0.5, 1.0]` | exact |
| `image_scripting_property_value` | `[0.0, 0.5, 1.0]` | exact |
| `in_band_asset` | `[0.0, 0.5, 1.0]` | exact |
| `interactive_scrolling` | `[0.0, 0.5, 1.0]` | exact |
| `interpolate_to_end` | `[0.0, 0.5, 1.0]` | exact |
| `interpolation_zero_duration` | `[0.0, 0.5, 1.0]` | exact |
| `jellyfish_test` | `[0.0, 0.991667, 1.983333]` | exact |
| `joel_v3` | `[0.0, 0.083333, 0.166667]` | exact |
| `keyboard_listener` | `[0.0, 0.5, 1.0]` | exact |
| `library` | `[0.0, 0.5, 1.0]` | exact |
| `library_export_animation_test` | `[0.0, 0.5, 1.0]` | exact |
| `library_view_model_test` | `[0.0, 0.5, 1.0]` | exact |
| `library_vmtest_1_host` | `[0.0, 0.5, 1.0]` | exact |
| `library_with_text_and_image` | `[0.0, 0.5, 1.0]` | exact |
| `list_focus_order` | `[0.0, 0.5, 1.0]` | exact |
| `list_index_script_access` | `[0.0, 0.5, 1.0]` | exact |
| `list_items` | `[0.0, 0.5, 1.0]` | exact |
| `list_to_length_test` | `[0.0, 0.5, 1.0]` | exact |
| `listener_action_inputs` | `[0.0, 0.5, 1.0]` | exact |
| `listener_view_model` | `[0.0, 0.5, 1.0]` | exact |
| `local_bounds` | `[0.0, 0.5, 1.0]` | exact |
| `magic_alley_db_reduced_export` | `[0.0, 0.5, 1.0]` | exact |
| `modifier_test` | `[0.0, 0.5, 1.0]` | exact |
| `modifier_to_run` | `[0.0, 0.5, 1.0]` | exact |
| `multi_listeners` | `[0.0, 0.5, 1.0]` | diverges (V28) |
| `multitouch` | `[0.0, 0.5, 1.0]` | exact |
| `multitouch_enter` | `[0.0, 0.5, 1.0]` | exact |
| `n_slice_triangle` | `[0.0, 0.5, 1.0]` | exact |
| `nested_artboard_origin_override_test` | `[0.0, 0.5, 1.0]` | exact |
| `nested_event_test` | `[0.0, 0.5, 1.0]` | exact |
| `nested_events` | `[0.0, 0.5, 1.0]` | exact |
| `nested_hug` | `[0.0, 0.5, 1.0]` | exact |
| `nested_needs_advance` | `[0.0, 0.5, 1.0]` | exact |
| `new_text` | `[0.0, 0.5, 1.0]` | not-yet (V29) |
| `number_to_list_nested_children` | `[0.0, 0.5, 1.0]` | exact |
| `path_effect_with_feathers` | `[0.0, 0.5, 1.0]` | diverges (V30) |
| `pause_nested_artboard` | `[0.0, 0.5, 1.0]` | exact |
| `pointer_exit` | `[0.0, 0.5, 1.0]` | exact |
| `rebind_with_nested_viewmodel` | `[0.0, 0.5, 1.0]` | exact |
| `recursive_data_bind` | `[0.0, 0.5, 1.0]` | exact |
| `relative_data_binding` | `[0.0, 0.5, 1.0]` | exact |
| `replace_vm_instance` | `[0.0, 0.5, 1.0]` | exact |
| `reset_phase` | `[0.0, 0.5, 1.0]` | exact |
| `reset_shared_viewmodel_instance_test` | `[0.0, 0.5, 1.0]` | exact |
| `reuse_path_in_effect` | `[0.0, 0.5, 1.0]` | exact |
| `rewards_demo` | `[0.0, 0.5, 1.0]` | diverges (V31) |
| `runtime_nested_text_runs` | `[0.0, 0.5, 1.0]` | exact |
| `saturation` | `[0.0, 0.5, 1.0]` | exact |
| `scope_probe` | `[0.0, 0.5, 1.0]` | exact |
| `script_advance_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_affects_has_changed` | `[0.0, 5.0, 10.0]` | exact |
| `script_artboard_opacity_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_artboard_origin_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_artboard_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_color_data_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_create_viewmodel_instance` | `[0.0, 0.5, 1.0]` | exact |
| `script_dependency_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_dependency_test2` | `[0.0, 0.5, 1.0]` | exact |
| `script_dependency_test_using_library` | `[0.0, 0.5, 1.0]` | exact |
| `script_dependency_test_using_library_v2` | `[0.0, 0.5, 1.0]` | exact |
| `script_inputs_test_1` | `[0.0, 0.5, 1.0]` | exact |
| `script_layout_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_namespace_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_path_effects_test` | `[0.0, 0.5, 1.0]` | exact |
| `script_string_converter_test` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_as_path` | `[0.0, 0.5, 1.0]` | diverges (V32) |
| `scripted_color` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_data_context` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_listener_action` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_listener_context` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_memory_leak` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_property_image` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_transition_condition` | `[0.0, 0.5, 1.0]` | exact |
| `scripted_viewmodel_cache` | `[0.0, 0.5, 1.0]` | exact |
| `scripting_root_viewmodel` | `[0.0, 0.5, 1.0]` | exact |
| `scroll_intent` | `[0.0, 0.5, 1.0]` | exact |
| `scroll_snap` | `[0.0, 0.5, 1.0]` | exact |
| `scroll_test` | `[0.0, 0.5, 1.0]` | exact |
| `scroll_threshold` | `[0.0, 0.5, 1.0]` | exact |
| `shared_viewmodel_instance` | `[0.0, 0.5, 1.0]` | exact |
| `smi_test` | `[0.0, 0.5, 1.0]` | exact |
| `solid_affects_has_changed` | `[0.0, 0.5, 1.0]` | exact |
| `sorted_listeners` | `[0.0, 0.5, 1.0]` | exact |
| `spotify_kids_app_icon` | `[0.0, 0.541667, 1.083333]` | exact |
| `spotify_kids_demo` | `[0.0, 0.25, 0.5]` | exact |
| `state_transition_fire_trigger` | `[0.0, 0.5, 1.0]` | exact |
| `stateful_artboard_swap` | `[0.0, 0.5, 1.0]` | exact |
| `stateful_keyed_trigger` | `[0.0, 0.5, 1.0]` | diverges (V33) |
| `stateful_multi_property` | `[0.0, 0.5, 1.0]` | exact |
| `stateful_nested` | `[0.0, 0.5, 1.0]` | diverges (V34) |
| `stateful_source_switch` | `[0.0, 0.5, 1.0]` | diverges (V35) |
| `superbowl` | `[0.0, 0.5, 1.0]` | diverges (V36) |
| `swappable_artboards_focus` | `[0.0, 0.5, 1.0]` | exact |
| `tape` | `[0.0, 1.0, 2.0]` | exact |
| `target_event` | `[0.0, 0.5, 1.0]` | exact |
| `test_modifier_run` | `[0.0, 0.5, 1.0]` | exact |
| `text_feather_falloff` | `[0.0, 0.5, 1.0]` | exact |
| `text_follow_path_shape_length` | `[0.0, 0.5, 1.0]` | exact |
| `text_input` | `[0.0, 0.5, 1.0]` | exact |
| `text_listener_simpler` | `[0.0, 0.5, 1.0]` | exact |
| `text_opacity_modifier` | `[0.0, 0.5, 1.0]` | exact |
| `text_stroke_test` | `[0.0, 0.5, 1.0]` | exact |
| `text_vertical_trim_test` | `[0.0, 0.5, 1.0]` | diverges (V37) |
| `time_based_interpolation` | `[0.0, 0.5, 1.0]` | exact |
| `transition_actions` | `[0.0, 0.25, 0.5]` | exact |
| `transition_artboard_condition_test` | `[0.0, 0.5, 1.0]` | exact |
| `transition_duration_bind_list` | `[0.0, 0.5, 1.0]` | exact |
| `transition_duration_bind_nested` | `[0.0, 0.5, 1.0]` | exact |
| `transition_index_condition` | `[0.0, 0.5, 1.0]` | exact |
| `transition_self_comparator_test` | `[0.0, 0.5, 1.0]` | exact |
| `trigger_based_listeners` | `[0.0, 0.5, 1.0]` | exact |
| `trigger_fires_single_change` | `[0.0, 0.5, 1.0]` | exact |
| `two_bone_ik` | `[0.0, 0.5, 1.0]` | exact |
| `unbound_stateful_component` | `[0.0, 0.5, 1.0]` | exact |
| `vertical_align_ellipsis` | `[0.0, 0.5, 1.0]` | exact |
| `viewmodel_access` | `[0.0, 0.5, 1.0]` | exact |
| `viewmodel_based_condition` | `[0.0, 0.5, 1.0]` | exact |
| `viewmodel_from_context` | `[0.0, 0.5, 1.0]` | exact |
| `viewmodel_image_reset` | `[0.0, 0.5, 1.0]` | exact |
| `viewmodel_instance_to_artboard` | `[0.0, 0.5, 1.0]` | diverges (V38) |
| `viewmodel_list_trigger` | `[0.0, 0.5, 1.0]` | exact |
| `virtualize_blendmode` | `[0.0, 2.0, 4.0]` | diverges (V39) |
| `virtualized_artboard_databound_children` | `[0.0, 0.5, 1.0]` | exact |
| `walle` | `[0.0, 0.5, 1.0]` | exact |
| `word_joiner_test` | `[0.0, 0.5, 1.0]` | exact |
| `zero_width_space_line_break` | `[0.0, 0.5, 1.0]` | exact |
| `zombie_skins` | `[0.0, 1.0, 2.0]` | diverges (V40) |
| `bidirectional_stateful_property` | `[0.0, 0.5, 1.0]` | exact |
| `paused_nested_artboard_opacity` | `[0.0, 0.5, 1.0]` | not-yet (V41) |
| `solo_index_test` | `[0.0, 0.5, 1.0]` | not-yet (F6) |
| `stateful_component_image_test` | `[0.0, 0.5, 1.0]` | not-yet (V42) |
| `image_computed_transform_bind` | `[0.0, 0.5, 1.0]` | exact |
| `layout_grid_stack` | `[0.0, 1.0, 2.0]` | exact |
| `layout_text_match` | `[0.0, 0.5, 1.0]` | exact |
| `artboard_opacity_and_transform_test` | `[0.0, 0.5, 1.0]` | not-yet (V44) |

## Gates

- `make scripted-golden-compare`: PASS — 356 entries; 319 exact; 1,051 exact segments; 27 named divergences; 10 named not-yet rows.
- `make silver-corpus-test`: PASS — 21 library tests, FL-E8 differential, 3 passed/1 documented ignore in frame-loop backfill, and 19 generator tests.
- `make check`: PASS — workspace compiles (existing warnings remain non-fatal).
- `make parity-scorecard-test`: PASS — 26 tests.
- `cargo test -p golden-compare`: PASS — 20 tests.
- `rustfmt --edition 2024 --check tools/golden-compare/src/bin/densify-corpus.rs`: PASS.
- `git diff --check`: PASS.
- Densifier dry run: PASS — `densified=0`.
