# Data Binding Graph OperationViewModel Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterOperationViewModel` on main-`ToTarget | TwoWay` number binds.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited target back to the source.
The manual target edit survives explicit `advancedDataContext()`, and the next
normal `StateMachineInstance::advance()` reapplies source-to-target conversion
with the resolved secondary view-model number operand.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Primary root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` primary source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterOperationViewModel` converters whose `sourcePathIds`
  resolve against the imported default root view-model instance.
- Secondary operation sources that resolve to `ViewModelInstanceNumber`.
- `TwoWay` number data binds without the `ToSource` direction flag.
- Initial source-to-target operation-view-model flushing through a normal
  state-machine advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` operation-view-model target-to-source behavior,
  covered by
  `docs/prototypes/data-binding-graph-operation-viewmodel-target-to-source-runtime-contract.md`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Live dependency/dirt propagation when the secondary operation source changes.
- Recomputing the secondary operand for imported or owned context rebinding.
- Missing, non-number, relative-path, parent-path, or nested secondary
  operation sources.
- Dedicated grouped `DataConverterOperationViewModel` parity coverage beyond
  the existing generic group executor.
- Formula, interpolator, number-to-list, and scripted converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the operation-view-model
  output from the unchanged primary source and imported secondary operand to
  the bindable number target.
- Mutating the operation-view-model target on a main-`ToTarget | TwoWay` bind
  preserves the manual value through explicit data-context advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged source using C++ operation-view-model forward arithmetic.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
