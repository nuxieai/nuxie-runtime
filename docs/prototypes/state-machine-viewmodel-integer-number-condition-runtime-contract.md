# State Machine ViewModel Integer Number Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the boolean
`TransitionViewModelCondition` slice. It closes the remaining numeric
view-model condition shape that C++ already handles: integer bindable
properties compared through numeric transition comparators.

## Formal Goal

Implement the narrow integer-as-number `TransitionViewModelCondition` runtime
behavior in `nuxie-runtime`: when the left comparator is a
`TransitionPropertyViewModelComparator` resolved to `BindablePropertyInteger`
and the right comparator is `TransitionValueNumberComparator`, read the
per-instance integer value, cast it to `f32`, and evaluate it using the same
numeric operation rules as C++.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyInteger` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyInteger.propertyValue` default.
- Mutate that per-instance integer value through the data-bind-index testing
  seam.
- Evaluate integer bindables against `TransitionValueNumberComparator.value`
  using numeric comparison rules.
- Preserve existing `BindablePropertyNumber` blend and condition behavior
  unchanged.
- Compare no-context, static-value, and mutated-value cases against the C++
  probe.

## Scope Lock

This slice owns only integer bindables used as numeric
`TransitionViewModelCondition` comparands.

It does not implement:

- integer bindables as blend-state sources;
- string, color, enum, asset, view-model, trigger, or general uint
  comparands;
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
   `BindablePropertyInteger` view-model property and whose right comparator is
   a numeric literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-numeric-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_integer_number_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
