Implemented the complete round-9 corrective with both binding policies enforced.

Key changes:

- Correct cfg evaluation, module/associated-type resolution, and fail-closed macro/attribute scanning in [main.rs](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:74).
- Exact per-site registry validation—including same-function drift—in [check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:1503) and [test_check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_check.py:4469).
- Error retention now occurs after complete bubbling/audio unwind in [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13115).
- Added honest BlendDirect clone/remount coverage in [state_machine.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine.rs:2007).
- Updated registry, FL-G03, ratchets, and closure policy in [runtime-frame-loop-gaps.toml](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-gaps.toml:68) and [runtime-frame-loop-fl-c5-closure.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-closure.md:915).

Acceptance:

- Runtime: 726 passed
- Tools differentials: 823 passed
- Nuxie library: 147 passed
- Checker: 71 passed
- Ordinary and scripted goldens: 317/317 entries, 647 exact segments, zero divergences
- Standalone detector clean-cache `--locked` build: passed
- Formatting and `git diff --check`: passed
- Independent standards and spec reviews: no remaining findings

The live checker reports only the spec-deferred E7 trace fingerprint and runner-provenance refreshes. No tests were weakened, no rm-style commands were used, and no commit was created.

Goal completed in 33m 27s using 512,882 tokens.