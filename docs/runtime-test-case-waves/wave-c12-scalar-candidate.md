# Wave C12 scalar/blob fragment

Status: **CANDIDATE FRAGMENT; PENDING MERGE AND INDEPENDENT REVIEW**

This fragment covers the six assigned non-silver cases in pinned
`scripting_properties_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`: ordinals 2, 5, 6, 7, 8, and
17. It does not claim the eight previously accepted cases or silver ordinals
9-16.

## Census

- 6/6 pinned identities mapped;
- five executable passing cases;
- one individually forceable expected red;
- zero pending or proxy cases.

## Owner evidence

Every case executes the literal pinned Luau source through the live `ScriptVm`
and reaches the retained `ScriptViewModel` plus its real scripted-property
userdata. The color, string, boolean, and enum cases select the authored
instance from the exact upstream fixture, preserve the host/script mutation
order, and assert every console line in order.

The integrated property case uses the exact `data_binding_test.riv` schemas,
five host mutations, direct and named rotation reads, both listener forms, and
both trigger directions. It exposes one real divergence: Rust invokes the two
rotation listeners newest-first (`changed with context`, then `changed`), while
pinned C++ requires registration order (`changed`, then `changed with
context`). The ignored expected-red test retains the pinned assertion and fails
at that live owner seam when forced.

For the fixture-free blob case, pinned C++ can construct a standalone property
owner. Rust's safe wrapper requires a schema-backed owner, so the test uses the
existing vendored blob schema and then preserves the exact `{10,20,30}` asset,
literal script, fresh wrapper calls, pre-write size/byte assertions, `abcd`
write, retained asset bytes, and post-write size/byte assertions.

## Gates

- focused non-incremental run: five pass and one ignored expected red;
- forced non-incremental expected red: fails with the exact reversed listener
  stream at the pinned ordered-console assertion;
- correspondence checker: 157 files / 1,404 cases;
- correspondence checker unit suite: 24/24;
- scoped format/diff and evidence-locator checks: pass.

No production behavior changed. The only parent-file edit is the late
`cfg(test)` child-module declaration.
