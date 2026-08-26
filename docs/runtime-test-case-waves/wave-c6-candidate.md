# Wave C6 buffer-extension candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_buffer_ext_test.cpp`

## Candidate verdict

Candidate for fresh independent review: **35 direct executable passes, 0 expected-red, 0 pending**.

- 35 distinct, discoverable `#[test] fn` definitions
- 35/35 identities and evidence locators accepted by the strict resolver
- no aggregate or macro-generated denominator evidence
- no production behavior changes

The two additional tests, `buffer_convert_rejects_overflowing_component_spans_without_panicking` and `buffer_convert_non_finite_float_to_integer_uses_rust_saturation_policy`, remain valuable Rust-safety coverage but are outside the pinned 35-case denominator.

## Exact-stream audit

Every pinned body was reread against its corresponding production `ScriptVm` test. A mechanical token audit, ignoring whitespace only, found all 35 Luau programs identical to the pinned source. This covers every buffer allocation size, byte offset, value, format, count, component width, source/destination stride, return expression, and intended error path.

The existing tests had preserved the programs but collapsed some separate Catch checks into tuple equality and substituted exact equality for upstream `Approx` checks. The candidate expands every denominator assertion into the pinned order and uses Catch's exact default float epsilon, explicit margins, infinity handling, and comparison relation. Error cases retain the exact `out of bounds` or `unknown buffer format` substring assertion.

All 35 cases are classified direct. C++ `ScriptingTest`, `lua_Number`, and raw Lua buffer representation are host wrappers rather than asserted observables in these bodies; the Rust tests execute the same literal programs through the production `ScriptVm` buffer-extension owner, so no structured adaptation is necessary.

## Validation

- Focused non-incremental suite: 37 passed, zero failed, zero ignored (35 denominator cases plus two extra safety tests).
- Strict Wave C6 identity/status/locator audit: 35/35 direct passes, zero pending.
- Literal program token audit: 35/35 exact after whitespace normalization.
- Assertion-stream audit: 35/35 preserve the pinned individual `CHECK` count and order.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24 passed.
- Non-test release LLVM IR: no Wave C6 test symbols retained.
- Scoped formatting and `git diff --check`: passed.

This is candidate evidence only and does not self-accept Wave C6.
