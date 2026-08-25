# Wave B1 semantic correction candidate

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Correction target: independent rejection receipt `817a8c8b3`

Status: **pending fresh independent review; not self-accepted**

## Candidate census

All 16 rejected rows now preserve the pinned fixture, action order, production
owner, and assertion semantics, or stop at the first concrete divergent seam.
The 54 previously accepted rows were not intentionally changed.

| classification | rows |
|---|---:|
| executable passing evidence | 49 |
| executable expected-red evidence | 21 |
| total | 70 |

Two rows formerly declared passing now expose real absent seams: decoded font
assignment does not replace the property backing `FontAsset` owner, and the
live-image null assignment is rejected before the pinned draw flow. This moves
the candidate census from 51/19 to 49/21 without hiding either failure.

## Corrected semantics

- Restored Catch `Approx` behavior for all seven rejected numeric rows.
- Observed the backing `FontAsset` owner and exact root/nested `ImageAsset`
  identities instead of byte-allocation or recording proxies.
- Executed the live-image null action directly.
- Restored generated fit/alignment reads, writes, restores, both no-scale local
  transform assertions, and all three 20-frame phases.
- Executed concrete empty-list-item insert/swap/remove mutations and compared
  the accumulated transition stream.
- Restored enum-name mutation, runtime view-model name/type-cache/enum-name
  assertions, post-advance TwoWay source assertions, and all three custom
  trigger owner assertions.

## Focused gates

- All ten corrected passing rows executed successfully.
- All six corrected expected-red rows were forced and failed inside the named
  owner flow at their documented seam; no unconditional panic or proxy-only
  assertion is used.
- The four deterministic keyframe ports pass together; their one unrelated
  pre-existing expected-red remains ignored.
- Exact root and nested image-owner evidence passes in the runtime owner module.
- No production behavior was modified. The only runtime-crate source edit is a
  test-module include under the existing `cfg(test)` owner.

Final evidence locators and the strict Wave A frozen-locator gate are owned by
the campaign's consolidated locator refresh and are intentionally not
re-adjudicated in this receipt.
