REJECT. At frozen Rust commit `dc96e571`, I found four blocking family-ownership or behavioral mismatches. The production candidate does not faithfully represent the complete pinned C++ family.

## Blocking findings

1. **BLOCKING — Deferred script initialization suppresses unrelated public behavior**

C++ completes scripted-object cloning, DataContext assignment, initialization/hydration, facility collection, hit sorting, and focus-tree construction synchronously inside the constructor before returning ([C++ constructor](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2072), [final phases](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2121)).

Rust publicly returns the instance immediately ([artboard.rs](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/artboard.rs:4508)), then uses `scripted_data_context_prepare_pending` ([scripted_object_lifecycle.rs](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/scripted_object_lifecycle.rs:67)) to short-circuit both `update_listeners` before reset/prepare/process ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5863)) and raw advance before C++ bookkeeping ([state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12101)).

Concrete exposing input: a machine with one ordinary Down listener on a hittable shape plus any fixed scripted object. Construct through public `artboard.state_machine_instance(0)` and immediately call `pointer_down` or `advance_and_apply`. C++ has finished script initialization and processes the listener/frame; Rust returns `None`/`false` and suppresses unrelated work.

The focused tests conceal this difference by manually marking script initialization complete ([test helper](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:13752)). This contradicts the closure’s constructor and hit/advance claims ([closure](/Users/levi/dev/worktrees/nuxie-flc5-review/docs/runtime-frame-loop-fl-c5-closure.md:473)).

2. **BLOCKING — Typed named-input lookup is absent**

C++ `getNamedInput` requires both the requested input type and exact name, and `getBool`, `getNumber`, and `getTrigger` expose that behavior ([C++ implementation](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2690), [typed methods](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2703)).

Rust only provides `input_named`, which returns the first non-null matching name regardless of type ([Rust implementation](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5043)). There are no corresponding typed lookup methods.

Concrete exposing input: authored inputs `Number("x")` followed by `Bool("x")`. C++ `getBool("x")` returns the Bool occurrence. Rust `input_named("x")` returns the Number, making the claimed typed-first behavior impossible through the represented API.

The closure nevertheless marks `getNamedInput` and all three typed methods closed ([closure](/Users/levi/dev/worktrees/nuxie-flc5-review/docs/runtime-frame-loop-fl-c5-closure.md:357)).

3. **BLOCKING — `fireSemanticAction` discards family-owned dispatch**

C++ performs the manager/node/data lookup and then dispatches tap, increase, or decrease based on the requested action ([C++ implementation](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2509)).

Rust checks for a semantic manager, discards both arguments, and always returns `false` ([Rust implementation](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5276)). The closure’s recorded seam permits manager/tree/node internals to remain external, but explicitly assigns dispatch orchestration to FL-C5 ([directives](/Users/levi/dev/worktrees/nuxie-flc5-review/docs/runtime-frame-loop-fl-c5-walk/directives.md:27)).

Concrete exposing input: an enabled semantic manager with node `77`, valid SemanticData, and action `tap`. C++ calls the node’s tap action; Rust produces no callback.

The focused test blesses the no-op ([test](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:19233)), while the negative ratchet merely accepts an empty `fire_semantic_action` function shape ([ratchet](/Users/levi/dev/worktrees/nuxie-flc5-review/tools/runtime-frame-loop-port/test_check.py:3380)). The cited proof therefore proves the wrong contract.

4. **BLOCKING — Audio-event selection and seam invocation are missing**

C++ locally dispatches, bubbles synchronously, then selects only `AudioEvent` reports and calls `play()` ([C++ implementation](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3155)).

Rust invokes `reach_recorded_audio_event_seam` once for every nonempty event batch ([call site](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7165)); that function only records a phase marker and performs neither AudioEvent selection nor an audio-specific invocation ([seam](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7198)).

Deferring `AudioEvent::play` internals is legitimate. Deferring the type selection and decision to invoke it is not: those are owned by this C++ family file.

Concrete exposing input: one ordinary Event followed by one AudioEvent. C++ plays only the AudioEvent after bubbling. Rust treats both identically and plays/forwards neither. This contradicts the closure’s `playsAudio == true` and listener→bubble→audio-seam claim ([closure](/Users/levi/dev/worktrees/nuxie-flc5-review/docs/runtime-frame-loop-fl-c5-closure.md:671)).

## Twelve-row adversarial audit

| Row | Result |
|---|---|
| Definition import and collection ownership | **NON-BLOCKING proof gap:** live fixture covers three ordinary inputs, not the claimed null holes plus duplicate binds/scripts. |
| Occurrence construction order | **BLOCKING:** deferred constructor gate above. The cited trace checks nine coarse markers, not all twelve phases and failure boundaries. |
| Ordered duplicates and nullable slots | **NON-BLOCKING proof gap:** hardest cross-collection duplicate/null fixture is absent. |
| Transition search and state change | Verified, including weighted boundaries, wrap, NaN, interruption, and convergence. |
| Hit listener and focus ownership | **BLOCKING:** constructor gate suppresses the three passes; semantic dispatch is absent. The behavioral three-pass test also clears groups beforehand, so reset is only structurally checked. |
| DataContext bind/rebind/clear | Implementation branches appear faithful; **NON-BLOCKING proof gap:** live comparison does not exercise the full null matrix claimed. |
| Event application and chained reports | Cursor, event-before-VM, exactly-100 batches, batch-101 retention, and nested-relative boundary verified. **BLOCKING:** audio tail above. |
| Zero-second and floating-point edges | Verified. |
| Advance return and pending work | Raw/facade terms and five unconditional settlement probes verified, except the blocking pre-bookkeeping script gate. |
| Keyframe DataBind lifecycle | Holder-before-clone and enrollment structure match; **NON-BLOCKING proof gap:** initialization→converter→enrollment order is not observed end-to-end. |
| Clone remount and teardown isolation | **NON-BLOCKING proof gap:** cited test omits several claimed mutable families, including reporting queues, hit groups, bind graphs, contexts, and callback sinks. |
| Direct C++ file correspondence | **BLOCKING:** typed inputs, semantic dispatch, and audio selection. Also omits an explicit closure row for `unbind`. |

## Other non-blocking closure defects

- `StateMachineInstance::unbind` is present in the C++ inventory ([C++](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2949)) and correctly represented in Rust ([Rust](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:10389)), but has no member row in the purported complete closure.
- `sortHitComponents` itself matches the C++ swap-derived ordering ([C++](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2255), [Rust](/Users/levi/dev/worktrees/nuxie-flc5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:3129)). However, the cited proofs do not behaviorally exercise the exact adversarial swap order.
- The main, ungated `updateListeners` path preserves reset→prepare→process, strongest `HitResult`, opacity propagation, and cleanup semantics. The rejection is caused by the observable early gate, not the core tri-state algorithm.
- The archived `.flc5/out` receipts cited by the closure are absent from this checkout, and no `RIVE_CPP_PROBE` executable is configured. The test/probe sources can be inspected, but the claimed frozen live-run evidence cannot be independently reproduced from the packet as supplied.

REJECT