# Wave C15 literal assertion-stream correction

Status: **CANDIDATE; PENDING SAME-REVIEWER REREVIEW**

Independent rejection: `a673e8c90`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This correction removes only the unpinned Rust boolean assertions identified
by the rejection. The shared setup for artboard cases 1-9 still calls the
combined default ViewModel bind in the same position and order, but ignores
its return. Case 10 still calls `enable_semantics` twice and then the combined
bind in the pinned order, but no longer asserts any of their Rust returns.

No fixture, owner access, settle/action sequence, pinned assertion, outcome,
lifecycle test, evidence locator, or production behavior changed. Lifecycle
cases 7 and 8 have their ledger reasons aligned byte-for-byte with their
unchanged executable `#[ignore]` reasons to unblock the official shard
validator.

## Gates

- focused non-incremental artboard suite: 10/10 green;
- exact scoped diff freeze: only the four rejected boolean assertions and the
  two authorized lifecycle reason fields changed;
- official strict Wave C15 shard: 19/19 green, with 16 direct and three
  adapted rows, 17 passes and two expected-reds, and zero pending;
- all ten artboard evidence locators remain unchanged and exact;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green.

All relied-on Cargo invocations use `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`.
