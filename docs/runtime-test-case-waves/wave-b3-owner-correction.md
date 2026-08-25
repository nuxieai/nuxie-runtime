# Wave B3 exact-owner correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Corrected candidate: `bd78d45e5`

Independent rejection receipt: `4907be33c`

Scope: rejected cases 1, 3, 4, 18, 46, 48, 49, 53, 54, 55, 68, 76,
77, and 80-85 from the 85 active `TEST_CASE`s in
`tests/unit_tests/runtime/focus_test.cpp`. The independently accepted 66 rows
remain semantically unchanged.

## Corrected census

- 50 direct rows and 35 Rust-safety-adapted rows;
- 70 passing executable rows;
- 12 executable expected-red rows;
- three safe-Rust null-receiver exclusions;
- zero pending or unverified rows.

The 19 rejected rows now split into 13 passes and six exact expected-red
owner seams. The latter are cases 3, 4, 18, 54, 76, and 80.

## Owner-exact changes

- Case 1 now asserts the explicit false `isScope` default.
- Cases 3, 4, and 18 retain the exact FocusNode/Focusable/FocusManager owner,
  key and text values, gamepad snapshot values, and action order. They fail
  only because direct FocusNode delegation and direct FocusManager primary-
  owner input routing are absent.
- Cases 46 and 49 use fresh empty StateMachineInstances. Cases 53-55 have
  separate tests for plain/keyboard/plain switching, the selected external
  manager, and the StateMachine clear facade. Case 54 exposes a real red:
  external selection preserves focus but clears the focused owner's keyboard
  capability.
- Case 48 independently proves both the false default and true override for
  `acceptsKeyboardInput`.
- Case 68 maps the focused retained owner to the newly mounted immediate
  artboard occurrence.
- Case 76 reaches the exact named List structural scope and proves its shared
  manager, flags, and absent focusable before failing on the missing pinned
  scope name. Case 77 proves the list's retained parent and live manager
  parent are the immediate container's direct FocusData node.
- Case 80 forces the mounted list-item manager mismatch, rebuilds the parent
  topology, proves the row and its children survive, and fails only because
  the child StateMachine manager is not reinstalled.
- Cases 81-85 assert the complete immediate-artboard name order. They retain
  and compare exact primary node ids where upstream retains a focus pointer,
  and case 83 additionally proves the foreign mounted state machine shares
  the root manager domain.

## Production boundary

No production behavior changed. The runtime-source additions are all
`#[cfg(test)]` owner access/construction seams that mutate or inspect the real
retained manager rather than implementing a second focus model.

## Validation

- focused corrected-owner pass sweep: 13 pass candidates green;
- all six corrected expected-red tests forced at their declared exact seams;
- strict Wave B3 identity/name/source/evidence/ignore/census validation;
- repository correspondence checker and its unit suite;
- strict Wave A 259-row validator after the consolidated locator refresh.

This is a correction candidate pending fresh independent semantic rereview;
it does not self-accept the rejected rows.
