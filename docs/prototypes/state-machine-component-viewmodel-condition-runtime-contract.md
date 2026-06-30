# State Machine Component ViewModel Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the artboard/component
condition slice. It adds the next self-contained `TransitionViewModelCondition`
comparand shape: component scalar properties compared with default-context
ViewModel bindable scalar properties.

## Formal Goal

Implement C++-compatible component/ViewModel scalar comparisons in
`rive-runtime`.

The goal is complete when the runtime slice can:

- Recognize `TransitionPropertyComponentComparator` paired with
  `TransitionPropertyViewModelComparator` in either left/right order.
- Classify the component property with the CoreRegistry field-kind rules
  already used by the component slices.
- Classify the ViewModel comparator from its latest imported
  `BindableProperty` using C++ kind compatibility rules.
- Admit numeric pairs when both sides are number-compatible:
  `NumberDouble` and `NumberFromUint`.
- Compare integer-like `NumberFromUint` pairs as exact uint values and compare
  mixed double/integer-like numeric pairs as floats.
- Compare bool, string/bytes, color, enum, and asset scalar pairs with the same
  operation limits as the existing ViewModel and component comparands.
- Preserve C++ data-context presence gating whenever either side is a
  ViewModel comparator.
- Read mutable transform values from the current `ArtboardInstance` for
  component double transform properties.
- Preserve C++ missing/unsupported component target behavior by using the
  component field kind's default value before applying the operation.
- Match C++ probe behavior for component-left/ViewModel-right and
  ViewModel-left/component-right order, static component values, mutable
  component transform values, mutated ViewModel values, no-context gating,
  missing/unsupported component targets, incompatible kind rejection, and
  integer-like numeric pairs.

## Scope Lock

This slice owns only scalar component/ViewModel comparisons for:

- number and integer-like numeric fields;
- bool;
- string/bytes;
- color;
- enum;
- asset.

It does not implement:

- trigger component/ViewModel comparisons;
- artboard component/ViewModel comparisons;
- ViewModel pointer component comparisons;
- `TransitionSelfComparator` behavior for components;
- live data-binding updates, observer queues, converters, or source/target
  propagation beyond existing explicit bindable mutation helpers;
- runtime-layout-driven artboard dimensions;
- layout solving, rendering, hit testing, listener dispatch, input dispatch,
  scripting, nested state-machine forwarding, or cloning.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is exactly one side `TransitionPropertyComponentComparator`?
2. Is exactly one side `TransitionPropertyViewModelComparator`?
3. Does the ViewModel comparator resolve to a supported bindable scalar kind?
4. Does the component property classify to a compatible supported scalar kind?
5. Can the component side be represented with existing mutable transform state
   or imported source-object stored values, falling back to C++ defaults when
   the target/property is missing or unsupported?

If not, defer it to a later trigger, artboard, pointer, data-bind, layout, or
runtime API slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_component_viewmodel_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
