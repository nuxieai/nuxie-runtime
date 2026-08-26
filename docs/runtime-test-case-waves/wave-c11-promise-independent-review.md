# Wave C11 Promise independent adversarial review

Candidate: `c07debf1e00ec5d096723708890cf1470cf63d5a`

Executable-test author commit: `b7e6c13a91`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_promise_test.cpp`

Verdict: **REJECTED — 41/48 exact executable streams accepted; seven altered
programs and 47 invalid evidence locators**

This review kept the candidate tests, ledger, and production behavior frozen.
It compared every pinned Luau program and assertion stream one-for-one against
the claimed Rust owner test, including fulfillment/rejection values, status
snapshots, cancellation propagation and hooks, coroutine resume order,
non-throwing await results, retry behavior, and every promise-flattening path.

## Exact-stream census

- Exact executable streams accepted: **41**.
- Executable streams rejected: **7** — cases 25, 27, 28, 30, 31, 39, and 40.
- Case 12's structured C++-language-only adaptation: **accepted**.
- Focused execution: **48 passed, zero failed, zero ignored**.
- Machine evidence locators accepted by the repository's Rust-test resolver:
  **1/48** (case 12 only).

Whitespace, line wrapping, optional statement separators, and optional trailing
table commas were treated as syntax-preserving presentation differences. They
did not cause rejection. The seven rows below remove function parameters or
named intermediate values from the executable program itself.

## Accepted C++-language-only adaptation

Case 12, `async coroutine inherits thread data (print works)`, executes the
pinned program with both exact messages in exact order:

1. `first resume` before `await(Promise.resolve(1))`;
2. `post-await resume` after coroutine resumption.

The test installs the real Rust host `print` callback, captures both calls, and
asserts the exact two-element ordered result. Under the campaign's assigned
exception, only the raw C++ `ScriptingContext*` / `lua_setthreaddata` identity
is inapplicable because mlua does not expose that pointer contract. No other
observable is omitted. The ledger row describes this correctly.

The candidate prose should nevertheless use the pinned message literals above;
its sole-adaptation summary currently calls them `before await` and
`after await`, which are not the strings in either executable program.

## Rejected executable streams

1. `scripting_promise_test.cpp#25`, `cancel sets state to Cancelled` — the
   pinned executor is `function(resolve, reject, onCancel) end`; the Rust
   program changes it to `function() end`. The final status assertion is
   preserved, but the claimed literal program is not.
2. `scripting_promise_test.cpp#27`, `cancel propagates down to consumers` —
   the parent Promise executor drops the pinned `resolve`, `reject`, and
   `onCancel` parameters. The child-status assertion remains correct, but this
   is a simplified program rather than a one-for-one port.
3. `scripting_promise_test.cpp#28`, `cancel propagates up when all consumers
   cancelled` — the parent executor again drops all three pinned parameters.
   It also removes the pinned `afterSecond = p:getStatus()` snapshot and reads
   `p:getStatus()` later in the return expression. That can erase an ordering
   regression if work is inserted between the second cancellation and the
   final observation.
4. `scripting_promise_test.cpp#30`, `cancelled promise does not fire andThen
   callbacks` — the Promise executor drops the pinned three-parameter list.
   The callback result is preserved, but the executable source is altered.
5. `scripting_promise_test.cpp#31`, `getStatus returns correct strings` — the
   pinned program stores `cancelled:getStatus()` in `cancelledStatus` before
   constructing the result. Rust removes that status snapshot and performs a
   later inline getter call instead.
6. `scripting_promise_test.cpp#39`, `Promise.resolve flattens a fulfilled
   promise` — the pinned program creates the named `outer` promise and invokes
   `outer:andThen(...)`. Rust removes that owner binding and chains directly
   from `Promise.resolve(inner)`.
7. `scripting_promise_test.cpp#40`, `Promise.resolve flattens a rejected
   promise` — the pinned program similarly retains `outer` before installing
   its catch handler. Rust removes the binding and chains directly from the
   constructor expression.

These are small changes, but this campaign exists specifically to avoid
semantic simplification by inspection. Restore the pinned programs rather than
arguing that the current synchronous implementation makes the differences
unobservable.

## Blocking machine-evidence defect

The ledger's 47 macro-generated cases point their `rust-test` evidence at the
identifier argument inside `number_case!`, `string_case!`,
`string_contains_case!`, or `bool_case!`. Those lines are not Rust function
definitions and do not carry a discoverable `#[test]` attribute. The shared
`resolve_rust_test` gate therefore rejects every such locator with:

```text
does not resolve <symbol> at crates/nuxie-scripting/src/vm/lua_promise.rs:<line>
```

Only case 12 points at an explicit `#[test] fn` and passes the resolver. Cargo
does expand the macros into 47 distinct runnable test entrypoints, so this is
not an aggregate-execution finding. It is still a blocking correspondence
defect: the machine ledger cannot prove those entrypoints using the repository's
declared `rust-test` locator contract.

Correct this by giving each row an explicit, discoverable `#[test] fn` locator
(shared evaluation helpers are fine), while keeping each case's literal Luau
program and assertion stream visible and distinct. Do not weaken the resolver
or replace the cases with a shared loop.

## Gates

- Pinned checkout: exact SHA confirmed.
- Pinned census, ids, ordinals, source lines, and exact Catch names: **48/48
  valid**.
- Ledger JSON and schema envelope: valid; 47 direct, one adapted, 48 pass,
  zero pending.
- Strict per-evidence `resolve_rust_test` audit: **1 valid / 47 invalid**.
- Focused non-incremental owner suite:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_INCREMENTAL=false cargo test -p
  nuxie-scripting --features compiler --lib upstream_promise_tests:: --
  --test-threads=1` — **48 passed**.
- Repository correspondence checker: **157 files / 1,404 cases, green**. The
  global checker currently consumes the all-pending global case ledger, not
  this Wave C11 shard, so it does not cure the shard-locator failures.
- Correspondence checker unit suite: **24/24 green**.
- Non-test release build and LLVM IR symbol scan: green; no Wave C11 test
  module or test symbol retained.
- Candidate/test scoped `git diff --check`: green.

Wave C11 acceptance remains **0/48** until the seven exact programs and 47
locators are corrected and independently re-reviewed. The other 41 executable
semantic streams, including case 12's adaptation, need not be reopened unless
their source changes during correction.
