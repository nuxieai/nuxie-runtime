Implemented the architecturally honest integration:

- Removed P3F command transport from frame-loop source/file/member ownership.
- Retained only the verified FL‑E scripting fragment of `command_server.cpp`.
- Preserved `faithful=349`, `pending=0`; trace hashes were untouched.
- Added fail-closed `verified-fragment` checker support and regression coverage.
- Kept F3 tracking in both correspondence and port manifests.
- Standards and spec reviews found no issues.

All gates pass:

- `make runtime-frame-loop-port-check`
- `make rust-attribution-check`
- `cargo check -p nuxie --features scripting`

The sandbox prevented writing the linked Git index, so no commit was created. The exact atomic commit map and requested subject are in [P3F2-report.md](/Users/levi/dev/worktrees/nuxie-mr-c16/P3F2-report.md).