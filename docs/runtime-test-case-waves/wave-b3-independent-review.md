# Wave B3 independent semantic review

Reviewed commit: `bd78d45e5bd3ae43f049d5f2f79837c9c2d583b8`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 85 active `TEST_CASE`s in
`tests/unit_tests/runtime/focus_test.cpp`.

Verdict: **REJECTED**

## Acceptance rule

Every row was checked against the pinned C++ fixture/setup, action order,
production owner, and assertions. Stable Rust identities and event streams are
valid adaptations only when they answer the same semantic question. A public
facade, traversal count, global graph-membership check, or generic fixture
failure does not substitute for an assertion about a particular FocusNode,
StateMachineInstance, list row, nested-artboard occurrence, or shared manager.

## Exact review census

| disposition | direct | adapted | total |
|---|---:|---:|---:|
| accepted pass | 42 | 15 | 57 |
| accepted executable expected-red | 6 | 0 | 6 |
| accepted not-applicable | 0 | 3 | 3 |
| incomplete or different semantic question | 4 | 15 | 19 |
| **total** | **52** | **33** | **85** |

The accepted set is 66 rows: 57 passing executable rows, six executable
expected-red rows, and the three safe-Rust null-receiver exclusions. The 19
rejected rows are cases **1, 3, 4, 18, 46, 48, 49, 53, 54, 55, 68, 76, 77,
and 80-85**. Seventeen are declared passing rows with narrower evidence; cases
76 and 77 are declared expected-red but fail at a proxy assertion.

## Blocking semantic findings

### FocusNode and StateMachine owner flows are replaced by nearby behavior

- **Case 1** never asserts the pinned `isScope() == false` default. That
  boolean is not a raw pointer address and cannot be removed by the declared
  Rust-safety adaptation. The repository's own ignored focus-surface finding
  still names per-node `isScope()` as an absent observable.
- **Case 3** replaces direct FocusNode-to-Focusable keyboard, text, focused,
  and blurred delegation with StateMachine scripted-input routing plus separate
  manager event tests. It does not assert the exact key/text values or the
  focused/blurred callbacks on the same retained focusable owner.
- **Case 4** clears focus on an imported StateMachine and sends input through
  the StateMachine facade. It never invokes key, text, focused, and blurred on
  a FocusNode whose focusable is absent.
- **Case 18** combines the same already-focused scripted-input helper with the
  case-4 StateMachine no-focus path. The no-focus gamepad assertion is absent,
  and the focused path only checks a three-call total rather than the pinned
  per-input counters and exact last key/text/snapshot values.
- **Case 46** loads an authored fixture that already has focus nodes. It omits
  the initial `hasFocusNodes() == false` state and the exact act of adding two
  focusables to the StateMachineInstance's manager before exercising the three
  StateMachine facades.
- **Case 48** points to a test of RuntimeFocusable construction and default
  gamepad dispatch. It never calls `acceptsKeyboardInput`, never proves its
  false default, and never exercises the true override.

### A shared helper erases case-specific FocusState assertions

- **Case 49** does not construct and query a fresh empty StateMachineInstance;
  its main helper begins already focused, while the supporting test only checks
  a bare FocusManager.
- **Case 53** mutates keyboard capability on one focused occurrence. Its
  supporting test switches two manager nodes that have no distinct keyboard
  capabilities. Neither executes the pinned plain -> keyboard -> plain switch
  while asserting both FocusState fields after every switch.
- **Case 54** verifies manager-selection phase traces and retained identity,
  but never asserts the selected external manager's focused, keyboard-accepting
  state through `StateMachineInstance::focusState` after installation.
- **Case 55** reaches an empty state through `set_focus(None)` and separately
  clears a FocusManager. It does not exercise the pinned
  `StateMachineInstance::clearFocus` facade and then assert both FocusState
  fields.

Cases 50-52 are accepted: despite sharing the helper, each required
StateMachineInstance FocusState result is explicitly executed and asserted.

### Bindable/list adaptations discard the identity and topology under test

- **Case 68** proves only that the Focusable source graph occurs somewhere in
  the nested graph walk and that some focus exists after traversal. It does not
  prove that primary focus belongs to the newly swapped nested occurrence, the
  pinned `primaryFocusImmediateArtboard() == focusableInstance` assertion.
- **Case 80** does not reproduce the regression setup. The pinned test clears a
  selected list item's external manager, verifies the mismatch, runs
  `cleanupFocusTree` plus `buildFocusTree`, and checks the item's subtree is
  under its row and shares the parent manager. The Rust test merely rebinds the
  parent machine's existing context and checks generic focus preservation and
  traversal. This is a different mutation and a different invariant.
- **Cases 81, 82, 84, and 85** reduce named immediate-artboard order and exact
  held-focus identity to two FocusState booleans and a count of successful
  `focus_next` calls. Reordered stops, focus on the wrong occurrence, or a
  replacement focus with the same booleans can all pass.
- **Case 83** additionally treats a source graph id from an independently
  loaded file as proof of foreign occurrence identity. Graph ids are local
  metadata, and the test never asserts that the swapped-in state machine shares
  the parent's focus manager, which is half of the upstream case.

These observables are semantic identities, not an attempt to compare C++ raw
pointer addresses. A stable Rust occurrence id, focused-listener owner chain,
or explicit shared-manager identity would be a valid adaptation; global
membership and traversal counts are not.

### Cases 76 and 77 are not valid expected-red ports

Both tests import `component_list_1.riv` and fail immediately at the same
generic `machine.has_focus_nodes()` assertion. Case 76 instead inspects the
named ArtboardComponentList's structural list-scope node, including its shared
manager, name, flags, and absent focusable. A structural scope can satisfy that
contract while contributing no focus stop, so the current failure does not
reach the asserted upstream seam. Case 77 finds the list's Node parent and, if
it has a direct FocusData child, compares `findClosestFocusNode(list)` with that
exact node. The Rust test performs none of that owner flow.

The six accepted expected-red rows are cases **31, 69-72, and 78**. Each
executes the material pinned setup/actions and fails at an actual divergent
runtime or frozen-SRIV result. Cases 76 and 77 must remain pending until the
list-scope/closest-focus owners are observable and the exact assertions can
run.

## Accepted Rust-safety exclusions

Cases **47, 64, and 66** are valid not-applicable rows. They exist solely to
call a C++ instance method with a null receiver. Safe Rust requires an owned
receiver and exposes no equivalent callable state; the exclusions do not hide
any non-null action behavior.

## Mechanical and execution gates

- pinned runtime HEAD verified as
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`;
- strict shard identity, line, name, evidence-locator, discovery,
  ignore-reason, and declared-census validation: 85/85 green, mechanically
  reporting 52 direct, 33 adapted, 74 pass, eight expected-red, and three
  not-applicable;
- focused runtime target: 81 pass, four ignored;
- focused Silver target: three pass, four ignored;
- all eight declared expected-red tests forced individually: each selected
  exactly one test and failed; six failures are accepted semantically, while
  the two generic component-list failures are rejected above;
- repository correspondence checker: 157 files and 1,404 pinned cases;
- correspondence-checker unit suite: 24/24;
- `git diff --check` for the candidate commit: green;
- the candidate commit changes only the Wave B3 Rust tests, Silver tests, shard,
  and closeout document; no production runtime source was changed.

Mechanical success does not promote the 19 semantically incomplete rows. Wave
B3 needs owner-exact corrections and another independent review before it can
be closed at 85/85.
