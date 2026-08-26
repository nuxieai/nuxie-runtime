# Wave C17 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C17 covers all 36 active Catch cases in pinned
`tests/unit_tests/runtime/semantic_label_inference_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This candidate changes test and
evidence semantics only. It does not modify production behavior and does not
declare Wave C17 accepted.

## Exact census

- 36/36 cases mapped, zero pending;
- classifications: 35 direct, one C++-language-only adapted, zero differential;
- outcomes: 36 pass, zero expected-red;
- label inference and role classification: 15 pass;
- spatial and diff ordering: eight pass;
- manager-scoped id allocation and identity: four pass;
- incremental content/bounds refresh and removal: nine pass.

## Executable evidence

All rows execute the live `SemanticManager` and retained
`SemanticNodeHandle` owners. The four disjoint test modules preserve the
pinned ids, role values, labels, bounds, tree shapes, mutation and drain order,
authoritative diff arrays, manager-scoped lookup identity, and individual
assertion streams. Shared helpers only construct or mutate those concrete
owners; expected values remain literal at each test.

Case 15 is classified `cxx-language-only`: Rust exposes one `u32`
`is_interactive_role` owner rather than C++ enum and `uint32_t` overloads, so
both pinned assertion streams use explicit enum-to-`u32` conversion.

No report/corpus metadata, source-string anchors, test-local resolver,
aggregate substitute, fake semantic owner, proxy/facade, constant failure,
unconditional panic, or production behavior change is used. Because every
literal case passes, this candidate contains no ignored expected-red row.

## Gates

- focused non-incremental owner suite: 36/36 green;
- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, evidence-locator, and symbol validator: 36/36 green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0`. The only shared
source edit is four late `cfg(test)` module declarations in
`semantic_manager.rs`; executable additions are confined to the four
disjoint Wave C17 test files.
