Implemented P2-e and left it uncommitted.

Highlights:

- Added the v2 little-endian decoder, validation, and per-device snapshots in [gamepad_batch.rs](/Users/levi/dev/worktrees/nuxie-p1c-importers/crates/nuxie-runtime/src/state_machine/gamepad_batch.rs).
- Wired runtime and `Frame` facade submission.
- Added eight decoder/lifecycle tests.
- Enabled the full 19-batch silver scenario; it now records an honest render divergence instead of an ingestion blocker.
- Updated manifests, correspondence counts, generator sources, and ratchets.
- Completed standards/spec review and fixed its stale generator finding.
- `Cargo.lock` is unchanged; neither stash was popped.

All six requested gates pass. Full details are in [P2E-report.md](/Users/levi/dev/worktrees/nuxie-p1c-importers/P2E-report.md).

One non-requested full port-manifest check still reports an unrelated pre-existing missing `core_uint64_type.cpp` row; this is documented in the report.