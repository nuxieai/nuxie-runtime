# State Machine Fire Trigger Runtime Contract

Date: 2026-06-29

Relative path boundary update: 2026-07-01

This document continues roadmap item `#11` after the view-model number
condition slice. It closes the smallest remaining state-machine fire-action
gap: scheduled `StateMachineFireTrigger` actions mutating a trigger property on
the bound data context.

## Formal Goal

Implement the narrow `StateMachineFireTrigger` runtime behavior in
`nuxie-runtime`: carry imported fire-trigger actions from `nuxie-binary`, bind an
authored view-model instance as the state-machine data context, resolve encoded
`viewModelPathIds` against that context, increment matching
`ViewModelInstanceTrigger.propertyValue` on scheduled state/transition
occurrences, and compare trigger counts against the C++ probe.

The goal is complete when the runtime slice can:

- Store `StateMachineFireTrigger` actions beside existing
  `StateMachineFireEvent` actions for states and transitions.
- Preserve existing `StateMachineFireEvent` reported-event behavior unchanged.
- Bind a minimal authored view-model-instance data context for a
  `StateMachineInstance`.
- Resolve absolute `viewModelPathIds` to imported `ViewModelInstanceTrigger`
  values through existing `nuxie-binary` data-context lookup rules.
- Preserve the C++ boundary for claimed relative `DataBindPath` fire-trigger
  actions in this scheduled state-machine slice: the imported action remains
  present, but the trigger source is treated as unresolved and no trigger count
  mutation occurs.
- Increment the target trigger value when the action's `occursValue` matches
  `atStart` or `atEnd`.
- Compare the trigger value after public state-machine advances against the C++
  probe.

## Scope Lock

This slice owns only scheduled `StateMachineFireTrigger::perform` behavior for
the existing state/transition scheduling path.

It does not implement:

- full live `DataContext` ownership or public view-model APIs;
- parent-context fallback;
- firing relative/name-resolved paths in listener-owned or nested-artboard data
  contexts;
- `TransitionViewModelCondition` trigger comparators;
- `ListenerViewModelChange` or listener-owned view-model dispatch;
- data-bind queues, source/target binding, observers, converters, or
  push/poll scheduling;
- `ViewModelInstanceTrigger::advanced()` reset scheduling;
- nested artboard contexts or nested state-machine forwarding;
- scripted actions, keyframe callbacks, rendering, or audio/open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for a scheduled `StateMachineFireTrigger` action to increment
   an imported `ViewModelInstanceTrigger` on the bound data context?
2. Can it be resolved through existing imported view-model instance data and
   compared through the C++ probe?
3. Does it avoid general data-binding lifecycle or listener dispatch behavior?

If not, defer it to a later data-context, listener, data-binding, or nested
artboard slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_fire_trigger -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
