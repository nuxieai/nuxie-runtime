# State Machine Component Artboard Unsupported Direction Audit

Date: 2026-06-29

This document closes the ambiguous roadmap item for component-left/artboard-right
`TransitionViewModelCondition` comparisons.

## Finding

C++ does not support the reverse direction of the artboard/component comparand
shape.

`TransitionPropertyArtboardComparator` only appends a comparable kind when it is
the left comparand. `TransitionPropertyComponentComparator` can participate on
either side, but when the artboard comparator is on the right there is no right
comparable kind to intersect. C++ therefore builds `ConditionComparisonNone`
for component-left/artboard-right conditions.

## Port Decision

Rust should preserve the C++ rejection behavior for:

- left `TransitionPropertyComponentComparator`;
- right `TransitionPropertyArtboardComparator`.

The existing left-artboard/right-component runtime slice remains the only
supported artboard/component direction.

## Scope Lock

This audit does not implement new runtime behavior.

It does not add:

- mirrored component-left/artboard-right comparisons;
- runtime-layout-driven artboard dimensions;
- component/ViewModel trigger, artboard, or pointer comparisons;
- layout solving, rendering, live data-binding, listener dispatch, input
  dispatch, scripting, nested state-machine forwarding, or cloning.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_component_artboard_unsupported_direction_matches_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
