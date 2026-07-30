REJECT — runtime behavior is unchanged and the full held-behavior set remains clean, but round 10 still has three blocking detector/registry bypasses.

## Standards / mechanics findings

- **BLOCKING — macro expansion order bypasses fragment scanning.** The detector concatenates identifiers in lexical order and searches that single normalized string ([main.rs:813](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:813)). I replayed this against exact detector digest `81d0ab38…`:

  ```rust
  macro_rules! reverse_join {
      ($suffix:ident, $prefix:ident) => {
          paste! { owner.[<$prefix $suffix>](owner, child, Some(host), &batch); }
      };
  }
  fn evades() { reverse_join!(events, notify_); }
  ```

  Expansion produces `notify_events`, but the detector emitted nothing and exited 0. The permanent negative tests only source-order `[<notify_ events>]` ([test_check.py:4664](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4664)). This violates the fail-closed future-spelling claim ([closure:935](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:935)) and W63’s any-call-form requirement ([W63:61](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:61)).

- **BLOCKING — an alias exported from the exempt owner module defeats cross-file selection detection.** The checker wholly skips `state_machine_instance.rs` ([check.py:64](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:64), [check.py:1646](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1646)). Therefore that module can re-export `RuntimeNestedAnimationInstance::StateMachine as Chosen`; a non-owner can then match `Chosen(owner)` without containing any guarded final segment. Direct replay of the non-owner import/match emitted nothing. The round-10 cross-file test places its exporting bridge in a non-owner file, where the export itself trips, so it misses this owner-origin form ([test_check.py:4590](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4590)). This still permits nested-animation selection in a non-owner file contrary to W63.

- **BLOCKING — the registry is fungible within one enclosing item.** Registry identity is only `(file, kind, anchor, guarded_name)` ([check.py:1539](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1539)); suppression consumes the first matching `(anchor, guarded_name)` without binding the recorded `site_offset` ([check.py:1680](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1680)). Moving a blessed selection into a forbidden branch of the same function produced:

  ```text
  match selection 33 68 Approved::choose StateMachine
  match selection 33 90 Approved::choose StateMachine
  ```

  Both satisfy the same registry row. The substitution negative changes `Approved::choose` to `Rejected::choose`, so it never exercises same-anchor relocation ([test_check.py:4828](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4828)). This is not an exact blessed-site allowlist.

- **NON-BLOCKING — adversarial-row count prose is stale.** The packet says “Twelve” and later “all twelve,” but candidate configuration and the checklist contain 13 completed rows ([closure:480](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:480), [closure:998](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-closure.md:998), [ownership:30](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-ownership.toml:30)). Enforcement covers all 13.

## Spec / behavior verification

No behavioral findings.

- **No runtime delta:** `afcb7058`, candidate `e729dd74`, and publication `4ecce48a` all have identical `crates` tree object `473036c1…`. `nuxie-runtime` is `84af0b36…`, `nuxie` is `35c5a2e4…`, and the runtime tests subtree is `91cd2cec…` at all three commits. The delta’s 14 paths are exclusively documentation and `tools/runtime-frame-loop-port`; `git diff --quiet … -- crates` is clean.

- **Both W76 behavioral closures hold:** the failing owner captures the error, completes ancestor bubbling and audio unwind, and only then retains `script_error` ([production:13148](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13148)); the direct test proves mid-chain invisibility and `parent-local → root-local → root-audio → parent-audio` ([test:18634](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:18634)). BlendDirect’s Rust-only clone/remount handle proof remains present ([state_machine.rs:2007](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:2007)).

- **Blend shapes hold:** Blend1D retains only its occurrence vector, `from`, `to`, and reset, with reset-before-blend application ([state_machine.rs:820](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:820), [state_machine.rs:1043](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)). BlendDirect retains only its occurrence vector, whose elements contain definition handle, animation, and mix ([blend_state_direct_instance.rs:3](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/blend_state_direct_instance.rs:3), [blend_state_direct_instance.rs:163](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/blend_state_direct_instance.rs:163)). The symmetric same-owner differential still covers both ([cpp_probe.rs:20271](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:20271)).

- **Callback-major delivery holds:** each nested-simple callback completes its singleton chain, zeros overshoot, and precedes mix ([production:12934](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12934)); the live differential asserts `[1,1]` batches, zero delays, complete callback-major ordering, and matching final mix ([cpp_probe.rs:84925](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:84925)).

- **O1–O3 hold:** construction retains preparation failure without suppressing the machine ([lib.rs:3993](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/lib.rs:3993)); typed named-input lookup remains authored-order and type-specific ([state_machine_instance.rs:5326](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5326)); semantic actions still resolve manager node ID through the recorded resolver before dispatch ([state_machine_instance.rs:5574](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5574)).

- **Retained arenas/reset order hold:** occurrences retain the definition arenas passed at construction ([state_instance.rs:27](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_instance.rs:27)); interruption teardown and reassignment precede replacement-reset construction ([state_machine_layer_instance.rs:733](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733)).

- **All seven FL-B proofs remain:** invalid interpolator ([19594](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19594)), importer cursor ([19626](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19626)), doomed sink ([19676](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19676)), negative-speed remap ([19703](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19703)), signed loop ([20004](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:20004)), empty-baseline reset ([24489](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:24489)), and NaN direct blend ([24592](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:24592)).

## Candidate-mode packet spot-check

Candidate mode is genuinely active ([ownership:22](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-ownership.toml:22)) and requires completed configured rows ([check.py:1110](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1110)). The live checker passed with 342 files, 75 members, 10 gaps, and all ratchets at their declared bounds.

Member-row proof spot-checks were consistent:

- `advance(seconds,newFrame)` names the live trigger differential, present at [cpp_probe.rs:77992](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:77992).
- `stateChangedByIndex` names the generic-layer occurrence differential, present at [cpp_probe.rs:82410](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/tests/cpp_probe.rs:82410).
- `applyEvents` points to both synchronous-flow proofs at [flow_session.rs:6643](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/flow_session.rs:6643) and [flow_session.rs:6703](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/flow_session.rs:6703).
- `pointerDown`/`initScriptedObjects` proofs remain at [scripted_listener_action_lifecycle_tests.rs:1749](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/scripted_listener_action_lifecycle_tests.rs:1749), [3825](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/scripted_listener_action_lifecycle_tests.rs:3825), and [4559](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie/src/scripted_listener_action_lifecycle_tests.rs:4559).
- E8 trace binds candidate `e729dd74`, 7,310 files, fingerprint `51eedc8f…`, and matching runner provenance ([trace:7850](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-trace.json:7850)).

The behavior/spec axis is clear. The structural publication gate is not fail-closed.

**REJECT**