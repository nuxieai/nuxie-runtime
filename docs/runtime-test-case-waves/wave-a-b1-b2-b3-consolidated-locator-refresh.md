# Wave A/B1/B2/B3 consolidated locator refresh

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Status: **locator-only refresh after semantic source/test freeze**

## Why this refresh exists

The final B1, B2, and B3 semantic corrections changed executable test-file
lengths while the four reviewed shards retained exact source locators. The
source and test owners were frozen before this pass. Every typed Rust evidence
symbol was then re-resolved once against the same shared working tree.

This pass refreshes 123 line fields: four in Wave A, 25 in Wave B1, ten in Wave
B2, and 84 in Wave B3. It does not change an upstream identity, evidence path
or symbol, proof classification, outcome, expected-red reason, fixture,
action, or assertion. Wave B2's one intentional evidence-path change is
separately justified by the PointsPath owner correction receipt.

## Strict result

All 459 typed Rust locators resolve uniquely at their declared line and
symbol:

- Wave A: 259 rows, 258 typed locators, zero stale;
- Wave B1: 70 rows, 70 typed locators, zero stale;
- Wave B2: 45 rows, 45 typed locators, zero stale; and
- Wave B3: 85 rows, 86 typed locators, zero stale.

The frozen shard censuses remain:

- Wave A: `240 direct / 4 differential / 15 adapted`, `217 pass / 42 expected-red`;
- Wave B1: `70 direct`, `51 pass / 19 expected-red`;
- Wave B2: `28 direct / 17 adapted`, `31 pass / 14 expected-red`; and
- Wave B3: `50 direct / 35 adapted`, `70 pass / 12 expected-red / 3 not-applicable`.

JSON parsing, the 1,404-case repository correspondence checker, its 24-test
unit suite, and scoped diff checking are green. This receipt records locator
integrity only; it does not accept any semantic candidate or promote the main
1,404-case ledger.
