Fixed the regression without committing.

Root cause: P1-k removed the unconditional instance snapshot refresh after attachment. Identical bytes skipped decoded-owner propagation, leaving stale sibling snapshots unchanged.

Changes:

- Restored unconditional refresh in [lib.rs](/Users/levi/dev/worktrees/nuxie-p1o-rb3rec/crates/nuxie/src/lib.rs:4554).
- Added a non-scripting sibling regression test at [lib.rs](/Users/levi/dev/worktrees/nuxie-p1o-rb3rec/crates/nuxie/src/lib.rs:6409).
- Documented diagnosis, C++ semantics, and verification in [FIX1-report.md](/Users/levi/dev/worktrees/nuxie-p1o-rb3rec/FIX1-report.md:1).

All requested gates passed:

- `cargo test -p nuxie --features scripting`
- `cargo test -p nuxie-runtime` — 851 passed, 1 ignored
- `make golden-compare` — 319/319 exact, 649 segments, 0 divergences
- `cargo fmt --all -- --check`
- `git diff --check`

Note: `origin/main` advanced by three P1-p commits while this work was running. I left this branch at its original `58d927da` base and did not rebase or commit.