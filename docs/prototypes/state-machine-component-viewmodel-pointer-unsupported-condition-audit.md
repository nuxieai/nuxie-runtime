# State Machine Component ViewModel Pointer Unsupported Audit

Date: 2026-06-29

This document closes the ambiguous roadmap item for component/ViewModel pointer
`TransitionViewModelCondition` comparisons.

## Finding

C++ currently does not construct component-side ViewModel pointer comparands.

The C++ kind classifier can describe a
`TransitionPropertyComponentComparator` whose property key is
`ViewModelInstanceViewModel.propertyValue` as `ComponentComparandKind::ViewModel`.
However, the `ComparisonShape::ViewModel` branch in `makeComparand()` only
constructs `ConditionComparandViewModelBindable` for
`TransitionPropertyViewModelComparator`. It does not construct the declared
`ConditionComparandComponentViewModel` class for component comparators.

As a result, component/ViewModel pointer pairs build `ConditionComparisonNone`
and evaluate false in both directions.

## Port Decision

Rust should preserve the current C++ rejection behavior for:

- left `TransitionPropertyComponentComparator`, right
  `TransitionPropertyViewModelComparator`;
- left `TransitionPropertyViewModelComparator`, right
  `TransitionPropertyComponentComparator`;
- compatible ViewModel pointer property keys.

The already-supported ViewModel-vs-ViewModel pointer condition remains the only
runtime pointer comparison shape in this area.

## Scope Lock

This audit does not implement new runtime behavior.

It does not add:

- component-side ViewModel pointer comparands;
- live ViewModel reference mutation or data-binding propagation;
- component/ViewModel trigger or artboard comparisons;
- runtime-layout-driven artboard dimensions;
- layout solving, rendering, listener dispatch, input dispatch, scripting,
  nested state-machine forwarding, or cloning.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_component_viewmodel_pointer_unsupported_matches_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
