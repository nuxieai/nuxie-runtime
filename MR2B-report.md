C02 split wave is committed as `481f7617`.

- Moved 6 rows cleanly.
- Recorded 8 justified entanglement exceptions.
- Queued 5 cross-owner rows.
- Updated both manifests, source sets, attribution, imports, and compatibility plumbing.
- Preserved the unrelated `MRMAJ-report.md` modification.

Validation passed:

- `cargo check --workspace --exclude nux-capi`
- `make runtime-frame-loop-port-check`
- Formatting and diff checks

`make rust-attribution-check` reports only three pre-existing unclassified files; no C02-created findings remain.

Full disposition: [MR2B-report.md](/Users/levi/dev/worktrees/nuxie-mr-c12/MR2B-report.md)

Sandbox worktree locks prevented switching/rebasing from stale `levi/mr2-c02` to requested `levi/mr2b-c02`. The commit is ready for integrator transplant onto current `origin/main`.