# Upstream Test Findings — open coverage gaps and production divergences

Live findings from porting the upstream C++ unit-test suite. Each finding is
anchored by an `#[ignore]`d Rust test that retains the upstream assertion
exactly as written — production behavior was not changed and no assertion was
weakened. When a finding is fixed, the ignored test is activated and its entry
here is removed. The full per-file disposition audit this was distilled from is
in git history (`docs/runtime-frame-loop-test-backfill-bc.md`); per-row status
lives in `test-correspondence-manifest.toml`.

Of the 402 finding-recorded upstream assertions, 5 are literal production
failures (4 click-sequence sites and 1 fresh-FocusNode site); 397 are blocked
capability/harness observables retained by linked ignored tests.

### Finding: click up outside

This is a production-behavior failure, not merely a missing harness surface.
The literal pinned `click_event.riv` port in
`cpp_probe.rs::upstream_click_event_fixture_reports_exact_group_click_sequence`
passes through the setup and first click (A1–A8, now active in
`upstream_click_event_fixture_initial_and_first_click_contract`), then fails
A9–A12. Upstream `hittest_test.cpp:284–310` requires cumulative event counts
`[1, 1, 1, 2, 3]`; Rust reports `[1, 2, 2, 3, 4]`, beginning with the
pointer-down at `(75,75)` and pointer-up at `(300,75)`. The full sequence test
is ignored exactly as written.

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

The exact Focusable pointer/delegation cases and 16 remaining fixture cases
require bindable-artboard swaps, VM assets, component-list occurrences,
and repeated focus-tree builds through an occurrence-facing API not exposed by
the Rust focus test seam. They are retained by
`focus.rs::upstream_focusable_identity_and_fixture_swap_contracts_need_runtime_occurrence_surface`;
generic focus-manager tests are intentionally not claimed as equivalents.

### Finding: silver hit-test fixtures

28 unsupported-action assertions terminate in pinned
`SerializingFactory::matches` goldens, but their action streams contain
layout-computed pointer expressions or long generated loops that the current
silver action interpreter cannot encode. They are retained by
`cpp_probe.rs::upstream_hit_test_fixtures_require_unsupported_dynamic_pointer_actions`.
The two multitouch silvers and the four hit-test silvers are exact active
tests in `silver_backfill_cases.rs`.

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
