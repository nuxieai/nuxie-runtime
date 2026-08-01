Completed both P1G tasks without committing.

- Promoted `lua_color` to faithful.
- Kept `lua_data_context`, `lua_data_value`, `lua_state`, and `lua_gradient` pending with exact missing semantics documented.
- Added host logging, `_G.print` routing, production `File` sink configuration, reentrant-print fidelity, and unprefixed Lua error routing.
- Updated both manifests and the legacy generator/checker.
- Full details are in [P1G-report.md](/Users/levi/dev/worktrees/nuxie-p1g-luarec/P1G-report.md).

Verification:

- `cargo test -p nuxie-scripting`: 181 passed
- `cargo test -p nuxie --features scripting`: 458 passed
- Host logging: 6 passed
- Manifest tests: 17 passed
- Formatting and `git diff --check`: clean
- Final standards/spec reviews: no remaining findings
- No commit created.