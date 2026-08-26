# Wave C15 final independent acceptance

Original candidate: `7690e928b`

First rejection: `f2d1f4668`

Owner correction: `aa8b4c4e5`

Literal-stream rejection: `a673e8c90`

Final correction: `8dd579808cb388f9f0575ae29e4f8057c77b7fa8`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — 19/19 exact executable cases**

The final correction is exactly the narrow change required by the prior
receipt. It removes the four unpinned Rust boolean assertions while retaining
all calls in their pinned positions:

- the shared fixture for artboard cases 1-9 still enables semantics, performs
  the combined Artboard-then-StateMachine default-context bind, then settles
  10 frames at 0.1 seconds; and
- artboard case 10 still calls `enable_semantics` twice, performs the combined
  default-context bind, settles, drains, and asserts the exact four unique
  fandom ids.

No pinned assertion, owner lookup, fixture, action, settle, classification,
outcome, or evidence locator changed. The accepted internal-manager
`node_by_id` checks and retained `parent_id` ancestry walk remain byte-frozen.

Lifecycle executable code and outcomes are also frozen. The only lifecycle
ledger edits make cases 7 and 8's `expected_red_reason` fields byte-identical
to their unchanged `#[ignore]` strings. The previously accepted real
scope-drop owner evidence was therefore not reopened.

## Gates

- Correction-scoped diff: only the four rejected boolean assertions, their
  explanatory correction note, and the two authorized reason fields changed.
- Focused non-incremental artboard suite: **10 passed / zero failed / zero
  ignored**.
- Strict Wave C15 shard: **19/19** rows; 16 direct and three adapted; 17 pass
  and two expected-red; zero pending.
- Evidence audit: **19/19** unique locators resolve to the declared symbol at
  the declared line.
- Lifecycle cases 7 and 8 ledger reasons byte-match their executable ignore
  reasons.
- Repository correspondence checker: **157 files / 1,404 cases, green**.
- Correspondence checker unit suite: **24/24 green**.
- Correction diff check and JSON parsing: green.

Wave C15 contributes **19 accepted cases** to the exact runtime-test campaign.
Existing user and other-lane workspace changes were preserved and are not part
of this receipt.
