Completed the D16 reconciliation and left the worktree uncommitted.

Key outcomes:

- Typed script advance failures now park owners exactly like C++ `false`, while still surfacing `ScriptError`.
- Replaced the retry regression with park+surface coverage for all three owner families.
- Added the persistent-failure Rust/C++ differential; both advance exactly once.
- Removed D16 and FLR-21 ceiling framing.
- Updated the three manifest rows and corrected closure provenance.
- Final standards/spec review has no remaining findings.

All requested checks pass:

- `cargo test -p nuxie-runtime`
- `cargo test -p nuxie --features scripting`
- `make scripted-golden-compare` — 319/319 entries, 649/649 segments
- `make runtime-frame-loop-port-check`
- Formatting and diff checks also pass.

Full report: [P1O2-report.md](/Users/levi/dev/worktrees/nuxie-p1o-rb3rec/P1O2-report.md)