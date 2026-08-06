# Upstream Test Findings — open coverage gaps and production divergences

Live findings from porting the upstream C++ unit-test suite. Each finding is
anchored by an `#[ignore]`d Rust test that retains the upstream assertion
exactly as written — production behavior was not changed and no assertion was
weakened. When a finding is fixed, the ignored test is activated and its entry
here is removed. The full per-file disposition audit this was distilled from is
in git history (`docs/runtime-frame-loop-test-backfill-bc.md`); per-row status
lives in `test-correspondence-manifest.toml`.

Of the 420 finding-recorded upstream assertions, 22 were literal production
failures (4 click-sequence sites, 1 fresh-FocusNode site, and 17 silver
sites); all 22 were fixed at their root causes in commit `9684cdf7` and their
pinned tests are active and unweakened
(`cpp_probe.rs::upstream_click_event_fixture_reports_exact_group_click_sequence`,
`focus_data.rs::upstream_focus_node_fresh_focusable_defaults_to_null`, and the
silver-corpus test `upstream_fl_bc_resolved_silver_assertions`). The remaining
398 are blocked capability/harness observables retained by linked ignored
tests — including one silver assertion site
(`multi_listeners`) whose replay needs a scripting-capable silver runner, not
a runtime change.

### Finding: mutable animation quantize

`linear_animation_test.cpp:85–87` switches the imported definition's
`quantize` property off and requires the half-second sample to become `200`.
Rust imports and correctly tests the authored `true` value (`160`) but exposes
`RuntimeLinearAnimation` definitions read-only, so the mutation assertion is
retained by
`cpp_probe.rs::upstream_quantize_toggle_requires_missing_mutable_definition_api`.

### Finding: focus fixture surface

The exact Focusable pointer/delegation cases and 16 remaining fixture cases
require bindable-artboard swaps, VM assets, component-list occurrences,
and repeated focus-tree builds through an occurrence-facing API not exposed by
the Rust focus test seam. Upstream's per-node `isScope()` and `manager()`
pointer observables (`focus_test.cpp:91/93`) are likewise not represented
per node: scope topology and manager ownership live on `FocusManager`. They
are retained by
`focus_data.rs::upstream_focusable_identity_and_fixture_swap_contracts_need_runtime_occurrence_surface`;
generic focus-manager tests are intentionally not claimed as equivalents.

### Finding: silver hit-test fixtures

28 unsupported-action assertions terminate in pinned
`SerializingFactory::matches` goldens, but their action streams contain
layout-computed pointer expressions or long generated loops that the current
silver action interpreter cannot encode. They are retained by
`cpp_probe.rs::upstream_hit_test_fixtures_require_unsupported_dynamic_pointer_actions`.
Two multitouch silvers are byte-exact active tests; the four hit-test silvers
that were literal failing findings are fixed and byte-exact in the active
silver-corpus test `upstream_fl_bc_resolved_silver_assertions`.

### Finding: silver scripted-listener harness gap

Upstream `state_machine_test.cpp:600` ("Listeners with multiple types of
events") terminates in `silver.matches("multi_listeners")` — one assertion
site. `multi_listeners.riv` carries five `ScriptAsset`/`ScriptedListenerAction`
pairs; upstream's `File` import auto-creates the scripting VM when script
assets are present (`src/file.cpp:688-694`) and
`ScriptedListenerAction::performStateful` runs the script on dispatch
(`src/animation/scripted_listener_action.cpp`). The silver runner's
`Execution::run` builds raw `nuxie-runtime` instances with no
`nuxie-scripting` VM and never attaches the fixture's `ScriptAsset`
occurrences, so the scripted action is inert and the replay diverges at
frame 2/op 253 (expected `makeRenderPath`, got `drawPath`). This is a silver
harness capability, not a runtime divergence: the same fixture is `exact` in
the scripted golden lane (`corpus.toml` `multi_listeners`, samples 0/0.5/1),
where both runners execute scripting. Retained by the silver-corpus test
`silver_backfill_cases.rs::upstream_fl_bc_multi_listener_scripted_action_assertion`;
the other nine formerly-divergent streams (17 assertion sites) are active in
`upstream_fl_bc_resolved_silver_assertions`.

### Finding: state-machine fixture surface

57 capability assertions require exact fixture occurrence, reset-pool,
view-model rebinding, or silver observables not discharged by the
similarly-shaped synthetic differentials. They are retained by
`cpp_probe.rs::upstream_state_machine_fixture_contracts_without_exact_runtime_equivalents`.
C++ reset-pool counts were not mislabeled class-D skips because each source
case also contains runtime state assertions.

### Finding: scripting fixture oracles

80 assertions require pinned script-console, view-model-result, or silver
outputs. Wrapper/lifecycle tests are useful but not assertion-equivalent, so
the gap is retained by
`cpp_probe.rs::upstream_scripting_fixture_contracts_require_script_and_silver_oracles`.
