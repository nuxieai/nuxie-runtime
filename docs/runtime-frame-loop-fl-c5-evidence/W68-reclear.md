Not cleared: findings 1–4 and 6 are closed, but the ownership ratchets remain structurally evadable.

## Standards

One hard violation remains. The scanner only recognizes `Type::member` or `.member` paths ([check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:541)). Valid angle-bracket UFCS returns zero hits:

```rust
let send = <StateMachineInstance>::notify_events;
```

The same bypass works for `reported_event` and `flush_deferred_owner_audio_events`, contrary to W63’s explicit “any call form incl. UFCS” requirement ([W63 spec](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:61)).

## Spec

1. **Closed:** Nested state-machine `applyEvents` batches now bubble before continuation into binds/layers ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12833), [owner loop](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13022)), matching pinned C++ ordering ([state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2555)).

2. **Closed:** Recursive notify now performs only `update_data_binds_false`; the zero-time full advance is gone ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13145)), matching C++ `notifyEventListeners` then `updateDataBinds(false)` ([state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3041)).

3. **Closed:** The deep differential observes live report-time source values at both actual notify seams, compares Rust directly with C++, and separately proves both differ from their matching final mixed value ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:84829)).

4. **Closed:** Rust notify-entry batch sizes are asserted exactly `[1, 1]`; a batched `[2]` implementation fails ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:85084)).

5. **Still open:** All prior probes now hit: function-item aliases, const/static aliases, direct enum-variant aliases, third-file relocation, and literal-free cross-file audio aliases. Recursive globs cover all Rust source files. However, my new enum-type alias evades selection detection:

```rust
use RuntimeNestedAnimationInstance as Anim;
if let Anim::StateMachine(owner) = animation {
    displace(owner);
}
```

It returns zero hits because selection detection recognizes only the literal enum type, `Self`, or direct variant aliases ([check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:602)). Together with the UFCS bypass, three of the four ratchets remain cheaply evadable.

6. **Closed:** Status and trace truthfully label this as pre-E5, non-operative work while retaining E4 as the operative publication ([status](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-status.md:7), [trace](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-trace.json:7834)). The checker’s three stale-evidence failures are therefore expected and honestly documented.

The full Python unittest could not create its temporary directory under the read-only sandbox; direct detector probes, the read-only checker, syntax parsing, and `git diff --check` ran successfully.

**FIX-FIRST**