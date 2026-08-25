# Wave B3 final independent semantic rereview

Reviewed commits: `bd78d45e5`, `f2c0a7831`, and `8c3d9c963`

Prior rejection receipt: `4907be33c`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 85 active `TEST_CASE`s in
`tests/unit_tests/runtime/focus_test.cpp`.

Verdict: **REJECTED for seven stale evidence locators; semantically accepted
85/85**

## Exact semantic census

| disposition | direct | adapted | total |
|---|---:|---:|---:|
| accepted pass | 44 | 26 | 70 |
| accepted executable expected-red | 6 | 6 | 12 |
| accepted not-applicable | 0 | 3 | 3 |
| semantic reject | 0 | 0 | 0 |
| **total** | **50** | **35** | **85** |

The 19 cases rejected by `4907be33c` are now semantically acceptable. Their
final split is 13 passes (cases 1, 46, 48, 49, 53, 55, 68, 77, and 81-85)
and six executable expected-red owner seams (cases 3, 4, 18, 54, 76, and
80). The other 66 rows retain their prior semantic acceptance: 57 passes,
six executable expected-red cases, and three safe-Rust null-receiver
exclusions.

This receipt nevertheless rejects the combined candidate because an evidence
locator is part of the proof. Seven Silver locators introduced by the
corrected shard at `f2c0a7831` and preserved by the consolidated locator commit
do not resolve to their declared tests.

## Re-adjudication of the 19 corrected cases

- Case 1 now asserts every fresh-node default, including the pinned false
  structural-scope result. This is the exact owner rule because upstream
  `FocusNode::isScope()` is defined by whether that node's children are
  non-empty.
- Cases 3 and 4 retain the exact FocusNode and Focusable relationship and
  attempt key, text, focused, and blurred in pinned order. Case 18 additionally
  proves the no-focus key/text/gamepad false results, selects the retained
  primary node, and reaches the missing FocusManager primary-owner input
  routing seam. None fails through an unconditional panic or a
  StateMachine-level proxy.
- Cases 46, 49, and 53-55 execute on the actual StateMachineInstance focus
  facade. They cover the fresh empty manager, two explicit additions,
  plain-to-keyboard-to-plain switching, external manager selection, and the
  clear facade. Case 54's expected-red is the concrete loss of keyboard
  capability after external-manager installation.
- Case 48 independently proves the exact false default and true override of
  the Focusable keyboard-input owner.
- Case 68 maps the selected focus owner to the newly mounted immediate
  artboard occurrence, rather than accepting global graph membership or a
  traversal count.
- Case 76 selects the named List structural-scope node, proves shared-manager
  membership, false focus/traversal flags, and absent Focusable, and then fails
  only at the missing pinned scope name. Case 77 resolves the list's closest
  focus node to its immediate container's direct FocusData owner. Neither is
  the former generic `has_focus_nodes` proxy.
- Case 80 deliberately removes the mounted list item's external manager,
  proves the mismatch, performs cleanup/build, proves the retained row and
  child topology, and fails only when the child's manager is not reinstalled
  into the parent's manager domain.
- Cases 81-85 assert complete immediate-artboard name order and stable retained
  focus identities where required. Case 83 also proves that the foreign leaf
  state machine shares the root manager. Case 85 assigns the runtime's explicit
  `u32::MAX` null-artboard sentinel before the first advance, observes the
  initially empty order, then inserts the first swap at the authored middle
  position.

## Prior-accepted 66-case audit

Only the 19 previously rejected rows changed status, outcome, evidence, note,
or adaptation metadata between the rejected and corrected shards. For the 63
executable rows in the previously accepted set, every primary Rust test body
is token-identical between `bd78d45e5` and `f2c0a7831` after whitespace is
removed; the visible rewrites are formatting only. Cases 47, 64, and 66 remain
valid `not-applicable` rows because their sole upstream observable is invoking
an instance method through a null C++ receiver, a state not constructible by
the safe owned Rust API.

## Blocking locator defect

`tools/silver-corpus/tests/wave_b3.rs` is clean and unchanged since
`bd78d45e5`. Its seven evidence symbols currently begin at these lines:

| case | symbol | declared line | actual line |
|---:|---|---:|---:|
| 70 | `wave_b3_focus_collapsing` | 43 | 39 |
| 71 | `wave_b3_keyboard_listener` | 49 | 43 |
| 72 | `wave_b3_keyboard_listener_keyboard_input` | 55 | 47 |
| 74 | `wave_b3_focus_traversal` | 60 | 50 |
| 75 | `wave_b3_focusable_element` | 65 | 53 |
| 78 | `wave_b3_list_focus_order` | 71 | 57 |
| 79 | `wave_b3_focus_test` | 76 | 60 |

The strict validator therefore resolves 78/85 Wave B3 rows and rejects these
seven. This is a metadata-only correction: classifications, outcomes, symbols,
fixtures, actions, assertions, and test code must remain unchanged.

## Execution and mechanical gates

- pinned upstream HEAD: exact;
- focused runtime integration target: 81 pass, four ignored;
- corrected exact-owner unit set: 11 pass, six ignored;
- focused Silver target: three pass, four ignored;
- all 12 declared expected-red rows forced individually: 12/12 selected one
  test and failed at their declared concrete seams;
- repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green;
- correspondence-checker unit suite: 24/24 green;
- non-test LLVM IR contains none of the corrected test fixtures, helper seams,
  or test symbols; production behavior remains unaffected;
- scoped `git diff --check`: green;
- strict Wave B3 upstream identity, name, source-line, classification,
  adaptation, and ignore-reason checks are otherwise green, but exact Rust
  locator validation is red for the seven rows above.

No candidate or production source was changed by this review.
