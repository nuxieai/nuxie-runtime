Two blocking detector defects remain. FL-B behavior itself is still clear.

## Findings

- **BLOCKING — cross-file re-exports evade the ownership detector.** The checker analyzes each Rust file independently ([check.py:1593](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/check.py:1593)), while module resolution descends only into inline `mod { … }` bodies ([main.rs:299](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:299)). Consequently:

  ```rust
  // bridge.rs
  pub(crate) use crate::artboard::RuntimeNestedAnimationInstance as Anim;

  // parent.rs
  mod bridge;
  if let bridge::Anim::StateMachine(owner) = animation { … }
  ```

  produces no resolved selection: the unresolved qualified path returns without recording a hit ([main.rs:545](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:545)), and the lexical fallback runs only when parsing fails ([main.rs:781](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:781)). The permanent negative tests only an inline module ([test_check.py:4370](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/test_check.py:4370)). This violates the requirement to detect nested-animation selection in every non-owner file ([W63:63](/Users/levi/dev/worktrees/nuxie-e7-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:63)).

- **BLOCKING — the registry is a fungible file quota, not an exact blessed-site allowlist.** For a registered `(file, kind)`, the checker validates only the number of sites and then discards every hit in that file/kind ([check.py:1621](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/check.py:1621)). The production row contains only `artboard.rs`, `selection`, and count `1` ([gaps.toml:1425](/Users/levi/dev/worktrees/nuxie-e7-verify/docs/runtime-frame-loop-gaps.toml:1425)). Deleting the approved selection and inserting one forbidden selection elsewhere in `artboard.rs` therefore remains green. Tests cover count changes to two or zero, but not one-for-one substitution ([test_check.py:4528](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/test_check.py:4528)). That contradicts W63’s “allowlisting only the blessed policy entry calls” requirement.

- **NON-BLOCKING — none.**

## Held FL-B verification

The `99ef7700..afcb7058` FL-B production sweep changes only the restored test in `state_machine.rs`.

- BlendDirect’s proof is honestly labeled and symmetric in claim: it clones one Rust occurrence, remounts, advances that clone, and asserts retained handles; it makes no C++ differential-parity claim ([state_machine.rs:2007](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:2007)).
- Error withholding is confined to nested-event delivery after ancestor dispatch and audio unwind ([state_machine_instance.rs:13148](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13148)), outside FL-B’s enumerated production closure ([FL-B spec:38](/Users/levi/dev/worktrees/nuxie-e7-verify/docs/runtime-frame-loop-fl-b-spec.md:38)).
- Reset teardown/reassignment still precedes reset construction ([state_machine_layer_instance.rs:733](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733)).
- Blend1D retains exactly its occurrence vector, `from`/`to`, and single construction-time reset ([state_machine.rs:820](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:820)); apply remains reset-then-blend ([state_machine.rs:1043](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)).

| Historical regression | Status |
|---|---|
| Signed loop override | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:20004)) |
| Invalid interpolator erasure | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19594)) |
| Importer-cursor survival | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19626)) |
| Doomed-owner sink | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19676)) |
| Negative-speed remap | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19703)) |
| NaN direct blend | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:24592)) |
| Empty-baseline reset | Clear ([test](/Users/levi/dev/worktrees/nuxie-e7-verify/crates/nuxie-runtime/tests/cpp_probe.rs:24489)) |

Packaging is structurally repaired: standalone `[workspace]` ([Cargo.toml:7](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/rust-owner-detector/Cargo.toml:7)), committed dependency lock ([Cargo.lock:1](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/rust-owner-detector/Cargo.lock:1)), and exact `cargo build --locked` invocation ([check.py:424](/Users/levi/dev/worktrees/nuxie-e7-verify/tools/runtime-frame-loop-port/check.py:424)). Locked offline metadata resolution passed. The sandbox prohibited all temporary-directory writes, so the forced fresh build and checker could not produce an independent green execution receipt; they failed before candidate code ran.

Standards axis: 2 blocking findings. Spec/behavior axis: no additional findings.

REJECT