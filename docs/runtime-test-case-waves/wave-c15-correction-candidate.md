# Wave C15 correction candidate

Original candidate: `7690e928b`

Independent rejection: `f2d1f4668`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected verdict

Candidate for fresh independent rereview: **17 executable passes, two individually forceable expected-reds, zero pending**.

- 16 direct cases
- three retained Rust-safety adaptations in lifecycle cases 1, 2, and 4
- 17 pass outcomes
- two expected-red outcomes in lifecycle cases 7 and 8
- no production behavior changes

## Artboard correction

All ten artboard ports now live in an internal `cfg(test)` owner module. Every fixture creates the authored default ViewModel context through `bind_default_view_model_context_on_artboard`, which binds the Artboard and StateMachine owners in the pinned order, before the exact ten-frame settle.

The four formerly pending owner checks use the real retained state:

- Cases 1, 4, and 5 query `semantic_tree.manager.node_by_id` for every exact id.
- Case 9 first preserves the flattened list/listItem count and parent-id assertions, then resolves each listItem with the authoritative manager and walks real `SemanticNode::parent_id` links until it finds the enclosing list role.

The remaining artboard fixtures preserve the pinned pointer/semantic actions, settle ordering, labels, traits, states, bounds, cycle counts, and idempotence assertions. With the corrected dual binding, all ten pass; the previous five reds were fixture-caused and have been reclassified.

## Lifecycle destruction reds

Cases 7 and 8 now construct a real `RuntimeSemanticData`, create and register its retained node in a real `SemanticManager`, and let the owner leave lexical scope without calling `detach`.

- Case 7 verifies the initial lookup, then fails because `node_by_id(captured_id)` remains present after owner destruction.
- Case 8 drains the initial add, destroys the owner, then fails because the next diff has zero removals rather than the exact one captured id.

Both tests are independently ignored/forceable expected-reds. They fail on the exact missing `Drop` cleanup and contain no placeholder, replay manager, explicit detach, or invented test-only teardown.

## Validation

- Focused internal sweep: 16 passed, two ignored expected-reds.
- Accepted lifecycle case 9 integration test: passed.
- Forced lifecycle case 7 alone: failed at post-drop manager lookup removal.
- Forced lifecycle case 8 alone: failed at exact removed-count assertion (`0` versus `1`).
- Strict Wave C15 identities and official evidence locators: 19/19 valid, zero pending.
- Repository correspondence checker and its unit suite: passed.
- Scoped formatting and diff checks: passed.
- Non-test release LLVM IR: no Wave C15 test-owner symbols retained.

This correction receipt is candidate evidence only and does not self-accept Wave C15.
