# State Machine Component Pair Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the component literal
condition slice. It adds the next self-contained `TransitionViewModelCondition`
component-comparand shape: component property on the left compared with
component property on the right.

## Formal Goal

Implement `TransitionPropertyComponentComparator` pair comparisons in
`rive-runtime` when both sides resolve to component property comparators.

The goal is complete when the runtime slice can:

- Classify each side with the same CoreRegistry field-kind rules used by C++:
  double, generic uint-as-number, bool, string/bytes, color, enum, trigger,
  asset, artboard, and view-model property-value keys.
- Choose a comparison only when C++ `componentKindsCompatible` would find a
  compatible pair.
- Compare numeric component pairs as floats when either side is a double and as
  exact uint values when both sides are generic `uint`/integer-like values.
- Compare bool, string/bytes, color, enum, trigger, asset, and artboard pairs
  with the same operation limits as C++.
- Read mutable transform values from the current `ArtboardInstance` and stored
  values from the imported source object for currently static properties.
- Preserve C++ missing/unsupported target behavior by using the field kind's
  default value for that side before applying the operation.
- Match C++ probe behavior for static, mutable, missing-target, unsupported,
  and incompatible component-pair cases.

## Scope Lock

This slice owns only `TransitionPropertyComponentComparator` on both sides.

It does not implement:

- component-vs-literal comparisons beyond the previous contract;
- component-vs-view-model comparisons;
- artboard-vs-component or component-vs-artboard comparisons;
- component view-model pointer comparisons;
- `TransitionSelfComparator` behavior for components;
- live data-binding updates, observer queues, converters, or source/target
  propagation;
- layout solving, runtime layout dimensions, rendering, hit testing, listener
  dispatch, input dispatch, scripting, nested state-machine forwarding, or
  cloning.

## Admission Rule

Before adding behavior to this slice, answer:

1. Are both comparators `TransitionPropertyComponentComparator`?
2. Do the two property keys classify to compatible C++ component comparand
   kinds?
3. Can each side be represented with existing mutable transform state or
   imported source-object stored values, falling back to C++ defaults when the
   target/property is missing or unsupported?

If not, defer it to a later component/ViewModel, artboard/component, data-bind,
layout, or runtime API slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_component_pair_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
