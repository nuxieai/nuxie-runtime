# State Machine ViewModel Enum Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the string
`TransitionViewModelCondition` slice. It closes the narrow enum comparison
shape that C++ supports for view-model bindable properties.

## Formal Goal

Implement `BindablePropertyEnum` transition conditions in `nuxie-runtime`: when
the left comparator is a `TransitionPropertyViewModelComparator` resolved to
`BindablePropertyEnum` and the right comparator is
`TransitionValueEnumComparator`, read the per-instance enum id and compare it
with C++'s enum operation behavior.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyEnum` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyEnum.propertyValue` default.
- Mutate that per-instance enum value through the data-bind-index testing seam.
- Evaluate enum bindables against `TransitionValueEnumComparator.value`.
- Match C++ operation behavior for enum comparisons: equality and inequality
  compare ids, while ordered operations evaluate false.
- Compare no-context, static-equal, mutated-equal, static-not-equal, and
  ordered-operation cases against the C++ probe.

## Scope Lock

This slice owns only enum bindables used as
`TransitionViewModelCondition` comparands.

It does not implement:

- asset, view-model, trigger, artboard, or general id comparands;
- component enum comparands;
- enum value-name lookup, enum data converters, live data-context APIs, or
  data-bind queues;
- source/target observers, converters, delegation suppression, dirt
  propagation, or push/poll scheduling;
- nested artboard contexts or nested animation/state-machine forwarding;
- listener-owned view-model dispatch, keyframe callbacks, rendering, scripting,
  audio, or open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is a
   `BindablePropertyEnum` view-model property and whose right comparator is an
   enum id literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-enum-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_enum_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
