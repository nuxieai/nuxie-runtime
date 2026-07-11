# State Machine ViewModel Color Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the integer-as-number
`TransitionViewModelCondition` slice. It closes the narrow color comparison
shape that C++ supports for view-model bindable properties.

## Formal Goal

Implement `BindablePropertyColor` transition conditions in `nuxie-runtime`: when
the left comparator is a `TransitionPropertyViewModelComparator` resolved to
`BindablePropertyColor` and the right comparator is
`TransitionValueColorComparator`, read the per-instance color value and compare
it with C++'s color operation behavior.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyColor` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyColor.propertyValue` default.
- Mutate that per-instance color value through the data-bind-index testing
  seam.
- Evaluate color bindables against `TransitionValueColorComparator.value`.
- Match C++ operation behavior for color comparisons: equality and inequality
  compare values, while ordered operations evaluate false.
- Compare no-context, static-equal, mutated-equal, and static-not-equal cases
  against the C++ probe.

## Scope Lock

This slice owns only color bindables used as
`TransitionViewModelCondition` comparands.

It does not implement:

- string, enum, asset, view-model, trigger, or general id comparands;
- component color comparands;
- color data converters, live data-context APIs, or data-bind queues;
- source/target observers, converters, delegation suppression, dirt
  propagation, or push/poll scheduling;
- nested artboard contexts or nested animation/state-machine forwarding;
- listener-owned view-model dispatch, keyframe callbacks, rendering, scripting,
  audio, or open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is a
   `BindablePropertyColor` view-model property and whose right comparator is a
   color literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-color-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_color_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
