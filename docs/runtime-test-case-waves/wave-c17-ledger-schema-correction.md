# Wave C17 adaptation-schema correction

Rejected independent receipt: `0dbc09b16`

Author commit: `801cd0e23`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **CORRECTED; PENDING SAME-REVIEWER REREVIEW**

Case 15 now records the required structured `adaptation` object with its
`cxx-language-only` kind, concrete Rust-overload rationale, and the exact C++
compile-time overload-resolution observable that is inapplicable in Rust. The
obsolete flat `adaptation_kind` field was removed.

No test code, production code, evidence locator, classification, outcome, or
other ledger row changed. The official strict shard validator accepts all
36 rows: 35 direct, one adapted, 36 pass, zero pending or ignored.

Validation reran JSON parsing, focused evidence-locator checks, the repository
correspondence checker and its 24-test unit suite, plus scoped diff checks. This
receipt does not self-accept Wave C17; the same reviewer should rereview the
ledger-only correction.
