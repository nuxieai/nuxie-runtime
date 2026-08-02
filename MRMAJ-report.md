Completed C06’s hotspot work.

- Moved 17 wholly owned rows into dedicated importer modules with manifest updates.
- Skipped exceptions `B6-0213` and `B6-0214`.
- Marked `B6-0235`–`0237` split-needed/cross-root for C07+C17; foreign roots remain untouched.
- Preserved all `audit_record` and B6 verdict fields.
- Required workspace checks passed; all 72 focused C++ importer tests passed.
- Full tests were blocked only by missing `.riv` fixtures.

Report: [MRMAJ-report.md](/Users/levi/dev/worktrees/nuxie-mr-c16/MRMAJ-report.md)

The initial commits succeeded, but sandbox ref-lock restrictions prevented squashing the corrected target-module moves with their manifest changes. The corrected worktree is the source of truth; the orchestrator must squash it against pinned base `2dafa25d` using the intended commit message recorded in the report.