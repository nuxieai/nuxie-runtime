# Wave C16 independent adversarial review

Author commit: `22ca9b3198f178eb7066d72e7941a5c1eee9558b`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Sources:

- `tests/unit_tests/runtime/semantic_dispatch_test.cpp` (16 cases)
- `tests/unit_tests/runtime/semantic_focus_list_test.cpp` (two cases)

Verdict: **ACCEPTED — 18/18 exact case streams**

This review kept the author code and Wave C16 ledger frozen while comparing
every pinned Catch case against its executable Rust owner. It reviewed fresh
fixture construction, listener identity and counts, action and mutation order,
manager-assigned ids, lookup and removal identity, focused sibling flags,
negative focus routing, and every focus-list geometry and ordering assertion.

## Exact-stream verdict

- Cases 1-6 preserve the exact listener multiplicity, action order, positive
  counts, zero counts, removal identity, and silent no-listener calls. No
  dispatch case is merged with another.
- Case 7 creates a live `RuntimeSemanticData` node and asserts the node's
  production-retained owner local id against the creating owner. The omitted
  C++ observable is only raw-pointer equality.
- Case 8 preserves the complete manager lookup -> returned live node ->
  retained owner identity -> concrete `RuntimeSemanticData` owner -> fire
  chain. The concrete owner is retained in an owner collection and selected
  by the identity written by `RuntimeSemanticData::semantic_node`; no resolver,
  back-reference, or node identity is injected. Reference identity and the
  listener effect are both asserted before the listener is removed.
- Cases 9-11 preserve the exact unknown id, boundary-node state and absent
  owner, registered node identity, removal order, and post-removal lookup.
- Cases 12-13 preserve the exact `Selected | Expanded` initial flags, both
  focused-state calls, all six sibling/focused assertions, and the two false
  no-op calls before node creation.
- Cases 14-16 execute the production `SemanticManager::request_focus` owner.
  The callback returns true unconditionally, so each false result proves that
  the unknown, ownerless, or boundary route was rejected rather than hidden by
  a test callback. Case 15 also directly asserts the absent retained core-owner
  identity. The inapplicable C++ observables are raw owner/sibling pointer
  traversals only; the Rust stable-identity and callback boundary is the
  structured ownership adaptation.
- Cases 17-18 load and settle the exact pinned fixture independently. Case 17
  preserves all root parent ids, button roles, sibling indices, four bounds per
  slot, the root children update, its length, and child-id order. Case 18
  preserves the independent minimum-id scan and final slot assertion.

No merged aggregate, injected owner identity, fake resolver, test-local owner
algorithm, separate-green composition, altered constant, omitted false/no-op
call, proxy observable, unconditional failure, or production behavior change
was found. The author commit adds only `#[cfg(test)]` module declarations,
test/evidence files, and two test-symbol renames in a pre-existing focus-list
port.

## Expected-red verification

- Case 17 was forced alone and failed in the live diff at the pinned slot
  geometry assertion: `min_y` disagrees with expected slot 1 value `75`.
- Case 18 was forced alone from a fresh fixture and failed at the independent
  minimum-id ordering assertion: Rust reports slot `0`, pinned C++ requires
  slot `3`.

Both failures reached their declared production seams; neither depends on the
other ignored test.

## Gates

- Focused non-incremental owner suite: 16 passed, zero failed.
- Individually forced non-incremental expected-reds: 2/2 failed at the declared
  live assertions.
- Strict Wave C16 ledger validation: 18/18 green; 14 direct and four adapted,
  with 16 pass and two expected-red.
- Repository correspondence checker: 157 files / 1,404 cases, green.
- Correspondence checker unit suite: 24/24 green.
- Non-incremental `cargo check -p nuxie-runtime`: green.
- Scoped rustfmt, JSON parse, author-diff whitespace, locator, exact pinned
  line/name, and production-freeze checks: green.

Wave C16 is independently accepted at **18/18** cases.
