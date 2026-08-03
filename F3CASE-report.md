F3CASE is complete and the worktree is clean at `fa3b2927`.

- Promoted all 62 non-F6 cases.
- Ledger: 66 complete, 0 pending non-F6, 13 pending F6, 4 S4-45 WATCH.
- `cargo check -p nuxie` passed.
- Focused suite passed: 73 tests, 0 failures.
- Independent standards/spec reviews are clean after fixes.
- No full batteries or golden comparisons were run.
- CommandQueue/CommandServer correspondence rows remain unpromoted because F6 is outstanding; `lua_scripted_context.cpp` remains untouched.
- Scatter remains 154; Luau pin unchanged.

See [F3CASE-report.md](/Users/levi/dev/worktrees/nuxie-mr-c16/F3CASE-report.md) and [the case ledger](/Users/levi/dev/worktrees/nuxie-mr-c16/docs/p3f-command-queue-test-ledger.md).