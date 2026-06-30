# State Machine Artboard Component Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the component pair condition
slice. It adds the next self-contained `TransitionViewModelCondition`
comparand shape: an artboard property on the left compared with a numeric
component property on the right.

## Formal Goal

Implement C++-compatible `TransitionPropertyArtboardComparator` versus
`TransitionPropertyComponentComparator` numeric comparisons in `rive-runtime`.

The goal is complete when the runtime slice can:

- Recognize the C++-supported shape where
  `TransitionPropertyArtboardComparator` is the left comparand and
  `TransitionPropertyComponentComparator` is the right comparand.
- Treat artboard `width`, `height`, and `ratio` as number/double comparands.
- Classify the right component property with the same CoreRegistry field-kind
  rules already used by the component slices, admitting `NumberDouble` and
  `NumberFromUint` only.
- Compare the artboard number against the component number with the authored
  transition operation.
- Read mutable transform values from the current `ArtboardInstance` for
  component double transform properties.
- Preserve C++ missing/unsupported target behavior by using the component
  field kind's default value before applying the operation.
- Match C++ probe behavior for static component values, mutable transform
  values, missing component targets, unsupported component properties, and
  integer-like component numeric properties.

## Scope Lock

This slice owns only:

- left `TransitionPropertyArtboardComparator`;
- right `TransitionPropertyComponentComparator`;
- numeric-compatible right component properties.

It does not implement:

- component-left/artboard-right comparisons;
- artboard comparisons against view-model comparands;
- component-vs-view-model comparisons;
- component view-model pointer comparisons;
- runtime-layout-driven artboard dimensions;
- live data-binding updates, observer queues, converters, or source/target
  propagation;
- layout solving, rendering, hit testing, listener dispatch, input dispatch,
  scripting, nested state-machine forwarding, or cloning.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the left comparator `TransitionPropertyArtboardComparator`?
2. Is the right comparator `TransitionPropertyComponentComparator`?
3. Does the right component property classify as `NumberDouble` or
   `NumberFromUint`?
4. Can the right side be represented with existing mutable transform state or
   imported source-object stored values, falling back to C++ defaults when the
   target/property is missing or unsupported?

If not, defer it to a later component/ViewModel, artboard/ViewModel,
data-bind, layout, or runtime API slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_artboard_component_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
