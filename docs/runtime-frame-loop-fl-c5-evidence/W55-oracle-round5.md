REJECT. Round 5 fixes separate nested-artboard siblings, but the policy boundary is still too coarse to match C++ for every nested event source.

## BLOCKING

1. Per-source chain atomicity fails when one `NestedArtboard` owns multiple reporting animations/state machines.

Rust invokes `advance_nested_event_source_with` once per `AdvancingComponentKind::NestedArtboard` ([artboard.rs:5913](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:5913)). Inside that component, it advances every animation owner into one shared report vector ([artboard.rs:10169](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10169), [state_machine_instance.rs:12628](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12628)). Ancestor dispatch and source-audio flush happen only after the entire component returns ([state_machine_instance.rs:12585](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12585), [state_machine_instance.rs:12599](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12599)).

A host may legally contain multiple `NestedStateMachine` animations; the importer appends every matching child ([artboard.rs:10714](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10714)), and existing tests already construct three under one host ([artboard.rs:11872](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:11872)).

Pinned C++ instead loops those animations individually ([nested_artboard.cpp:989](/Users/levi/dev/oss/rive-runtime/src/nested_artboard.cpp:989)). Each `NestedStateMachine::advance` synchronously enters its state-machine advance ([nested_state_machine.cpp:16](/Users/levi/dev/oss/rive-runtime/src/animation/nested_state_machine.cpp:16)), which applies events before returning ([state_machine_instance.cpp:2555](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2555)); notification then recursively bubbles and completes audio unwind in the same call ([state_machine_instance.cpp:3155](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3155)).

For two reporters A/B on one host, Rust can therefore produce:

`A-local, B-local, ancestor-[A,B], ancestor-audio-[A,B], A-audio, B-audio`

instead of C++:

`A-local, ancestor-A, ancestor-A-audio, A-audio, B-local, ancestor-B, ancestor-B-audio, B-audio`.

The same granularity reopens the error path. Rust advances the host’s animation owners first, then its child subtree ([artboard.rs:10169](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10169), [artboard.rs:10200](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10200)). If a later scripted child fails, the error branch flushes source audio but never dispatches the already-collected reports to ancestors ([state_machine_instance.rs:12587](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12587)). C++ had already completed ancestor notification and every audio tail before advancing that later work.

The round-5 unit and live fixtures miss this shape: both create two separate nested-artboard hosts, each with one state machine ([artboard.rs:11635](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:11635), [cpp_probe.rs:2198](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/tests/cpp_probe.rs:2198)).

2. Nested linear-animation event notifiers still drop their reports.

C++ registers both nested state machines and nested linear animations as event notifiers ([state_machine_instance.cpp:2025](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2025)). `NestedSimpleAnimation` advances with the animation instance as its keyed-callback reporter ([nested_simple_animation.cpp:13](/Users/levi/dev/oss/rive-runtime/src/animation/nested_simple_animation.cpp:13)), and `LinearAnimationInstance::reportEvent` synchronously notifies its parent listeners ([linear_animation_instance.cpp:442](/Users/levi/dev/oss/rive-runtime/src/animation/linear_animation_instance.cpp:442)).

Rust records `LinearAnimation` registrations ([state_machine_instance.rs:3224](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:3224)), but `RuntimeNestedAnimationInstance::Simple` uses the eventless advance facade ([artboard.rs:10443](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/artboard.rs:10443)). That facade calls `LinearAnimationInstance::advance`, which explicitly passes no report or keyed-callback destinations ([animation.rs:2277](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/animation.rs:2277)). Consequently nested timeline Event/AudioEvent callbacks never reach the registered parent state machine. This is not introduced by the round-5 delta, but it prevents confirming that the previously accepted O4 family remained complete.

## NON-BLOCKING

- The live atomicity differential does not directly observe C++ audio chronology. It observes local/root report names, then constructs the expected audio steps by appending `"-audio"` ([cpp_probe.rs:84068](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/tests/cpp_probe.rs:84068)). Source inspection establishes the C++ order, but the claimed “live differential” is weaker than the prose suggests.

- Publication status is stale after E3 publication: it still instructs publishing the staged E3 packet ([runtime-frame-loop-status.md:831](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-status.md:831)). This is the same class of publication-pointer inconsistency flagged in round 4.

## Verified held

- O1/O2/O3 remain intact. Ordinary behavior is still independent of unavailable scripted preparation; typed lookup filters by type and name in authored order ([state_machine_instance.rs:5205](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5205)); semantic dispatch uses the resolver rather than ordinal projection ([state_machine_instance.rs:5463](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5463)).

- Audio selection itself correctly filters only typed AudioEvents and invokes the seam once per occurrence ([state_machine_instance.rs:7402](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7402)).

- The retained-definition correction now covers transition timing, exit-time hold/apply, reset construction, and reconstructed states ([state_machine_layer_instance.rs:693](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:693), [state_machine_layer_instance.rs:733](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733), [animation_reset_factory.rs:102](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/src/state_machine/animation_reset_factory.rs:102)). The wrong-artboard differential exercises the public state-machine path ([cpp_probe.rs:19914](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/tests/cpp_probe.rs:19914)).

- No regression was found in the previously accepted FL-B set: signed loop overrides, invalid-interpolator erasure, importer-cursor survival, doomed-object sink, negative-speed remap, NaN direct blend, and empty-baseline reset. Their differentials remain present from [cpp_probe.rs:19331](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/tests/cpp_probe.rs:19331) through [cpp_probe.rs:24264](/Users/levi/dev/worktrees/nuxie-e3-review/crates/nuxie-runtime/tests/cpp_probe.rs:24264). The live checker also accepts the frozen FL-B scope.

- Round-4 structural items are closed: repository-wide semantic scanning ([check.py:1347](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/check.py:1347)), recursive Git-based receipt enumeration ([stamp_floor_receipt.py:19](/Users/levi/dev/worktrees/nuxie-e3-review/tools/runtime-frame-loop-port/stamp_floor_receipt.py:19)), 23/23 tracked receipt stamps valid, tools-enabled commands, 67-test prose, dated scope authorization, and candidate/runner provenance at `691c5262` ([runtime-frame-loop-trace.json:7810](/Users/levi/dev/worktrees/nuxie-e3-review/docs/runtime-frame-loop-trace.json:7810)).

Focused prebuilt-candidate tests for separate-host atomicity, later root-level ScriptError audio, two-ancestor ordering, O1/O2/O3, and loop semantics passed. The live checker and formatting/diff checks passed. Full Cargo/differential replay and `codex review --commit 691c5262` were blocked solely by the frozen environment’s prohibition on temporary files and lock creation.

REJECT