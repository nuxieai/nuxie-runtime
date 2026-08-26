# Wave C17 independent adversarial review

Author commit: `801cd0e23`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/semantic_label_inference_test.cpp`

Verdict: **REJECTED — one case-ledger schema defect; executable semantics are
36/36 accepted**

This review kept the author code and Wave C17 ledger frozen. It compared all
36 pinned Catch cases one-for-one against the four executable Rust modules,
including fixture construction, node ids, roles, labels, bounds, tree shape,
mutation and drain order, retained lookup identity, and every authoritative
`added`, `removed`, `updatedSemantic`, `updatedGeometry`, and
`childrenUpdated` assertion.

## Exact-stream verdict

- Cases 1-14: accepted.
- Case 15: executable assertion stream accepted. Rust has a single `u32`
  `is_interactive_role` owner, so explicit enum-to-`u32` conversions preserve
  both pinned C++ overload assertion streams without inventing an owner.
- Cases 16-36: accepted.
- In particular, cases 19, 22-27, and 35-36 preserve every pinned constant,
  empty-array guard, identity check, tree order, mutation sequence, and
  authoritative diff assertion. No scenario is merged, collapsed, or replaced
  by a test-local expected-value computation.
- No fake owner, proxy observable, unconditional failure, ignored test, or
  production behavior change was found.

The executable semantics therefore pass review 36/36. The wave is not
accepted because its machine evidence fails closed as described below.

## Blocking finding

### Case 15 — adaptation metadata does not satisfy the shard schema

`docs/runtime-test-case-waves/wave-c17.json` marks case 15 as `adapted` but
records only:

```json
"adaptation_kind": "cxx-language-only"
```

The current `nuxie-test-case-correspondence/v1` validator requires an
`adaptation` object containing all three fields:

- `kind`;
- `rationale`;
- `inapplicable_observable`.

The strict gate stops at case 15 with:

```text
tests/unit_tests/runtime/semantic_label_inference_test.cpp#15 adapted case requires adaptation metadata
```

This also makes the candidate note's claim that the strict classification and
adaptation validator is 36/36 green incorrect. Correct only the case 15 ledger
metadata, preserving its accepted Rust test and the other 35 rows, then rerun
the strict shard validator for independent closeout.

## Gates

- Focused non-incremental owner suite:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_INCREMENTAL=false cargo test -p nuxie-runtime wave_c17_ -- --nocapture`
  — 36 passed, zero failed, zero ignored.
- Pinned identity, ordinal, source-line, exact-name, symbol, and evidence-locator
  audit — 36/36 valid.
- Strict Wave C17 case-ledger validator — rejected at case 15 for missing
  required adaptation metadata.
- Repository correspondence checker — 157 files / 1,404 cases, green.
- Correspondence checker unit suite — 24/24 green.
- JSON parse, author-commit `git diff --check`, and production-freeze review —
  green.

Acceptance count remains **0/36 for Wave C17** until the frozen machine ledger
passes its required strict gate. The executable semantic review need not be
reopened unless the correction changes author code or any assertion stream.
