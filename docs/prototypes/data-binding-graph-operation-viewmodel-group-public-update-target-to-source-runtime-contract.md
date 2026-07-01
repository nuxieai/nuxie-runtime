# Data Binding Graph OperationViewModel Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit grouped `DataConverterOperationViewModel` public
`updateDataBinds(true)` target-to-source behavior for a main-`ToTarget |
TwoWay` number bind.

This slice proves that graph-owned converter groups reverse their children in
C++ order when one child is a context-aware operation-view-model converter:
the edited bindable number target is reverse-converted through the group,
written to the primary default view-model number source, then forward-applied
back to the target during the same public update.

## In Scope

- Default root view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Main-`ToTarget | TwoWay` state-machine `DataBindContext` records targeting
  `BindablePropertyNumber.propertyValue`.
- A direct root primary `ViewModelInstanceNumber.propertyValue` source.
- A `DataConverterGroup` containing `DataConverterOperationValue` followed by
  `DataConverterOperationViewModel`.
- Operation-view-model secondary operands that resolve to another imported
  default root `ViewModelInstanceNumber`.
- `StateMachineInstance::update_data_binds_apply_target_to_source`, matching
  C++ `StateMachineInstance::updateDataBinds(true)`.
- Exact C++ probe comparison for source, target, state-machine advance, and
  component update reports.

## Out Of Scope

- Other grouped operation-view-model compositions.
- Grouped live dependency/dirt propagation when the secondary operation source
  changes.
- Missing, non-number, relative-path, parent-path, or nested secondary
  operation sources.
- Imported and owned runtime contexts.
- Formula, interpolator, number-to-list, list, string, mixed stateful groups,
  scripted converters, and broader dirty-list scheduler parity.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, layout, and rendering.

## Completion Checks

- A mutated bindable target is reverse-converted through the converter group in
  C++ reverse child order.
- The operation-view-model child uses the imported secondary number operand
  during reverse conversion.
- The primary default number source and bindable number target match C++ after
  public update and after subsequent state-machine advancement.
- Existing direct operation-view-model and operation-value group public-update
  tests continue to pass.
