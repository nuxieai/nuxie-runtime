# Wave C15 independent adversarial review

Status: **REJECT**

Candidate: `7690e928b114b291fe7eccb9c8733d6be23c8d4f`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 10 cases in `semantic_artboard_test.cpp` and all nine cases in
`semantic_data_lifecycle_test.cpp`.

This was a review-only adjudication. It did not change candidate tests,
ledgers, fixtures, or production behavior. The `implement` and `tdd` skills
were explicitly excluded.

## Exact census

- Candidate ledger: 19 rows; eight pass / five expected-red / six pending.
- Accepted semantic evidence: **seven cases**, lifecycle cases 1-6 and 9.
- Rejected semantic evidence: **12 cases**, all 10 artboard cases and
  lifecycle cases 7-8.

## Rejected fixture and outcomes

The Rust artboard fixture does not preserve the pinned setup. Pinned
`loadFixture` creates the default ViewModel instance and binds it to both the
Artboard and StateMachine before the 10-frame settle. Rust `load_fixture`
constructs an owned handle and calls only
`StateMachineInstance::bind_owned_view_model_handle`; it never binds the
Artboard. The runtime already exposes
`bind_default_view_model_context_on_artboard` as the exact combined owner path,
and lifecycle case 9 uses that path successfully.

This invalidates artboard cases 2, 3, 6, 7, 8, and 10 as parity evidence. All
five declared reds were forced individually, but they fail after the incorrect
setup: case 2 at the initial Expanded bit, case 3 at the first missing fandom
entry, case 6 at the initial fandom set, case 7 at the initial Expanded bit,
and case 10 at zero rather than four fandom ids. Case 8 passes only after the
same incorrect setup, so its pointer/semantic convergence result is not
accepted either. A red caused by an incorrectly ported fixture is not a
renderer/runtime divergence.

## Six pending rows are executable

1. Artboard cases 1, 4, and 5 are not ownerless. `StateMachineInstance`
   retains `semantic_tree`, whose real `RuntimeSemanticTree::manager` owns the
   authoritative public `SemanticManager::node_by_id` lookup. A child
   `cfg(test)` module can observe that retained manager without changing
   production behavior or replaying the flattened diff as a proxy.
2. Artboard case 9 can use the same retained manager and real node handles.
   `SemanticNode::parent_id` plus `SemanticManager::node_by_id` can walk the
   authoritative retained ancestor chain and assert the list role after the
   flattened parent-id assertions.
3. Lifecycle cases 7 and 8 do not require an explicit detach substitute.
   `RuntimeSemanticData`, `SemanticManager::add_child`, `node_by_id`, and
   `drain_diff` are callable now. Letting the real `RuntimeSemanticData` leave
   scope and then asserting lookup removal and the one-id removed diff creates
   exact executable expected-reds. The missing `Drop` teardown is the behavior
   under test, not an unavailable seam.

No candidate executable row uses a placeholder panic, injected node identity,
or explicit detach as lifecycle evidence. The four removed artboard
placeholder tests were correctly deleted, but deleting them does not justify
pending status when the retained owner remains reachable internally.

## Mechanical and execution gates

- Pinned upstream checkout resolves to the declared SHA.
- All 19 ledger identities, ordinals, names, source lines, classifications,
  outcomes, and reasons were inspected; all 13 executable evidence locators
  resolve to unique symbols at their declared lines.
- Focused non-incremental Wave C15 suite: eight pass / five ignored, green.
- Five expected-reds forced separately with both incremental settings
  disabled: five failures at live assertions, but rejected for the fixture
  defect above.
- Repository correspondence checker: 157 files / 1,404 cases, green.
- Correspondence checker unit suite: 24/24 green.
- Candidate diff check: green; candidate changes no production behavior.

Existing user and other-lane workspace changes were preserved and are not part
of this receipt.
