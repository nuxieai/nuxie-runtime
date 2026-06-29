# State Machine Artboard Comparand Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the fire-trigger action
slice. It closes a narrow `TransitionViewModelCondition` gap: numeric
conditions whose authored comparand reads a property from the current artboard.

## Formal Goal

Implement the narrow `TransitionPropertyArtboardComparator` runtime behavior in
`rive-runtime`: build `TransitionViewModelCondition` objects whose left
comparator is an artboard property and whose right comparator is a numeric
value, read the imported artboard width, height, or ratio, and evaluate the
condition with the same `StateMachineOp` comparison rules as C++.

The goal is complete when the runtime slice can:

- Import `TransitionPropertyArtboardComparator.propertyType` for state-machine
  transition conditions.
- Support C++ `ArtboardProperty` values `width = 0`, `height = 1`, and
  `ratio = 2`.
- Compare the selected artboard value against a
  `TransitionValueNumberComparator.value`.
- Use the existing transition-condition count gate, so unsupported condition
  shapes still prevent the transition from running.
- Compare width, height, ratio, and non-matching thresholds against the C++
  probe.

## Scope Lock

This slice owns only static artboard numeric comparands evaluated during
state-machine transition checks.

It does not implement:

- layout solving or runtime mutation of `layoutWidth()` / `layoutHeight()`;
- component comparands;
- non-number artboard comparands;
- view-model comparands beyond the existing number-only slice;
- data-context binding, data-bind queues, converters, observers, or
  source/target invalidation;
- nested artboard contexts or nested animation/state-machine forwarding;
- hit testing, pointer/keyboard/gamepad inputs, rendering, keyframe callbacks,
  scripting, or listener dispatch.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` with a
   `TransitionPropertyArtboardComparator` and a
   `TransitionValueNumberComparator`?
2. Can the compared value be computed from imported artboard width/height
   without running layout?
3. Can the behavior be compared through the existing C++ state-machine probe?

If not, defer it to a later layout, component-comparand, data-binding, nested
artboard, or input/runtime slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_artboard_comparand_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
