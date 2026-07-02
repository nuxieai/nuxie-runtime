# Data Binding Graph Formula Random List Fallback Bindable-List Target Runtime Contract

## Purpose

Pin random-function list-target behavior for `DataConverterFormula` list
fallback sources.

C++ keeps a default-context `ViewModelInstanceList` source represented for a
`BindablePropertyList.propertyValue` target even when the bind has a direct
`DataConverterFormula` whose output token is `FunctionType::random`. For list
inputs, the random function does not consume Rust's supplied random values for
the observable fallback result: the source reports the original list size, the
list target is preserved during explicit data-context advancement, and later
normal state-machine advancement applies the numeric fallback target value.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyList.propertyValue` state-machine targets.
- Direct `DataConverterFormula` with a `FormulaTokenFunction` output token.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- Explicit `advance_data_context` followed by normal state-machine
  advancement.
- Existing C++ probe list binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value stays unchanged during explicit data-context advancement,
  - target value becomes the formula fallback scalar on later normal advance.

## Out Of Scope

- Deterministic `FormulaTokenInput` list-target behavior, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-target-runtime-contract.md`.
- Public-update and target-to-source scheduling for formula list targets.
- Target-dirty scheduling for formula list targets.
- `DataConverterNumberToList`, which is covered by the existing bindable-list
  and number-to-list contracts.
- Generated list item creation, generated item identity, item-level binding,
  list layout, virtualization, and `DataBindListItemConsumer` behavior.
- Artboard-owned `ArtboardComponentList` targets.
- Imported, owned, relative, parent, and nested view-model contexts.
- Real random generation, random call counts, secondary dependency
  invalidation, and full dirty-list scheduler parity.

## Completion Checks

- A random-function formula list source bound to
  `BindablePropertyList.propertyValue` is admitted into the runtime data-bind
  graph.
- Rust reports the same source list size as C++ for the formula list bind.
- Explicit data-context advancement preserves the list target scalar.
- Later normal state-machine advancement applies the formula fallback scalar to
  the list target.
- Random modes `0`, `1`, and `2` match C++ even when Rust is supplied non-zero
  random values.
- Existing deterministic formula list-target, direct list, and
  `DataConverterNumberToList` bindable-list probes continue to pass.
