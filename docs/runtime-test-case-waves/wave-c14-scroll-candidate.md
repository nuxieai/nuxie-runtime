# Wave C14 scroll exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

This narrow Wave C14 slice covers all four active Catch cases in pinned
`scroll_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It does not change production
behavior or declare the slice accepted.

## Exact census

- four of four cases are distinct direct executable ports;
- all four reach the real pinned SRIV parser/comparator and expose genuine
  production divergences;
- there are no pending, adapted, proxy, aggregate, or placeholder rows.

## Owner evidence

Each test independently encodes its pinned fixture and action stream in Rust.
The shared helpers only import and instantiate the selected real artboard,
initialize its renderer, bind view-model instance zero where authored, draw,
and compare serialized bytes. No action or outcome is sourced from the Silver
manifest or an aggregate runner.

All four first differences are the authoritative C++ negative-zero transform
component versus Rust positive zero in frame zero. The former
`missing_silver_match` unconditional-panic file was deleted, so it no longer
competes as evidence.

## Gates

- focused non-incremental suite: zero pass / four ignored genuine reds;
- all four expected-reds forced individually and reproduced their exact first
  differences;
- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, locator, and ignored-reason validator: 4/4 green;
- pinned checkout, both fixtures, and all four SRIV baselines: present;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green;
- default non-test Silver-corpus LLVM IR contains no Wave C14 scroll symbols.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`.
