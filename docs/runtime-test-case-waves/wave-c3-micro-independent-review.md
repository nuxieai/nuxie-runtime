# Wave C3 micro independent adversarial review

Status: **REJECTED; CORRECTION REQUIRED**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Candidate reviewed: `fae9e184300a8b0fd49ea75787c35de3f81fa296`

Scope: the 23 active Catch2 cases in `lite_rtti_test.cpp`,
`malformed_file_import_test.cpp`, `math_test.cpp`, and `node_test.cpp`.

## Verdict

The denominator and all evidence locators are real, and every executable test
is green. The candidate is nevertheless not exact-owner evidence for 10 of its
20 claimed executable cases.

- 10 cases are accepted as executable evidence: lite RTTI #1, malformed import
  #1-#2, and math #1, #2, #9-#12, and #14.
- 10 claimed executable cases are rejected: math #3-#8, #13, #15, and node
  #1-#2.
- The candidate's three existing pending rows, math #16-#18, remain honestly
  pending.
- Corrected strict result: **10 executable / 13 pending**, not 20 executable /
  3 pending.

The accepted language adaptations use Rust's actual language/library owner for
functionality that only exists upstream as a C++ portability abstraction:
IEEE floating division, bit reinterpretation, clamping, leading-zero count,
rotation, most-significant-bit derivation, and Rust-safe RTTI/ownership. The
malformed-import cases execute the exact pinned fixture and import lifecycle;
math #14 calls the production `positive_mod` owner.

## Rejected evidence

### Math #3-#8: test-local mixed-integer implementation

The candidate defines `MixedInteger` plus six private comparison functions in
the test module, then proves those new functions. No production runtime owner
is exercised. These functions are a second implementation of the pinned C++
`cmp_*` templates and therefore cannot certify the runtime port. Rust rejecting
mixed-sign comparisons does not authorize hiding a missing source owner inside
the test.

Correction: mark all six cases pending until the source-correspondence phase
either establishes a genuine Rust owner or adjudicates them as an explicit
language-level non-applicability. Do not manufacture an expected red.

### Math #13: test-local round-up implementation

`round_up_to_multiple_of` is recreated as a closure inside the test. The test
therefore validates the closure, not a retained runtime owner. This is the same
failure mode correctly rejected for the three bitmask helpers.

Correction: mark the case pending until a real owner exists or the source row
is explicitly adjudicated.

### Math #15: fallback path was collapsed

The pinned case independently exercises `count_set_bits` and
`internal::count_set_bits_fallback`. The candidate repeats Rust `count_ones`
twice. That is acceptable evidence for the public portability operation, but
it does not preserve the assertion stream against the separately named
fallback owner.

Correction: keep the case pending unless source correspondence proves that the
fallback is an approved C++-only portability detail. Repetition of one Rust
primitive is not evidence for two upstream owners.

### Node #1-#2: dynamic arena is not a `Node` owner

The pinned cases construct a concrete `Node`, read its own default `x`, mutate
that same object's `x`, and read it again. The candidate instead manufactures a
three-record `RuntimeFile`, builds an `InstanceObjectArena`, and accesses a
dynamic property by local id and string name. The arena is real production
code, but it is an aggregate/proxy surface for this test. The repository has no
concrete runtime `Node` owner corresponding to the pinned source class.

Treating these as direct would conceal precisely the packed-source gap this
campaign is meant to expose. Mark both pending until source correspondence
creates or identifies the one-to-one `Node` authority. Do not replace the
missing owner with a fixture graph.

## Gates run by the reviewer

All commands used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`:

- `cargo test -p nuxie-runtime --lib wave_c3_math_`
- `cargo test -p nuxie-runtime --lib wave_c3_node_`
- `cargo test -p nuxie-runtime --test upstream_language_wave_c3`
- `cargo test -p nuxie --test upstream_malformed_import_wave_c3`

All executable candidates pass. This rejection is semantic, not a compile or
test failure.
