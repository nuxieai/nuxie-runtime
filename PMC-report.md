Reconciliation completed and committed as `30e4315d`:

`[P3-f] Reconcile command-transport status across ledgers`

Adjudication:

- Both command transport files are honestly `partial`.
- Correspondence encodes this as `pending` + `pending-verification` + legacy `partial`, because its schema does not permit `partial`.
- `command_server.cpp` now consistently lists both Rust owners.
- Evidence is corrected to 14 focused tests and 4/83 complete upstream cases.
- B6 audit fields and MR exception remain intact.

All gates pass:

- `port-manifest-check`: 20/20 tests, 447/447 rows
- `runtime-frame-loop-port-check`
- `rust-attribution-check`: 10/10 tests

Review has zero unresolved standards or spec findings. The worktree is clean and the stash remains untouched.

Full rationale: [PMC-report.md](/Users/levi/dev/worktrees/nuxie-mr-c13/PMC-report.md)