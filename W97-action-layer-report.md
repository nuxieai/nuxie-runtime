# Silver corpus executable action-layer report

## Outcome

`make silver-corpus` now executes every classified runtime entry that the
current Rust runtime and action vocabulary can faithfully replay. The runner
imports the pinned `.riv` fixture, selects the manifest artboard/state machine
or animation, applies the ordered actions, serializes Rust renderer calls as
SRIV v1, and feeds that stream to the existing operation-aware comparator.

The runtime lane has no remaining `pending` entries:

| Classification | Count |
| --- | ---: |
| Byte-exact | 0 |
| Epsilon-exact | 0 |
| Divergent | 91 |
| Unsupported feature | 104 |
| Runtime total | 195 |

The whole corpus remains 238 entries: 195 runtime, 41 `pending-scripted`, and
2 `provenance-unknown`. All 238 C++ baselines parse successfully
(34,038,592 bytes; 1,191,317 operations). The `cpp-rust-exact` ratchet remains
0 and did not decrease.

The executable subset includes ordered artboard/state-machine/animation
advances, fixed-count frame loops, frame boundaries, draws, default view-model
binding, and literal pointer down/move/up/exit events (including timestamp and
pointer ID). The interpreter also supports named boolean/number/trigger input
changes, text input, and artboard-size mutation for newly encoded cases.

## Divergence finding

All 91 replayable cases diverge in the renderer resource-allocation prefix.
The C++ baseline mutates the first retained paint (`color` or `style`), while
the Rust stream allocates another `makeRenderPaint`. A few asset-heavy cases
reach the same mismatch at operation 2–4. This is recorded as a genuine
Rust-vs-C++ finding; production runtime code was not changed.

Every divergence and first divergent operation:

- `advance_blend_mode-inputs` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `advance_blend_mode-vms` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `animated_clipping-layout` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `animated_clipping-nodes` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `artboard_list_map_rules` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `artboard_list_overrides_horizontal` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `artboard_list_overrides_vertical` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `artboard_width_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `bankcard` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `clear_viewmodel_list` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `clipping_and_draw_order` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `collapse_data_binds-test_1` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `collapse_data_binds-test_2` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `collapsing_elements` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `component_stateful` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `computed_root_transform-list` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `computed_root_transform-nested_artboard` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `computed_values_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `custom_property_enum` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `data_bind_solo-solos-to-values` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `data_bind_solo-values-to-solos` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `data_converter_to_number` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `databind_artboard` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `event_trigger_event` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `fill_trim_path` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `focus_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `focus_traversal` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `follow_path_constraint` — frame 0, op 1 (style): expected style, got makeRenderPaint
- `format_number_with_commas` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `global_viewmodels_test-auto_instance` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `group_effect-main-missing-targets` — frame 0, op 1 (style): expected style, got makeRenderPaint
- `hide_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hittest_ab1` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hittest_ab1_grand_parent` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hittest_ab1_parent` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hittest_collapsed_layouts` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hittest_nested` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `hunter_x_demo` — frame 0, op 2 (color): expected color, got makeRenderPaint
- `layout_display` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `layout_paint` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `multi_listeners` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `multitouch` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `multitouch_enter` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `n_slice_triangle` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `nested_artboard_origin_override_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `nested_events` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `nested_hug` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `number_to_list_nested_children` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `pause_nested_artboard` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `recursive_data_bind` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `relative_data_binding` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `reset_phase_multi_main` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `saturation` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `sorted_listeners` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `spotify_kids_app_icon` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `spotify_kids_demo` — frame 0, op 2 (color): expected color, got makeRenderPaint
- `stacked_path_effects` — frame 0, op 1 (style): expected style, got makeRenderPaint
- `state_transition_fire_trigger` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `superbowl` — frame 0, op 4 (color): expected color, got makeRenderPaint
- `target_event` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `text_input` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `text_stroke_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `text_vertical_trim_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `time_based_interpolation` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `transition_actions` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `transition_artboard_condition_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `transition_index_condition` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `trigger_based_listeners` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `trigger_fires_single_change` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `unbound_stateful_component` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `vertical_align_ellipsis` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `viewmodel_based_condition` — frame 0, op 3 (color): expected color, got makeRenderPaint
- `virtualize_blendmode` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `component_list_child_origin` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `component_list_follow_path_distance` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `follow_path_animate_shape` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `follow_path_animate_solo` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `follow_path_animate_target` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `image_fit_alignment_2` — frame 0, op 3 (color): expected color, got makeRenderPaint
- `image_fit_alignment_3` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `image_fit_alignment_updated_test` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `layout_anim_bound` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `layout_anim_component_list` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `layout_anim_nested` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `layout_aspect_ratio` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `nested_needs_advance` — frame 0, op 1 (style): expected style, got makeRenderPaint
- `path_effect_with_feathers` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `stateful_keyed_trigger` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `text_follow_path_shape_length` — frame 0, op 1 (style): expected style, got makeRenderPaint
- `transition_duration_bind_list` — frame 0, op 1 (color): expected color, got makeRenderPaint
- `transition_duration_bind_nested` — frame 0, op 1 (color): expected color, got makeRenderPaint

## Unsupported blockers

The 104 unsupported entries are explicitly named in the manifest:

- `view-model-mutation`: 56
- `pointer-expression-encoding`: 19
- `named-view-model-instance`: 7
- `layout-scroll-physics`: 6
- `focus-keyboard-dispatch`: 5
- `runtime-object-mutation`: 4
- `global-view-model-setup`: 2
- `runtime-frame-loop-nontermination`: 1 (`db_health_tracker`)
- `renderer-paint-allocation`: 1 (`echo_show_demo`)
- `gamepad-input-sequence`: 1 (`gamepad_test`)
- `runtime-derived-loop`: 1 (`juice`)
- `layout-mutation`: 1 (`layout_hug_artboard`)

The dynamic pointer blocker is deliberately conservative: literal coordinates,
timestamps, and pointer IDs execute; C++ expressions depending on live layout,
loop variables, or mutable locals remain unsupported rather than being
approximated. Known focus/keyboard, gamepad, global/named view-model actions
are also rejected as named blockers instead of silently thinning their C++
bodies. View-model/list/object mutation is the largest remaining FL-D/E area.

## Verification

- `make silver-corpus` — green; 8 Rust corpus tests and 11 generator tests pass,
  generated manifest is current, all runtime classifications execute/validate.
- `cargo test -p nuxie-render-api sriv_serializer_matches_cpp_smoke_stream` —
  green.
- `cargo fmt --all --check` — green.
- `cargo clippy -p nuxie-render-api --all-targets --no-deps -- -D warnings` —
  green.
- `cargo clippy -p silver-corpus --all-targets --no-deps -- -D warnings` —
  green.

A workspace dependency-wide clippy invocation still reaches an unrelated
pre-existing `manual_contains` warning in `crates/nuxie-schema/src/lib.rs:263`.
No production code was changed to silence it. No commit was created.
