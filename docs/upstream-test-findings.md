# Upstream Test Findings — open coverage gaps and production divergences

Live findings from porting the upstream C++ unit-test suite. Each finding is
anchored by an `#[ignore]`d Rust test that retains the upstream assertion
exactly as written — production behavior was not changed and no assertion was
weakened. When a finding is fixed, the ignored test is activated and its entry
here is removed. The full per-file disposition audit this was distilled from is
in git history (`docs/runtime-frame-loop-test-backfill-bc.md`); per-row status
lives in `test-correspondence-manifest.toml`.

Of the 420 finding-recorded upstream assertions, 23 are literal production
failures (4 click-sequence sites, 1 fresh-FocusNode site, and 18 silver sites);
397 are blocked capability/harness observables retained by linked ignored tests.

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
Two multitouch silvers are byte-exact active tests; four other hit-test
silvers are literal failing findings below.

### Finding: silver runtime divergences

The silver-corpus test
`upstream_fl_bc_divergent_silver_assertions` replays ten literal action
streams and compares them to the pinned `.sriv` files. It is ignored after
reporting these production divergences:
`focus_traversal` (frame 0/op 95), `hittest_ab1` (frame 1/op 153),
`hittest_ab1_parent` (frame 1/op 192), `hittest_ab1_grand_parent` (frame
2/op 304), `hittest_nested` (frame 1/op 155), `multi_listeners` (frame 2/op
253), `sorted_listeners` (frame 0/op 32), `transition_actions` (frame 2/op
72), `transition_duration_bind_list` (frame 0/op 13), and
`transition_duration_bind_nested` (frame 0/op 57). These account for 18
upstream assertion sites.

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
