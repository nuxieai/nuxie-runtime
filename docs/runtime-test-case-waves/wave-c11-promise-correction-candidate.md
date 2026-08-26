# Wave C11 Promise correction candidate

Original candidate: `c07debf1e`

Independent rejection: `47ab577e2`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected verdict

Candidate for fresh independent rereview: **48 executable passes, 0 expected-red, 0 pending**.

- 47 direct cases
- 1 accepted structured `cxx-language-only` adaptation (case 12)
- 48 explicit, discoverable `#[test] fn` definitions
- 48/48 evidence locators accepted by the repository's official `resolve_rust_test` resolver
- no production behavior changes

The 47 macro-generated entrypoints were replaced with explicit test definitions while retaining one distinct literal Luau program and assertion stream per pinned case. Only evaluation setup is shared; there is no aggregate loop or duplicate competing suite.

## Seven exact-stream corrections

The seven semantic rejects now preserve the pinned source token streams:

- Case 25 restores the `resolve, reject, onCancel` executor parameters.
- Case 27 restores the `resolve, reject, onCancel` executor parameters.
- Case 28 restores those parameters and the immediate `afterSecond` status snapshot before the return expression.
- Case 30 restores the `resolve, reject, onCancel` executor parameters.
- Case 31 restores the named `cancelledStatus` snapshot before result construction.
- Case 39 restores the named fulfilled `outer` promise and invokes `outer:andThen`.
- Case 40 restores the named rejected `outer` promise and invokes `outer:catch`.

A whitespace/optional-semicolon-insensitive token audit compared each of those seven executable Rust Lua strings with its exact pinned C++ raw program and found seven exact matches. Their exact expected result/status assertions remain local to their explicit tests.

Case 12 remains unchanged semantically: it executes the exact `first resume` / `post-await resume` program and asserts those two ordered host callback messages. Only raw C++ `ScriptingContext*` / `lua_setthreaddata` identity remains the structured language-only exception.

## Validation

- Official strict resolver: 48/48 discovered Rust tests, zero ignored.
- Focused non-incremental suite: 48 passed, zero failed, zero ignored.
- Exact seven source-token audit: 7/7 matched.
- Strict Wave C11 identity/status/adaptation audit: 48/48 valid; 47 direct, one adapted, zero pending.
- Repository correspondence checker: passed.
- Correspondence checker unit suite: 24 passed.
- Scoped Rust formatting and `git diff --check`: passed.
- Non-test release LLVM IR scan: no Wave C11 test symbols retained.

This correction receipt is candidate evidence only and does not self-accept Wave C11.
