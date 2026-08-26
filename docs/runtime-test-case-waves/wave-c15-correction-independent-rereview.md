# Wave C15 correction independent rereview

Status: **REJECT**

Correction: `aa8b4c4e59fff50240e8005bf33927f937cf8212`

Prior rejection: `f2d1f4668`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This fresh review kept the correction frozen and rechecked every prior
finding, every evidence locator, all three structured adaptations, the two
expected-reds, and the focused owner suites. The correction fixes every owner
and fixture defect from the prior receipt, but all 10 artboard rows still
violate the campaign's literal assertion-stream rule.

## Exact census

- Correction ledger: 19 executable rows; 17 pass / two expected-red.
- Accepted semantic evidence: **nine lifecycle cases**; seven pass / two
  executable expected-red.
- Rejected semantic evidence: **all 10 artboard cases**, solely for added
  Rust boolean assertions absent from the pinned tests.

## Remaining literal-stream defect

Pinned `loadFixture` calls `enableSemantics`, creates the default ViewModel
instance, conditionally binds it to the Artboard and StateMachine, and settles.
It does not assert a return from either operation. Corrected Rust
`load_fixture`, shared by artboard cases 1-9, adds:

```rust
assert!(
    state_machine.bind_default_view_model_context_on_artboard(&mut artboard),
    "default ViewModel instance binds to the Artboard and then StateMachine"
);
```

That boolean is not an upstream observable and is not required to reach any
owner. It can fail a behaviorally faithful port whose combined bind reports no
change, so it is not acceptable as strengthening under the campaign's literal
assertion-stream rule. Artboard cases 1-9 are rejected until the call remains
in the pinned position but its Rust return value is ignored.

Artboard case 10 separately adds three unpinned boolean assertions:

```rust
assert!(machine.enable_semantics());
assert!(!machine.enable_semantics());
assert!(machine.bind_default_view_model_context_on_artboard(&mut artboard));
```

Pinned case 10 calls `enableSemantics` twice, binds the default instance when
present, settles, and asserts only the four unique fandom ids. The Rust calls
and their order are correct, but their return values must not become extra
parity requirements. Case 10 is rejected until those three calls remain and
only the added boolean assertions are removed.

## Corrected owners accepted in substance

After the boolean assertions above are removed, no prior owner defect remains:

- the combined default-context call binds the Artboard before the StateMachine
  and precedes the exact 10 frames at 0.1 seconds;
- cases 1, 4, and 5 query exact ids through the selected internal
  `semantic_tree.manager.node_by_id` index;
- case 9 preserves flattened list/listItem counts and parent ids, then resolves
  each retained node through the same manager and follows real
  `SemanticNode::parent_id` links to a list-role ancestor; and
- all pinned labels, roles, flags, bounds, cycle counts, pointer/semantic
  actions, and final assertions are otherwise complete and correctly ordered.

No replay map, facade, synthetic manager, injected identity, placeholder
panic, or production behavior change replaces the authoritative owners.

## Lifecycle correction accepted

Lifecycle cases 7 and 8 create and register a live `RuntimeSemanticData` node
in a real `SemanticManager`, then let the data owner leave lexical scope.
Neither calls `detach` or a surrogate teardown. Forced individually, case 7
fails only because the post-drop lookup remains present, and case 8 fails
because the next removed array has length zero rather than one. The exact
captured-id assertion remains after the count assertion.

Previously accepted lifecycle cases 1-6 and 9 retain their fixtures,
constants, setter order, identities, actions, and assertion streams. All seven
continue to pass. All nine lifecycle rows are accepted.

## Mechanical and execution gates

- Strict Wave C15 ledger audit: 19/19 identities, ordinals, pinned lines,
  exact names, unique symbols, and evidence lines valid; 16 direct, three
  adapted, 17 pass, two expected-red, zero pending.
- All three `rust-safety` adaptations for lifecycle cases 1, 2, and 4 are
  structurally complete and identify only raw C++ pointer observables.
- Focused non-incremental internal suite: 16 passed / two ignored; lifecycle
  case 9 integration suite: one passed.
- Both expected-reds forced independently with both incremental settings
  disabled: two failures at exact post-drop owner assertions.
- Repository correspondence checker: 157 files / 1,404 cases, green.
- Correspondence checker unit suite: 24/24 green.
- Correction diff check and JSON parsing: green.
- Production freeze: green; executable additions are contained by `#[cfg(test)]`
  modules and the remaining changes are test relocation and evidence.

Existing user and other-lane workspace changes were preserved and are not part
of this receipt.
