## WOULD-REJECT

1. Nested state-machine reports still bubble after the source’s remaining layer/mix work.

Rust advances the complete child machine first, then extracts and dispatches its reports ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12746)). Those reports are processed before Rust’s authored layers ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12488)), but are not bubbled until after layers run at line 12529.

Pinned C++ runs `applyEvents()` and recursively notifies ancestors before advancing layers ([state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2555), [recursive notify](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3155)). An ancestor listener reading a property mixed later in the source frame therefore sees pre-mix state in C++ but post-mix state in Rust. This violates W63’s report-time/pre-mix contract.

2. Recursive settlement is too early and too broad.

For an intermediate owner, Rust queues its bubble/audio ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7388)), then executes an entire zero-time `advance_on_artboard` before delivering the bubble ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12869)). That advance includes binds, layers, converters, and input advancement.

C++ orders local listeners → recursive ancestors → current audio, then runs only `updateDataBinds(false)` ([state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3041), [state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3155)). Rust can expose settled bind/layer state to the root too early.

3. The deep differential misses both production defects.

Its C++ side flattens event names across `0.25` and two later `0.0` advances ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:84750)); Rust asserts only synthetic phase markers ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:84797)). There is no bind or source-layer value observed at the ancestor, so neither report-time nor pre-/post-mix visibility is proved.

4. Singleton-batch evidence distinguishes C++ batching, but not Rust batching.

The probe records C++ notify batches ([main.cpp](/Users/levi/dev/worktrees/nuxie-fl-c/tools/cpp-probe/main.cpp:383)), and the test asserts `[1, 1]` only for C++ ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:84894)). Rust uses markers emitted by `complete_nested_report_chain`, downstream of callback invocation ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12802)).

A batched Rust implementation could collect both callbacks, drain them individually before mix, and still produce the asserted two phase triples, zero delays, and final mixed value. The unit pre-mix check also permits this evasion because both observations merely expect the same authored value ([artboard.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/artboard.rs:14294)). W63 explicitly requires batched Rust to be distinguishable.

5. The four “structural” ratchets remain cheaply evadable.

The detector is still spelling/call-shape based ([check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:420)). Direct in-memory probes returned no hits for:

```rust
let deliver = StateMachineInstance::notify_events;
deliver(owner, child, Some(host), &batch);

let finish = StateMachineInstance::flush_deferred_owner_audio_events;
finish(owner);

use RuntimeNestedAnimationInstance::StateMachine as Machine;
if let Machine(owner) = animation { /* displaced policy */ }
```

The negatives cover collection aliases, but not dispatch/audio function aliases or enum aliases; the supposedly renamed audio negative still contains the literal word `audio` ([test_check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_check.py:3612)).

Additionally, all four ratchets scan only `artboard.rs` and `nested_state_machine.rs` ([runtime-frame-loop-gaps.toml](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-gaps.toml:1397)). Relocating identical mechanics to any third non-owner file bypasses them, contrary to W63’s “in non-owner files” requirement.

6. Packet status is false at `f4f013dd`.

The status still identifies E4 candidate `9434b39c…` as Current and claims the checker is green on the exact tree ([runtime-frame-loop-status.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-status.md:7)), while Next requests round-seven review against “this publication” ([runtime-frame-loop-status.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-status.md:832)). The trace likewise remains pinned to `9434b39c…` ([runtime-frame-loop-trace.json](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-trace.json:7834)).

Running the checker at `f4f013dd` fails for stale Rust ref, candidate fingerprint, and runner provenance. The packet needs to say explicitly that round seven is pre-E5, not yet the operative publication.

## NIT

- The tracked W63 spec still links its source verdicts through untracked `.flc5/out/W60…W62` paths ([W63-round7-spec.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:3)). Its requirements are restated, but the archived packet is not fully navigable.

## Verified held

- Both restored parity pairs genuinely use `0.25` on C++ and Rust ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:21471), [cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:83968)).
- Nested-simple production currently zeroes overshoot and invokes the callback chain before mix ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12701)).
- Blend1D clone/remount is enabled ([cpp_probe.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/cpp_probe.rs:20275)).
- No additional orchestration creep was found beyond the allowed authored artboard loop/borrow adapter.

**FIX-FIRST**