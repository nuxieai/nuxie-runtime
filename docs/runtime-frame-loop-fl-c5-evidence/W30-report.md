Implemented all eight FL-C5 round-one fixes against pinned C++:

- Failure 1: generic states now expose `LayerState` core type 60 in [state_machine.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine.rs:437), matching [layer_state_base.hpp](/Users/levi/dev/oss/rive-runtime/include/rive/generated/animation/layer_state_base.hpp:13).
- Failures 2 and 8: trigger-reset dirt refreshes bindings without rearming `needsAdvance` in [data_bind_graph.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/data_bind_graph.rs:6741) and [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:11581), matching [viewmodel_instance_trigger.cpp](/Users/levi/dev/oss/rive-runtime/src/viewmodel/viewmodel_instance_trigger.cpp:22) and [state_machine_instance.cpp](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:2546). This also fixes `sound`.
- Failures 3–7: root Artboard layout bounds are initialized for imports and clones in [artboard.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/artboard.rs:2569), while Shape hits retain the ancestor/clipping chain in [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5649). This matches [artboard.cpp](/Users/levi/dev/oss/rive-runtime/src/artboard.cpp:264), [shape.cpp](/Users/levi/dev/oss/rive-runtime/src/shapes/shape.cpp:241), and [component.cpp](/Users/levi/dev/oss/rive-runtime/src/component.cpp:97).

Acceptance receipts:

- C++ probes: 804/804
- `nuxie --lib`: 146/146
- Runtime library: 713/713
- Scripting library: 205/205
- `sound`: exact 1/1
- Frame-loop port check: 56/56 plus ownership/ratchet check green
- Formatting and `git diff --check`: green
- Final spec and standards reviews: no findings

The source-bound [runtime-frame-loop-trace.json](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-trace.json) was refreshed. No tests, goldens, or closure checklists changed. No commit was created; `HEAD` remains `f9809441`.

Goal run completed in about 38 minutes.