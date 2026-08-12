# Runtime drift queue

Generated from the checked-in parity ledgers. JSON is authoritative; this view highlights clusters and the highest-discovery candidates.

- Upstream ref: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
- Candidates: 808
- Clusters: 34

## Dispositions

| disposition | candidates |
|---|---:|
| extension | 3 |
| intentional-decision | 14 |
| known-divergence | 101 |
| pending-proof | 205 |
| stale-proof | 446 |
| unknown | 4 |
| unsupported | 35 |

## Filter fields

Filter the JSON `candidates` array by `owner_family`, `subsystem`, `evidence_state`, `disposition`, or sort by descending `discovery_value`. The complete deterministic value sets are in `filters`.

## Clusters

| cluster | boundary | owner family | candidates | max discovery value |
|---|---|---|---:|---:|
| `cluster:layout:ownership` | ownership | layout | 14 | 128 |
| `cluster:unresolved:ownership` | ownership | unresolved | 43 | 125 |
| `cluster:runtime-tests:lifecycle` | lifecycle | runtime-tests | 75 | 115 |
| `cluster:runtime-tests:ordering` | ordering | runtime-tests | 21 | 115 |
| `cluster:runtime-tests:mutation` | mutation | runtime-tests | 1 | 110 |
| `cluster:runtime-tests:ownership` | ownership | runtime-tests | 141 | 110 |
| `cluster:runtime-tests:invalidation` | invalidation | runtime-tests | 2 | 105 |
| `cluster:unresolved:float-behavior` | float-behavior | unresolved | 3 | 105 |
| `cluster:unresolved:invalidation` | invalidation | unresolved | 3 | 105 |
| `cluster:unresolved:lifecycle` | lifecycle | unresolved | 5 | 105 |
| `cluster:unresolved:mutation` | mutation | unresolved | 7 | 105 |
| `cluster:unresolved:ordering` | ordering | unresolved | 16 | 105 |
| `cluster:animation:ownership` | ownership | animation | 93 | 103 |
| `cluster:data_bind:ownership` | ownership | data_bind | 45 | 103 |
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
| `cluster:unresolved:unsupported-observable` | unsupported-observable | unresolved | 4 | 80 |
| `cluster:runtime-tests:unsupported-observable` | unsupported-observable | runtime-tests | 31 | 75 |
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
| 125 | `gap:F10` | unknown | `unresolved-owner` | **Behavioral-verify candidates** — concrete typeKeys with no bespoke handler: `ClampedScrollPhysics`/`ElasticScrollPhysics` (524/525), `ListPath` (619), `ListenerInputTypeEvent/Te… |
| 115 | `golden:editor_scripted_vector_v7` | known-divergence | `unresolved-owner` | line 4: rust `makeRenderPaint {id=3,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}` vs c++ `source file="fixtures/editor/editor_scripted_vect… |
| 115 | `golden:group_effect` | known-divergence | `unresolved-owner` | line 86: rust `drawPath path={id=2,fillRule=2,path={verbs=[move,line,line,line,line,move,line,line,line,line,move,line,line,line,line,move,line,line,line,line,move,line,line,line,… |
| 115 | `golden:path_effect_with_feathers` | known-divergence | `unresolved-owner` | line 82: rust `drawPath path={id=5,fillRule=2,path={verbs=[move,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,line,… |
| 115 | `golden:superbowl` | known-divergence | `unresolved-owner` | line 2156: rust `makeEmptyRenderPath {id=101,fillRule=0,path={verbs=[],points=[]}}` vs c++ `drawPath path={id=61,fillRule=2,path={verbs=[move,cubic,cubic,cubic,cubic,cubic,cubic,c… |
| 115 | `silver:artboard_list_overrides_horizontal` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 303 (rewind): expected rewind, got drawPath. |
| 115 | `silver:artboard_list_overrides_vertical` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 303 (rewind): expected rewind, got drawPath. |
| 115 | `silver:clear_viewmodel_list` | known-divergence | `tests/unit_tests/runtime/data_bind_lists_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 10 (makeRenderPaint): expected makeRenderPaint, got save. |
| 115 | `silver:collapsable_data_binding` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 14 (save): expected save, got color. |
| 115 | `silver:collapse_data_binds-test_1` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 10, op 760 (rewind): expected rewind, got drawPath. |
| 115 | `silver:collapse_data_binds-test_2` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 15, op 315 (addRawPath): expected 151 fields, got 256. |
| 115 | `silver:collapsing_elements` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 943 (rewind): expected rewind, got drawPath. |
| 115 | `silver:component_list_child_origin` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 448 (rewind): expected rewind, got drawPath. |
| 115 | `silver:computed_root_transform-list` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 255 (rewind): expected rewind, got drawPath. |
| 115 | `silver:computed_values_test` | known-divergence | `tests/unit_tests/runtime/data_binding_computed_values_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 191 (addRawPath), field point: expected (301.00003, -0.0 (0x80000000)), … |
| 115 | `silver:data_bind_keyframes_test` | known-divergence | `tests/unit_tests/runtime/data_binding_keyframes.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 4, op 159 (save): expected save, got restore. |
| 115 | `silver:data_bind_solo-solos-to-values` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 81 (addRawPath): expected 752 fields, got 669. |
| 115 | `silver:data_binding_artboards_test_recursive` | known-divergence | `tests/unit_tests/runtime/data_binding_artboards_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 118 (makeRenderPaint): expected makeRenderPaint, got frame. |
| 115 | `silver:data_converter_interpolator_reset` | known-divergence | `tests/unit_tests/runtime/data_binding_converters_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 30 (save): expected save, got color. |
| 115 | `silver:draw_index_list` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 35 (color): expected color, got makeRenderPaint. |
| 115 | `silver:fit_font_size_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 199 (makeRenderPath): expected makeRenderPath, got rewind. |
| 115 | `silver:focus_collapsing` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 192 (color), field paint_id: expected 6, got 11. |
| 115 | `silver:group_effect` | known-divergence | `tests/unit_tests/runtime/path_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 46 (addRawPath): expected 163 fields, got 3. |
| 115 | `silver:hittest_ab_shape_parent` | known-divergence | `tests/unit_tests/runtime/hittest_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 353 (save): expected save, got color. |
| 115 | `silver:image_fit_alignment` | known-divergence | `tests/unit_tests/runtime/data_binding_images_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 115 (transform), field tx: expected 462.03198, got -197.96802. |
| 115 | `silver:interpolate_to_end` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 63 (addRawPath): expected 954 fields, got 975. |
| 115 | `silver:interpolation_zero_duration` | known-divergence | `tests/unit_tests/runtime/data_binding_converters_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 38 (transform), field tx: expected 0, got 200. |
| 115 | `silver:keyboard_listener` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 85 (color): expected color, got save. |
| 115 | `silver:keyboard_listener-KeyboardInput` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 214 (color): expected color, got save. |
| 115 | `silver:layout_anim_bound` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 145 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_anim_component_list` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 88 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_anim_nested` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 85 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_aspect_ratio` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133). |
| 115 | `silver:layout_display` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 188 (makeRenderPath): expected makeRenderPath, got rewind. |
| 115 | `silver:layout_fixed_fill` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 56 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_grid_stack_grid_with_layouts` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 1, op 228 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_grid_stack_grid_with_layouts_size_span_changing` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 32, op 1592 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_grid_stack_grid_with_layouts_span` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 34, op 1116 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_grid_stack_stack_with_layouts` | known-divergence | `tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned gridStackSilver helper actions; first difference: frame 1, op 228 (rewind): expected rewind, got drawPath. |
| 115 | `silver:layout_paint` | known-divergence | `tests/unit_tests/runtime/layout_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 77 (drawPath): expected drawPath, got makeRenderPath. |
| 115 | `silver:layout_scroll_drag_multiplier_layouts` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 38 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_drag_multiplier_list` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_drag_multiplier_virtualized` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_snap_padding_layouts` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 38 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_snap_padding_list` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_snap_padding_virtualized` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 24 (makeRenderPaint): expected makeRenderPaint, got frameSize. |
| 115 | `silver:layout_scroll_visibility` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 130 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 115 | `silver:layout_text_match` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 61 (save): expected save, got frame. |
| 115 | `silver:list_focus_order` | known-divergence | `tests/unit_tests/runtime/focus_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 78 (addRawPath), field point: expected (-0.0 (0x80000000), 137.20052), g… |
| 115 | `silver:nested_artboard_quantize_and_speed` | known-divergence | `tests/unit_tests/runtime/nested_artboard_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 75 (transform), field xx: expected 0.95105654, got 1. |
| 115 | `silver:number_to_list_nested_children` | known-divergence | `tests/unit_tests/runtime/data_bind_lists_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 141 (color): expected color, got save. |
| 115 | `silver:paused_nested_artboard_opacity` | known-divergence | `tests/unit_tests/runtime/state_machine_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 103 (rewind): expected rewind, got drawPath. |
| 115 | `silver:relative_data_bind_path-fire-trigger` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 48 (color): expected color, got save. |
| 115 | `silver:relative_data_bind_path-listener` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 72 (makeRenderPath): expected makeRenderPath, got drawPath. |
| 115 | `silver:relative_data_bind_path-scripted-input` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 39 (transform), field tx: expected 115.56351, got 250. |
| 115 | `silver:scroll_intent` | known-divergence | `tests/unit_tests/runtime/layout_scroll_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 69 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 115 | `silver:stateful_multi_property` | known-divergence | `tests/unit_tests/runtime/component_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 134 (rewind): expected rewind, got drawPath. |
| 115 | `silver:text_feather_falloff` | known-divergence | `tests/unit_tests/runtime/text_modifier_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 29 (feather), field paint_id: expected 12, got 8. |
| 115 | `silver:text_input` | known-divergence | `tests/unit_tests/runtime/text_input_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 25 (transform), field xy: expected -0.0 (0x80000000), got 0. |
| 115 | `silver:text_vertical_trim_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 3, op 220 (rewind): expected rewind, got drawPath. |
| 115 | `silver:time_based_interpolation` | known-divergence | `tests/unit_tests/runtime/data_binding_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 1, op 65 (transform), field tx: expected 250.07309, got 250.29443. |
| 115 | `silver:transition_artboard_condition_test` | known-divergence | `tests/unit_tests/runtime/serialized_rendering_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 16 (frameSize), field width: expected 983, got 984. |
| 115 | `silver:unbound_stateful_component` | known-divergence | `tests/unit_tests/runtime/data_binding_viewmodels_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 0, op 10 (save): expected save, got color. |
| 115 | `silver:virtualized_artboard_databound_children` | known-divergence | `tests/unit_tests/runtime/component_list_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 5, op 365 (makeRenderPaint): expected makeRenderPaint, got save. |
| 115 | `silver:word_joiner_test` | known-divergence | `tests/unit_tests/runtime/text_test.cpp` | Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE actions; first difference: frame 2, op 262 (transform), field ty: expected -39.996094, got -15.796875. |
| 113 | `owner:src/layout/layout_sizing_style.cpp` | pending-proof | `src/layout/layout_sizing_style.cpp` | behavioral proof is unverified |
| 110 | `test:tests/unit_tests/runtime/global_view_model_binding_test.cpp` | pending-proof | `tests/unit_tests/runtime/global_view_model_binding_test.cpp` | The listed File catalog case is a direct Rust port over global_variables_test.riv. The eight state-machine binding cases remain unclaimed. |
| 110 | `test:tests/unit_tests/runtime/layout_participant_test.cpp` | pending-proof | `tests/unit_tests/runtime/layout_participant_test.cpp` | S4-38 directly asserts the seven fixture outcomes; the four animation cases are F15/UNIV-1603 direct ports (linear, retarget, disable, cubic). Still unclaimed: cubic retarget-whil… |
| 110 | `test:tests/unit_tests/runtime/rounded_rect_path_test.cpp` | pending-proof | `tests/unit_tests/runtime/rounded_rect_path_test.cpp` | S4-41 directly compares the new raw rounded-rectangle path against the prior Rectangle construction. Offset-origin and ShapePaintPath cases remain unclaimed. |
| 105 | `gap:A2` | pending-proof | `unresolved-owner` | **Native device-output control remains absent** — the Rust Artboard facade now exposes headless engine and volume control, but CPAL start/stop and the portable C boundary remain l… |
| 105 | `gap:A5` | pending-proof | `unresolved-owner` | **`nux-capi` cannot read events at all**; VM coverage is bool/number/string set-only (no color/enum/trigger/image/artboard/list, no getters/observers); no `pointer_exit`; no input… |
| 105 | `gap:A7` | pending-proof | `unresolved-owner` | **Artboard resize/layout override not first-class** (`width(x)`, `layoutWidth/Height`, `updateLayoutBounds`, `resetArtboardSize`) — only `raw_mut().set_artboard_dimensions`. Respo… |
| 105 | `gap:C1` | pending-proof | `unresolved-owner` | 28 schema-known typeKeys appear in **zero** corpus files; after removing abstract bases, the live list: `Folder`, `TextVariationModifier`, `TextStyleFeature`, `NSlicerTileMode`, `… |
| 105 | `gap:C2` | pending-proof | `unresolved-owner` | Only ~14/317 entries exercise any pointer input; `structural` verification mode used by zero entries (fine — but means it's untested machinery). \| Grow input-script corpus alongs… |
| 105 | `gap:F1` | pending-proof | `unresolved-owner` | **Audio** — `src/audio/**` engine/source/sound/reader, `audio_event.cpp` firing, `Artboard::volume` \| 1,030+ \| PARTIAL (P2F1/P2F2) \| Symphonia WAV/MP3/FLAC source/reader decode… |
| 105 | `gap:F13` | pending-proof | `unresolved-owner` | Historical backlog ceilings (from the original port's status log): full ListenerGroup drag/opaque behavior, nested pointer/listener hit propagation beyond event bubbling, live dat… |
| 105 | `gap:F15` | pending-proof | `unresolved-owner` | **Participant layout animation** — the C++ `ParticipantAnimation` lifecycle (`layout_participant.cpp:29-43,398-455,508-644`: `cascadeLayoutStyle` allocation, `advanceComponent`, `… |
| 105 | `gap:F7` | pending-proof | `unresolved-owner` | **Unported Lua bindings** — `lua_gpu` 3,734, `lua_promise` 1,323, `lua_scripted_context` 583, `lua_buffer_ext` 538, `lua_audio` 507, `lua_data_value` 503, `lua_image_decode` 467, … |
| 105 | `gap:H1` | pending-proof | `unresolved-owner` | Cycle-3 approval granted 2026-07-21 for the fixed `d788e8ec..b73bc675` cut: PORT TextInput (`1b4df2ad`) and static-link (`b73bc675`), profiler (`079305d7`) deferred WATCH, both de… |
| 105 | `gap:H3` | pending-proof | `unresolved-owner` | Two `TODO(golden)` markers: `state_machine.rs:797` (port `addToHitLookup`), `draw.rs:3555` (unify layout-bounds path). |
| 105 | `gap:RB-2` | pending-proof | `unresolved-owner` | Focus ownership/projection \| `RuntimeFocusTree::sync` descriptor projection plus `target_nodes` rebuild instead of retained live `Focusable`/`FocusData` relationships \| OPEN |
| 105 | `gap:RB-4` | pending-proof | `unresolved-owner` | Scalar ScriptInput binding \| `rehydrate_script_listener_actions` rescans and hydrates scalar inputs at scene rebind instead of retaining the C++ `ScriptInput`/`DataBindContext` p… |
| 105 | `gap:RB-5` | pending-proof | `unresolved-owner` | SolidColor paint mutation \| `solid_color_paint_revisions` defers the C++ `SolidColor::colorValueChanged` retained-paint mutation to a later draw handoff \| OPEN |
| 105 | `gap:V1` | pending-proof | `unresolved-owner` | **The two oracles never compose.** `corpus.toml` proves runtime→draw-calls; `corpus-r.toml` replays pre-serialized `.rive-stream` fixtures through the renderer. Nothing runs `.riv… |
| 105 | `gap:V10` | pending-proof | `unresolved-owner` | **BLOCKING RATCHET INSTALLED 2026-08-04; direct ratio parity remains open.** `make perf-gate` measures 24 manifest-owned files with scripting-enabled release C++/Rust runners, 100… |
| 105 | `gap:V16` | pending-proof | `unresolved-owner` | **`bankcard`: authored gradient/inner-feather dependency topology is ported; one compound inner-path contour remains displaced.** Rust now interleaves `LinearGradient`/`RadialGrad… |
| 105 | `gap:V18` | pending-proof | `unresolved-owner` | **`clipping_and_draw_order`: post-zero transform diverges.** Rust emits translation (1121,259) where C++ emits identity. \| `corpus.toml` milestone V18, samples 0/0.5/1 \| Reconci… |
| 105 | `gap:V2` | pending-proof | `unresolved-owner` | **PARTIAL — the animated-corpus sampling hole is closed; the ratchet is withheld.** All 226 entries that combined `LinearAnimation` coverage with a sole `t=0` sample now retain `t… |
| 105 | `gap:V24` | pending-proof | `unresolved-owner` | **PARTIAL 2026-08-03 — retained-paint loss is closed; a later gradient-order gap remains.** Rust now completes all samples and paint global 584 remains addressable through its con… |
| 105 | `gap:V25` | pending-proof | `unresolved-owner` | **Script-update invalidation ported; chained GroupEffect output remains divergent.** A true scripted path-effect advance now schedules `ScriptUpdate` at the effect dependency slot… |
| 105 | `gap:V26` | pending-proof | `unresolved-owner` | **PARTIAL 2026-08-03 — retained-gradient rematerialization is closed; a later path gap remains.** Gradient 189 is no longer recreated at t=0.5. The next mismatch is draw path 397 … |
| 105 | `gap:V3` | pending-proof | `unresolved-owner` | **Differential fuzzing was planned (V2 map, "Long-Tail Strategy" §2) but never built.** `fuzz/` targets are panic-only — no C++ comparison, no randomized times/inputs. The long-ta… |
| 105 | `gap:V30` | pending-proof | `unresolved-owner` | **Feather invalidation is wired, but its upstream effect path remains divergent.** V25's `ScriptUpdate` reaches `ShapePaint` before feather preparation; fresh dense capture still … |
| 105 | `gap:V31` | pending-proof | `unresolved-owner` | **PARTIAL 2026-08-04 — retained-gradient rematerialization and root-gradient construction order are closed; a weighted rounded-path gap remains.** Gradient 51 is not recreated at … |
| 105 | `gap:V41` | pending-proof | `unresolved-owner` | **`paused_nested_artboard_opacity`: nested opacity differs after enrollment.** Rust emits alpha `0xf7` where C++ emits `0xff` for the same `0x6e0000` color payload. \| `corpus.tom… |
| 105 | `gap:V43` | pending-proof | `unresolved-owner` | **`data_bind_blob_test`: data-bound blob geometry differs.** At the first differing draw, Rust's rectangle height is 2098.35938 while C++ uses 926.574219. \| `corpus.toml` milesto… |
| 105 | `gap:V44` | pending-proof | `unresolved-owner` | **`artboard_opacity_and_transform_test`: the Rust runner lacks nested-child data binding.** It exits on data-bind global 29 (`data-binding-nested-child`, target `Artboard`) before… |
