# Data Binding Graph OperationViewModel Secondary Source Mutation Runtime Contract

## Purpose

Admit direct `DataConverterOperationViewModel` dependency dirtiness when its
secondary default view-model number source changes.

C++ `DataConverterOperationViewModel::bindFromContext()` stores the resolved
secondary `ViewModelInstanceNumber` and registers the owning `DataBind` as a
dependent. When that secondary number changes, the dependent bind recomputes
with the new operand.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Primary root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- Direct `DataConverterOperationViewModel` converters whose `sourcePathIds`
  resolve to a second imported default root `ViewModelInstanceNumber`.
- Mutating that secondary number through
  `StateMachineInstance::set_default_view_model_number_source_for_data_bind`.
- Refreshing the direct converter operand and dirtying the dependent source so
  source-to-target reapplication uses the new secondary value.
- C++ probe coverage through a `BlendState1DViewModel` consumer with a second
  ordinary number bind exposing the secondary source.

## Out Of Scope

- Grouped `DataConverterOperationViewModel` dependency dirtiness.
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

- The initial state-machine advance uses the imported default secondary number
  operand.
- Mutating the secondary default number source refreshes the direct
  operation-view-model converter operand.
- The dependent primary bind is re-applied with C++ operation arithmetic using
  the new secondary value.
- The primary converter-bound bind and the secondary ordinary bind both match
  the C++ probe after each explicit runtime action.
