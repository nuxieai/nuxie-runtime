# Transition-condition owner-family closure

This is the publication checklist for the complete pinned-C++ transition-condition
family at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. The family is not eligible for
review until every row below has either a live C++ differential or an explicit
source-cited structural proof. All statuses remain pending until independent
acceptance.

## Source-to-Rust closure

| Pinned C++ owner | Required semantics | Focused Rust owner | Evidence |
| --- | --- | --- | --- |
| `src/animation/state_transition.cpp` | authored condition order; first allowed candidate; exit-time floating-point behavior, including zero-duration `NaN`; mix lifecycle; trigger use after allowance | `crates/nuxie-runtime/src/state_machine/state_transition.rs` | `state_machine_exit_time_transition_matches_cpp_probe`; `state_machine_early_exit_transition_matches_cpp_probe`; `state_machine_transition_handoff_matches_cpp_probe`; `state_machine_timed_transition_mixing_matches_cpp_probe`; `state_machine_blend_state_transition_*`; `state_machine_random_transition_matches_cpp_probe`; structural rules `state_machine_transition_search_early_exit` and `state_machine_invented_transition_return_guard` |
| `src/animation/transition_condition.cpp` | base dispatch, default allowance, clone/occurrence boundary | `crates/nuxie-runtime/src/state_machine/transition_condition.rs` | module dispatch tests plus the complete condition differentials below; dispatch definition lives in this file rather than a subtype file |
| `src/animation/transition_input_condition.cpp` | importer lookup; bounds validation; subtype validation; nullable input occurrence accepted | `crates/nuxie-runtime/src/state_machine/transition_input_condition.rs` | `state_machine_input_conditions_reject_wrong_types_and_bad_indices_like_cpp`; `state_machine_input_conditions_preserve_cpp_null_slots_and_evaluate_them_true`; `runtime_import_status_counts_state_machine_null_input_placeholders` |
| `src/animation/transition_bool_condition.cpp` | exact bool operator; missing/null input evaluates true | `crates/nuxie-runtime/src/state_machine/transition_bool_condition.rs` | `state_machine_input_transitions_match_cpp_probe`; wrong-type, bad-index, and null-slot tests above |
| `src/animation/transition_number_condition.cpp` | all six floating-point comparisons; missing/null input evaluates true | `crates/nuxie-runtime/src/state_machine/transition_number_condition.rs` | `state_machine_input_transitions_match_cpp_probe`; wrong-type, bad-index, and null-slot tests above |
| `src/animation/transition_trigger_condition.cpp` | per-layer fireability; use only after allowance; missing/null input evaluates true | `crates/nuxie-runtime/src/state_machine/transition_trigger_condition.rs` | `state_machine_trigger_input_drives_zero_duration_transition_once`; wrong-type, bad-index, and null-slot tests above |
| `src/animation/transition_focus_condition.cpp` | right/left focus comparator lookup; descendant focus; duplicate and failed candidates; missing or wrong comparator occurrence retained and false | `crates/nuxie-runtime/src/state_machine/transition_focus_condition.rs` | `focus_transition_conditions_match_cpp_for_duplicate_and_failing_candidates`; `focus_transition_conditions_without_component_comparator_stay_blocked_like_cpp` |
| `src/animation/scripted_transition_condition.cpp` | per-occurrence cloned script state; ordered script inputs; exact boolean result; failing candidate continuation | `crates/nuxie-runtime/src/state_machine/scripted_transition_condition.rs` | live scripting-oracle test `scripted_transition_conditions_match_live_cpp_success_and_failure_order` |
| `src/animation/transition_comparator.cpp` | shared comparison dispatch | `crates/nuxie-runtime/src/state_machine/transition_comparator.rs` | integer, property, artboard-comparand, and view-model comparison matrix differentials below |
| `src/animation/transition_property_comparator.cpp` | property lookup and comparison ownership | `crates/nuxie-runtime/src/state_machine/transition_property_comparator.rs` | `transition_integer_comparison_matches_pinned_cpp_probe`; `state_machine_artboard_comparand_conditions_match_cpp_probe` |
| `src/animation/transition_property_viewmodel_comparator.cpp` | view-model property lookup; integer/string/enum/bool/number/trigger comparison edges | `crates/nuxie-runtime/src/state_machine/transition_property_viewmodel_comparator.rs` | `state_machine_viewmodel_comparator_matrix_edges_match_cpp_probe` |
| `src/animation/transition_viewmodel_condition.cpp` | owns left/right comparator occurrences and comparison; resolves after import; incompatible or unsupported comparators retain `ConditionComparisonNone` and evaluate false; exact dispatch | `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` | `state_machine_viewmodel_comparator_matrix_edges_match_cpp_probe`; `state_machine_artboard_comparand_conditions_match_cpp_probe`; `state_machine_component_pair_conditions_match_cpp_probe`; `state_machine_component_artboard_unsupported_direction_matches_cpp_probe`; `state_machine_component_viewmodel_conditions_match_cpp_probe`; `state_machine_component_viewmodel_pointer_unsupported_matches_cpp_probe` |
| `src/importers/state_machine_importer.cpp` | ordered nullable input slots and condition import context | `crates/nuxie-binary/src/lib.rs`, `crates/nuxie-runtime/src/state_machine/state_machine_input.rs`, `state_machine_input_instance.rs` | `runtime_import_status_validates_transition_input_condition_types`; `runtime_import_status_counts_state_machine_null_input_placeholders`; source proof from `StateMachineImporter::readNullObject` and `StateMachineInstance` null occurrence construction |
| `src/importers/state_transition_importer.cpp` | conditions attach to the current transition in authored order | `crates/nuxie-binary/src/lib.rs`, `crates/nuxie-runtime/src/state_machine/state_transition.rs` | transition differentials above; structural rules `state_machine_per_advance_collection_rebuild`, `state_machine_occurrence_definition_copy`, and `state_machine_replacement_candidate_container` |
| `src/importers/transition_viewmodel_condition_importer.cpp` | comparator occurrence ownership and left-before-right assignment | `crates/nuxie-binary/src/lib.rs`, `crates/nuxie-runtime/src/state_machine/transition_viewmodel_condition.rs` | view-model comparison matrix differential and importer-context fixture |
| `src/animation/state_machine_instance.cpp:1742-1766,2901-2905`; `src/data_bind/data_bind_container.cpp:25-33` | retain every transition-duration DataBind occurrence in insertion order; clone each per instance; live nested path; last applied occurrence supplies the value | `crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs`, `crates/nuxie-runtime/src/data_bind_graph.rs`, `crates/nuxie-runtime/src/state_machine/instance.rs` | `transition_duration_bindings_preserve_duplicate_authored_order_like_cpp`; `nested_transition_duration_path_resolves_before_same_advance_fire_like_cpp` |

Mapped member rows covered by this family are `state_machine.conditions` and
`state_machine.transitions`. Neither row is promoted by this checklist.

## Adversarial publication review

- [x] Wrong input type: bool, number, and trigger imports rejected exactly like C++.
- [x] Out-of-range input index: bool, number, and trigger imports rejected exactly like C++.
- [x] Nullable input occurrence: slot and later indices retained; bool, number, and trigger evaluate true. The synthetic pin crashes during later unrelated finalization, so this is explicitly a source-cited structural/behavioral proof rather than a claimed live differential.
- [x] Duplicate candidates: focus and scripted candidate order includes both success and failure.
- [x] Duplicate DataBinds: every occurrence retained in authored order; no ID-keyed replacement map.
- [x] Zero duration: C++ division and resulting `NaN` allowance preserved without an invented guard.
- [x] Nested path: unresolved transition-duration path survives import, resolves against live instance data, and affects set → fire → `advance(0)` without an extra frame.
- [x] Same-frame timing: transition selection, input consumption, and bound duration are compared on the same zero-second advance.
- [x] Dispatch: base condition enum and dispatch live in `transition_condition.rs`; subtype files own subtype behavior.
- [x] Lifecycle/cloning: definitions remain shared and immutable; occurrence state and DataBind clones are per `StateMachineInstance`.
- [x] Unsupported comparators: authored ViewModel-condition occurrences are retained with a false `ConditionComparisonNone` evaluator rather than being deleted and turning the transition unconditional.
- [x] Malformed focus comparators: missing and wrong comparator occurrences remain attached and evaluate false rather than being deleted and turning the transition unconditional.
- [x] Permanent structural ratchets: collection rebuild, definition copy, replacement candidate container, event/listener rescan, transition-search early exit, invented return guard, and dropped ViewModel/focus conditions all have injected negative controls.

## Publication boundary

Use focused tests while editing. Once every row above is closed, run the expensive
runtime/workspace-probe, ordinary/scripted golden, renderer pixel, ABI, size,
format/lint, structural checker, and performance floors once against the frozen
candidate. Publish one immutable SHA and wait for one independent acceptance
verdict before starting the next owner family.
