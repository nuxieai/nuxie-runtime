# Wave C14 scroll independent adversarial review

Verdict: **ACCEPTED — 4/4 direct expected-red ports**

Reviewed candidate: `0daa56d4c691d5b2834c19f9f8cac3f954aa624a`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Body-level correspondence

I independently compared every body in pinned
`tests/unit_tests/runtime/scroll_test.cpp` with its distinct Rust test in
`tools/silver-corpus/tests/upstream_scroll.rs`.

- case 1 selects `scroll_test.riv`, the default artboard, and state machine
  zero; preserves the initial advance/draw, both transition advances, both
  pointer-down streams, all 62 right and left drag frames, every pointer
  coordinate, both release sequences, every frame boundary, and every draw;
- cases 2–4 each select `scroll_threshold.riv`, the exact named artboard
  (`vertical-scroll`, `horizontal-scroll`, or `all-scroll`), state machine zero,
  and view-model instance zero; each preserves the initial frame, both pointer
  streams, the exact `40/10`, `40/10`, or `50/32` loop limits, the decrement by
  eight, pointer-up ordering, advances, frame boundaries, and draws;
- all four compare the completed serialization from the live
  `ArtboardInstance` and `StateMachineInstance` through the real
  `SerializingFactory` SRIV parser/comparator against the exact pinned
  baseline;
- no test sources actions or expected results from the corpus manifest or an
  aggregate action runner, and no helper computes the expected outcome;
- the prior `missing_silver_match` placeholder file is deleted and none of its
  symbols or evidence locators remain mapped.

The Rust-only pointer id/pressure arguments, import/renderer initialization,
and owned handle used to retain the view-model instance are host API spellings
of the same authored operations, not behavioral substitutions.

## Expected-red verification

All four ignored tests were forced individually after a non-incremental build.
Each completed its entire authored action stream and then failed through the
real comparator at the ledgered first difference:

1. `scroll_test`: frame 0, op 53, `transform.xy`, expected `-0.0
   (0x80000000)`, got `0`;
2. `scroll_threshold-vertical-scroll`: frame 0, op 69, `transform.xy`, expected
   `-0.0 (0x80000000)`, got `0`;
3. `scroll_threshold-horizontal-scroll`: frame 0, op 79, `transform.xy`,
   expected `-0.0 (0x80000000)`, got `0`;
4. `scroll_threshold-all-scroll`: frame 0, op 82, `transform.xy`, expected
   `-0.0 (0x80000000)`, got `0`.

The comparator deliberately compares signed zero by bits, so these are real
frozen-byte divergences rather than epsilon matches or diagnostic stand-ins.

## Ledger and gates

- pinned checkout identity and the source, both RIV fixtures, and all four
  SRIV baseline blob identities match the pinned commit;
- strict shard identity, ordinal, source line, exact case name, classification,
  outcome, evidence line/symbol, and ignore-reason validation: 4/4 green;
- focused suite: zero pass / four ignored genuine expected-reds;
- non-incremental focused compile: green;
- repository correspondence census: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- JSON parsing and scoped diff checks: green;
- candidate commit changes only test evidence and documentation; no production
  source changed.

Every relied-on Cargo build used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`.
