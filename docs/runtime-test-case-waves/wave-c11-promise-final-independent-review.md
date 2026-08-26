# Wave C11 Promise final independent review

Correction: `2987ba0ab67b6094e697a4f4c64388f57d9ec411`

Prior rejection: `47ab577e2`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_promise_test.cpp`

Verdict: **ACCEPTED — 48/48 exact executable cases**

This fresh review kept the correction frozen and rechecked every prior finding,
all evidence locators, the structured case 12 adaptation, and the focused owner
suite. Wave C11 is accepted as 47 direct passes and one adapted pass, with zero
expected-red and zero pending cases.

## Correction closeout

- All 47 former macro-generated cases are now explicit, unique `#[test] fn`
  definitions.
- The official Rust-test evidence resolver accepts all 48 ledger locators.
- The focused suite discovers exactly 48 Wave C11 tests. There is no surviving
  macro-generated duplicate suite, aggregate case loop, or competing
  entrypoint.
- Shared code is limited to three typed evaluation helpers. Each test retains
  its own Luau program and its own exact result, substring, status,
  cancellation, hook, ordering, retry, await, or flattening assertion stream.

## Seven prior semantic rejects

Cases 25, 27, 28, 30, 31, 39, and 40 now token-match their pinned raw Luau
programs:

- cases 25, 27, 28, and 30 restore the pinned Promise executor parameter
  lists;
- case 28 also restores the immediate named `afterSecond` status snapshot;
- case 31 restores the named `cancelledStatus` snapshot; and
- cases 39 and 40 restore the named `outer` Promise owners and invoke their
  handlers through those bindings.

A complete 48-program token audit found no remaining program delta after
ignoring only whitespace, optional statement semicolons, and optional trailing
table commas. The seven corrected expected values and assertions remain local
to their explicit tests.

## Case 12 adaptation regression check

Case 12 remains an accepted structured `cxx-language-only` adaptation. Its
literal program prints `first resume`, suspends through
`await(Promise.resolve(1))`, then prints `post-await resume`. The test captures
and asserts exactly those two messages in order through the installed Rust host
callback. Only the raw C++ `ScriptingContext*` / `lua_setthreaddata` identity is
classified inapplicable because mlua does not expose that pointer contract.
The ledger retains the required adaptation kind, rationale, and inapplicable
observable.

## Gates

- Pinned SHA, ids, ordinals, source lines, and exact Catch names: **48/48**.
- Evidence symbols are unique: **48/48**.
- Official `resolve_rust_test` evidence audit: **48/48 valid, zero ignored**.
- Exact-program token audit: **48/48**, including **7/7** prior rejects.
- Focused non-incremental suite:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_INCREMENTAL=false cargo test -p
  nuxie-scripting --features compiler --lib upstream_promise_tests:: --
  --test-threads=1` — **48 passed, zero failed, zero ignored**.
- Repository correspondence checker: **157 files / 1,404 cases, green**.
- Correspondence checker unit suite: **24/24 green**.
- Correction-scoped `git diff --check`: green.
- Production freeze: green; the Rust correction is contained by the existing
  `#[cfg(all(test, feature = "compiler"))]` test module, and the other changes
  are Wave C11 evidence documents.

Wave C11 contributes **48 accepted cases** to the exact runtime-test campaign.
