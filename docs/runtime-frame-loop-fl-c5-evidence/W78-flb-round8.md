## Findings

**BLOCKING — the syn-AST ownership ratchet cannot be built from the frozen checkout.**

The detector inherits workspace metadata ([detector Cargo.toml](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/Cargo.toml:1)), but the root workspace neither includes nor excludes it ([root Cargo.toml](/Users/levi/dev/worktrees/nuxie-e6-review/Cargo.toml:2)). Consequently, the checker’s exact build-on-demand command ([check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/check.py:424)) fails with:

> current package believes it's in a workspace when it's not

A clean-cache candidate therefore cannot execute the mandatory ratchet, contradicting the published 67/67/live-checker claims ([closure](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-closure.md:937)).

W73’s exact glob-import probe is permanently present ([negative](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/test_check.py:3949)), and the AST logic correctly binds glob variants ([resolver](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:215)). Direct replay through pre-existing detector binaries produced `hit selection 40`. The logic is correct, but the immutable tree cannot build or enforce it reproducibly. This remains a shared blocking ownership-ratchet defect.

**NON-BLOCKING — none.**

## Held verification

Blend1D is behaviorally clean. The invented `last_applied_*` state is gone; Rust constructs its sole reset in `new` and only applies that retained reset before blending ([constructor](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:820), [apply](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)). This matches pinned C++ exactly ([constructor](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:10), [apply](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:67)). Same-owner parity and separate clone/remount handle coverage remain present ([differential](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:20271), [unit proof](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:1926)).

| Historical FL-B correction | 99ef7700 |
|---|---|
| Signed loop override | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:20004)) |
| Invalid interpolator erasure | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:19594)) |
| Importer-cursor survival | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:19626)) |
| Doomed-owner sink | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:19676)) |
| Negative-speed remap | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:19703)) |
| NaN direct blend | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:24592)) |
| Empty-baseline reset | Clear ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:24489)) |

Interrupted-transition teardown/reassignment still precedes reset construction ([production](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733), [tripwire](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:83990)). Exact FL-B scope remains 45 unique files with the importer included and scripted listener excluded ([ledger](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-ownership.toml:184)).

The publication `Next` pointer is corrected ([status](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-status.md:837)). The dated interim-floor deferral is explicit coordinator policy and is not a finding ([policy](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:23)).

REJECT