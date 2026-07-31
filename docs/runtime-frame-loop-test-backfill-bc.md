# Runtime frame-loop FL-B/FL-C upstream test backfill

This audit discharges the “likely represented” assessment in
`W65-unit-test-triage.md` against the pinned checkout at
`/Users/levi/dev/oss/rive-runtime`
(`d788e8ec6e8b598526607d6a1e8818e8b637b60c`). The scope is the 19
FL-B/FL-C class-A, class-B, and class-C files listed below: 198 `TEST_CASE`s
and 1,225 `REQUIRE`/`CHECK` assertion sites.

## Disposition rules and evidence

- `A1–An` identifies every assertion in source order inside that upstream
  `TEST_CASE`; a range is an assertion-by-assertion mapping of every member of
  that range to the cited Rust test(s).
- **covered-by** means an equivalent assertion already existed.
- **ported-at** means this backfill added the equivalent assertion. Fixture
  values are checked in Rust and, where the C++ probe supports the fixture,
  differentially against the pinned C++ runtime.
- **finding / ignored-at** means the unchanged assertion either fails against
  production or requires a production API/harness surface that Rust does not
  expose. The ignored test records rather than weakens the upstream contract.
- **skipped-because** is reserved for class-D C++ container-mechanics
  observables. There are no such skips in this scope.

Evidence names below are exact Rust test function names. Paths are relative to
the repository root.

| Evidence | Rust location |
|---|---|
| `B-state` | `crates/nuxie-runtime/src/animation.rs::upstream_animation_state_speed_start_and_spilled_time_matrix` |
| `B-instance` | `crates/nuxie-runtime/src/animation.rs::upstream_linear_animation_instance_sequences` |
| `B-loop-override` | `crates/nuxie-runtime/src/animation.rs::raw_loop_override_retains_cpp_minus_one_sentinel` |
| `B-definition` | `crates/nuxie-runtime/src/animation.rs::upstream_linear_animation_definition_timing_and_keep_going` |
| `B-quantize` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_quantize_and_looping_timeline_event_fixtures` |
| `B-missing-keyed` | `crates/nuxie-runtime/tests/cpp_probe.rs::keyed_property_importer_cursor_survives_next_animation_like_cpp_probe` |
| `B-events` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_quantize_and_looping_timeline_event_fixtures` |
| `B-fixtures` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_cubic_and_elastic_fixture_assertions_match_cpp_probe` |
| `B-elastic` | `crates/nuxie-runtime/src/animation.rs::upstream_elastic_ease_numeric_contract` |
| `C-default` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_default_state_machine_fixture_contract` |
| `C-focus-state` | `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs::fl_c5_focus_semantic_focus_state_and_owner_safe_focus_accessors` |
| `C-focus-actions` | `crates/nuxie-runtime/src/state_machine/focus_action_traversal.rs::traversal_maps_all_cpp_values_invalid_to_next_and_clear_is_idempotent` and `focus_action_target.rs::action_targets_the_first_direct_focus_data_before_and_after_topology_build` |
| `C-focus-conditions` | `crates/nuxie-runtime/tests/cpp_probe.rs::focus_transition_conditions_match_cpp_for_duplicate_and_failing_candidates` and `focus_transition_conditions_without_component_comparator_stay_blocked_like_cpp` |
| `C-focus-io` | `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs::focused_keyboard_dispatch_bubbles_leaf_to_parent_and_stops_when_handled`, `text_input_parent_precedes_scripted_and_listener_keyboard_dispatch`, and `gamepad_listener_dispatches_all_payloads_fifo_marks_advance_and_returns_false` |
| `C-hit-core` | `state_machine_instance.rs::fl_c5_hit_result_is_tristate_and_aggregates_strongest`, `fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false`, `fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order`, `fl_c5_hit_click_only_duplicate_groups_require_down_and_up`, and `fl_c5_hit_component_identity_reuses_owner_but_retains_duplicate_groups` |
| `C-hit-diff` | `cpp_probe.rs::fl_c5_hit_component_shared_target_down_up_matches_cpp_probe`, `fl_c5_nested_pointer_authored_child_routing_matches_cpp_probe`, and `fl_c5_component_list_pointer_reverse_overlap_matches_cpp_probe` |
| `C-draw-hit` | the geometry/layout hit tests in `crates/nuxie-runtime/src/draw.rs`, especially `geometry_hit_test_rejects_visible_shape_geometry_outside_its_active_clip`, `layout_proxy_routes_hits_and_opaque_flags_to_exact_layout_owner`, and `concrete_hit_dedup_keeps_overlapping_component_list_occurrences` |
| `C-listener-flags` | `crates/nuxie-runtime/src/state_machine/listener_action.rs::upstream_listener_action_flag_decode_occurrence_and_import_routing_matrix` |
| `C-align` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_listener_align_target_fixture_contract` |
| `C-nested` | `crates/nuxie-runtime/src/artboard.rs::upstream_runtime_nested_inputs_fixture_aliases_share_live_occurrences` |
| `C-inputs` | `crates/nuxie-runtime/tests/cpp_probe.rs::upstream_state_machine_input_fixture_contract`, `crates/nuxie/tests/public_api.rs::public_api_drives_default_state_machine_and_inputs`, and `crates/nuxie-runtime/src/state_machine/state_machine.rs::fl_c5_definition_authored_order_duplicates_names_and_null_slots` |
| `C-sm-core` | `cpp_probe.rs::upstream_rocket_and_blend_state_machine_fixture_structure`, `state_machine_animation_state_advances_through_public_runtime_seam`, `state_machine_input_transitions_match_cpp_probe`, `state_machine_transition_handoff_matches_cpp_probe`, `state_machine_blend_state_1d_input_matches_cpp_probe`, and `state_machine_blend_state_direct_matches_cpp_probe` |
| `C-sm-tail` | `cpp_probe.rs::state_machine_blend_state_transition_reset_matches_cpp_probe`, `state_machine_transition_interruption_matches_cpp_probe`, and `state_machine_component_viewmodel_pointer_unsupported_matches_cpp_probe` |
| `C-silver-exact` | `tools/silver-corpus/tests/runtime_frame_loop_backfill_bc.rs::upstream_fl_bc_exact_silver_assertions` |
| `C-silver-findings` | `tools/silver-corpus/tests/runtime_frame_loop_backfill_bc.rs::upstream_fl_bc_divergent_silver_assertions` |
| `C-script-input` | `state_machine_instance.rs::scripted_input_scalar_trigger_and_artboard_projection_failures_match_cpp`, `scripted_hydration_validation_failure_applies_no_inputs_or_init`, `scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order`, `scripted_hydration_accepts_valid_null_viewmodel_and_continues_to_init`, `scripted_hydration_typed_artboard_failure_stops_later_inputs_and_init`, `scripted_drawable_subtypes_register_keyboard_text_and_gamepad_paths`, and `scripted_listener_action.rs::typed_script_inputs_apply_live_cpp_core_values_and_keep_occurrences_isolated` |
| `C-script-action` | `cpp_probe.rs::scripted_listener_action_occurrence_lifecycle_matches_cpp_probe` and `state_machine_instance.rs::scripted_listener_actions_keep_authored_fifo_and_prefer_perform_action` |
| `C-script-pointer` | `state_machine_instance.rs::successive_pointer_events_preserve_previous_position_and_timestamp` and `crates/nuxie-scripting/src/vm/listener_invocation.rs::pointer_wrapper_matches_cpp_uint8_id_and_owned_payload` |
| `C-script-transition` | `cpp_probe.rs::scripted_transition_conditions_match_live_cpp_success_and_failure_order` and `scripted_transition_condition.rs::scripted_transition_requires_an_exact_true_boolean` |
| `C-script-gamepad` | `listener_invocation.rs::owned_text_and_gamepad_payloads_survive_source_drop` plus `listener_input_type_gamepad.rs::{button_phase_masks_and_threshold_match_cpp,standard_mapping_requires_matching_standard_intent}` |

## FL-B files

### `animation_state_instance_test.cpp` — class A, 14 cases, 30 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| AnimationStateInstance advances in step with animation speed 1 (13; A1–A2) | **ported-at** `B-state` |
| advances twice as fast when speed is doubled (47; A1–A2) | **ported-at** `B-state` |
| advances half as fast when speed is halved (83; A1–A2) | **ported-at** `B-state` |
| advances backwards when speed is negative (118; A1–A2) | **ported-at** `B-state` |
| starts a positive animation at the beginning (154; A1) | **ported-at** `B-state` |
| starts a negative animation at the end (182; A1) | **ported-at** `B-state` |
| negative state speed starts a positive animation at the end (211; A1) | **ported-at** `B-state` |
| negative state speed starts a negative animation at the beginning (241; A1) | **ported-at** `B-state` |
| spilledTime accounts for Nx speed with oneShot (272; A1–A3) | **ported-at** `B-state` |
| spilledTime accounts for 1/Nx speed with oneShot (313; A1–A3) | **ported-at** `B-state` |
| spilledTime accounts for Nx speed with loop (354; A1–A3) | **ported-at** `B-state` |
| spilledTime accounts for 1/Nx speed with loop (394; A1–A3) | **ported-at** `B-state` |
| spilledTime accounts for -Nx speed with oneShot (434; A1–A3) | **ported-at** `B-state` |
| spilledTime accounts for -Nx speed with loop (477; A1–A3) | **ported-at** `B-state` |

### `cubic_value_test.cpp` — class B, 1 case, 6 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| test cubic value load and interpolate properly (8; A1–A6) | **ported-at** `B-fixtures`; actual pinned `cubic_value_test.riv`, including named node/timeline, interpolator count, and both exact samples; live C++ differential |

### `elastic_easing_test.cpp` — class B, 2 cases, 15 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| test elastic easing loads properly (10; A1–A10) | **ported-at** `B-fixtures`; actual pinned `test_elastic.riv`, including easing/amplitude/period, shape/timeline, initial value, and both samples; live C++ differential |
| elastic easer computes correct actual amplitude (38; A1–A5) | **ported-at** `B-elastic` |

### `linear_animation_instance_test.cpp` — class A, 10 cases, 115 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| LinearAnimationInstance oneShot (8; A1–A8) | **ported-at** `B-instance` |
| LinearAnimationInstance speed (43; A1–A3) | **ported-at** `B-instance` |
| negative advance adds absolute total time (70; A1–A4) | **ported-at** `B-instance` |
| oneShot ← (100; A1–A14) | **ported-at** `B-instance` |
| loop → (149; A1–A8) | **ported-at** `B-instance` |
| loop ← (182; A1–A14) | **ported-at** `B-instance` |
| loop ← work area (225; A1–A20) | **ported-at** `B-instance` |
| pingpong → (283; A1–A24) | **ported-at** `B-instance` |
| pingpong ← (340; A1–A16) | **ported-at** `B-instance` |
| override loop (386; A1–A4) | **covered-by** `B-loop-override` |

### `linear_animation_test.cpp` — class A, 6 cases, 38 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| positive speed has normal start/end seconds (14; A1–A5) | **ported-at** `B-definition` |
| negative speed has reversed start/end seconds (40; A1–A5) | **ported-at** `B-definition` |
| quantize goes to whole frames (66; A1–A5) | A1–A4 **ported-at** `B-quantize` using pinned `quantize_test.riv`; A5 **finding / ignored-at** `cpp_probe.rs::upstream_quantize_toggle_requires_missing_mutable_definition_api` because imported definitions are read-only |
| reports when to keep going correctly (92; A1–A6) | **ported-at** `B-definition` |
| keeps initializing after missing keyed object (139; A1–A4) | **covered-by** `B-missing-keyed` |
| looping timeline events load correctly and report (174; A1–A13) | **ported-at** `B-events` using pinned `looping_timeline_events.riv` and the exact five-advance time/count sequence |

## FL-C focus and input files

### `default_state_machine_test.cpp` — class C, 1 case, 6 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| default state machine is detected at load (13; A1–A6) | **ported-at** `C-default` using the pinned `entry.riv`: valid default index/name, default occurrence construction, and default-scene state-machine selection |

### `focus_test.cpp` — class A, 81 cases, 393 assertions

For focus rows, Rust’s manager-owned node IDs replace C++ raw node/manager
pointers. The mapped assertion is the same lifetime/topology observable
(parent, child, attachment, focus, notification, or traversal result), not the
pointer address itself.

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| FocusNode default properties (79; A1–A11) | A1–A5, A7–A8, A10 **ported-at** `focus.rs::upstream_focus_node_defaults_and_property_setters`; A6, A9, A11 **finding / ignored-at** `focus.rs::upstream_focus_node_fresh_focusable_scope_and_manager_defaults` |
| FocusNode property setters (96; A1–A6) | **ported-at** `focus.rs::upstream_focus_node_defaults_and_property_setters` |
| FocusNode with Focusable (119; A1–A7) | **finding / ignored-at** `focus.rs::upstream_focusable_identity_and_fixture_swap_contracts_need_runtime_occurrence_surface`; Rust does not expose a per-node Focusable identity/delegation seam |
| FocusNode without Focusable does not crash (143; A1–A2) | **finding / ignored-at** the same focus-surface finding; a fresh Rust node is not the upstream null-Focusable representation |
| FocusNode setFocusable/clearFocusable (154; A1–A3) | **finding / ignored-at** the same focus-surface finding; Rust has no set/clear Focusable pointer API |
| FocusNode hierarchy (168; A1–A6) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` and `detaching_a_focused_subtree_preserves_focus_for_reattachment` |
| FocusManager basic focus operations (191; A1–A7) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` and `clearing_focus_blurs_leaf_and_ancestors` |
| focus change notifications (212; A1–A4) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` and `clearing_focus_blurs_leaf_and_ancestors` |
| respects canFocus (231; A1) | **covered-by** `direct_focus_on_an_ineligible_scope_does_not_reach_its_child` |
| hierarchy (243; A1–A9) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` and `moving_between_siblings_does_not_renotify_the_common_ancestor` |
| hasFocus with descendants (270; A1–A6) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` |
| removeChild clears focus (292; A1–A3) | **covered-by** `removing_a_focused_subtree_blurs_and_invalidates_every_node` |
| list-row reparent preserves primary focus (307; A1–A4) | **covered-by** `detaching_a_focused_subtree_preserves_focus_for_reattachment` |
| input routing (335; A1–A11) | **covered-by** `C-focus-io` |
| traversal basic (369; A1–A4) | **covered-by** `next_and_previous_traversal_follow_stable_tab_order_and_rest_on_leaves` |
| traversal with tabIndex (397; A1–A3) | **covered-by** `next_and_previous_traversal_follow_stable_tab_order_and_rest_on_leaves` |
| traversal skips non-traversable (423; A1) | **covered-by** `only_unbacked_structural_scopes_are_transparent_to_traversal` |
| edge behavior closedLoop (443; A1) | **covered-by** `closed_loop_scope_wraps_at_both_edges` |
| edge behavior stop (463; A1) | **covered-by** `stop_scope_does_not_move_past_its_boundary` |
| ancestor notification on focus (483; A1–A6) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` |
| common ancestor optimization (509; A1–A7) | **covered-by** `moving_between_siblings_does_not_renotify_the_common_ancestor` |
| traversal focuses leaves only (538; A1–A4) | **covered-by** `direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf` |
| nested scopes focus deepest leaf (560; A1–A3) | **covered-by** `direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf` |
| parentScope exits to parent (578; A1–A2) | **covered-by** `parent_scope_edges_continue_with_the_scopes_siblings` |
| clearFocus clears hasFocus chain (605; A1–A6) | **covered-by** `clearing_focus_blurs_leaf_and_ancestors` |
| removeChild clears manager reference (631; A1–A2) | **covered-by** `detaching_a_focused_subtree_preserves_focus_for_reattachment` |
| freeing parent clears surviving child parent (643; A1–A4) | **covered-by** `removing_a_focused_subtree_blurs_and_invalidates_every_node`; Rust observes ID invalidation and parent removal rather than a raw pointer write |
| addChild removes migrating root from previous manager (668; A1–A5) | **covered-by** `migrating_a_subtree_preserves_ids_after_the_old_manager_is_dropped` |
| migrated scope survives destruction of previous manager (689; A1–A3) | **covered-by** `migrating_a_subtree_preserves_ids_after_the_old_manager_is_dropped` |
| backward from first leaf exits scope (711; A1) | **covered-by** `parent_scope_edges_continue_with_the_scopes_siblings` |
| closedLoop wraps backward (735; A1) | **covered-by** `closed_loop_scope_wraps_at_both_edges` |
| stop prevents backward traversal (755; A1) | **covered-by** `stop_scope_does_not_move_past_its_boundary` |
| hasFocusNodes ignores non-traversable scopes (775; A1) | **covered-by** `focusable_content_ignores_empty_structural_scopes_but_counts_authored_nodes` |
| hasFocusNodes sees leaves under transparent scope (791; A1–A2) | **covered-by** `only_unbacked_structural_scopes_are_transparent_to_traversal` |
| hasFocusNodes counts currently ineligible focus data (821; A1) | **covered-by** `focusable_content_ignores_empty_structural_scopes_but_counts_authored_nodes` |
| traversal descends transparent scope and keeps sibling order (845; A1–A5) | **covered-by** `only_unbacked_structural_scopes_are_transparent_to_traversal` |
| drops focus when transparent-scope leaf becomes hidden (885; A1–A2) | **covered-by** `focus_is_dropped_when_the_primary_node_becomes_ineligible` |
| rebuilding one scope preserves sibling-scope focus (912; A1–A2) | **covered-by** `inserting_an_existing_subtree_reorders_without_blurring` |
| traversal action next (951; A1) | **covered-by** `C-focus-actions` |
| traversal action previous (975; A1) | **covered-by** `C-focus-actions` |
| traversal action unknown defaults next (1000; A1) | **covered-by** `C-focus-actions` |
| StateMachineInstance focus accessors (1024; A1–A4) | **covered-by** `C-focus-state` |
| traversal action ignores null StateMachineInstance (1049; no assertion macros) | **covered-by** no-panic/null branch in `C-focus-actions` |
| Focusable acceptsKeyboardInput defaults false (1064; A1–A2) | **covered-by** `C-focus-io` |
| focusState reports no focus (1073; A1–A2) | **covered-by** `C-focus-state` |
| focusState reports focused non-keyboard focusable (1088; A1–A2) | **covered-by** `C-focus-state` |
| focusState reports keyboard expectation (1108; A1–A2) | **covered-by** `C-focus-state` |
| focusState clears when focus clears (1128; A1–A3) | **covered-by** `C-focus-state` |
| focusState tracks switches (1151; A1–A6) | **covered-by** `C-focus-state` and `moving_between_siblings_does_not_renotify_the_common_ancestor` |
| focusState uses external manager (1189; A1–A3) | **covered-by** `fl_c5_focus_semantic_manager_switch_is_identity_noop_and_restores_internal` |
| clearFocus clears internal manager (1215; A1–A3) | **covered-by** `C-focus-state` |
| setFocus on scope descends first leaf (1238; A1) | **covered-by** `direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf` |
| setFocus on scope descends depth-first (1255; A1) | **covered-by** `direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf` |
| setFocus scope with no eligible leaf falls back (1274; A1) | **covered-by** `direct_focus_on_an_ineligible_scope_does_not_reach_its_child` |
| setFocus on ineligible scope is no-op (1291; A1) | **covered-by** `direct_focus_on_an_ineligible_scope_does_not_reach_its_child` |
| setFocus on leaf unchanged (1309; A1) | **covered-by** `focusing_child_notifies_leaf_and_ancestors` |
| Tab after focusing scope traverses siblings (1325; A1–A2) | **covered-by** `next_and_previous_traversal_follow_stable_tab_order_and_rest_on_leaves` |
| FocusActionClear clears primary focus (1350; A1–A2) | **covered-by** `C-focus-actions` and `clearing_focus_blurs_leaf_and_ancestors` |
| FocusActionClear no-op without focus (1372; A1–A2) | **covered-by** `C-focus-actions` |
| FocusActionClear ignores null StateMachineInstance (1389; no assertion macros) | **covered-by** no-panic/null branch in `C-focus-actions` |
| TransitionFocusCondition reassigned type key (1401; A1–A3) | **covered-by** `C-focus-conditions` (successful import/type dispatch and evaluation) |
| TransitionFocusCondition false for null instance (1418; A1) | **covered-by** `C-focus-conditions` |
| TransitionFocusCondition false without comparator (1428; A1) | **covered-by** `focus_transition_conditions_without_component_comparator_stay_blocked_like_cpp` |
| swapping bindable artboard registers nested focus nodes (1446; A1–A15) | **finding / ignored-at** focus-surface finding; exact `bindable_focus_tree_swap.riv` occurrence swap is not exposed |
| swapping nested artboard preserves other focus (1509; A1–A11) | **finding / ignored-at** focus-surface finding |
| skips collapsed and transparent nodes (1555; A1–A8) | **finding / ignored-at** focus-surface finding for exact `focus_collapsing.riv` sequence |
| focused elements receive keyboard inputs (1674; A1) | **finding / ignored-at** focus-surface finding for exact `keyboard_listener.riv` result |
| keyboard key combinations (1754; A1–A14) | **finding / ignored-at** focus-surface finding for exact fixture key/modifier matrix |
| text input handled on focused nodes (1870; A1–A12) | **finding / ignored-at** focus-surface finding for exact `text_input_event.riv` results |
| focus traversal listener actions (1917; A1–A2) | **finding / ignored-at** `C-silver-findings`; pinned `focus_traversal.sriv` differs at frame 0, operation 95 |
| traversal clears focus at root edge (2000; A1–A2) | **finding / ignored-at** focus-surface finding for exact fixture sequence |
| ArtboardComponentList scope uses shared manager (2051; A1–A11) | **finding / ignored-at** focus-surface finding for exact `focusable_element.riv` list occurrence |
| list under Node resolves closest FocusData (2082; A1–A7) | **finding / ignored-at** focus-surface finding |
| focus built and updated for lists (2121; A1–A2) | **finding / ignored-at** focus-surface finding for `component_list_1.riv` |
| focus based transitions work (2202; A1–A2) | **ported-at** `C-silver-exact`; pinned `focus_test.riv`/`focus_test.sriv` action stream is byte-exact |
| list item focus tree remains under row during rewire (2235; A1–A13) | **finding / ignored-at** focus-surface finding for `list_focus_order.riv` |
| swappable slot keeps tab-order place (2320; A1–A35) | **finding / ignored-at** focus-surface finding for `swappable_artboards_focus.riv` |
| repeat focus-tree build preserves nested focus (2418; A1–A11) | **finding / ignored-at** focus-surface finding |
| cross-file swaps keep slot order/share manager (2466; A1–A16) | **finding / ignored-at** focus-surface finding |
| unresolvable swap leaves focus/tab order untouched (2545; A1–A18) | **finding / ignored-at** focus-surface finding |
| initially empty slot keeps authored position on first swap (2601; A1–A19) | **finding / ignored-at** focus-surface finding |

### `gamepad_test.cpp` — class B, 7 cases, 24 assertions

#### Finding: gamepad batch buffer API

The parser/state assertions require C++
`StateMachineInstance::submitGamepadsFromBuffer(const uint8_t*, size_t)`.
`nuxie-runtime` currently exposes already-decoded
`StateMachineInstance::gamepad_dispatch`; it has no byte-batch parser, wire
version, connected-device table, or device-ID lifecycle API. Typed dispatch is
well covered, but it cannot discharge these parser/state assertions. Because
the requested change is tests-only, the missing production API is recorded as
a finding at
`cpp_probe.rs::upstream_gamepad_batch_buffer_contract_requires_missing_runtime_api`,
which is ignored and deliberately panics. These are not class-D container
mechanics and are not marked skipped.

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| batch accepts a single connected record (105; A1–A2) | A1 **ported-at** `cpp_probe.rs::upstream_gamepad_wire_size_and_fixture_load_contract`; A2 **finding / ignored-at** gamepad batch finding |
| tracks multiple device IDs independently (130; A1–A2) | **finding / ignored-at** gamepad batch finding |
| rejects update for unknown device ID (158; A1) | **finding / ignored-at** gamepad batch finding |
| handles disconnect among several devices (179; A1–A2) | **finding / ignored-at** gamepad batch finding |
| allows reconnecting same device ID (215; A1–A3) | **finding / ignored-at** gamepad batch finding |
| tolerates disconnect of unknown device ID (249; A1) | **finding / ignored-at** gamepad batch finding |
| file processes multiple gamepad input types (266; A1–A13) | A1 **ported-at** `cpp_probe.rs::upstream_gamepad_wire_size_and_fixture_load_contract`; A2–A13 **finding / ignored-at** gamepad batch finding; post-parse typed listener behavior is independently **covered-by** `C-script-gamepad` and `state_machine_instance.rs::gamepad_listener_dispatches_all_payloads_fifo_marks_advance_and_returns_false` |

### `hittest_test.cpp` — class A, 21 cases, 111 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| hittest-basics (21; A1–A2) | **covered-by** `draw.rs::geometry_hit_test_includes_the_exact_cubic_boundary_without_hit_slop` and `geometry_clockwise_fill_rule_uses_rive_hit_tester_parity` |
| hittest-mesh (51; A1) | **covered-by** `draw.rs::geometry_hit_test_includes_the_exact_cubic_boundary_without_hit_slop` |
| hit test on opaque target (70; A1–A12) | **covered-by** `fl_c5_hit_result_is_tristate_and_aggregates_strongest`, `fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false`, and `C-hit-diff` |
| opaque nested artboard (123; A1–A14) | **covered-by** `fl_c5_nested_pointer_authored_child_routing_matches_cpp_probe` and nested pointer unit paths |
| early out on listeners (196; A1–A29) | **covered-by** `fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false`, `fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order`, and `fl_c5_hit_component_identity_reuses_owner_but_retains_duplicate_groups` |
| click event (258; A1–A12) | A1–A8 **ported-at** `cpp_probe.rs::upstream_click_event_fixture_initial_and_first_click_contract`; A9–A12 **finding / ignored-at** `cpp_probe.rs::upstream_click_event_fixture_reports_exact_group_click_sequence`; see failing assertion finding below |
| multiple shapes with mouse movement behavior (316; A1–A14) | **finding / ignored-at** `cpp_probe.rs::upstream_hit_test_fixtures_require_unsupported_dynamic_pointer_actions` for the exact `click_event.riv` art-2 sequence |
| shape clipped by parent layout (394; A1–A2) | **finding / ignored-at** `C-silver-findings`; `hittest_ab1.sriv` differs at frame 1, operation 153 |
| shape clipped by parent artboard (432; A1–A2) | **finding / ignored-at** `C-silver-findings`; `hittest_ab1_parent.sriv` differs at frame 1, operation 192 |
| shape clipped by parent and grand-parent artboard (471; A1–A2) | **finding / ignored-at** `C-silver-findings`; `hittest_ab1_grand_parent.sriv` differs at frame 2, operation 304 |
| artboard list scrolling behavior (515; A1–A2) | **finding / ignored-at** hit-test silver finding |
| virtualized/carousel list scrolling (594; A1–A2) | **finding / ignored-at** hit-test silver finding |
| text in rotated/scaled layouts (676; A1–A2) | **finding / ignored-at** hit-test silver finding |
| shapes in layouts (726; A1–A2) | **finding / ignored-at** hit-test silver finding |
| objects inside shapes (776; A1–A2) | **finding / ignored-at** `C-silver-findings`; `hittest_nested.sriv` differs at frame 1, operation 155 |
| pointer exit (818; A1–A2) | **finding / ignored-at** hit-test silver finding for exact `pointer_exit.riv` output |
| multi-touch events (879; A1–A2) | **ported-at** `C-silver-exact`; pinned `multitouch.riv`/`multitouch.sriv` action stream is byte-exact |
| multi-touch nested artboard + exit (956; A1–A2) | **ported-at** `C-silver-exact`; pinned `multitouch_enter.riv`/silver action stream is byte-exact |
| multi-touch list + exit (1043; A1–A2) | **finding / ignored-at** hit-test silver finding |
| multi-touch multi-scroll (1118; A1–A2) | **finding / ignored-at** hit-test silver finding |
| collapsed-layout leaves (1155; A1) | **covered-by** `runtime_drawable_dispatch_stream_treats_collapsed_clip_paths_as_empty_like_cpp_probe` and `layout_proxy_routes_hits_and_opaque_flags_to_exact_layout_owner` |

### `listener_action_flags_test.cpp` — class C, 7 cases, 30 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| parentKind decodes bits 1–2 (22; A1–A4) | **ported-at** `C-listener-flags` |
| flag fields are independent (45; A1–A8) | **ported-at** `C-listener-flags` |
| matchesScheduledOccurrence covers both phases (72; A1–A4) | **ported-at** `C-listener-flags` |
| import routes Transition parent despite listener importer (121; A1–A4) | **ported-at** `C-listener-flags` |
| import routes State parent despite listener importer (136; A1–A5) | **ported-at** `C-listener-flags` |
| import routes Listener parent when both present (157; A1–A3) | **ported-at** `C-listener-flags` |
| Listener parent without importer fails (171; A1–A2) | **ported-at** `C-listener-flags` |

### `listener_align_target_test.cpp` — class B, 2 cases, 12 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| preserve offset off (18; A1–A6) | **ported-at** `C-align` using pinned `align_target.riv` |
| preserve offset on (50; A1–A6) | **ported-at** `C-align` using pinned `align_target.riv` |

### `nested_input_test.cpp` — class A, 4 cases, 50 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| nested boolean get/set (14; A1–A14) | **ported-at** `C-nested` using pinned `runtime_nested_inputs.riv` and all three aliases |
| nested number get/set (49; A1–A14) | **ported-at** `C-nested` using the pinned fixture and all three aliases |
| nested trigger fire (84; A1–A8) | **ported-at** `C-nested` using the pinned fixture |
| boolean get/set multiple artboards deep (110; A1–A14) | **ported-at** `C-nested` using the two-level pinned occurrence |

### `state_machine_input_test.cpp` — class C, 1 case, 17 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| file with state-machine inputs loads (18; A1–A17) | **ported-at** `C-inputs` using the pinned `smi_test.riv`: nested artboard identity/transform import, nested state-machine occurrence, trigger/bool/number type and authored input-ID retention |

### `state_machine_test.cpp` — class A, 18 cases, 172 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| file with state machine can be read (30; A1–A29) | **ported-at** `cpp_probe.rs::upstream_rocket_and_blend_state_machine_fixture_structure` using pinned `rocket.riv` |
| file with blend states loads correctly (106; A1–A28) | **ported-at** `cpp_probe.rs::upstream_rocket_and_blend_state_machine_fixture_structure` using pinned `blend_test.riv` |
| animation state with no animation does not crash (163; A1–A11) | **finding / ignored-at** `cpp_probe.rs::upstream_state_machine_fixture_contracts_without_exact_runtime_equivalents` for exact `multiple_state_machines.riv` structure/advance |
| 1D blend keeps keepsGoing true after animations stop (192; A1–A3) | **finding / ignored-at** state-machine fixture finding for exact `oneshotblend.riv` sequence |
| duration transition completes state before state change (219; A1–A13) | **finding / ignored-at** state-machine fixture finding for exact `state_machine_transition.riv` component/color sequence |
| blend animations with reset (264; A1–A30) | **covered-by** `state_machine_blend_state_transition_reset_matches_cpp_probe` |
| transitions with reset (359; A1–A19) | **covered-by** `state_machine_blend_state_transition_reset_matches_cpp_probe` and `state_machine_transition_interruption_matches_cpp_probe` |
| triggers consumed only on allowed state changes (436; A1–A19) | **finding / ignored-at** state-machine fixture finding for exact `state_machine_triggers.riv` sequence |
| list-index transition compares to number (514; A1) | **ported-at** `C-silver-exact`; `transition_index_condition.riv`/silver action stream is byte-exact |
| listeners sorted in correct order (537; A1) | **finding / ignored-at** `C-silver-findings`; `sorted_listeners.sriv` differs at frame 0, operation 32 |
| listeners with multiple event types (600; A1) | **finding / ignored-at** `C-silver-findings`; `multi_listeners.sriv` differs at frame 2, operation 253 |
| listeners with multiple event types and rebinding (650; A1) | **finding / ignored-at** state-machine fixture finding for exact rebinding silver |
| nested state-machine transition duration bindable (719; A1) | **finding / ignored-at** `C-silver-findings`; nested-duration silver differs at frame 0, operation 57 |
| artboard-list state-machine transition duration bindable (750; A1) | **finding / ignored-at** `C-silver-findings`; list-duration silver differs at frame 0, operation 13 |
| replace view-model instances at multiple levels (782; A1–A2) | **finding / ignored-at** state-machine fixture finding for exact `rebind_with_nested_viewmodel.riv` sequence |
| component-based transition conditions (837; A1–A4) | **finding / ignored-at** state-machine fixture finding for exact `component_based_conditions.riv` sequence |
| component transition conditions with other properties (877; A1–A4) | **finding / ignored-at** state-machine fixture finding |
| transitions and state layers trigger actions (917; A1–A4) | **finding / ignored-at** `C-silver-findings`; `transition_actions.sriv` differs at frame 2, operation 72 |

## FL-C scripting files

The triage placed these under the recursive FL-C scripting wave. They are
included here rather than treated as a separate follow-up.

### `scripting/scripting_gamepad_event_test.cpp` — class C, 2 cases, 12 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| GamepadEvent reads snapshot fields and standard names (9; A1–A10) | **covered-by** `C-script-gamepad`; device ID, mapping, button/axis values, semantic standard button names, and stick axes are all exercised |
| unknown mapping clears semantic buttons (73; A1–A2) | **covered-by** `C-script-gamepad` |

### `scripting/scripting_input_test.cpp` — class A, 15 cases, 128 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| scripted object set/read number input (23; A1–A11) | **covered-by** `C-script-input` typed input and live-core update tests |
| set/read string input (88; A1–A11) | **covered-by** `C-script-input`, including raw-byte/string boundary coverage |
| set/read boolean input (157; A1–A11) | **covered-by** `C-script-input` |
| set/read integer input (222; A1–A11) | **covered-by** `C-script-input` |
| multiple inputs (287; A1–A18) | **covered-by** `scripted_listener_action.rs::typed_script_inputs_apply_live_cpp_core_values_and_keep_occurrences_isolated` |
| inputs used in advance (407; A1–A12) | **covered-by** scripted converter/advance tests in `C-script-input` |
| ScriptedDrawable created and initialized (486; A1–A4) | **covered-by** scripted object hydration/initialization lifecycle tests |
| ScriptedLayout created and initialized (565; A1–A6) | **covered-by** `state_machine_instance.rs::scripted_drawable_subtypes_register_keyboard_text_and_gamepad_paths` and hydration tests |
| ScriptedDataConverter created and initialized (661; A1–A9) | **covered-by** scripted converter hydration/bind/advance tests |
| scriptDispose clears view-model listeners immediately (760; A1–A12) | **finding / ignored-at** `cpp_probe.rs::upstream_scripting_fixture_contracts_require_script_and_silver_oracles`; existing lifecycle coverage does not reproduce the pinned listener/console sequence |
| scripted input color and trigger (827; A1–A3) | **finding / ignored-at** scripting fixture finding for the exact 30-frame `script_inputs_test_1.riv` silver |
| no listener-attachment memory leak (863; A1–A3) | **finding / ignored-at** scripting fixture finding for exact `scripted_memory_leak.riv` construction/lifetime |
| ensureScriptInitialized then hydrate twice keeps self ref (879; A1–A7) | **covered-by** repeated hydration/table identity lifecycle tests |
| hydrateScriptInputs runs init without DataContext when allowed (922; A1–A5) | **covered-by** `crates/nuxie-scripting/src/vm.rs::attached_empty_context_rejects_only_when_init_requests_missing_data` |
| hydration preflight fails atomically for unresolved VM input (968; A1–A5) | **covered-by** failed-init poison/retry and atomic preflight tests in `vm.rs` |

### `scripting/scripting_listener_action_test.cpp` — class A, 3 cases, 62 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| scripted listener action (13; A1–A2) | **finding / ignored-at** scripting fixture finding for exact `scripted_listener_action.riv` silver |
| listener action inputs (50; A1) | **finding / ignored-at** scripting fixture finding for exact `listener_action_inputs.riv` silver |
| action script receives pointer types and data (82; A1–A59) | **finding / ignored-at** scripting fixture finding for exact `scripted_listener_context.riv` view-model results |

### `scripting/scripting_pointer_event_test.cpp` — class C, 2 cases, 2 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| pointer event can be constructed (7; A1) | **covered-by** `C-script-pointer` |
| pointer event constructed with specified position (16; A1) | **covered-by** `C-script-pointer` |

### `scripting/scripting_transition_condition_test.cpp` — class A, 1 case, 2 assertions

| Upstream `TEST_CASE` (line; assertion IDs) | Disposition |
|---|---|
| scripted transition condition (12; A1–A2) | **covered-by** `C-script-transition` for success, false/non-boolean result, protected failure, and authored candidate order |

## Findings

### Finding: click up outside

This is a production-behavior failure, not merely a missing harness surface.
The literal pinned `click_event.riv` port in
`cpp_probe.rs::upstream_click_event_fixture_reports_exact_group_click_sequence`
passes through the setup and first click (A1–A8, now active in
`upstream_click_event_fixture_initial_and_first_click_contract`), then fails
A9–A12. Upstream `hittest_test.cpp:284–310` requires cumulative event counts
`[1, 1, 1, 2, 3]`; Rust reports `[1, 2, 2, 3, 4]`, beginning with the
pointer-down at `(75,75)` and pointer-up at `(300,75)`. The full sequence test
is ignored exactly as written. Production was not changed and the assertion
was not weakened.

### Finding: mutable animation quantize

`linear_animation_test.cpp:85–87` switches the imported definition's
`quantize` property off and requires the half-second sample to become `200`.
Rust imports and correctly tests the authored `true` value (`160`) but exposes
`RuntimeLinearAnimation` definitions read-only, so the mutation assertion is
retained by
`cpp_probe.rs::upstream_quantize_toggle_requires_missing_mutable_definition_api`.

### Finding: FocusNode representation

`focus_test.cpp:91` requires a fresh node's Focusable pointer to be null.
`FocusNode::new()` currently stores `has_focusable = true`; the literal
assertion therefore fails. Upstream's per-node `isScope()` and `manager()`
pointer observables are also not represented directly. The ignored test is
`focus.rs::upstream_focus_node_fresh_focusable_scope_and_manager_defaults`.

### Finding: focus fixture surface

The exact Focusable pointer/delegation cases and 16 remaining fixture cases listed in the
table require bindable-artboard swaps, VM assets, component-list occurrences,
and repeated focus-tree builds through an occurrence-facing API not exposed by
the Rust focus test seam. They are retained by
`focus.rs::upstream_focusable_identity_and_fixture_swap_contracts_need_runtime_occurrence_surface`;
generic focus-manager tests are intentionally not claimed as equivalents.

### Finding: silver hit-test fixtures

The 28 unsupported-action assertions identified in the table terminate in
pinned `SerializingFactory::matches` goldens, but their action streams contain
layout-computed pointer expressions or long generated loops that the current
silver action interpreter cannot encode. They are retained by
`cpp_probe.rs::upstream_hit_test_fixtures_require_unsupported_dynamic_pointer_actions`.
Two multitouch silvers are now byte-exact active tests; four other hit-test
silvers are literal failing findings below.

### Finding: silver runtime divergences

`tools/silver-corpus/tests/runtime_frame_loop_backfill_bc.rs::upstream_fl_bc_divergent_silver_assertions`
replays all ten literal action streams and compares them to the pinned `.sriv`
files. It is ignored after reporting these production divergences:
`focus_traversal` (frame 0/op 95), `hittest_ab1` (frame 1/op 153),
`hittest_ab1_parent` (frame 1/op 192), `hittest_ab1_grand_parent` (frame
2/op 304), `hittest_nested` (frame 1/op 155), `multi_listeners` (frame 2/op
253), `sorted_listeners` (frame 0/op 32), `transition_actions` (frame 2/op
72), `transition_duration_bind_list` (frame 0/op 13), and
`transition_duration_bind_nested` (frame 0/op 57). These account for 18
upstream assertion sites.

### Finding: state-machine fixture surface

The 57 capability assertions identified in the table require exact fixture occurrence,
reset-pool, view-model rebinding, or silver observables not discharged by the
similarly-shaped synthetic differentials. They are retained by
`cpp_probe.rs::upstream_state_machine_fixture_contracts_without_exact_runtime_equivalents`.
C++ reset-pool counts were not mislabeled class-D skips because each source
case also contains runtime state assertions.

### Finding: scripting fixture oracles

The 80 assertions identified in the table require pinned script-console,
view-model-result, or silver outputs. Wrapper/lifecycle tests are useful but
not assertion-equivalent, so the gap is retained by
`cpp_probe.rs::upstream_scripting_fixture_contracts_require_script_and_silver_oracles`.

## Totals

| Disposition | Upstream assertions |
|---|---:|
| covered by pre-existing equivalent Rust tests | 407 |
| ported in this backfill | 398 |
| finding recorded by ignored literal/capability tests | 420 |
| skipped as class-D C++ container mechanics | 0 |
| **total** | **1,225** |

The 398 newly ported assertions include all 30 animation-state assertions, 111
of 115 linear-animation-instance assertions, 16 definition/work-area
assertions, 21 cubic/elastic assertions, 17 of 18 quantize/event assertions, 6
default-state-machine assertions, 17 state-machine-input assertions, 14 direct
FocusNode defaults/setters, 2 gamepad setup assertions, all 30 listener-flag
assertions, all 12 align-target assertions, all 50 nested-input assertions, and
all 57 rocket/blend fixture assertions, plus 7 exact focus/multitouch/list-index
silver assertions and 8 click-event setup/first-click assertions. The remaining
click sequence is counted as a finding because its upstream assertions fail.
Of the 420 findings, 23 are literal production failures (4 click-sequence
sites, 1 fresh-FocusNode site, and 18 silver sites); 397 are blocked
capability/harness observables retained by
linked ignored tests rather than mislabeled as class-D skips.
