# Wave C11 Promise candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: every one of the 48 `TEST_CASE` bodies in `tests/unit_tests/runtime/scripting/scripting_promise_test.cpp`.

## Verdict

Candidate for independent review: **48 executable passes, 0 expected-red, 0 pending**.

- 47 direct cases
- 1 `cxx-language-only` adaptation
- 48 distinct Rust test entrypoints and evidence locators
- no production or test behavior changes

Each pinned body was re-read against its corresponding test in `crates/nuxie-scripting/src/vm/lua_promise.rs`. The correspondence is one test per pinned case—not a loop or aggregate count—and retains the literal Luau program plus its exact returned value, status string, callback/cancellation result, or promise-flattening result.

## Sole adaptation

`scripting_promise_test.cpp#12`, “async coroutine inherits thread data (print works),” directly preserves the observable program: `print("before await")`, suspension through `await(Promise.resolve(1))`, `print("after await")`, and the exact two captured console messages. It is classified `cxx-language-only` because mlua does not expose the raw C++ `ScriptingContext*` identity installed through `lua_setthreaddata`; the installed Rust host callback is the runtime-observable equivalent. The structured adaptation and inapplicable observable are recorded on the row itself.

## Validation

- Strict shard audit: 48 identities, 48 current symbol locators, 47 direct, 1 adapted, 0 pending.
- Focused owner suite: `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_INCREMENTAL=false cargo test -p nuxie-scripting --features compiler --lib upstream_promise_tests:: -- --test-threads=1` — 48 passed.
- Repository correspondence checker: passed.
- Correspondence checker unit suite: 24 passed.
- Scoped `git diff --check`: passed.
- Non-test release IR scan: no Wave C11 test symbols retained.

This receipt is candidate evidence only and does not self-accept Wave C11.
