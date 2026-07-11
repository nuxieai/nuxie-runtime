# State Machine ViewModel Boolean Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the number-only
`TransitionViewModelCondition` and trigger lifecycle slices. It closes the next
smallest view-model condition shape: boolean bindable properties compared
against authored boolean values.

## Formal Goal

Implement the narrow boolean `TransitionViewModelCondition` runtime behavior in
`nuxie-runtime`: when the left comparator is a
`TransitionPropertyViewModelComparator` resolved to `BindablePropertyBoolean`
and the right comparator is `TransitionValueBooleanComparator`, evaluate the
condition using the same C++ data-context presence gate and equality operation
rules.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyBoolean` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyBoolean.propertyValue` default.
- Mutate that per-instance boolean value through the existing data-bind-index
  testing seam.
- Evaluate boolean `TransitionViewModelCondition` values against
  `TransitionValueBooleanComparator.value`.
- Preserve existing number condition behavior unchanged.
- Compare no-context, static-value, and mutated-value cases against the C++
  probe.

## Scope Lock

This slice owns only boolean view-model transition conditions for the already
audited state-machine data-bind target shape.

It does not implement:

- string, color, enum, uint, trigger, view-model, or asset comparands;
- component comparands;
- live data-context APIs or data-bind queues;
- source/target observers, converters, delegation suppression, dirt
  propagation, or push/poll scheduling;
- nested artboard contexts or nested animation/state-machine forwarding;
- listener-owned view-model dispatch, keyframe callbacks, rendering, scripting,
  audio, or open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is a
   `BindablePropertyBoolean` view-model property and whose right comparator is
   a boolean literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-boolean-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_boolean_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
