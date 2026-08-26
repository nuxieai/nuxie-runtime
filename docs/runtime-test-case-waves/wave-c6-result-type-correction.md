# Wave C6 exact-result type correction candidate

Original candidate: `d095c9721`

Independent rejection: `cf956662e`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected scope

Only the three rejected result conversions changed:

- case 21, `buffer.convert f32 to u8norm`;
- case 22, `buffer.convert u8norm clamps out-of-range f32`;
- case 26, `buffer.convert u8 to u16`.

Each case now deserializes its Lua result tuple as `f64`, matching pinned `lua_tonumber` double conversion, and retains its distinct exact equality assertions in the pinned order. The literal Lua programs, all other 32 accepted cases, both extra safety tests, production behavior, and ledger classifications are unchanged.

## Counterexample

A temporary forced counterexample changed only case 21's second returned Lua value from `128` to `128.9`. The corrected test failed exactly at `left: 128.9, right: 128.0`. The rejected `i64` conversion would have truncated that value to `128` and falsely passed. The pinned return expression was restored before the final suite and lexical audit.

## Validation

- Focused non-incremental suite: 37 passed, zero failed, zero ignored.
- Strict Wave C6 shard: 35/35 identities and locators, 35 direct passes, zero pending.
- Literal program audit after restoration: 35/35 token-identical to pinned upstream.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24 passed.
- Non-test release LLVM IR: no Wave C6 test/helper symbols retained.
- Scoped formatting and `git diff --check`: passed.

This is a correction candidate for fresh independent rereview and does not self-accept Wave C6.
