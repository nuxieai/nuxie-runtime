# Data Binding Graph List To Length Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterListToLength` on main-`ToTarget | TwoWay` list-to-number binds.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited number target back to the
list source. The manual target edit survives explicit
`advancedDataContext()`, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion through `DataConverterListToLength`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Imported `ViewModelInstanceList` sources and their attached imported
  `ViewModelInstanceListItem` children.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterListToLength` on a `TwoWay` number data bind without
  the `ToSource` direction flag.
- Initial list-length flushing through a normal state-machine advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- List targets and `BindablePropertyList` behavior.
- List mutation APIs and update-queue propagation.
- `DataConverterNumberToList` and generated runtime list items.
- Main-`ToSource | TwoWay` target-to-source behavior for
  `DataConverterListToLength`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Owned runtime view-model list contexts.
- Relative-path, parent-path, nested-path, listener-owned behavior, nested
  artboards, and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the imported list length to
  the bindable number target.
- Mutating the `DataConverterListToLength` target on a main-`ToTarget | TwoWay`
  bind preserves the manual number value through explicit data-context
  advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged imported list source length.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
