# Runtime exact-parity campaign

This campaign applies the renderer-port lesson to the pinned Rive runtime:
establish the complete upstream test oracle first, then make source ownership
comparable, then correct demonstrated translation discrepancies. It is not a
general refactor or improvement campaign.

## Operating constraints

- Do not use the `implement` or `tdd` skills during this campaign. The phase
  order and evidence rules in this document are the execution method.
- Port pinned behavior before improving it. When a Rust result differs, find
  the incorrect source translation or an already-approved adaptation; do not
  invent a replacement behavior from the failing test.
- Preserve the permanent adaptation ceilings: Taffy remains the layout owner,
  the Rust-native audio and scripting backends remain, Rust slices and checked
  arithmetic replace C++ container helpers, and safe Rust ownership is not
  forced to reproduce undefined behavior or allocator bookkeeping.
- A phase has one branch and one PR. Do not mix the next phase into the current
  PR, even if an immediately obvious production change would make a newly
  translated test pass.
- CI plumbing, packaging optimization, editor work, and broad unsupported-host
  validation are separate work and cannot become parity blockers.

## Phase 1 — complete upstream unit-test port

Pinned upstream: `rive-app/rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

The frozen denominator is 157 files and 1,404 active Catch2 `TEST_CASE`s from
`tests/unit_tests/runtime/*.cpp` and
`tests/unit_tests/runtime/scripting/*.cpp`. The starting correspondence census
was 655 covered cases and 749 uncovered cases.

A case counts as ported only when its evidence points to a discovered Rust test
that retains the upstream fixture, action order, and complete assertion
sequence. A currently failing production assertion may remain `#[ignore]`, but
the test body and expected value must remain literal and its reason must name
the missing production behavior. Nearby coverage, a source citation, a test
that only proves its own helper, or a weakened assertion does not count.

Approved language/backend adaptations must preserve the meaningful observable
sequence and record any inapplicable C++-only assertion explicitly. They must
not silently be called literal parity. Tests may be red; Phase 1 changes no
production runtime behavior merely to make them green.

Phase acceptance:

1. Every active upstream case has direct or differential Rust evidence, or an
   explicit adaptation disposition for an inapplicable C++-only observable.
2. `test-correspondence-manifest.toml` has no `pending` or `partial` file row.
3. The correspondence checker recounts the pinned 157/1,404 denominator from
   source and accepts every evidence path.
4. The PR reports active, failing, and ignored totals honestly. Green is not an
   acceptance condition.
5. The PR contains tests, fixtures, correspondence evidence, and Phase 1
   tooling/documentation only—no parity-motivated production fixes.

## Phase 2 — one-to-one source correspondence

After Phase 1 merges, create one primary private Rust owner for each packed
upstream behavioral source owner. Split the large multi-owner Rust files along
the pinned C++ source boundaries without changing behavior. Preserve public
Rust interfaces unless a private seam is required to expose an already-owned
behavior to its translated test.

Phase acceptance is a bijective source ledger for applicable behavioral
owners, behavior-neutral compilation/test results, and one source-
correspondence PR.

## Phase 3 — paired source audit

Review every one-to-one source pair against the pin. For each discrepancy,
classify it as an approved adaptation, an intentional Rust safety correction,
or an incorrect/missing translation. Change production behavior only for the
last class and bind the correction to the Phase 1 test that exposed it.

## Phase 4 — denominator closure

Activate the expected-red tests as their source discrepancies close. Re-run the
complete direct and differential denominators, preserving explicit approved
adaptations rather than rewriting them as literal equivalence.

## Phase 5 — final parity closeout

Run the full supported product matrix, freeze byte/provenance evidence, review
for forbidden fallbacks and unrecorded adaptations, and publish the remaining
known differences. Improvements beyond pinned behavior begin only after this
closeout PR merges.
