# Wave C6 buffer-extension independent adversarial review

Verdict: **REJECTED — 32/35 exact executable ports; 3 assertion-conversion
corrections required**

Reviewed candidate: `d095c9721a9fc0e36115bd2f9b542fe53af2a252`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_buffer_ext_test.cpp`

## Accepted correspondence

The denominator is exactly 35 distinct, explicit Rust tests. The two following
Rust-safety tests remain outside that denominator:

- `buffer_convert_rejects_overflowing_component_spans_without_panicking`;
- `buffer_convert_non_finite_float_to_integer_uses_rust_saturation_policy`.

An independent lexical audit extracted every pinned `ScriptingTest` source and
every corresponding Rust raw string, then compared actual Luau token streams
rather than deleting all whitespace. All 35 programs have identical tokens.
Thus no allocation size, offset, literal value, format name, count, component
width, source stride, destination stride, return expression, or error path was
changed by whitespace normalization.

The candidate also expands the previously collapsed tuple assertions into the
pinned number and order of individual checks. All 35 cases have the same
per-case assertion count, and all approximate and error-substring streams are
ordered one-for-one.

The shared `assert_catch_approx` helper faithfully implements this pinned
Catch2 version's `Approx` relation: fixed margin first, then
`100 * f32::EPSILON * abs(expected)`, with zero relative scale for infinite
expected values. Its symmetric `lhs + margin >= rhs` comparisons preserve the
pinned positive-infinity, negative-infinity, zero-margin, explicit-margin, and
default-epsilon behavior.

## Rejected exact-result conversions

Cases 21, 22, and 26 do not yet preserve the pinned exact numeric comparison:

- `buffer.convert f32 to u8norm`;
- `buffer.convert u8norm clamps out-of-range f32`;
- `buffer.convert u8 to u16`.

The pinned bodies read each result through `lua_tonumber` and compare the
resulting `double` exactly with `0`, `128`, `255`, or `200`. The Rust tests
instead deserialize the return values into `i64` tuples before asserting them.
The active `luaur-rt` `FromLua for i64` conversion truncates finite floating
values toward zero. Consequently, for example, an observed value of `128.9`
would satisfy the Rust assertion for `128` while failing the pinned Catch
check. Splitting the tuple assertion did not remove that lossy conversion.

Correction is narrow: deserialize these three result tuples as `f64` values
and retain the existing individual exact-equality assertions in pinned order.
No production change, proxy, adaptation, or expected-red is justified.

## Gates

- pinned checkout and source bytes: exact SHA and clean source path;
- strict Wave C6 identity, source-line, name, status, outcome, and evidence
  resolver: 35/35 valid;
- lexical Luau stream audit: 35/35 token-identical;
- assertion-count audit: 35/35 counts match, with no shared loop or aggregate
  denominator evidence;
- focused non-incremental suite: 37 passed, zero failed, zero ignored (35
  denominator plus two extras);
- repository correspondence census: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- non-test release LLVM IR: no Wave C6 test/helper symbols retained;
- candidate-scoped `git diff --check`: green;
- candidate diff changes only the Wave C6 test and evidence documents.

The ledger's claim of 35 direct passes must not be accepted until the three
lossy integer result conversions are corrected and independently rereviewed.
