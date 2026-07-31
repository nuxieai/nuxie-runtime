REJECT — production correctives look behaviorally correct, but the mandatory structural ratchet remains evadable.

## Standards

**BLOCKING — nested-animation selection can escape the W63 ownership ratchet.**

W63 requires structural detection of nested-animation selection in every non-owner file ([W63 §7](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:61)). The detector recognizes direct paths and explicit aliases, but not glob imports ([check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/check.py:602)):

```rust
use RuntimeNestedAnimationInstance::*;

fn displaced() {
    if let StateMachine(owner) = animation {
        displace(owner);
    }
}
```

Direct detector replay returned `[]`; the equivalent fully qualified control returned `[0]`. Existing negatives cover direct and explicit-alias forms but omit glob imports ([test_check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/test_check.py:3733)). Therefore the green checker does not satisfy the binding structural-ratchet requirement.

## Spec

**NON-BLOCKING — publication status still contains a pre-publication instruction.**

W63 requires `Next` to be true at the publication commit, specifically “no publish-this instruction” ([W63 §8](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:72)). Published HEAD still says “Publish the staged E5 evidence/docs packet” ([status](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-status.md:829)).

No blocking behavioral/spec defect found.

## Corrective verification

- Pinned C++ HEAD resolves exactly to `d788e8ec…`. Its callback loop reports each crossed keyframe immediately ([keyed_property.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/keyed_property.cpp:90)); `Scene` executes the callback synchronously ([scene.cpp](/Users/levi/dev/oss/rive-runtime/src/scene.cpp:33)); and `LinearAnimationInstance` constructs a singleton vector before notifying ([linear_animation_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/linear_animation_instance.cpp:442)).
- Rust now calls its sink inside the crossing loop ([animation.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/animation.rs:1403)), zeroes nested-simple delay and finishes each singleton before mix ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12969)).
- The C++ recorder retains batch boundaries ([main.cpp](/Users/levi/dev/worktrees/nuxie-e5-review/tools/cpp-probe/main.cpp:395)); the differential asserts `[1,1]` for both C++ and Rust ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:84937)). The focused prebuilt Rust production test passed.
- Blend1D and BlendDirect both enable and execute clone/remount before the second advance ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:20271)), consistent with C++’s stable occurrence vector and `from`/`to` pointers.

## Regression replay

| Check | Result |
|---|---|
| Exact FL-B scope | Clear: one unique 45-file row; importer included, scripted listener excluded ([ledger](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-ownership.toml:184)) |
| Signed loop | Differential intact ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:20004)) |
| Invalid interpolator | Differential intact ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:19594)) |
| Importer cursor | Differential intact; doomed-owner sink also remains covered ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:19626)) |
| Negative-speed remap | Differential intact ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:19703)) |
| NaN direct blend | Differential intact ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:24607)) |
| Empty-baseline reset | Differential intact ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:24504)) |
| Reset order | Teardown/reassignment precedes reset construction and is guarded differentially ([production](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733), [test](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:84005)) |

Fixture-based live reruns were attempted but stopped before assertions because the frozen sandbox permits no temporary-file writes. Likewise, the checker’s 67-test wrapper had 63 temp-directory setup errors and four passes. The live fail-closed checker itself exited green, and independent fingerprint recomputation matched 7,295 files and `92a2588f…`.

The boundary merge changed no `crates/nuxie-runtime` file. Its only scoped production delta is `crates/nuxie/src/scene.rs`; the FL-B-adjacent changes merely add authoring export mappings for interpolator schema records ([scene.rs](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie/src/scene.rs:5780)), without changing runtime animation owners, callback delivery, resets, or blends. All reviewed delta ranges pass `git diff --check`; the checkout remains clean.

Axis summary: Standards has one blocking finding; Spec has one non-blocking finding.

REJECT for FL-B reacceptance.