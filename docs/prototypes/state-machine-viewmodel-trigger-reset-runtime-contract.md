# State Machine ViewModel Trigger Reset Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the fire-trigger action
slice. It closes the reset half of the same trigger lifecycle:
`StateMachineFireTrigger` increments a bound `ViewModelInstanceTrigger`, and the
explicit data-context advance step resets that trigger value.

## Formal Goal

Implement the narrow `ViewModelInstanceTrigger::advanced()` behavior exposed
through C++ `StateMachineInstance::advancedDataContext()` in `rive-runtime`:
when a state machine is bound to the minimal authored default view-model data
context, advancing that context resets imported trigger counts to zero.

The goal is complete when the runtime slice can:

- Expose a Rust state-machine instance method matching the explicit C++
  `advancedDataContext()` seam.
- Reset bound `ViewModelInstanceTrigger.propertyValue` counts to zero.
- Leave ordinary state-machine input trigger reset behavior unchanged.
- Avoid resetting trigger values during ordinary `StateMachineInstance::advance`
  unless the new explicit data-context-advance seam is called.
- Compare the increment-before-reset and zero-after-reset values against the
  C++ probe.

## Scope Lock

This slice owns only explicit data-context advancement for the minimal default
view-model context already supported by the fire-trigger slice.

It does not implement:

- full live `DataContext` ownership or public view-model APIs;
- data-bind queues, observers, converters, delegation suppression, dirt
  propagation, or source/target invalidation;
- nested view-model values, parent contexts, or relative data-bind paths;
- listener-owned view-model dispatch or `ListenerViewModelChange`;
- trigger comparator conditions;
- nested artboard contexts, rendering, scripting, keyframe callbacks, audio, or
  open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it part of the explicit C++ `StateMachineInstance::advancedDataContext()`
   path?
2. Does it only reset imported trigger counts on the currently bound default
   view-model context?
3. Can the before/after trigger values be compared through the C++ probe?

If not, defer it to a later data-context, data-binding, listener, nested
artboard, or runtime-behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_viewmodel_trigger_reset_matches_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
