# Wave C14 vector candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_vector_test.cpp`

## Candidate verdict

Candidate for fresh independent review: **12 executable passes, 1 individually forceable expected-red, 0 pending**.

- 10 direct passing cases
- 2 passing structured adaptations (cases 8 and 12)
- 1 direct expected-red (case 13)
- 13 distinct, discoverable `#[test] fn` definitions and 13/13 strict evidence locators
- no production behavior changes

The extra `vector_arithmetic_preserves_pinned_finite_operation_boundaries` test remains useful coverage but is outside the pinned 13-case denominator.

## Exact stream audit

All 13 pinned bodies were reread against their distinct executable Rust tests. Cases 1-12 preserve their literal Luau programs, evaluation order, result counts, exact values, Catch `Approx`-equivalent float comparisons, buffer offsets and out-of-bounds failures, vector indexing/metamethod behavior, and closure callback stream.

Case 8 is a `native-scripting` adaptation. The Rust backend cannot expose the C++ Luau fork's FASTCALL bytecode versus `lua_vec2d` C-binding route identity, but its exact direct-static, namecall, and indirect-call program executes through production `ScriptVm` and preserves every result and error assertion.

Case 12 is a `cxx-language-only` adaptation. Rust cannot observe the pinned raw `lua_State` allocator/free callback, compiled bytecode buffer, or C closure upvalue representation. Its executable test preserves the exact `index = 222`, `callMyFunc()` source, `test_source` chunk name, sandboxing, callback invocation, and captured-value assertion through the production `ScriptVm` Lua owner.

Case 13 preserves `N = 1,000,000`, three warmups, five measured runs, both static and namecall forms of dot/distance/length/lerp, and the final `-5.0 * N` dot sanity assertion. When forced individually, it reaches the real first benchmark program and fails at the production 100,000-script-safepoint quota. The quota was not lowered or bypassed.

## Validation

- Focused non-incremental suite: 13 passed, zero failed, one ignored (12 denominator passes plus the extra finite-boundary test).
- Individually forced non-incremental benchmark: failed at the production safepoint quota as expected.
- Strict Wave C14 identities and evidence locators: 13/13 resolved; 12 pass/adapted, one expected-red, zero pending.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24 passed.
- Non-test release LLVM IR: no Wave C14 test symbols retained.
- Scoped formatting and `git diff --check`: passed.

This is candidate evidence only and does not self-accept Wave C14.
