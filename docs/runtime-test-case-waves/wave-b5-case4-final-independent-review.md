# Wave B5 case 4 final independent review

Reviewed correction: `54be89418`

Prior rejection: `4ff8d1713`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — Wave B5 is 31/31 semantically accepted**

## One-row verdict

`hittest_test.cpp#4`, **hit test on opaque nested artboard**, now preserves the
exact pinned semantic question.

- It selects the artboard slot named `second-nested`, then requires
  `nestedAnimations()[0]` itself to be the nested `StateMachine` occurrence.
- It retains that exact host, occurrence, animation, and state-machine identity
  and revalidates the same owner at every later nested-input read. Stable Rust
  IDs replace only unsafe C++ pointer retention; no traversal or topology proxy
  replaces the owner question.
- It advances the outer artboard first and the outer state machine second, then
  asserts the selected nested machine's `bool-target` is false. The rejected
  pre-initialization assertion and alternate `update_components` sequence are
  gone.
- It executes the pinned pointer sequence. The forced test reaches the real
  `pointer_down(301, 50)` divergence and fails at the immediately following
  assertion because `second-gray-toggle` is `false` instead of `true`.

The row is therefore accepted as a precise executable expected-red. It would
pass when the real nested-artboard hit-boundary behavior is corrected.

## Final census

| upstream file | cases | accepted pass | accepted expected-red | rejected |
|---|---:|---:|---:|---:|
| `hittest_test.cpp` | 21 | 13 | 8 | 0 |
| `ik_constraint_test.cpp` | 1 | 1 | 0 | 0 |
| `ik_test.cpp` | 2 | 2 | 0 | 0 |
| `image_asset_test.cpp` | 2 | 2 | 0 | 0 |
| `image_decoders_test.cpp` | 5 | 3 | 2 | 0 |
| **total** | **31** | **21** | **10** | **0** |

The shard remains 29 direct and two `rust-safety` adaptations. The two
adaptations are the previously accepted stable ImageAsset identity translations.
The platform-conditional bad PNG and JPEG decoder classifications remain
accepted exactly as adjudicated in the initial independent review.

## Scope and gates

The correction changes only case 4's owner helper and test body. The other 30
accepted rows are unchanged. Its larger exact-owner helper mechanically shifted
seven direct hittest symbols; all seven refreshed ledger locators resolve to
their exact function declarations.

- pinned upstream HEAD: exact;
- strict ledger: 31/31 case identities and evidence locators exact;
- shard census: 29 direct / two adapted, 21 pass / ten expected-red;
- corrected row forced individually: selected one ignored test and failed at
  the post-`x=301` assertion (`false` versus `true`), after the exact setup and
  assertion prefix;
- unaffected direct hittest pass rows: five passed and two expected-red rows
  remained ignored in the focused module run;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR contains no Wave B5 test module or symbol;
- JSON parsing and scoped `git diff --check`: green.

The focused compile and execution gates were run in a clean detached checkout
of `54be89418`, because unrelated in-progress Wave B4 edits in the shared
worktree do not currently compile. That unrelated state does not affect this
candidate or verdict.
