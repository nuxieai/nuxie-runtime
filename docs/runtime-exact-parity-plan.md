# Runtime exact-parity campaign

> **Workflow correction (2026-08-26):** The rigid phase boundary in this
> document created a missing-owner deadlock during the test-port campaign.
> Continue under
> [`runtime-exact-parity-workflow-correction.md`](runtime-exact-parity-workflow-correction.md),
> which preserves the parity rules but closes the in-flight test checkpoint and
> moves one-to-one source correspondence ahead of the remaining blocked tests.

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
classified 655 cases as covered and 749 as uncovered by promoting file-level
statuses. That estimate is retained only as historical navigation metadata. It
is not behavioral proof.

The machine-owned case ledger is `test-correspondence-cases.json`. Its truthful
Phase 1 baseline is **0 accepted dispositions and 1,404 pending cases**. Every
row is keyed by pinned upstream path plus one-based source ordinal, and also
freezes the declaration line and case name. That makes duplicate Catch2 names
unambiguous while allowing the checker to reject missing, duplicate, reordered,
or stale rows against the pin. `test-correspondence-manifest.toml` remains the
compatible file-level navigation/history ledger; changing one of its file
statuses never promotes a case.

A case counts as ported only when a case-level row points to a discovered Rust
test that retains the upstream fixture, action order, and complete assertion
sequence, or to a live C++/Rust differential that executes that case. A
file-level status plus an evidence path is not case-level proof. A currently
failing production assertion may remain `#[ignore]`, but the test body and
expected value must remain literal and its reason must name the missing
production behavior. Nearby coverage, a source citation, a test that only
proves its own helper, or a weakened assertion does not count.

Case ledger dispositions are deliberately narrow:

- `direct` resolves one exact Rust source path, function line, and discovered
  test symbol. `pass` evidence cannot be ignored. `expected-red` must resolve to
  `#[ignore = "expected-red: …"]`, with the same reason recorded in the row.
- `differential` identifies an executable `.rs`, `.py`, or `.sh` harness, its
  stable differential id, both language entry points, and its argv command.
- `adapted` names an approved adaptation kind, the inapplicable observable, and
  a rationale. A wholly `not-applicable` row is reserved for C++-language-only
  behavior; backend and Rust-safety adaptations still require executable proof.
- `pending` is `unverified` and carries no evidence, adaptation, or note.

The case ratchet is independent from the historical file ratchet. Its
`max_pending` value may only fall, and a case that has appeared as direct,
differential, or adapted in tracked history may not regress to pending. The
all-pending generator refuses to overwrite an existing ledger unless `--force`
is given, because regeneration otherwise destroys proof.

Recount and validate the live ledger with:

```sh
python3 tools/runtime-frame-loop-port/check_test_correspondence.py \
  --rive-runtime-dir /path/to/pinned/rive-runtime
```

`generate_test_case_ledger.py` exists only to establish or deliberately reset
the all-pending skeleton. Day-to-day work edits individual case rows and lowers
`ratchet.max_pending` by the same number of accepted promotions.

Approved language/backend adaptations must preserve the meaningful observable
sequence and record any inapplicable C++-only assertion explicitly. They must
not silently be called literal parity. Tests may be red; Phase 1 changes no
production runtime behavior merely to make them green.

Phase acceptance:

1. Every active upstream case has direct or differential Rust evidence, or an
   explicit adaptation disposition for an inapplicable C++-only observable.
2. Every one of the 1,404 cases has its own accepted disposition; a file row
   cannot promote unnamed cases transitively.
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
