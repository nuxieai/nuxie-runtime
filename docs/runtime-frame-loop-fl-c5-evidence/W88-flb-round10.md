REJECT. FL-B production behavior remains clear, but round 10 still does not close the shared detector/registry blockers.

## Standards

- **BLOCKING — neutral cross-file owner wrappers evade detection.** The checker skips `state_machine_instance.rs` entirely ([check.py:1647](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1647)) and propagates cross-file aliases only for audio ([check.py:1580](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1580)). A neutral `DELIVER` wrapper around `notify_events` in the skipped owner, called from a non-owner file, produces no dispatch hit because matching recognizes only `notify_events*` ([main.rs:39](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:39)). I replayed this against the exact candidate detector: the owner body emitted a hit, while the non-owner `state_machine_instance::DELIVER(...)` call emitted nothing. This violates W63’s any-call-form and consistent-helper-rename requirements ([W63:63](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:63)). The permanent owner-originating cross-file negative covers audio only ([test_check.py:4240](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4240)).

- **BLOCKING — registry substitution remains possible under a forged/same anchor.** Although the detector emits `site_offset`, the checker authorizes only `(anchor, guarded_name)` ([main.rs:664](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:664), [check.py:1673](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1673)). Thus relocation within the same enclosing item passes. The added substitution test changes `Approved::choose` into `Rejected::choose`, so it never tests a preserved or forged anchor ([test_check.py:4828](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4828)).

## Spec

- **BLOCKING — a different function can impersonate a blessed policy-entry anchor.** `qualified_anchor` contains impl/module context plus the immediate item name, but omits the enclosing outer function ([main.rs:680](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:680), [main.rs:1003](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:1003)). I mutated the real focused-input source in memory:

  1. Deleted the blessed key-input selection at [focused_input_dispatch.rs:55](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/focused_input_dispatch.rs:55).
  2. Inserted a local function named `dispatch_nested_key_input_at_focus` inside the unrelated text-input method.
  3. Put the forbidden `StateMachine` selection in that local function.

  The candidate detector emitted exactly:

  ```text
  ArtboardInstance::dispatch_nested_text_input_at_focus StateMachine
  ArtboardInstance::dispatch_nested_key_input_at_focus StateMachine
  ArtboardInstance::dispatch_nested_gamepad_at_focus StateMachine
  ArtboardInstance::broadcast_nested_gamepad_to_scripted_drawables StateMachine
  ```

  These are precisely the four registered keys ([gaps.toml:1439](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-gaps.toml:1439)). The forged key is therefore consumed as blessed at [check.py:1687](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/check.py:1687), with no extra selection hit. The round-9 one-for-one substitution attack remains possible despite the new anchors.

## Requested confirmations

- **Delta:** Confirmed. `afcb7058..e729dd74` touches no `crates/nuxie-runtime/src/**` file, including all five enumerated FL-B production files ([FL-B spec:38](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-b-spec.md:38)).

- **Behavior and shapes:** Held by source identity. Blend1D still owns only its occurrence vector, `from`/`to` handles, and one construction-time reset; apply remains reset-before-blend ([state_machine.rs:820](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:820), [state_machine.rs:1043](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)). BlendDirect retains definition handles without copied definition payloads ([blend_state_direct_instance.rs:4](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/blend_state_direct_instance.rs:4)). Both clone/remount proofs remain intact ([state_machine.rs:1926](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:1926), [state_machine.rs:2007](/Users/levi/dev/worktrees/nuxie-e8-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:2007)). All seven historical regression proofs remain unchanged: signed loop, invalid interpolator, importer cursor, doomed-owner sink, negative-speed remap, NaN direct blend, and empty-baseline reset.

- **Verbatim bridge:** Confirmed tripping. The W83 cross-file `bridge::Anim::StateMachine` form emitted `hit selection` and a matching selection record; its permanent negative is at [test_check.py:4590](/Users/levi/dev/worktrees/nuxie-e8-verify/tools/runtime-frame-loop-port/test_check.py:4590). This does not cure the stronger wrapper and anchor-collision evasions above.

- **Candidate mode:** Reproduced green using the exact content-addressed candidate detector. The live checker exited 0, including all four owner ratchets at `0/0..0`. Candidate mode is active ([ownership.toml:22](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-ownership.toml:22)); the independently recomputed 7,310-file fingerprint exactly matched the trace and `e729dd74` reference ([trace.json:7850](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-trace.json:7850)).

- **Clean-cache locked build:** Locked offline metadata and dependency-tree resolution passed. A genuinely fresh target could not be independently rebuilt because this frozen sandbox denies every temporary-directory creation before Cargo runs. The clean-cache build therefore remains supported by the W84 receipt ([W84:9](/Users/levi/dev/worktrees/nuxie-e8-verify/docs/runtime-frame-loop-fl-c5-evidence/W84-report.md:9)), not independently reproduced here.

No non-blocking code findings.

REJECT for FL-B reacceptance.