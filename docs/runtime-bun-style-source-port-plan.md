# Bun-style runtime source port plan

Decision date: 2026-08-26

This plan supersedes the per-pair certification loop in
`docs/runtime-exact-parity-workflow-correction.md`. The campaign is returning
to the central Bun port tactic: mechanically translate the complete pinned
source tree into a corresponding Rust source tree before spending substantial
time certifying tests, fixtures, or platform matrices.

The immutable upstream remains Rive runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. Continue to ignore and not use the
`implement` and `tdd` skills. Preserve Taffy and the Rust-native audio and
scripting adaptations.

## Primary deliverable

For every applicable pinned C++ source owner, create or identify one primary
Rust source file that visibly mirrors the upstream owner.

The Rust should preserve the upstream file boundary, type and method names
where Rust permits, method order, retained fields and defaults, control flow,
meaningful branches, callback order, mutation order, error paths, and clone or
reset behavior. Rust-specific representation changes are allowed only for an
already approved adaptation and should be named briefly beside the translated
code.

The deliverable of this pass is the source translation itself. A ledger,
receipt, fixture census, or bespoke test is not a substitute for the missing
Rust source owner.

## Atomic workflow

For one complete source file at a time:

1. Read the complete pinned implementation and its primary handwritten
   header.
2. Create or normalize the corresponding Rust file so unrelated upstream
   owners are not packed into it.
3. Translate every executable body and retained state directly, keeping the
   Rust structure visibly comparable to the C++.
4. When existing Rust behavior differs, recover the pinned behavior from the
   source comparison. Do not invent a fix from a test or downstream symptom.
5. Give the complete pair one lightweight independent source read. Review the
   translation itself; do not create a separate rejection receipt, fixture
   census, consumer topology, or custom evidence campaign.
6. Apply concrete review corrections directly and commit the completed source
   pair. Record only a concise commit message and, when necessary, a short
   inline adaptation note.
7. Move immediately to the next source owner.

Do not require a new unit test merely because a source file was translated.
Add a focused test only when it is the shortest practical way to preserve a
subtle cross-language semantic difference discovered during translation.

## Batch checks

Run fast existing checks at natural directory or commit-batch boundaries, not
after every small function or review comment:

- compile/check the affected Rust crates;
- run the existing focused package tests that are already available;
- keep the worktree and adaptation boundaries intact.

Defer global correspondence, release IR, frozen-byte, forbidden-fallback,
fixture-wide scans, platform CI, and full differential matrices until a source
directory or PR-sized batch is complete.

## Explicitly deferred work

The source translation pass does not close or repeatedly re-adjudicate C7-C10,
the 1,404-case ledger, pending consumer counts, fixture-reference counts,
expected-red rendering cases, or platform CI. Preserve existing results as
historical evidence, but do not make their closeout a prerequisite for
translating the next source file.

After the complete source tree has one-to-one correspondence:

1. port or reconcile the full upstream test suite against the now-callable
   Rust owners;
2. run the broad differentials and global gates;
3. fix failures by returning to the exact translated source pair;
4. perform the final parity audit once, across the completed tree.

## Progress accounting

Use a minimal source checklist with only these states:

- `not started`;
- `translated`;
- `blocked by an approved adaptation decision`;
- `needs source correction`.

Do not count pending tests as translated source, and do not require test-case
or fixture accounting to mark a source file translated. A source file is
translated when every executable body and retained state in the complete pair
has a concrete, visibly comparable Rust owner or a named approved adaptation.

## Completion criterion

This pass is complete when the applicable pinned C++ source tree has a
one-to-one, mechanically comparable Rust source tree with no silently omitted
bodies or state. Test-suite parity and final runtime parity are subsequent
campaign gates; they are not interleaved into every source-file translation.
