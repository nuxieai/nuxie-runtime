# Wave C9 narrow correction candidate

This correction responds only to the independent rejection in `eade64796`.
It does not change runtime production behavior and does not self-accept Wave C9.

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Corrected evidence

- Semantic cases 2–5 and 7 are strict pending again. Their diff-projection tests
  were removed because they did not observe the pinned live `nodeById` owner.
- Event case 8 now applies Catch's exact float-relative margin after widening
  the `f32` operands.
- Event case 10 and state-machine cases 9–11, 13–14, and 16–19 each have a
  distinct literal Silver replay locator.
- `sorted_listeners` and `paused_nested_artboard_opacity` are executable
  expected-red cases at their exact first divergent operations. They are not
  collapsed into a group test or described as missing-owner pending cases.

The corrected ledger topology is 46 total: 28 pass, two executable
expected-red, and 16 pending. The 30 executable cases comprise 20 direct and
ten structured Rust-safety adaptations.

## Verification contract

The correction must pass both focused Rust suites, the eight non-red literal
Silver replays, individual forced execution of both expected-red tests, the
strict 46-row correspondence check, pinned source hashes, forbidden-proxy
search, and a scoped diff audit. Global and release-containment evidence may
be reused only where unchanged; the new integration-test symbols require a
targeted release-IR containment scan.
