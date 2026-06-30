# Data Binding Graph OperationValue Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterOperationValue` execution in the runtime graph
target-to-source path.

C++ target-to-source converter dispatch follows the binding's main direction:
main `ToSource` bindings call `convert`, while main `ToTarget` two-way bindings
call `reverseConvert`. The representative fixture in this contract is a
`ToSource | TwoWay` bind, so the numeric target is converted with
`DataConverterOperationValue::convert` before writing the view-model source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterOperationValue` on a `ToSource | TwoWay` data bind.
- C++ main-direction operation-value math for numeric sources and targets.
- C++ probe coverage with a representative multiply case, including exact
  source/target number binding reports for the mutating bind.

## Out Of Scope

- Reverse conversion for symbol-list-index sources.
- `DataConverterOperationViewModel`, system-operation converters, range mapper,
  interpolator, formula, string, number-to-list, or list converters.
- `DataConverterGroup::reverseConvert` ordering.
- Imported and owned view-model contexts.
- Pending dirty queues, pending add/remove behavior, observer-list parity, and
  re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated numeric target on an `OperationValue` target-to-source bind is
  converted according to C++ main-direction dispatch before writing the default
  view-model number source.
- The mutating bind's source and target values are compared directly against
  the C++ probe after each explicit runtime action.
- Existing direct number target-to-source and forward OperationValue tests
  still pass.
