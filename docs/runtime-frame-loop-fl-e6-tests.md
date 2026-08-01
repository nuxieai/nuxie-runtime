# FL-E6 W65 test disposition

W65 assigns `raw_text_input_test.cpp` (16 cases, class C) and
`text_input_test.cpp` (19 cases, class B) to FL-E. FL-E6 ports every
non-silver assertion sequence either literally in the direct owner modules or
against the authored `text_input.riv` fixture. No assertion, tolerance, or
conditional skip was weakened.

## Class C: `raw_text_input_test.cpp` (16/16)

| Upstream cases | Rust evidence |
|---|---|
| cursor operators | `text::cursor::tests::upstream_cursor_operators_are_ported` |
| visual cursor position; LTR, RTL, and mixed-bidi hit placement; multiline up/down; home/end | `upstream_text_input_visual_cursor_fixture_values_are_ported` loads the pinned Inter font and asserts the upstream `0`, `23.30859`, `65.17969`, and `396.0` caret coordinates plus exact line heights. `upstream_text_input_ltr_rtl_and_mixed_bidi_hits_are_ported` loads the pinned IBM Plex Arabic font, applies the upstream 500px width, asserts the C++ 2/3/5 bidi-aware shaped-line ranges, pins the real second-line visual-edge cursor indices, and checks six interior hit/caret samples captured from the pinned C++ owner (`23@143.50781`, `19@252.35156`, `15@383.66016` for RTL; `18@130.71094`, `26@273.41016`, `22@407.3203` for mixed bidi). `upstream_text_input_multiline_cursor_sequence_is_ported` ports the upstream 1→15→4→0→3→14→36→53 sequence, while `upstream_text_input_vertical_cursor_retains_the_ideal_column` pins repeated movement through an uneven short line. The authored key fixture also pins Home/End. |
| bounds and measurement cache | `upstream_text_input_measurement_cache_is_ported` asserts the upstream `446.51953×216` and `318.97266×324` bounds, same-size cache reuse, width-key invalidation, and text-change invalidation with the pinned Arabic font. |
| text insert/set | `upstream_insert_delete_selection_and_text_contracts_are_ported` and the authored committed-text fixture |
| word movement | `upstream_word_cursor_sequence_is_ported` |
| sub-word movement | `upstream_subword_cursor_sequence_is_ported` |
| right/left multi-codepoint movement; backward/forward multi-codepoint deletion | `upstream_combining_cluster_movement_and_deletion_are_ported`; both direction branches are asserted in the retained buffer implementation and exercised again through Backspace/Delete in the authored fixture. |
| word selection, including right-edge and symbol selection | `upstream_word_selection_edges_are_ported` |
| journal undo/redo, branch truncation, replacement-selection restoration | `upstream_journal_branching_sequence_is_ported` |

## Class B: `text_input_test.cpp` (19/19)

| Upstream cases | Rust evidence |
|---|---|
| file loads and owns 3 concrete drawables | `upstream_text_input_load_and_drawable_children_are_ported` |
| arrow keys; Backspace/Delete; undo/redo; unhandled/released keys; modifier boundaries; Shift selection; select-all; Home/End | `upstream_text_input_key_editing_and_selection_cases_are_ported` |
| raw `selectLine`; TextInput `selectWord`/`selectLine` wrappers | `upstream_multiline_select_all_and_line_contracts_are_ported`, `upstream_word_selection_edges_are_ported`, and `upstream_text_input_text_multiline_wrapper_and_radius_cases_are_ported` |
| double/triple click selection | `upstream_text_input_double_and_triple_click_selection_is_ported` drives the authored state machine through the pinned press/release sequence using the TextInput's live world transform; it retains the pinned C++ test's conditional pointer-hit guard but requires the authored state machine and cursor to exist. `upstream_multi_click_constants_and_initial_state_are_pinned` separately locks the interval, distance, and cold state. Deterministic wrapper tests assert word and line selection without that asset-layout guard. |
| committed text; multiline source/display toggling; single-line CR/LF stripping; Enter; selection radius | `upstream_text_input_text_multiline_wrapper_and_radius_cases_are_ported` and `upstream_single_line_break_stripping_is_ported` |
| state-machine key/text forwarding | `text_input_parent_precedes_scripted_and_listener_keyboard_dispatch` |
| serialized render | The `text_input` silver entry remains the rendering oracle and is reported from `make silver-corpus`; renderer goldens cover the shared draw-path boundary. |

The direct selection-path tests additionally pin empty, square, rounded, and
adjacent-rectangle union behavior. Clone-cold scroll/drag/edit ownership is
covered by `text_input_clone_rebuilds_scroll_link_and_drag_state_cold`.
