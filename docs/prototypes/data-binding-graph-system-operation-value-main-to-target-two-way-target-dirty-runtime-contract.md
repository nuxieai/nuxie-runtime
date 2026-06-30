# Data Binding Graph System OperationValue Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads` on
main-`ToTarget | TwoWay` number binds.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited target back to the source.
The manual target edit survives explicit `advancedDataContext()`, and the next
normal `StateMachineInstance::advance()` reapplies source-to-target conversion.
For these system converters on a main-`ToTarget` bind, that source-to-target
conversion uses forward operation-value arithmetic.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterSystemNormalizer` converters.
- Direct `DataConverterSystemDegsToRads` converters.
- Imported `operationType` and `operationValue`.
- `TwoWay` number data binds without the `ToSource` direction flag.
- Initial source-to-target system-converter flushing through a normal
  state-machine advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` system converter target-to-source behavior, covered
  by
  `docs/prototypes/data-binding-graph-system-operation-value-target-to-source-runtime-contract.md`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Symbol-list-index inputs to system converters.
- `DataConverterOperationViewModel`.
- Formula, interpolator, number-to-list, and scripted converters.
- Converter group compositions involving system converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the system-converter output
  from the unchanged source to the bindable number target.
- Mutating the system-converter target on a main-`ToTarget | TwoWay` bind
  preserves the manual value through explicit data-context advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged source using C++ system-converter forward operation-value
  arithmetic.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action for both concrete system converter classes.
