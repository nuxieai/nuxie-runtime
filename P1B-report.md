Implemented the C2 asset owner split with no commit.

- Added six direct owners under [assets](/Users/levi/dev/worktrees/nuxie-p1b-assets/crates/nuxie-binary/src/assets/mod.rs).
- Preserved C3 import-stack/importer regions in [lib.rs](/Users/levi/dev/worktrees/nuxie-p1b-assets/crates/nuxie-binary/src/lib.rs).
- Updated all six [manifest rows](/Users/levi/dev/worktrees/nuxie-p1b-assets/file-correspondence-manifest.toml) to `faithful` with anchors.
- Wrote the per-row [P1B-report.md](/Users/levi/dev/worktrees/nuxie-p1b-assets/P1B-report.md).

Required gates passed:

- `cargo test -p nuxie-binary` — 213 passed.
- `cargo test -p nuxie-runtime` — 885 passed, 1 existing ignored.
- `make cpp-binary-compare` — 70/70 passed.
- Shader ratchets — 12 decoder/registration and 16/16 resolution tests passed.
- Formatting and diff checks passed.

## Standards

No findings.

## Spec

No findings.

Review summary: 0 Standards findings, 0 Spec findings.

One unrelated baseline issue remains: `make b6-audit-check` reports its pre-existing verdict-census mismatch. This change modifies no `b6_verdict` fields.