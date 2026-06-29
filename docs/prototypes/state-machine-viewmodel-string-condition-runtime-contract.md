# State Machine ViewModel String Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the color
`TransitionViewModelCondition` slice. It closes the narrow string comparison
shape that C++ supports for view-model bindable properties.

## Formal Goal

Implement `BindablePropertyString` transition conditions in `rive-runtime`:
when the left comparator is a `TransitionPropertyViewModelComparator` resolved
to `BindablePropertyString` and the right comparator is
`TransitionValueStringComparator`, read the per-instance string payload and
compare it with C++'s string operation behavior.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyString` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyString.propertyValue` default as bytes.
- Mutate that per-instance string value through the data-bind-index testing
  seam.
- Evaluate string bindables against `TransitionValueStringComparator.value`.
- Match C++ operation behavior for string comparisons: equality and inequality
  compare values, while ordered operations evaluate false.
- Compare no-context, static-equal, mutated-equal, and static-not-equal cases
  against the C++ probe.

## Scope Lock

This slice owns only string bindables used as
`TransitionViewModelCondition` comparands.

It does not implement:

- enum, asset, view-model, trigger, or general id comparands;
- component string comparands;
- string data converters, live data-context APIs, or data-bind queues;
- source/target observers, converters, delegation suppression, dirt
  propagation, or push/poll scheduling;
- nested artboard contexts or nested animation/state-machine forwarding;
- listener-owned view-model dispatch, keyframe callbacks, rendering, scripting,
  audio, or open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is a
   `BindablePropertyString` view-model property and whose right comparator is a
   string literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-string-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_viewmodel_string_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
