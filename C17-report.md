Completed the 42-row C17 reconciliation:

- 3 canonical importer moves.
- 39 justified exceptions: 10 reconciled, 5 already exact, 24 plan amendments.
- Removed two orphaned compatibility shims and redirected 43 generated inventory mappings.
- No Rust behavior or item bodies changed.

Full dispositions are in [C17-report.md](/Users/levi/dev/worktrees/nuxie-mr-c17/C17-report.md:18); amendments are in [mr-move-plan.md](/Users/levi/dev/worktrees/nuxie-mr-c17/.parity-decomp/mr-move-plan.md:740).

Required gates pass:

- `cargo check --workspace --exclude nux-capi --tests`
- `make runtime-frame-loop-port-check` — 108 tests
- `make rust-attribution-check` — 10 tests
- Final standards and spec reviews — no findings

Commits could not be created because the sandbox cannot write the shared worktree Git metadata (`index.lock`). `C17-report.md` is also ignored and must be force-added by a writable integrator. The pre-existing `MR2-report.md` change was untouched.