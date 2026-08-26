# Wave C12 scripted-property Silver final independent acceptance

Reviewed metadata correction: `9e13b640408be4355e20deb7365cb3e8d4a44dee`

Accepted semantic rereview: `9c69a7cc7`

Corrected owner candidate: `257a7b17f2bba812a30de2b625ccf4b119ba3535`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — 1 pass / 6 executable expected reds / 1 honest pending**

This rereview was restricted to the six expected-red reason corrections. The
commit changes only `expected_red_reason` in cases 10, 11, 12, 14, 15, and 16,
plus its correction note. Every corrected ledger value now byte-matches the
complete discovered Rust `#[ignore]` string, including the `expected-red: `
prefix.

No test, shared harness, action stream, fixture, baseline, classification,
outcome, evidence locator, case 9 pass, or case 13 pending row changed. The
realized-owner semantic findings accepted in `9c69a7cc7` therefore remain
closed and were not reopened.

## Final census

- case 9: one exact realized-owner pass;
- cases 10, 11, 12, 14, 15, and 16: six independently forceable
  realized-owner expected reds;
- case 13: one honest pending/unverified case at nested graph 84's missing
  retained source File authority;
- seven distinct executable evidence locators;
- six of six expected-red ledger/ignore reasons byte-exact.

## Gates

- strict shard comparison: only the six authorized reason fields changed;
- strict locator/reason audit: 8 identities, 7 executable locators, and 6
  exact reasons, green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- correction JSON parse, diff check, test/harness freeze, and production-source
  freeze: green.

Wave C12 Silver is independently accepted with **7 executable cases and 1
honest pending case**.
