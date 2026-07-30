REJECT — four blocking findings. Blend1D itself is correctly repaired, but the failing-owner contract remains incomplete, a held BlendDirect proof was removed, and the replacement ownership ratchet has two direct bypasses.

## Standards

- **BLOCKING — production code under negated/mixed test cfg is invisible to the ratchet.** `cfg_test` classifies an item as test-only whenever `test` appears anywhere inside `#[cfg(...)]`, without respecting `not(test)` or mixed alternatives ([main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:131)). Such items are then skipped entirely ([main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:461)). Therefore this valid production relocation produces no hit:

  ```rust
  #[cfg(not(test))]
  fn displaced() {
      StateMachineInstance::notify_events(...);
  }
  ```

  The same problem affects `#[cfg(any(test, feature = "tools"))]`, which contains production code in tools builds. Tests cover plain `#[cfg(test)]` exclusion only ([test_check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/test_check.py:4249)). This violates W63’s every-non-owner-file requirement ([W63](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:63)).

- **BLOCKING — the allow tag is a self-service bypass, not a blessed-entry allowlist.** Any same-line `flc5-owner-ratchet-allow: <kind>` or `all` text suppresses a hit, with no validation of the file, function, or permitted policy entry ([main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:344), [main.rs](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:357)). The permanent test explicitly treats a direct macro-composed `notify_events` access as safe merely by appending that comment ([test_check.py](/Users/levi/dev/worktrees/nuxie-e6-review/tools/runtime-frame-loop-port/test_check.py:4080)). W63 permits only blessed policy entry calls ([W63](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:67)).

## Spec

- **BLOCKING — the failing owner’s error becomes observable before its full chain completes.** The correction queues the bubble and defers source audio, then returns `Err` ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7490)). `notify_events` immediately retains that error as terminal `script_error` state ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7345), [retention](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5132)); ancestor dispatch and audio unwind happen only afterward ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13159)).

  The new test exposes rather than closes the ordering gap: it invokes the failing owner first, then manually drains/notifies the root and flushes audio ([test](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:18575)). It checks `script_error` only after those manual steps, never proving it was withheld until completion. W63 requires the failing owner’s chain to complete through audio before propagation ([W63](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:31)). The test is red against the old loss behavior, but does not cover the full ordering contract.

- **BLOCKING — round 8 removes the held BlendDirect clone/remount proof.** W73 explicitly held that both Blend1D and BlendDirect executed clone/remount ([W73](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/W73-flb-round7.md:29)), matching FL-B’s requirement that blend occurrences retain their exact definition handles across clone/remount ([FL-B](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-b-spec.md:228)). The rewritten table performs same-owner advances for both cases with no clone/remount ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/tests/cpp_probe.rs:20271)); the sole replacement unit is Blend1D-specific ([state_machine.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:1926)). No BlendDirect clone/remount test remains. That weakens a held proof despite W63’s “never weaken a test” rule ([W63](/Users/levi/dev/worktrees/nuxie-e6-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:90)).

## Verified closures and held items

Blend1D production now contains only its occurrence vector, `from`/`to`, and constructor-time reset ([state_machine.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:820)); apply performs reset then blend ([state_machine.rs](/Users/levi/dev/worktrees/nuxie-e6-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)), matching pinned C++ construction and application ([C++ constructor](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:10), [C++ apply](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:67)). No `last_applied_*` or other test-compensation state remains. Its differential is genuinely symmetric, and its Rust-only clone/remount test is separately labeled.

All other W71-held behavior remains intact: callback-major overshoot handling, O1/O2/O3, typed audio selection, retained arenas, reset ordering, and the seven historical FL-B correction differentials. The stale publication pointer and nonzero-time explanation are also corrected. `git diff --check 171b5703..99ef7700` passes and the checkout is clean. Per the dated coordinator policy, floor7’s older candidate stamp is not a finding.

Standards: 2 blocking. Spec: 2 blocking.

REJECT