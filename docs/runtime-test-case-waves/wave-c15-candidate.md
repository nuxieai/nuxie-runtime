# Wave C15 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C15 covers all 19 active Catch cases in pinned
`semantic_artboard_test.cpp` and `semantic_data_lifecycle_test.cpp` at
upstream SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It does not change
production behavior or declare Wave C15 accepted.

## Exact census

- 19/19 rows mapped;
- classifications: nine direct, four Rust-safety adapted, six pending;
- outcomes: eight pass, five expected-red, six unverified pending;
- semantic artboard: one pass, five genuine expected-red, four pending;
- SemanticData lifecycle: seven pass, zero expected-red, two pending.

## Executable evidence

The six executable lifecycle owner cases use live `RuntimeSemanticData` and
its retained node. They preserve the exact authored constants, setter order,
trait/state bit combinations, identity checks, and assertions. The fixture
lifecycle case loads, binds, settles, fires the live semantic action, settles
again, and asserts the exact `updated_semantic` state transition.

The six complete artboard cases retain the pinned fixture, four fandom labels,
10-frame settle, pointer and semantic action order, bounds/state/trait
assertions, three collapse cycles, and unique-id count. Five are genuine
production reds; the pointer/semantic convergence case passes.

Six rows remain pending without executable placeholders:

- artboard cases 1, 4, and 5 require selected-manager `node_by_id`;
- artboard case 9 additionally requires authoritative parent/ancestor access;
- lifecycle cases 7 and 8 require scope-driven Drop teardown.

The prior panic-backed lookup bodies were removed. No replay-map-only claim,
explicit detach substitute, injected identity, aggregate stand-in,
report/corpus metadata, or production behavior change is used.

## Gates

- focused non-incremental owner suite: eight pass / five ignored / six pending;
- all five complete expected-reds forced individually at their live assertions;
- strict pinned identity, ordinal, source-line, exact-name, classification, and
  outcome validator: 19/19 green;
- focused evidence-locator, pending-reason, ignored-reason, and forbidden-proxy
  validator: 19/19 rows and 13/13 unique executable symbols green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green;
- default non-test LLVM IR contains no Wave C15 test-owner symbols.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`.
