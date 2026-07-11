# State Machine ViewModel Trigger Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the view-model pointer
`TransitionViewModelCondition` slice. It closes the narrow trigger/self
condition path that C++ implements through `ConditionComparisonSelf`, without
implementing the full data-binding scheduler.

## Formal Goal

Implement `BindablePropertyTrigger` transition conditions in `nuxie-runtime`
for the case where the left side is a
`TransitionPropertyViewModelComparator` resolved to `BindablePropertyTrigger`
and the right side is either `TransitionValueTriggerComparator` or
`TransitionSelfComparator`.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyTrigger` data-bind targets into
  per-instance runtime state.
- Resolve a trigger bindable to a default view-model `ViewModelInstanceTrigger`
  source when its target data bind is a `DataBindContext` whose raw
  `sourcePathIds` resolve against the file's default view-model instance.
- Track the C++ source facts needed by `ConditionComparisonSelf`: source
  changed, and source already used in the current state-machine layer.
- Mark the trigger source as used when a transition consuming the condition is
  taken.
- Clear source changed/used-layer state when the existing explicit data-context
  advance seam runs.
- Match C++ behavior for no-context, `TransitionValueTriggerComparator`,
  `TransitionSelfComparator`, and same-layer double-consumption suppression
  cases through the C++ probe.

## Scope Lock

This slice owns only default-context view-model trigger sources used by
`TransitionViewModelCondition`.

It does not implement:

- generic `TransitionSelfComparator` behavior for non-trigger bindables;
- component trigger comparands;
- trigger-value integer comparisons;
- relative, name-based, manifest-backed, parent, nested, or multi-view-model
  context lookup beyond the existing raw default-instance path helper;
- data-bind update queues, converter execution, target/source push-poll
  scheduling, observer registration, dirt propagation, or `DataBind::update`;
- public view-model runtime APIs for firing arbitrary trigger properties;
- listener-owned view-model dispatch, `ListenerViewModelChange`, scripting,
  rendering, hit testing, keyboard/pointer/gamepad dispatch, or nested
  state-machine forwarding.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is
   a `BindablePropertyTrigger` view-model property and whose right comparator
   is `TransitionValueTriggerComparator` or `TransitionSelfComparator`?
2. Does the bindable's target data bind resolve to an imported
   `ViewModelInstanceTrigger` on the file's default view-model instance?
3. Can source changed/used-layer behavior be driven by existing
   `StateMachineFireTrigger` actions and compared directly against the C++
   probe?

If not, defer it to a later data-binding, component-comparand, listener, or
runtime view-model API slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_trigger_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
