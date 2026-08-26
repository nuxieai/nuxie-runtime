# Wave B5 case 4 semantic correction

Corrected candidate: `d12cabb53277613cb70fb98c570eefc040f193f1`

Rejection receipt: `4ff8d1713`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Status: **CORRECTED; PENDING FRESH ONE-ROW REVIEW**

## Corrected row

`hittest_test.cpp#4`, **hit test on opaque nested artboard**, now selects
`nestedAnimations()[0]` exactly and requires that occurrence to be a nested
state machine. The test retains the selected host, occurrence, animation, and
state-machine identities and revalidates the same owner for every later nested
input read; it no longer scans for an arbitrary state-machine occurrence.

The action and assertion prefix now matches the pinned owner flow: advance the
outer artboard, advance and apply the outer state-machine instance, and only
then assert that the selected nested instance's `bool-target` is false. The
test subsequently executes the exact pointer sequence and reaches the real
divergence at `pointer_down(301, 50)`, where the parent
`second-gray-toggle` incorrectly becomes false.

Rust stable retained IDs replace only the unsafe raw-pointer identity relation;
they do not substitute graph membership, traversal order, or a different
observable.

## Scope

- One semantic row changed: `hittest_test.cpp#4`.
- The other 30 accepted Wave B5 rows are unchanged.
- The expanded exact-owner helper shifted all seven direct hittest test
  symbols; their Wave B5 locator lines were refreshed mechanically.
- No production behavior changed.

## Evidence gates

- strict Wave B5 census: 29 direct / two adapted, 21 pass / ten expected-red,
  zero pending or unverified;
- all 21 pass rows execute successfully;
- all ten expected-red rows were selected individually and fail at their
  documented concrete boundary;
- corrected case 4 fails at the post-`x=301` parent-toggle assertion after the
  exact nested owner and post-initialization assertion prefix;
- repository correspondence, checker unit, production-artifact, and scoped
  diff gates are recorded in the correction commit.

This receipt does not self-accept the corrected row. Wave B5 remains pending a
fresh independent semantic review of case 4.
