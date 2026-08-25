# Runtime drift queue

Generated from the checked-in parity ledgers. JSON is authoritative; this view highlights clusters and the highest-discovery candidates.

- Upstream ref: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
- Candidates: 699
- Clusters: 32

## Dispositions

| disposition | candidates |
|---|---:|
| extension | 3 |
| intentional-decision | 14 |
| known-divergence | 102 |
| pending-proof | 95 |
| stale-proof | 446 |
| unknown | 4 |
| unsupported | 35 |

## Filter fields

Filter the JSON `candidates` array by `owner_family`, `subsystem`, `evidence_state`, `disposition`, or sort by descending `discovery_value`. The complete deterministic value sets are in `filters`.

## Clusters

| cluster | boundary | owner family | candidates | max discovery value |
|---|---|---|---:|---:|
| `cluster:layout:ownership` | ownership | layout | 14 | 128 |
| `cluster:runtime-tests:lifecycle` | lifecycle | runtime-tests | 73 | 120 |
| `cluster:runtime-tests:ordering` | ordering | runtime-tests | 20 | 120 |
| `cluster:unresolved:ownership` | ownership | unresolved | 43 | 120 |
| `cluster:runtime-tests:ownership` | ownership | runtime-tests | 38 | 105 |
| `cluster:animation:ownership` | ownership | animation | 93 | 103 |
| `cluster:data_bind:ownership` | ownership | data_bind | 45 | 103 |
| `cluster:unresolved:float-behavior` | float-behavior | unresolved | 3 | 100 |
| `cluster:unresolved:invalidation` | invalidation | unresolved | 3 | 100 |
| `cluster:unresolved:lifecycle` | lifecycle | unresolved | 5 | 100 |
| `cluster:unresolved:mutation` | mutation | unresolved | 7 | 100 |
| `cluster:unresolved:ordering` | ordering | unresolved | 16 | 100 |
| `cluster:viewmodel:ownership` | ownership | viewmodel | 39 | 93 |
| `cluster:lua:ownership` | ownership | lua | 29 | 90 |
| `cluster:runtime-core:ownership` | ownership | runtime-core | 51 | 90 |
| `cluster:scripted:ownership` | ownership | scripted | 6 | 90 |
| `cluster:shapes:ownership` | ownership | shapes | 44 | 90 |
| `cluster:text:ownership` | ownership | text | 30 | 90 |
| `cluster:core:ownership` | ownership | core | 10 | 83 |
| `cluster:assets:ownership` | ownership | assets | 10 | 80 |
| `cluster:importers:ownership` | ownership | importers | 26 | 80 |
| `cluster:input:ownership` | ownership | input | 4 | 80 |
| `cluster:inputs:ownership` | ownership | inputs | 3 | 80 |
| `cluster:runtime-tests:unsupported-observable` | unsupported-observable | runtime-tests | 31 | 80 |
| `cluster:unresolved:unsupported-observable` | unsupported-observable | unresolved | 4 | 75 |
| `cluster:audio:ownership` | ownership | audio | 4 | 70 |
| `cluster:bones:ownership` | ownership | bones | 6 | 70 |
| `cluster:constraints:ownership` | ownership | constraints | 21 | 70 |
| `cluster:math:ownership` | ownership | math | 14 | 70 |
| `cluster:async:ownership` | ownership | async | 1 | 60 |
| `cluster:profiler:ownership` | ownership | profiler | 2 | 60 |
| `cluster:semantic:ownership` | ownership | semantic | 4 | 60 |

## Highest-discovery candidates

| value | candidate | disposition | owner | first signal |
|---:|---|---|---|---|
| 128 | `owner:src/layout/grid_item_placement.cpp` | known-divergence | `src/layout/grid_item_placement.cpp` | structural proof is divergent; behavioral proof is unverified |
| 128 | `owner:src/layout/grid_track.cpp` | known-divergence | `src/layout/grid_track.cpp` | structural proof is divergent; behavioral proof is unverified |
| 128 | `owner:src/layout/layout_participant.cpp` | known-divergence | `src/layout/layout_participant.cpp` | structural proof is divergent; behavioral proof is unverified |
| 120 | `gap:F10` | unknown | `unresolved-owner` | **Behavioral-verify candidates** — concrete typeKeys with no bespoke handler: `ClampedScrollPhysics`/`ElasticScrollPhysics` (524/525), `ListPath` (619), `ListenerInputTypeEvent/Te… |
| 120 | `silver:artboard_list_overrides_horizontal` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 303 (rewind): expected rewind, got drawPath. |
| 120 | `silver:artboard_list_overrides_vertical` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 303 (rewind): expected rewind, got drawPath. |
| 120 | `silver:clear_viewmodel_list` | known-divergence | `tests/unit_tests/runtime/data_bind_lists_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 10 (makeRenderPaint): expected makeRenderPaint, got save. |
| 120 | `silver:collapsable_data_binding` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 14 (save): expected save, got color. |
| 120 | `silver:collapse_data_binds-test_1` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 10, op 760 (rewind): expected rewind, got drawPath. |
| 120 | `silver:collapse_data_binds-test_2` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 15, op 315 (addRawPath): expected 151 fields, got 256. |
| 120 | `silver:collapsing_elements` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 943 (rewind): expected rewind, got drawPath. |
| 120 | `silver:component_list_child_origin` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 448 (rewind): expected rewind, got drawPath. |
| 120 | `silver:computed_root_transform-list` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 255 (rewind): expected rewind, got drawPath. |
| 120 | `silver:computed_values_test` | known-divergence | `tests/unit_tests/runtime/data_binding_computed_values_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 191 (addRawPath), field point: expected (301.00003, -0.0 (0x80000000)), … |
| 120 | `silver:data_bind_keyframes_test` | known-divergence | `tests/unit_tests/runtime/data_binding_keyframes.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 4, op 159 (save): expected save, got restore. |
| 120 | `silver:data_bind_solo-solos-to-values` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 81 (addRawPath): expected 752 fields, got 669. |
| 120 | `silver:data_binding_artboards_test_recursive` | known-divergence | `tests/unit_tests/runtime/data_binding_artboards_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 118 (makeRenderPaint): expected makeRenderPaint, got frame. |
| 120 | `silver:data_converter_interpolator_reset` | known-divergence | `tests/unit_tests/runtime/data_binding_converters_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 30 (save): expected save, got color. |
| 120 | `silver:draw_index_list` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 35 (color): expected color, got makeRenderPaint. |
| 120 | `silver:fit_font_size_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 199 (makeRenderPath): expected makeRenderPath, got rewind. |
| 120 | `silver:focus_collapsing` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 192 (color), field paint_id: expected 6, got 11. |
| 120 | `silver:group_effect` | known-divergence | `tests/unit_tests/runtime/path_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 46 (addRawPath): expected 163 fields, got 3. |
| 120 | `silver:hittest_ab_shape_parent` | known-divergence | `tests/unit_tests/runtime/hittest_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 353 (save): expected save, got color. |
| 120 | `silver:image_fit_alignment` | known-divergence | `tests/unit_tests/runtime/data_binding_images_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 115 (transform), field tx: expected 462.03198, got -197.96802. |
| 120 | `silver:interpolate_to_end` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 63 (addRawPath): expected 954 fields, got 975. |
| 120 | `silver:interpolation_zero_duration` | known-divergence | `tests/unit_tests/runtime/data_binding_converters_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 38 (transform), field tx: expected 0, got 200. |
| 120 | `silver:keyboard_listener` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 85 (color): expected color, got save. |
| 120 | `silver:keyboard_listener-KeyboardInput` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 214 (color): expected color, got save. |
| 120 | `silver:layout_anim_bound` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 145 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_anim_component_list` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 88 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_anim_nested` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 85 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_aspect_ratio` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133). |
| 120 | `silver:layout_display` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 188 (makeRenderPath): expected makeRenderPath, got rewind. |
| 120 | `silver:layout_fixed_fill` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 56 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_grid_stack_grid_with_layouts` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 1, op 228 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_grid_stack_grid_with_layouts_size_span_changing` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 32, op 1592 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_grid_stack_grid_with_layouts_span` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 34, op 1116 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_grid_stack_stack_with_layouts` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 1, op 228 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_paint` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 77 (drawPath): expected drawPath, got makeRenderPath. |
| 120 | `silver:layout_scroll_drag_multiplier_layouts` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 38 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_drag_multiplier_list` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_drag_multiplier_virtualized` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_snap_carousel` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 145 (rewind): expected rewind, got drawPath. |
| 120 | `silver:layout_scroll_snap_padding_layouts` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 38 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_snap_padding_list` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_snap_padding_virtualized` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 120 | `silver:layout_scroll_visibility` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 130 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 120 | `silver:layout_text_match` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 61 (save): expected save, got frame. |
| 120 | `silver:list_focus_order` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 78 (addRawPath), field point: expected (-0.0 (0x80000000), 137.20052), g… |
| 120 | `silver:nested_artboard_quantize_and_speed` | known-divergence | `tests/unit_tests/runtime/nested_artboard_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 75 (transform), field xx: expected 0.95105654, got 1. |
| 120 | `silver:number_to_list_nested_children` | known-divergence | `tests/unit_tests/runtime/data_bind_lists_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 141 (color): expected color, got save. |
| 120 | `silver:paused_nested_artboard_opacity` | known-divergence | `tests/unit_tests/runtime/state_machine_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 103 (rewind): expected rewind, got drawPath. |
| 120 | `silver:relative_data_bind_path-fire-trigger` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 48 (color): expected color, got save. |
| 120 | `silver:relative_data_bind_path-listener` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 72 (makeRenderPath): expected makeRenderPath, got drawPath. |
| 120 | `silver:relative_data_bind_path-scripted-input` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 39 (transform), field tx: expected 115.56351, got 250. |
| 120 | `silver:scroll_intent` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 69 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 120 | `silver:stateful_multi_property` | known-divergence | `tests/unit_tests/runtime/component_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 134 (rewind): expected rewind, got drawPath. |
| 120 | `silver:text_feather_falloff` | known-divergence | `tests/unit_tests/runtime/text_modifier_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 29 (feather), field paint_id: expected 12, got 8. |
| 120 | `silver:text_input` | known-divergence | `tests/unit_tests/runtime/text_input_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 25 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 120 | `silver:text_vertical_trim_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 220 (rewind): expected rewind, got drawPath. |
| 120 | `silver:time_based_interpolation` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 65 (transform), field tx: expected 250.07309, got 250.29443. |
| 120 | `silver:transition_artboard_condition_test` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 16 (frameSize), field width: expected 983, got 984. |
| 120 | `silver:unbound_stateful_component` | known-divergence | `tests/unit_tests/runtime/data_binding_viewmodels_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 10 (save): expected save, got color. |
| 120 | `silver:virtualized_artboard_databound_children` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 5, op 365 (makeRenderPaint): expected makeRenderPaint, got save. |
| 120 | `silver:word_joiner_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 262 (transform), field ty: expected -39.996094, got -15.796875. |
| 115 | `golden:editor_scripted_vector_v7` | known-divergence | `unresolved-owner` | line 4: rust `makeRenderPaint {id=3,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}` vs c++ `source file="fixtures/editor/editor_scripted_vect… |
| 115 | `golden:group_effect` | known-divergence | `unresolved-owner` | line 86: rust `drawPath path={id=2,fillRule=2,path={verbs=[move,line,line,line,line,move,line,line,line,line,move,line,line,line,line,move,line,line,line,line,move,line,line,line,… |
| 115 | `golden:path_effect_with_feathers` | known-divergence | `unresolved-owner` | line 82: rust `drawPath path={id=5,fillRule=2,path={verbs=[move,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,… |
| 115 | `golden:superbowl` | known-divergence | `unresolved-owner` | line 2156: rust `makeEmptyRenderPath {id=101,fillRule=0,path={verbs=[],points=[]}}` vs c++ `drawPath path={id=61,fillRule=2,path={verbs=[move,cubic,cubic,cubic,cubic,cubic,cubic,c… |
| 105 | `silver:interpolator` | unknown | `unresolved-owner` | No producer/reference exists in the pinned upstream runtime tests. |
| 105 | `silver:listener_action_inputs` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_listener_action_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:multitouch_debug` | unknown | `unresolved-owner` | No producer/reference exists in the pinned upstream runtime tests. |
| 105 | `silver:script_affects_has_changed` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_artboards` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_artboards_opacity` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_artboards_origin` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_create_viewmodel_instance` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_context_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_input_color_trigger` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_input_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:script_layout_grid` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_layout_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripted_artboard_render` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripted_data_context` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_context_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripted_data_converter_bound_input` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_converter_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripted_viewmodel_cache` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripting_linear_animation` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:scripting_root_viewmodel` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_context_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:stateful_list_props` | unknown | `unresolved-owner` | The only producer is deliberately disabled upstream inside a block comment; tests/unit_tests/runtime/component_test.cpp:282-480 says per-list-item stateful VM instances are not cr… |
| 105 | `silver:viewmodel_from_context` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_context_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 105 | `silver:viewmodel_instance_to_artboard` | pending-proof | `tests/unit_tests/runtime/scripting/scripting_context_test.cpp` | Scripted producer provenance is catalogued; scripted action/output replay is explicitly deferred to the next adoption step. |
| 103 | `owner:src/animation/keyframe_int.cpp` | pending-proof | `src/animation/keyframe_int.cpp` | behavioral proof is unverified |
| 103 | `owner:src/data_bind/context/context_value_asset_blob.cpp` | pending-proof | `src/data_bind/context/context_value_asset_blob.cpp` | behavioral proof is unverified |
| 103 | `owner:src/layout/layout_sizing_style.cpp` | pending-proof | `src/layout/layout_sizing_style.cpp` | behavioral proof is unverified |
| 100 | `gap:A2` | pending-proof | `unresolved-owner` | **Native device-output control remains absent** — the Rust Artboard facade now exposes headless engine and volume control, but CPAL start/stop and the portable C boundary remain l… |
| 100 | `gap:A5` | pending-proof | `unresolved-owner` | **`nux-capi` cannot read events at all**; VM coverage is bool/number/string set-only (no color/enum/trigger/image/artboard/list, no getters/observers); no `pointer_exit`; no input… |
| 100 | `gap:A7` | pending-proof | `unresolved-owner` | **Artboard resize/layout override not first-class** (`width(x)`, `layoutWidth/Height`, `updateLayoutBounds`, `resetArtboardSize`) — only `raw_mut().set_artboard_dimensions`. Respo… |
| 100 | `gap:C1` | pending-proof | `unresolved-owner` | 28 schema-known typeKeys appear in **zero** corpus files; after removing abstract bases, the live list: `Folder`, `TextVariationModifier`, `TextStyleFeature`, `NSlicerTileMode`, `… |
| 100 | `gap:C2` | pending-proof | `unresolved-owner` | Only ~14/317 entries exercise any pointer input; `structural` verification mode used by zero entries (fine — but means it's untested machinery). \| Grow input-script corpus alongs… |
| 100 | `gap:F1` | pending-proof | `unresolved-owner` | **Audio** — `src/audio/**` engine/source/sound/reader, `audio_event.cpp` firing, `Artboard::volume` \| 1,030+ \| PARTIAL (P2F1/P2F2) \| Symphonia WAV/MP3/FLAC source/reader decode… |
| 100 | `gap:F13` | pending-proof | `unresolved-owner` | Historical backlog ceilings (from the original port's status log): full ListenerGroup drag/opaque behavior, nested pointer/listener hit propagation beyond event bubbling, live dat… |
| 100 | `gap:F15` | pending-proof | `unresolved-owner` | **Participant layout animation** — the C++ `ParticipantAnimation` lifecycle (`layout_participant.cpp:29-43,398-455,508-644`: `cascadeLayoutStyle` allocation, `advanceComponent`, `… |
| 100 | `gap:F7` | pending-proof | `unresolved-owner` | **Unported Lua bindings** — `lua_gpu` 3,734, `lua_promise` 1,323, `lua_scripted_context` 583, `lua_buffer_ext` 538, `lua_audio` 507, `lua_data_value` 503, `lua_image_decode` 467, … |
