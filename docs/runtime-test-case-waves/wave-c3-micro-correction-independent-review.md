# Wave C3 micro correction independent rereview

Status: **REJECTED; LEDGER-SCHEMA CORRECTION REQUIRED**

Original candidate: `fae9e184300a8b0fd49ea75787c35de3f81fa296`

Independent rejection: `82a8d8b39`

Correction reviewed: `6886bc0ecfe497e988a87a122bd97be11306423b`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Frozen evidence verdict

The correction fixes the semantic evidence census identified by the first
review. Across the exact 23-case denominator, only the accepted ten executable
owners remain mapped: lite RTTI #1, malformed import #1-#2, and math #1, #2,
#9-#12, and #14. Their ten distinct locators resolve to live, non-ignored Rust
tests. The other 13 rows are `pending` / `unverified`, have empty evidence
arrays, and contain no placeholder or synthetic expected-red.

The rejected Wave C3-specific mixed-integer helpers and six tests, round-up
closure/test, duplicated fallback test, and both Node proxy tests are absent.
The Node proxy module is deleted and de-mapped from `objects.rs`. No substitute
proxy or production implementation was introduced. A legacy, uncredited
`crates/nuxie-runtime/tests/upstream_math.rs` still contains earlier test-local
math implementations; none is a Wave C3 locator or accepted evidence, and this
rereview does not count them.

The ten retained bodies preserve the pinned assertion streams through the
previously adjudicated Rust language/ownership adaptations. Math #14 alone is
direct and calls the production `positive_mod` owner. The correction changes no
runtime behavior; it removes only rejected test evidence, a test-module
declaration, and the associated ledger mappings.

## Blocking strict-schema failure

The corrected topology is not encoded in the repository's strict case-ledger
schema:

- all nine `adapted` rows use the obsolete top-level `adaptation_kind` field
  and omit the required structured `adaptation` object containing `kind`, a
  non-empty `rationale`, and a precise `inapplicable_observable`; and
- all 13 pending rows carry a `note`, while the strict schema requires a
  pending row to contain only the unverified outcome and empty evidence, with
  no adaptation or note claim.

The first strict failure is exact:
`tests/unit_tests/runtime/lite_rtti_test.cpp#1 adapted case requires adaptation metadata`.
Removing only that first failure exposes the same omission on the other eight
adapted rows, followed by the pending-note failures.

Correction is ledger-only. Give each of the nine adapted rows literal,
case-specific structured adaptation metadata. Remove `adaptation_kind`. Remove
`note` entirely from all 13 pending rows so each remains `unverified`, with no
evidence, adaptation, locator, or placeholder. Do not change executable tests,
production code, classifications, outcomes, or the 10/13 topology.

## Gates

- pinned checkout and all four source hashes: green;
- focused non-incremental executable sweep: seven math, one lite-RTTI, and two
  malformed-import tests passed; zero failed or ignored;
- 10/10 declared evidence locators resolve to their exact non-ignored symbol
  and line;
- strict Wave C3 shard: rejected on the metadata failure above;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- correction-scoped JSON parsing and `git diff --check`: green;
- default release LLVM IR contains no Wave C3 test, rejected proxy, or
  forbidden helper symbol.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and disabled
incremental compilation for the invoked profile. This receipt changes no
candidate evidence and does not accept Wave C3.
