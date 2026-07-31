Implemented the round-8 corrective exactly as specified.

- Reverted Blend1D’s invented per-apply reset/replay state and added symmetric same-owner C++ parity plus a labeled retained-handle clone/remount unit test: [state_machine.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043), [cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:20271)
- Failing reporting owners now complete bubbling/audio-tail work before propagating listener errors, with regression coverage: [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7427)
- Replaced all four regex ownership detectors with a build-on-demand syn AST resolver and fail-closed token/macro tripwire: [main.rs](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:14), [check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:399)
- Added permanent W66/W68/W71/W72/W73 negatives, including valid AST forms and allowlist coverage: [test_check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_check.py:3839)
- Published the nonzero-time explanation and detector closure note: [W67-report.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-evidence/W67-report.md:25), [closure.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-closure.md:889)
- `runtime-frame-loop-status.md` is untouched.

Acceptance results:

- `nuxie-runtime --lib`: 725 passed
- Full C++ probe: 823 passed
- `nuxie --lib`: 147 passed
- Both golden comparisons: 317 exact entries / 647 segments / 0 divergences
- Checker tests: 67/67 passed
- Live checker: only the two explicitly deferred E6 failures—source fingerprint and Rust runner provenance
- Formatting and `git diff --check`: clean

The scripted Make wrapper attempted an upstream clean and was denied; I did not retry that path. The identical scripted comparison was completed successfully through non-deleting build steps.

No commit was created; HEAD remains `3bef19dad2e127025991d993fb9456f8325cbac3`. Goal usage: 517,100 tokens over about 33m 35s.