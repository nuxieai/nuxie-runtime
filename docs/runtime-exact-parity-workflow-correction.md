# Runtime exact-parity workflow correction

> **Superseded for execution on 2026-08-26.** The per-pair certification
> workflow below again accumulated the bookkeeping and bespoke-proof overhead
> it was intended to remove. Continue the campaign using
> `docs/runtime-bun-style-source-port-plan.md`. This document remains as a
> historical record of the completed Phase 1A transition and the source-pair
> work already accepted; it is no longer the active execution loop.

Decision date: 2026-08-26

This document corrects the execution workflow in
`docs/runtime-exact-parity-plan.md`. It preserves the pinned denominator,
exact-behavior rules, approved adaptations, and prohibition on invented
production fixes. It replaces the rigid assumption that all 1,404 tests can be
completed before source correspondence begins.

## Why the workflow changed

The test-port campaign produced valuable evidence, but its phase boundary
became a deadlock. A growing number of upstream cases cannot execute exactly
because the Rust runtime has no callable one-to-one owner for an upstream
type, method, intermediate state, or ordering boundary. Phase 1 prohibited the
source work needed to create those owners while also requiring zero pending
tests.

The review process also accumulated disproportionate overhead: small waves
repeated global correspondence, release-IR, schema, candidate, rejection,
correction, and rereview cycles. The first independent semantic review often
found important defects, but repeated bookkeeping reviews displaced the main
Bun-style task: comparing complete upstream and Rust source files literally.

The work is retained. Independent review has already caught real translation
errors and false evidence, including numeric truncation, omitted renderer and
phase assertions, non-pinned fixture topology, backing-container proxies,
test-local algorithms, and Unicode-category logic substituted for shaped glyph
lookup.

## Correct accounting

Do not describe a pending case as certified, ported, or executable.

At the decision point, the inherited campaign total said 999 rows were
accepted. The immutable receipt audit completed in
`docs/runtime-phase-1a-checkpoint.md` corrected that inherited statement:

- 929 rows had durable independent acceptance: 852 executable, three approved
  C++-only not-applicable, and 74 pending owner blockers;
- 70 executable Wave B1 rows had only an explicitly named self-acceptance and
  therefore remain provisional rather than independently accepted.

The later C7-C9 closeout is recorded only in the Phase 1A checkpoint. This
document retains the decision-point numbers to explain why the workflow
changed, not as the current campaign total.

Future reporting must always separate:

1. executable exact ports;
2. genuine executable expected-red cases;
3. approved executable adaptations;
4. pending missing-owner blockers.

## Corrected sequence

### 1. Close the in-flight checkpoint

Finish only Waves C7, C8, and C9, including one independent semantic review
and any narrow correction needed for acceptance. Do not start another test
wave.

Publish a consolidated Phase 1A checkpoint that reports executable cases,
expected reds, adaptations, and pending owner blockers separately. A pending
row is useful inventory but does not satisfy the original Phase 1 acceptance
criterion.

### 2. Establish one-to-one source correspondence

Move immediately to the Bun-style atomic unit: one complete pinned C++ source
owner beside one primary Rust source owner.

For each applicable upstream behavioral source file:

1. read the complete pinned file;
2. enumerate every function, override, callback, static, meaningful branch,
   default, side effect, and ordering dependency;
3. map each item to a concrete Rust symbol and line;
4. classify it as mechanically equivalent, equivalent under a named approved
   adaptation, deliberately unsupported/not applicable, or missing/incorrect;
5. split packed Rust owners along upstream file boundaries without changing
   behavior;
6. have one fresh reviewer compare the complete pair without relying on the
   prior classification.

Prioritize source owners blocking pending tests, shared owners, multi-module
mappings, tracked gaps, divergent rows, and pending-verification rows, but
ultimately cover the complete source denominator.

### 3. Complete blocked test ports

As real one-to-one owners become callable, replace pending rows with literal
tests. Preserve the complete upstream fixture, action order, intermediate
observables, and assertion stream. Do not use a helper implementation, backing
container, static graph, downstream rendering result, or other proxy for a
missing owner.

### 4. Correct source discrepancies

Change production behavior only after the source-pair audit identifies an
incorrect or missing translation. Recover pinned behavior rather than inventing
a fix from the test. Bind every correction to the exact translated test or
state-trace differential that demonstrates it.

### 5. Close parity

Activate expected-red tests as their source discrepancies close. Then run the
global correspondence, release-IR, frozen-byte, forbidden-fallback, and
supported-product gates once at the appropriate PR closeout instead of after
every small source batch.

## Working rules

- Continue to ignore and not use the `implement` and `tdd` skills.
- Keep the pinned upstream SHA
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5` immutable.
- Preserve Taffy and the Rust-native audio and scripting adaptations.
- One source file/pair is the normal work and review unit. Group files only
  when the upstream ownership boundary genuinely spans them.
- Use one author and one independent semantic reviewer. After a rejection, the
  reviewer checks the correction delta and affected assertions; do not restart
  an unrelated full audit unless the correction changes the source authority.
- Run focused tests and the strict local evidence check per unit. Run expensive
  global/release/IR gates at checkpoint or PR closeout, except when containment
  of a new test-only seam specifically requires an immediate check.
- Keep candidate/rejection/correction evidence concise. Evidence exists to make
  source comparison trustworthy, not to become the primary deliverable.
- Do not shape source code around ledger coordinates. Refresh locators when
  exact code naturally moves.
- Do not count pending rows as completed parity.

## Completion claim

The runtime may claim exact source parity only when every pinned C++ function
and meaningful behavior has a concrete Rust counterpart or approved
adaptation, every pair has been independently reviewed at symbol level, every
translatable upstream test executes, and every remaining difference is
explicitly approved rather than hidden behind a proxy or accounting status.
