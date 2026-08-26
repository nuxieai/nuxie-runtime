# Wave C16 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C16 covers all 18 active Catch cases in pinned
`semantic_dispatch_test.cpp` and `semantic_focus_list_test.cpp` at upstream
SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It changes test and evidence
semantics only and does not declare Wave C16 accepted.

## Exact census

- 18/18 cases mapped, zero pending;
- classifications: 14 direct and four Rust-safety adapted;
- outcomes: 16 pass and two expected-red;
- listener dispatch/back-reference/focused-state: nine pass;
- manager lookup/removal/request-focus: seven pass;
- focus-list fixture ordering: two genuine production expected-reds.

## Executable evidence

Cases 1-16 exercise concrete `RuntimeSemanticData`, `SemanticNodeHandle`, and
`SemanticManager` owners. Two distinct listeners are retained where pinned;
the tests preserve action order, counts, sibling state flags, manager-assigned
ids, node identity, removal, and negative focus paths. Cases 7, 8, 15, and 16
use the structured Rust-safety adaptation for stable local owner identity in
place of raw C++ owner/core pointers. Case 8 follows the live
manager lookup -> retained node owner identity -> concrete owner -> fire chain.

Cases 17-18 load and settle the exact pinned
`focus_nodes_list_order.riv` fixture. They preserve all four expected bounds,
roles, parent ids, sibling indices, root child-id ordering, and minimum-id
search. Both are individually forceable ignored reds: case 17 fails at the
second slot's pinned `minY == 75` geometry, and case 18 fails with minimum-id
slot 0 rather than pinned slot 3.

No fake resolver, injected back-reference, local dispatch algorithm, merged
aggregate, composed separate green, report/corpus proxy, placeholder panic, or
production behavior change is used.

## Gates

- focused non-incremental suite: 16 pass / two ignored expected-red;
- individually forced expected-reds: 2/2 reached their declared live seams;
- official strict shard validator: 18/18 green, with 14 direct, four adapted,
  16 pass, and two expected-red;
- focused evidence-locator/owner validator: 18/18 unique live symbols;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green;
- default non-test LLVM IR contains no Wave C16 test-owner symbols.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`.
