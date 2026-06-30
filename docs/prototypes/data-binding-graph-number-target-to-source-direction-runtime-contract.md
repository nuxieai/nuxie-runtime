# Data Binding Graph Number Target-To-Source Direction Runtime Contract

## Purpose

Pin C++ converter-direction dispatch and exact source/target values for
graph-owned number target-to-source data binding.

C++ does not choose `reverseConvert` solely because data is flowing from target
to source. `DataBindContextValue::calculateUntypedDataValue` calls
`converter->convert` when the target-to-source pass is the binding's main
direction, and calls `converter->reverseConvert` when target-to-source is the
secondary direction of a main-to-target two-way bind. This slice mirrors that
main-direction rule for default-context number sources.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct number target-to-source with no converter.
- Direct `DataConverterOperationValue` on a `ToSource | TwoWay` data bind.
- `DataConverterGroup` containing ordered `DataConverterOperationValue`
  children on a `ToSource | TwoWay` data bind.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Full C++ dirty-list scheduling for ordinary source-to-target bindable target
  writes.
- Exact second-bind target-value parity for neighboring `ToTarget` binds that
  share the same source path.
- Imported and owned view-model contexts.
- Non-number target-to-source converter families.
- System-operation converters, range mapper, interpolator, formula, string,
  number-to-list, list, or scripted converters.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- The C++ probe reports `numberBindings` with `dataBindIndex`, `sourceValue`,
  and `targetValue`.
- A `ToSource | TwoWay` numeric bind does not eagerly source-to-target its own
  bindable target on initial data-context advance.
- A mutated direct numeric target writes the exact target value to the default
  view-model number source.
- A mutated `DataConverterOperationValue` numeric target uses C++ main-direction
  `convert` before writing the source.
- A mutated grouped `OperationValue` numeric target uses C++ main-direction
  group `convert` order before writing the source.
