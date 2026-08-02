Implemented and committed C17’s step-1 exception-note seed.

- Commit: `d4be1320 [MR-2/C17] Seed justified exception notes`
- Appended all 75 verbatim justifications in [file-correspondence-manifest.toml](/Users/levi/dev/worktrees/nuxie-mr-c17/file-correspondence-manifest.toml)
- Preserved every `rust_module`, `audit_record`, and B6 verdict
- No Rust/source files changed
- Moved rows: none
- Skipped rows: none
- Cross-root queue: none
- Report: [MR2-report.md](/Users/levi/dev/worktrees/nuxie-mr-c17/MR2-report.md)
- Branch is clean, ahead of `origin/main` by one commit
- No push or PR performed

Validation confirmed exactly 75 note-only row changes and valid TOML with 448 rows. `cargo check --workspace` was attempted normally and offline, but could not pass because `nux-capi`’s unchanged `cbindgen` step required uncached crates (`combine`/`alsa`) while network access was unavailable. Independent review found no standards or implementation issues; the unavailable green cargo gate was the sole finding.