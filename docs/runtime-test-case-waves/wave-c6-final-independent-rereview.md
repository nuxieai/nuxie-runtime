# Wave C6 buffer-extension final independent rereview

Verdict: **ACCEPTED — 35/35 direct executable ports; zero pending, adapted,
differential, or expected-red cases**

Original candidate: `d095c9721a9fc0e36115bd2f9b542fe53af2a252`

Independent rejection: `cf956662e`

Correction candidate: `e592d097ec2ef25217d3c6c9a19b842ce2d383e1`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_buffer_ext_test.cpp`

Pinned source SHA-256:
`460ab2cd428b05805ef3b5ad8d53c4a0a11991fe2f3b46839215675e5615800a`

## Correction verdict

The correction is the exact narrow change requested by the independent
rejection. Cases 21, 22, and 26 now deserialize their Lua result tuples as
`f64`, preserving the pinned `lua_tonumber` double conversion and exact
comparison semantics:

- `buffer.convert f32 to u8norm`;
- `buffer.convert u8norm clamps out-of-range f32`;
- `buffer.convert u8 to u16`.

Their individual assertions remain in pinned order. The other 32 previously
accepted cases, every literal Luau program, production behavior, evidence
locators, and ledger classifications are unchanged.

I independently reproduced the rejected conversion's concrete counterexample
in an isolated detached worktree. With only case 21's second observed value
changed to `128.9`, the corrected `f64` assertion failed with `left: 128.9` and
`right: 128.0`. Restoring the rejected `i64` result tuple made that same
counterexample falsely pass by truncating `128.9` to `128`. The isolated
worktree was then removed; neither experiment is present in the candidate or
this receipt commit.

## Exact correspondence

The denominator is exactly 35 distinct explicit Rust tests corresponding
one-for-one with the 35 pinned C++ cases. A fresh lexical audit compared actual
Luau token streams rather than whitespace-normalized text: all 35 programs are
token-identical. Every buffer size, offset, type, format, component count,
stride, literal, return expression, error result, and ordered assertion stream
therefore remains represented without an adaptation or proxy.

All 35 cases have the pinned per-case assertion count and order. No shared loop,
collapsed aggregate assertion, or extra safety case is counted as source
evidence. The two Rust-only safety tests remain explicitly outside the
denominator:

- `buffer_convert_rejects_overflowing_component_spans_without_panicking`;
- `buffer_convert_non_finite_float_to_integer_uses_rust_saturation_policy`.

The unchanged `assert_catch_approx` helper preserves the pinned Catch2
semantics: fixed margin first, then `100 * f32::EPSILON * abs(expected)`, zero
relative scale for infinite expected values, and symmetric comparisons. The
positive-infinity, negative-infinity, explicit-margin, zero-margin, and default
epsilon assertions remain one-for-one.

## Fresh gates

- strict Wave C6 resolver: 35 identities and source locators valid; 35 direct
  passes; zero pending, adapted, differential, expected-red, or
  not-applicable cases;
- focused non-incremental suite: 37 passed, zero failed, zero ignored — exactly
  35 denominator tests plus the two named extras;
- repository correspondence census: 157 files / 1,404 pinned cases, green;
- correspondence-checker unit suite: 24/24 green;
- JSON and pinned source resolution: green;
- non-test release LLVM IR: no Wave C6 test or helper symbols retained;
- correction-scoped `git diff --check`: green;
- correction commit scope: only the Wave C6 test file and its correction
  evidence document.

Wave C6 is accepted as 35/35 exact direct executable correspondence.
