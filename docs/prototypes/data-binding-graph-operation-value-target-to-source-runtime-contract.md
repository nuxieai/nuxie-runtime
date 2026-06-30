# Data Binding Graph OperationValue Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterOperationValue::reverseConvert` execution in the
runtime graph target-to-source path.

C++ applies `DataConverterOperationValue::reverseConvert` when a two-way or
to-source number bind writes a changed numeric target back into a numeric view
model source. Rust already models C++ operation-value forward math and carries
the matching reverse helper for system-operation converters; this slice wires
that reverse helper into graph-owned number target-to-source binding.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterOperationValue` on a `ToSource | TwoWay` data bind.
- C++ reverse operation-value math for numeric sources and targets.
- C++ probe coverage with a representative multiply reverse case, plus a
  second direct `ToTarget` bind to the same source observed through an existing
  blend-state consumer.

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
  reverse-converted before writing the default view-model number source.
- The same source path is marked dirty so a second ordinary `ToTarget` bind
  observes the reversed source value on the same explicit data-context advance.
- Existing direct number target-to-source and forward OperationValue tests
  still pass.
