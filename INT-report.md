C10 reconciliation is complete and committed as `71e7252c` (`[MR-2/C10] Extract no rows from crates/nuxie/src/lib.rs`).

- Moved: none. All 17 C10 rows are planned facade exceptions retaining `lib.rs`.
- Exceptions: 17, with existing manifest justifications preserved.
- Queued: 12 cross-root reconciliations; 5 rows already match their final retained-module sets.
- No changes to `lib.rs`, correspondence manifest, or ownership ledger.
- All required Cargo, frame-loop, and attribution gates pass.
- Review: 0 standards findings; no spec findings except branch-base drift.

Full details: [INT-report.md](/Users/levi/dev/worktrees/nuxie-mr-c14/INT-report.md:1).

`origin/main` advanced two unrelated documentation commits during verification. The branch is ahead 1/behind 2; sandbox restrictions prevented fast-forwarding or rebasing onto `4886b6cf`.