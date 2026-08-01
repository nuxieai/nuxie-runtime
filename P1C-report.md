Implemented the complete P1-c C3 importer split without committing.

- Added all 22 filename-owner modules under [importers/mod.rs](/Users/levi/dev/worktrees/nuxie-p1c-importers/crates/nuxie-binary/src/importers/mod.rs).
- Preserved ordered import-stack behavior and kept C2 asset helpers in [lib.rs](/Users/levi/dev/worktrees/nuxie-p1c-importers/crates/nuxie-binary/src/lib.rs).
- Updated all 22 [manifest rows](/Users/levi/dev/worktrees/nuxie-p1c-importers/file-correspondence-manifest.toml) to `faithful` with validated anchors.
- Added the per-row [P1C report](/Users/levi/dev/worktrees/nuxie-p1c-importers/P1C-report.md).
- Existing ratchets and differential tests remain unmodified.

All gates passed:

- `cargo test -p nuxie-binary`: 213 passed
- `cargo test -p nuxie-runtime`: 885 passed, 1 ignored
- `make cpp-binary-compare`: 70/70 passed
- `make golden-compare`: 319/319 exact, 649 segments, 0 divergences
- Manifest tests: 17 passed
- Formatting and diff checks: passed
- Final spec review: no findings
- Standards review: no hard violations

No commit was created.