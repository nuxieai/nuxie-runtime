# Luau on clients, lane H1: no script runs under a Core borrow (2026-11-14)

Branch `claude/luaur-lane-h1-borrow-396cb9`, based on `levi/luaur-client-stage1f`
(`b495c08c29`). Owned files: `crates/nuxie-runtime` outside H2's view-model
value files, plus the facade call path in `crates/nuxie-scripting/src/luau/facade/`
(`handles.rs`, `state.rs`) for the check.

## Commits

| Commit | Concern |
| --- | --- |
| `17b21f0daa` | The check: test builds report a script entered while a Core receiver is mutably lent |
| `4244065df8` | Data-bind context value lent out of its cell (take-out), not borrowed |
| `51d7d71f28` | Scripted interpolators run with no borrow of their callers |
| `c2e614ceaa` | Transition conditions run with the transition only shared |
| `ae2d6510e7` | A script's property listener runs once no Core receiver is lent |
| `700abd19e6` | A scripted listener action performs from its handle |
| `4b3e6feb3e` | An artboard script input hydrates with the input only shared |
| `e8d31962aa` | A layout script's resize runs with no borrow of its layout |
| `2dab138d83` | Fix to `51d7d71f28`: an interpolator advances through its converter capability |
| `1a931babb4` | Path effects run their scripts with no borrow of their paint or group |
| `b00d409418` | An artboard list row's instance is made with no borrow of its list |
| `54cc3ebefc` | H4 corpus: a NaN distance measures from a contour's start |
| `c9c0e69fb1` | The check aborts test builds (`ENFORCED = true`) |
| `7485432f97` | Ratchet: the handle-level paths add no panic-lint sites |
| `f69eef52d9` | rustfmt of the files this lane touched (no code change) |

## The check (step 1)

`crates/nuxie-runtime/src/script_borrow_check.rs`. `CoreHandle::with`/`with_mut`
(and `with_downcast*`, through them) and a runtime artboard's
`with_artboard`/`with_artboard_mut` hold a `Lent` guard for the life of the
borrow; in `test-support` builds it pushes `(Shared|Mutable, caller location)`
on a thread-local stack (`#[track_caller]` in those builds only). The facade
calls `script_entered(caller)` immediately before it runs Lua: in
`Function::call_with` (which `call`, `call_lent`, `Table::call_function_unit`
and `call_function_truthy` reach) and `Thread::resume_raw` (which `resume`,
`resume_module`, `Chunk::call` and `exec` reach). An entry while any receiver
is mutably lent:

* inside `record_script_borrow_reports(..)` is recorded and returned (how the
  tests name their entry point);
* with `NUXIE_SCRIPT_BORROW_SURVEY=1` is written to stderr once per distinct
  case and the run carries on (how the list below was made);
* otherwise writes the report and a backtrace to stderr and aborts the
  process, so a `catch_unwind` cannot swallow it. This is on from the last
  commit (`ENFORCED = true`); before it the check only reported.

Shared borrows are listed in a report but are not a failure. Without
`test-support` (every shipped build) `Lent` is a zero-size type and
`script_entered` is empty.

The data-bind context value is not counted: since `4244065df8` it is moved out
of its cell while in use, so there is no borrow to count.

## Entry points the check found

Survey of the three suites on `17b21f0daa` (the check alone), grouped by
script entry and the mutable loans open; line numbers as of that commit.
27 distinct (entry, loans) cases collapse to 10 families:

| # | Mutable loan(s) open (outermost first) | Reached by | Family |
| --- | --- | --- | --- |
| 1 | `data_bind.rs:500` / `:547` (`context.borrow_mut()`) | converters, dependency, viewmodels, wave_b | data-bind context |
| 2 | `data_bind.rs:500` → `data_converter.rs:301` → `data_converter_range_mapper.rs:187` | interpolator | interpolators |
| 3 | `state_machine_instance.rs:1469`, `:1482`; `data_converter_interpolator.rs:480` → `:264`; `layout_component.rs:6812`; `linear_animation_instance.rs:685`, `:697` | interpolator, nuxie unit | interpolators |
| 4 | `state_machine_instance.rs:1166` → `:1651` | listener_action, transition_condition | transition conditions |
| 5 | `state_machine_instance.rs:4379` (→ `data_bind.rs:547`) and `:987` (→ `:547`), entries `vm/view_model.rs:1542`, `:993`, `:998`, `embedded_chunk.rs:75` | viewmodels, wave_a_final, listener_action, audio_core, wave_b | property listeners raised under listener actions |
| 6 | `state_machine_instance.rs:4379`, entry `vm.rs:1172` | listener_action | scripted listener actions |
| 7 | `scripted_object.rs:1272` (`with_script_input`) | scripting_artboard, wave_b | artboard script input hydrate |
| 8 | `intrinsically_sizeable.rs:311` | scripted_layout | scripted layout resize |
| 9 | `component_graph.rs:880` → `shape_paint.rs:283` (→ `target_effect.rs:158` → `group_effect.rs:72`); `advancing_component.rs:144` | scripted_path, runtime path_effect | path effects (update and advance) |
| 10 | `artboard_component_list.rs:2507` (first seen under `data_bind.rs:500`) | runtime wave_b | artboard list row instancing |

`nuxie-scripting` reported nothing (its suite runs scripts with no Core
objects lent). The raw logs: survey of `nuxie` (24 cases), `nuxie-runtime`
(7 cases, one shared with nuxie), `nuxie-scripting` (0).

## Outcome per entry point

Three mechanisms, one per family:

* **take-out**: move the state out of its cell for the operation, so a re-entry
  finds it at rest (empty) rather than a `RefCell` panic;
* **handle**: read what the script needs under a borrow (shared where the read
  allows), end it, call the script, apply the result under a new borrow;
* **defer**: queue the call on the end of the outermost Core borrow
  (`picture_layout_mode::after_core_borrows`, G4's queue), for a notification
  whose result nothing waits for.

| Family | Mechanism | What changed | Test |
| --- | --- | --- | --- |
| 1 data-bind context | take-out | `RuntimeBindContextValue::lend` moves the boxed value out of its `Rc<RefCell<Option<..>>>` for each use and a guard puts it back (also on unwind). A re-entrant use of the same bind sees `None` and does nothing. | `data_bind.rs` `context_loan_tests` (2), `upstream_scripting_converter::scripted_converters_run_with_no_core_receiver_lent` |
| 2–3 interpolators | handle (+ take-out for the converter's advancer) | `KeyFrameInterpolator::transform_occurrence`/`transform_value_occurrence` read a `ScriptedInterpolator`'s instance under a shared borrow and call it after; every caller (state machine mix, layout, keyframes, linear animation, converters) uses them. The range mapper computes its range under a shared borrow and maps outside it (`convert_occurrence`). The interpolator converter's advance moves its `InterpolatorAdvancer` out for the step. | `upstream_scripting_interpolator::scripted_interpolators_run_with_no_core_receiver_lent` |
| 4 transition conditions | handle | `allowed` only reads the transition, so it is borrowed shared; the weight writes and `useInLayer` follow under their own borrow. A `ScriptedTransitionCondition` evaluates from its handle. | `upstream_scripting_transition_condition::scripted_transition_conditions_run_with_no_core_receiver_lent` |
| 5 property listeners | defer | `ScriptViewModel::property_listener_sink` queues its callback with `after_core_borrows`; it runs at once when no Core borrow is open. | `upstream_data_binding_viewmodels::property_listeners_run_with_no_core_receiver_lent`, `upstream_wave_a_final::state_listener_actions_notify_scripts_with_no_core_receiver_lent` |
| 6 scripted listener actions | handle | `perform_listener_action_occurrence`: a `ScriptedListenerAction` performs from its handle; other actions keep their borrow (their notifications are family 5). | `upstream_scripting_listener_action::scripted_listener_actions_run_with_no_core_receiver_lent` |
| 7 artboard input hydrate | handle | `ScriptInputArtboard::hydrate_occurrence` reads the referenced artboard, name and scripted object under a shared borrow and gives the script its artboard after. | `upstream_scripting_artboard::artboard_inputs_hydrate_with_no_core_receiver_lent` |
| 8 scripted layout | handle | `ScriptedLayout::control_size_occurrence` / `display_scale_changed_occurrence`: the size is written under the borrow, the resize runs after, an update request is applied under a new borrow. Used by `control_size_handle` and `ScriptBackend::set_display_scale`. | `scripted_layout::layout_scripts_resize_with_no_core_receiver_lent` |
| 9 path effects | handle | Fill and Stroke register a component-graph update handler (`Fill::update_occurrence`, `Stroke::update_occurrence`): the paint is borrowed to snapshot its source path (`begin_effect_pass`), the chain (`effects_container::update_effect_chain`) steps each effect from its handle with the paint only read, group and target effects run their children as a chain from their handles, a scripted effect is borrowed only before and after its script, and the paint is borrowed again to keep the source and (stroke) update its render paint, after the effects as before. `ScriptedPathEffect::advance_occurrence` does the same for the effect's advance. | `upstream_scripted_path::scripted_path_effects_update_with_no_core_receiver_lent`, runtime `upstream_scripting_path_effect::scripted_path_effects_run_with_no_core_receiver_lent` |
| 10 artboard list rows | handle | `artboard_source_for_index` reads the item's source artboard and override under the list's borrow; the instance (whose scripts hydrate) is made after it ends. | runtime `upstream_wave_b_expected_red::component_list_rows_instance_with_no_core_receiver_lent` |

The borrowed paths stay for callers that still hold the object (for example
`Fill::update` through `component_update`, `with_script_input`'s other uses):
they behave as before.

## Ordering differences

* **Property listeners (family 5)** now run when the outermost Core borrow on
  the thread ends (shared ones included), not inside the write. Within one
  release they run in the order raised, after any queued picture-layout
  refreshes raised before them. A listener raised with no borrow open runs at
  once, as before. Inside the Lua userdata's own suppression nothing changed.
  No suite, golden or scripted golden observed the change (results below).
* No other family reorders script calls: each handle-level path runs the same
  steps in the same order, only with the borrow released around the script.
  The stroke's render-paint update still follows its effects.

## Brief items

* **NaN distance in path and contour measures** (H4 corpus,
  `bindings/path_measure_nan_distance`, `bindings/contour_measure_nan_distance`):
  fixed in `contour_measure.rs`: `get_pos_tan` measures a NaN distance from
  the start (as a negative one), so `positionAndTangent` and `warp` no longer
  trip `find_segment`'s assert. Test `a_nan_distance_measures_from_the_start`.
  `get_segment` (extract) already clamped NaN away. C++ release builds would
  return NaN here (its asserts are debug-only); no existing content passes a
  NaN distance without panicking today, so nothing observable changes.

## What remains

Nothing the three suites reach: with `ENFORCED` on, all three run with no
report and no abort. Known paths no suite reaches with a script entered are
under "Not verified" (two ScriptInputArtboard setters, `measure_layout`).
H2's view-model value files were out of scope; G4's
`patch_property_userdata` row is theirs and did not appear in the survey.

## Checks

All on the tip before this doc (`f69eef52d9`), with `NUXIE_EXECUTION=local`,
check enforced (no survey variable). Load averages are `uptime`'s 1/5/15
minute figures at the start and end of each run; H2 was building on the same
host throughout.

| Check | Result | Load |
| --- | --- | --- |
| `runtime.py test -p nuxie-runtime` | 256/256 pass, 0 reports | 12.5 → 4.9 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42; the one red is the known `stateful_component_image_bind`; 0 reports | 4.9 → 6.7 |
| `runtime.py test -p nuxie-scripting` | 39/39 pass, 0 reports | 6.7 → 15.9 |
| Free disk | 62–68 GB throughout these runs (stop line 60 GB); an earlier dip to 56 GB paused the lane until the coordinator cleared it | — |
| `tools/ratchets/panic-lints.sh` | nuxie-scripting 0 = 0 (every lint); nuxie-runtime (no committed row) against a scratch baseline taken at `b495c08c29`: 5046 → 5042 (`expect_used` 1349 → 1346, `unwrap_used` 626 → 625, others +0) | 12.5 |
| `tools/ratchets/unsafe-count.sh` | nuxie-runtime 221 = 221 (scratch baseline at `b495c08c29`), nuxie-scripting 46 (committed baseline 47) | 12.5 |
| `make golden-compare` | exact 356, diverges 8 (364 entries) | 14.7 → 16.7 |
| `make scripted-golden-compare` | exact 348, diverges 16 (364 entries) | 19.3 → 16.6 |
| `codex review --base levi/luaur-client-stage1f` | round 1: no actionable findings; no further rounds needed | — |

The first ratchet run on the lane's tip found nuxie-runtime `expect_used` +3
and `unwrap_in_result` +4 against base (three new invariant expects, four
existing ones moved into functions now returning `Option`); `7485432f97`
removed them. The base counts were taken by checking the two crates' sources
out at `b495c08c29` in this worktree, running the scripts with
`--update --baseline` into a scratch file, and restoring `HEAD`.

## Not verified

* The tests record reports rather than reproduce a real `RefCell` panic. A
  real panic needs the script to touch the lent object, which the pinned
  `.riv` fixtures' scripts do not do (they would need, for example, a
  view-model binding into the lent object). The new tests were not run
  against the old code. What shows they would fail there: the survey of the
  check-only commit (`17b21f0daa`) reports each family's cases under the same
  test target the new test wraps, and a recorded report fails its assertion.
* Family 10's test wraps `wave_b_focus_test_078` (`list_focus_order.riv`); the
  survey does not name the test within the wave_b target (tests run in
  parallel), so the choice is from the fixture (`list_focus_order.riv`, scripted list rows), not confirmed by running the test on the old code.
* G4 listed `ScriptAsset::instantiate_for_occurrence`/`hydrate_occurrence`
  under `CoreHandle::clone_occurrence_into`, `patch_property_userdata` from
  `create_scripted_view_model_retained`, and display scale: none appeared
  under a mutable loan in the three suites. They may only occur under shared
  borrows, or need content the suites lack.
* Found by reading, not exercised: `ScriptInputArtboard::artboard_id_changed`
  and `update_artboard` still give the script its artboard under the input's
  own borrow (`sync_referenced_artboard`); `ScriptedLayout::measure_layout`
  calls the script under the layout's borrow. No suite reaches them with a
  script entered; with `ENFORCED` on, a test that does will abort and name
  them.

## Where the brief was wrong

* The brief's list was G4's; the check found the same families plus the artboard
  list row instancing (10), the scripted path effect's advance (9), and the
  interpolator range mapper and keyframe paths (2–3). Display scale and
  `clone_occurrence_into` did not appear under a mutable loan.
* "Write the test where the script reaches the borrowed object and panics
  today": not literally done (see Not verified).
* `nuxie-runtime__unit_test` links `nuxie-scripting__test`, which links its own
  `nuxie-runtime__test` copy, so loans taken in unit-test code are not seen by
  scripts it calls; the integration targets share one copy and are covered.
