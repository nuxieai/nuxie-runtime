# Wave C9 correction final independent rereview

Verdict: **REJECTED — event case 8 still computes the Catch relative margin
at `f32` precision before widening**

Reviewed candidate: `db5257cb22556e75f9d94aba9697fc86e0b7c131`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

The correction resolves every other finding from `eade64796`. Semantic cases
2–5 and 7 are strict pending rows with no proxy evidence. The ten callable
Silver cases have distinct literal replay locators; eight pass, while
`sorted_listeners` and `paused_nested_artboard_opacity` retain their exact
forced first-divergence reasons. The mechanical topology is correct: 46 total,
28 pass, two executable expected-red, and 16 pending; evidence status is 20
direct, ten structured Rust-safety adaptations, and 16 pending.

## Remaining blocker

Event case 8 currently computes:

```rust
f64::from(100.0 * f32::EPSILON * 0.1f32.abs())
```

That performs both multiplications in `f32` and then widens, yielding
`1.1920928955078125e-6`. The pinned Catch comparison—and the repository's
existing oracle—widens the epsilon and expected value before the relative
margin multiplication, yielding `1.1920929132713809e-6`:

```rust
f64::from(f32::EPSILON) * 100.0 * expected.abs()
```

Replace only that expression. No other row needs correction or rereview, and
the unchanged global, release, hash, and containment gates do not need to be
repeated.

## Focused evidence

- Event case 8 executes successfully, but with the non-equivalent narrower
  margin above; a green result cannot certify the wrong assertion semantics.
- The Wave C9 Silver suite reports eight passes and two ignored expected-red
  cases.
- Forced `sorted_listeners` execution fails exactly at
  `frame 5, op 134 (addRawPath): expected 180 fields, got 337`.
- Forced `paused_nested_artboard_opacity` execution fails exactly at
  `frame 1, op 103 (rewind): expected rewind, got drawPath`.
- All 46 evidence locators resolve, all 16 pending rows have empty evidence,
  and the scoped diff is clean.

This receipt changes no production source, test, fixture, or ledger row.
