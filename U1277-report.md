Implemented UNIV-1277 and UNIV-1278 companion work. Full details are in [U1277-report.md](/Users/levi/dev/worktrees/nuxie-p1a-binary/U1277-report.md).

Key outcomes:

- Added exact-zero effective-opacity gates for Shapes, nested artboards, and component lists.
- Geometry queries now fully settle deferred nested opacity propagation.
- Retained geometry remains inclusive.
- Added four Scene API lifecycle regressions.
- Added Transform parent/dependent construction assertions.
- Added codegen fixture, golden corpus entry, and Rust/C++ differential coverage.
- Resolved all standards/spec review findings.

All requested gates pass:

- `cargo test -p nuxie-runtime`
- `cargo test -p nuxie --features scripting`
- `make golden-compare` — 320/320 exact
- `make cpp-probe`
- `make cpp-binary-compare` — 71/71
- Formatting and diff checks

No commit was created.