# State Machine Runtime Artboard Dimension Comparand Contract

## Scope

This slice covers `TransitionPropertyArtboardComparator` inside
`TransitionViewModelCondition` for width, height, and ratio comparisons.

The Rust runtime must:

- Store imported artboard width and height on `ArtboardInstance`.
- Evaluate artboard property comparands from the current `ArtboardInstance`
  dimensions, not from a value frozen while building `RuntimeStateMachine`.
- Apply the same live dimension read path to artboard-vs-literal and
  artboard-vs-component numeric comparisons.
- Match C++ `ConditionComparandArtboardProperty`, where width, height, and
  ratio read `Artboard::layoutWidth()` / `Artboard::layoutHeight()`.
- Preserve existing operation semantics, including the existing ratio behavior
  for zero height.

## Out Of Scope

This slice does not implement Yoga layout, intrinsic/hug sizing,
`Artboard::calculateLayout`, layout dirt propagation, nested artboard layout,
component-list layout, data-binding-driven size changes, or renderer viewport
behavior.

The only mutation API admitted here is a narrow artboard-dimension setter used
to exercise the live comparand path and model the post-layout dimensions that
C++ `layoutWidth()` / `layoutHeight()` return. The C++ probe sets the test
layout rectangle directly; Rust does not solve that layout.

## Completion Checks

- C++ probe-backed coverage proves an artboard comparator observes a size
  mutation made after the runtime artboard instance exists and before condition
  evaluation. The Rust side additionally mutates after constructing the
  `StateMachineInstance` to guard against freezing the value there.
- Existing static width, height, ratio, and artboard-vs-component C++ probe
  cases continue to pass.
- `docs/porting-map.md` records that runtime artboard dimension reads are closed
  for no-layout builds, while layout solving remains future work.
