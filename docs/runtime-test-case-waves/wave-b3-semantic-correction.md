# Wave B3 semantic correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 85 `TEST_CASE`s in `tests/unit_tests/runtime/focus_test.cpp`.

The first Wave B3 draft created 85 compiling entry points, but that census was
not semantic proof. In particular, the fixture-heavy entries only imported a
fixture and reached its first focus stop, while several private action-owner
entries asserted a nearby `FocusManager` effect. This correction rejects those
rows instead of treating executable smoke coverage as exact parity.

## Corrected census

- 41 direct rows;
- 12 adapted rows (nine Rust stable-identity/event-stream adaptations and
  three C++ null-receiver call shapes that are not applicable in safe Rust);
- 32 pending rows;
- 47 passing executable rows;
- three executable expected-red rows;
- three not-applicable rows.

The 32 pending rows are deliberately unclaimed. They include incomplete
`Focusable` delegation/public-surface assertions, state-machine facade cases
whose exact private owner is not yet exposed by a dedicated test, transition
focus-condition guard coverage, and the fixture/Silver/swap/list scenarios
whose complete pinned action and assertion streams have not yet been
translated.

The three expected-red rows execute translated work and fail at concrete
runtime seams:

- case 31: removing a transient row erases the child that survives the parent
  in pinned C++;
- cases 76 and 77: `component_list_1.riv` builds no retained list focus
  topology, including the Node-hosted list variant.

## Validation

- strict pinned identity, line, name, evidence locator, and ignore-reason
  validation: 85/85 rows;
- focused Rust target: 82 pass, three ignored;
- all three expected-red tests forced individually: each selects exactly one
  test and fails at its declared seam;
- repository correspondence checker: 157 files and 1,404 pinned cases;
- checker unit suite: 24/24.

No production source was changed by Wave B3.
