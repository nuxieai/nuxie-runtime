Implemented the P1-i Promise family without committing.

Key artifacts:

- [Promise implementation](/Users/levi/dev/worktrees/nuxie-p1i-promise/crates/nuxie-scripting/src/vm/promise.rs:1)
- [47 scenarios and contract tests](/Users/levi/dev/worktrees/nuxie-p1i-promise/crates/nuxie-scripting/tests/promise_scenarios.rs:1)
- [Pinned C++ oracle](/Users/levi/dev/worktrees/nuxie-p1i-promise/tools/promise-oracle/main.cpp:1)
- [P1I report](/Users/levi/dev/worktrees/nuxie-p1i-promise/P1I-report.md:1)

Validated:

- `make promise-differential`: 47 upstream + 2 invalid-yield cases, all exact
- `cargo test -p nuxie-scripting`: passed
- `cargo test -p nuxie --features scripting`: passed
- `make scripted-golden-compare`: 319/319 exact, zero divergences
- Formatting, clippy, and focused manifest tests passed
- Final standards and specification reviews found no remaining issues

The manifest row is `faithful` with `pending-verification`, and image decode remains excluded. The C++ oracle now validates archive provenance, compiler, defines, configuration, and SHA-256 before reuse.