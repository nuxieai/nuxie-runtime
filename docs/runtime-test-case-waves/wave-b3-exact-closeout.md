# Wave B3 exact focus closeout

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 85 active `TEST_CASE`s in
`tests/unit_tests/runtime/focus_test.cpp`.

## Final census

- 52 direct rows;
- 33 Rust-safety-adapted rows;
- 74 passing rows;
- eight executable expected-red rows;
- three C++ null-receiver cases that are not applicable in safe Rust;
- zero pending or unverified rows.

The semantic correction had rejected 32 smoke/proxy rows. Their final split is
six direct passes, five direct expected-red cases, and 21 Rust-safety-adapted
passes. No row is accepted merely because an 85-entrypoint census exists.

## Exact executable owners added

- Seven complete Silver action streams now replay `focus_collapsing`, both
  `keyboard_listener` variants, `focus_traversal`, `focusable_element`,
  `list_focus_order`, and `focus_test` against frozen SRIV bytes.
- The text-input case executes every focus/key/text/advance action and every
  `isFocused`/`hasKeyed`/`hasTexted` assertion on the real fixture view model.
- The missing-comparator case imports a complete synthetic state machine with
  an authored `TransitionFocusCondition` and proves its guarded transition
  remains blocked.
- The bindable and swappable cases execute real view-model artboard mutations,
  verify the live nested source graphs, and assert focus preservation and
  complete traversal counts before and after each swap.
- Private Focusable delegation and FocusState cases resolve to executable
  owner-level tests for keyboard, text, gamepad, lifecycle callbacks, keyboard
  capability, manager selection, and clear/switch behavior.

## Expected-red seams

- freeing a transient parent erases the child that survives it upstream;
- swapping an unrelated bindable nested artboard clears focus held on Main;
- `component_list_1.riv` does not construct the two required retained list
  focus topologies;
- four exact Silver streams diverge at their first frozen operation: collapsing
  paint identity, both keyboard render streams, and the list path coordinate.

Every expected-red case was forced individually and failed after executing its
real setup and action stream.

## Rust-safety boundary

Rust intentionally does not expose C++ `FocusNode*`, `Focusable*`,
`FocusManager*`, or `primaryFocusImmediateArtboard()` pointer identity. Adapted
rows instead assert stable retained identities, attachment/topology, source
graph membership, focus state, traversal position/count, event order, and
mutation results. Case 80 likewise cannot manufacture the C++-only temporary
manager-pointer mismatch on a mounted list-item instance; it repeats the live
retained-context wiring through the public owner and asserts the row focus
invariant. The shard records those omitted pointer observables explicitly and
does not call them literal pointer parity.

## Validation

- strict pinned identity, name, source line, evidence locator, adaptation, and
  ignore-reason validation: 85/85;
- focused runtime target: 81 pass, four ignored;
- focused Silver target: three pass, four ignored;
- eight expected-red tests forced individually: eight concrete failures;
- repository correspondence checker: 157 files and 1,404 upstream cases;
- checker unit suite: 24/24.

No production runtime source was changed by this correction.
