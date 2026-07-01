# Data Binding Graph BooleanNegate Target-To-Source Runtime Contract

## Purpose

Admit the first reverse-converter runtime graph path for target-to-source data
binding.

C++ routes target-to-source writes through `DataConverter::reverseConvert`.
`DataConverterBooleanNegate` is symmetric: its reverse conversion is the same
boolean negation used for forward conversion. This slice covers that one
converter so reverse conversion enters the graph without taking on converter
groups, list consumers, or the broader dirty-queue scheduler.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` source values.
- `BindablePropertyBoolean.propertyValue` targets.
- A `ToSource | TwoWay` data bind with `DataConverterBooleanNegate`.
- Explicit `advance_data_context` draining the mutated boolean target through
  reverse conversion before normal source-to-target application.
- C++ probe coverage using a second direct `ToTarget` bind to the same source
  and an existing boolean transition-condition consumer.

Public `updateDataBinds(true)` coverage for direct boolean and BooleanNegate
main-`ToTarget | TwoWay` binds is covered separately by
`docs/prototypes/data-binding-graph-boolean-public-update-target-to-source-runtime-contract.md`.

## Out Of Scope

- Reverse conversion for `DataConverterToNumber`, `DataConverterToString`,
  operation, range, interpolator, formula, system, number-to-list, or list
  converters.
- `DataConverterGroup::reverseConvert` ordering.
- List source/target propagation and `DataBindListItemConsumer` targets.
- Imported and owned view-model contexts.
- Pending dirty queues, pending add/remove behavior, observer-list parity, and
  re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated boolean target on a `BooleanNegate` target-to-source bind is
  negated before writing the default view-model boolean source.
- The same source path is marked dirty so a second ordinary `ToTarget` bind
  observes the reversed source value on the same explicit data-context advance.
- Existing direct boolean target-to-source and forward boolean-negate tests
  still pass.
