# State Machine Component Literal Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the ViewModel
`TransitionViewModelCondition` family. It starts the component-comparand work
with the smallest runtime-evaluable C++ shape: a component property on the left
and a literal value comparator on the right.

## Formal Goal

Implement `TransitionPropertyComponentComparator` transition conditions in
`nuxie-runtime` when the left side resolves to an artboard-local component
property and the right side is a literal value comparator.

The goal is complete when the runtime slice can:

- Resolve `TransitionPropertyComponentComparator.objectId` through the
  instantiated artboard's local slots, matching C++ `Artboard::resolve`.
- Match C++ missing/unsupported target behavior: if the resolved object does
  not exist or `CoreRegistry::objectSupportsProperty(target, propertyKey)` would
  fail, the component comparand returns the field kind's default value and the
  normal comparison operation still runs.
- Classify the component property using the C++ CoreRegistry field kind:
  double, bool, string/bytes, color, or uint.
- Evaluate component double properties, including mutable transform properties,
  against `TransitionValueNumberComparator`.
- Evaluate component bool, string, and color properties against their matching
  literal comparator types.
- Evaluate generic component uint properties against
  `TransitionValueNumberComparator` as a numeric comparison, matching C++'s
  `NumberFromUint` path.
- Evaluate special component uint `propertyValue` keys for enum, trigger,
  asset, and artboard values only against their matching literal comparator
  type with exact equal/not-equal semantics.
- Match C++ probe behavior for supported, unsupported, and missing component
  target/property cases, including default-value comparisons.

## Scope Lock

This slice owns only `TransitionPropertyComponentComparator` on the left and a
literal value comparator on the right.

It does not implement:

- component-vs-component comparisons;
- component-vs-view-model comparisons;
- artboard-vs-component or component-vs-artboard comparisons;
- `TransitionSelfComparator` behavior for components;
- component view-model pointer comparisons;
- live data-binding updates, observer queues, converters, or source/target
  propagation;
- layout solving, runtime layout dimensions, render output, hit testing, input
  dispatch, listener dispatch, scripting, nested state-machine forwarding, or
  cloning.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the left comparator a `TransitionPropertyComponentComparator`?
2. Is the right comparator a literal `TransitionValue*Comparator`?
3. Can the left target be resolved through the current artboard instance, and
   can the property value be read from existing mutable runtime state or the
   imported source object's stored value?
4. Does the comparison shape match C++'s field-kind compatibility rules?

If not, defer it to a later component-cross-comparison, ViewModel binding,
layout, or runtime API slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_component_literal_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
