# Data Binding Graph OperationViewModel Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterOperationViewModel` execution in the graph-owned
target-to-source path for main-`ToSource | TwoWay` number binds.

C++ target-to-source converter dispatch follows the data bind's main direction.
For a main-`ToSource` `DataConverterOperationViewModel` bind, the edited
target value is passed through `convert`, using the resolved secondary
view-model number operand, before writing the primary view-model number source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Primary root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` primary source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterOperationViewModel` converters whose `sourcePathIds`
  resolve against the imported default root view-model instance.
- Secondary operation sources that resolve to `ViewModelInstanceNumber`.
- Main-`ToSource | TwoWay` target-to-source dispatch through `convert`.
- C++ operation-view-model arithmetic using the already-audited
  operation-value helper.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToTarget | TwoWay` operation-view-model dirty behavior.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity and
  public-queue `reverseConvert` behavior.
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

- A mutated numeric target on a main-`ToSource | TwoWay`
  `DataConverterOperationViewModel` bind is converted with C++ main-direction
  arithmetic before writing the default view-model number source.
- The secondary operand comes from the imported default root view-model number
  resolved by the converter's `sourcePathIds`.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct `OperationViewModel` source-to-target tests continue to pass.
