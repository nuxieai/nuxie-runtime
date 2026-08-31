---
description: Maintain the Rust port one upstream commit at a time, with separate translation and two review passes.
---

# Incremental upstream sync

Read `docs/upstream-sync-map.md` and `docs/PARITY_WORKFLOW.md` first. They define
the checkpoint, frozen destination, supported scope, and source-first method.

1. Confirm the current request authorizes writes. Scheduled inspection alone is
   read-only; this command does not grant merge permission.
2. Enumerate upstream commits oldest first from the completed checkpoint to the
   frozen target. Use a clean candidate worktree; leave normal checkouts alone.
3. Inspect the next complete diff, surrounding owners, tests, and dependencies.
   Port applicable changes, skip proven irrelevant/equivalent changes, and split
   mixed commits by actual source rather than title. Ask about major new scope.
4. Translate that delta and its relevant tests/assets. Do not load implement or
   TDD skills, restart a whole-runtime port, or restore superseded implementations.
5. After translation, perform source-equivalence review, then a separate
   Rust-integration review. Review subsequent semantic corrections too.
6. Run targeted checks against the matching upstream checkpoint. Commit with
   `Upstream-Commit: <full SHA>`; keep one upstream change's identity intact.
7. Continue in order within the authorized scope. Small PRs may contain several
   commits; validate the applicable broader harnesses before landing. Record
   skips and unverified coverage briefly in the PR, not a new ledger.
8. Advance the completed checkpoint and coherent active pins only through the
   fully accounted prefix. Preserve historical provenance and fail-closed checks.

Parallelize disjoint work, not integration order. Green samples are not proof of
complete translation. Preserve all approved Rust and browser-target adaptations.
