# Wave C14 vector independent adversarial review

Verdict: **ACCEPTED — 13/13 exact executable ports**

Reviewed candidate: `4b57d8ffc57445f508446988af7128beb5179ddb`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Body-level correspondence

I independently compared every body in pinned
`tests/unit_tests/runtime/scripting/scripting_vector_test.cpp` with its
distinct Rust test in
`crates/nuxie-scripting/tests/upstream_scripting_vector.rs`.

- cases 1–7 and 9–11 preserve every literal Luau program, fresh-VM action
  order, result arity, 2D/3D component value, static/namecall/metamethod/index
  route, buffer offset, and out-of-bounds `pcall` assertion;
- the two pinned `Catch::Approx` streams use a float-relative tolerance based
  on the pinned `f32` expectation rather than a widened hard-coded epsilon;
- case 8 executes the complete pinned direct-static, namecall, and indirect
  program, including all length, squared-length, normalize, distance,
  squared-distance, dot, three lerp factors, scale-add, scale-sub, cross, and
  zero-normalization equality/error checks;
- case 12 installs the same global `callMyFunc`, captures the same unsigned
  value `222`, executes the same sandboxed source under chunk name
  `test_source`, and proves the live callback observed the captured value;
- case 13 preserves `N = 1,000,000`, three warmups, five measured runs, all
  four static/namecall script pairs, fresh production `ScriptVm` evaluation
  for every timed run, best-run selection, and the final `-5.0 * N` static-dot
  sanity check.

There are 13 distinct, discoverable denominator tests. The additional finite
arithmetic-boundary test is explicitly outside the 13-case pinned denominator.

## Adaptation audit

Case 8's `native-scripting` adaptation is narrow. The current Rust scripting
backend does not expose the pinned C++ Luau fork's FASTCALL bytecode-versus-C
binding dispatch identity, but all three runtime-observable call forms execute
through the live production `ScriptVm` vector owner and retain every pinned
equality and error assertion. It does not substitute a test-local vector
implementation or expected-value proxy.

Case 12's `cxx-language-only` adaptation is also narrow. The custom C
reallocator, raw compiled-bytecode allocation, `lua_State*`, and C closure
upvalue representation are C++ host implementation details unavailable through
the Rust wrapper. The corresponding sandbox, global closure, captured value,
named source, invocation, and success stream execute through the live
production `ScriptVm`/Lua owner. No behavioral assertion is dropped.

## Expected-red verification

Case 13 was forced individually with the exact one-million-iteration workload.
It entered the first static-dot benchmark program and failed at
`best_run` through the production error:

`script cycle exceeds 100000 script safepoints`

The workload was not shortened, the safepoint ceiling was not changed or
bypassed, and the failure does not depend on an environment-only ignore. The
case is therefore an honest, individually forceable production-owner red.

## Ledger and gates

- focused non-incremental suite: 13 passed, one ignored (12 denominator passes
  plus the non-denominator finite-boundary test);
- forced non-incremental benchmark: expected production-safepoint failure;
- strict ledger identity, ordinal, source line, name, classification, outcome,
  evidence symbol, and locator validation: 13/13 green;
- repository correspondence census: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- pinned checkout identity and JSON parsing: green;
- release LLVM IR scan: no Wave C14 vector test symbols retained;
- scoped diff check: green;
- candidate changes only test evidence and documentation; no production source
  changed.

Every relied-on Cargo build used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false` for test gates.
