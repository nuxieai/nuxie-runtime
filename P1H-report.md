Implemented P1-h buffer extensions without committing.

- Added all four APIs in [buffer_ext.rs](/Users/levi/dev/worktrees/nuxie-p1h-buffers/crates/nuxie-scripting/src/vm/buffer_ext.rs): `readf16`, `writef16`, `stridedcopy`, and `convert`.
- Registered them before sandboxing in [vm.rs](/Users/levi/dev/worktrees/nuxie-p1h-buffers/crates/nuxie-scripting/src/vm.rs:1092).
- Ported all 35 upstream scenarios plus two overflow/non-finite hardening tests in [buffer_extensions.rs](/Users/levi/dev/worktrees/nuxie-p1h-buffers/crates/nuxie-scripting/tests/buffer_extensions.rs).
- Updated both correspondence manifests conservatively; the source row remains pending orchestrator verification.
- Wrote [P1H-report.md](/Users/levi/dev/worktrees/nuxie-p1h-buffers/P1H-report.md).

All gates pass:

- `cargo test -p nuxie-scripting`
- `cargo test -p nuxie --features scripting`
- `make scripted-golden-compare` — 319/319 exact entries, 649 exact segments, zero divergences
- Manifest checker, formatting, and `git diff --check`

Review results:

- Standards: no remaining findings.
- Spec: no remaining findings. The 35 scenarios are honestly classified `ported-direct` because the available C++ probe ABI cannot execute arbitrary inline Luau.

Summary: 0 standards findings, 0 spec findings. No commit created.