# Data Binding Graph OperationViewModel Group Secondary Source Mutation Runtime Contract

## Purpose

Admit grouped `DataConverterOperationViewModel` dependency dirtiness when the
converter group's secondary default view-model number source changes.

This extends the direct secondary-source mutation slice to the existing
`DataConverterGroup<OperationValue, OperationViewModel>` runtime path. C++
binds the operation-view-model child from the same data context and registers
the owning `DataBind` as a dependent of the resolved secondary
`ViewModelInstanceNumber`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- A primary root number source feeding a `BindablePropertyNumber.propertyValue`
  target through a `DataConverterGroup`.
- A group containing `DataConverterOperationValue` followed by
  `DataConverterOperationViewModel`.
- The operation-view-model child resolving `sourcePathIds` to a second imported
  default root `ViewModelInstanceNumber`.
- Mutating that secondary number through
  `StateMachineInstance::set_default_view_model_number_source_for_data_bind`.
- Refreshing the nested converter operand and dirtying the owning source so
  grouped source-to-target reapplication uses the new secondary value.
- C++ probe coverage through the existing blend-state consumer plus ordinary
  observer binds for the primary and secondary number sources.

## Out Of Scope

- Other grouped operation-view-model compositions.
- Imported and owned runtime contexts.
- Missing, non-number, relative-path, parent-path, nested, or name-resolved
  secondary operation sources.
- Target mutations that update the secondary source through target-to-source
  conversion.
- Formula, interpolator, number-to-list, list, string, mixed stateful groups,
  scripted converters, and broader dirty-list scheduler parity.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, layout, and rendering.

## Completion Checks

- The initial state-machine advance uses the grouped operation-value and
  operation-view-model operands in C++ order.
- Mutating the secondary default number source refreshes the nested
  operation-view-model operand.
- The grouped primary bind is re-applied with C++ arithmetic using the new
  secondary value.
- The grouped bind, primary observer, and secondary observer match the C++ probe
  after each explicit runtime action.
