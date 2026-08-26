# Wave C12 Silver expected-red reason correction

Rejected rereview: `9c69a7cc7`

Corrected owner candidate: `257a7b17f2bba812a30de2b625ccf4b119ba3535`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **CORRECTED; PENDING SAME-REVIEWER REREVIEW**

The ledger reasons for cases 10, 11, 12, 14, 15, and 16 now equal their
executable Rust `#[ignore]` reasons byte-for-byte, including the
`expected-red: ` prefix. No test, action stream, harness, fixture, baseline,
outcome, evidence locator, pass case 9, or pending case 13 changed.

Validation reran the strict six-row reason matcher, shard JSON and evidence
locators, production/test diff freeze, and the repository correspondence
checker with its unit suite:

- strict shard: 8/8 identities, 7/7 executable locators, and 6/6 exact reasons;
- repository correspondence: 157 files / 1,404 cases;
- correspondence checker unit suite: 24/24;
- test, harness, fixture, baseline, outcome, and production diffs: frozen.

This receipt does not self-accept the shard; the same reviewer should rereview
the metadata-only correction.
