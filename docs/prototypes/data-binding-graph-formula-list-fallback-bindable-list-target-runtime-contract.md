# Data Binding Graph Formula List Fallback Bindable-List Target Runtime Contract

## Purpose

Pin the first list-target behavior for `DataConverterFormula` list fallback
sources.

C++ keeps a default-context `ViewModelInstanceList` source represented for a
`BindablePropertyList.propertyValue` target even when the bind has a direct
`DataConverterFormula`. A deterministic `FormulaTokenInput` over that list
source reports the original source list size, preserves the list target during
explicit data-context advancement, and then applies the numeric fallback target
value on later normal state-machine advancement.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyList.propertyValue` state-machine targets.
- Direct `DataConverterFormula` with a single deterministic
  `FormulaTokenInput` output token.
- Explicit `advance_data_context` followed by normal state-machine
  advancement.
- Existing C++ probe list binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value stays unchanged during explicit data-context advancement,
  - target value becomes the formula fallback scalar on later normal advance.

## Out Of Scope

- Formula random-function tokens for list targets are covered separately by
  `data-binding-graph-formula-random-list-fallback-bindable-list-target-runtime-contract.md`.
- Explicit target-to-source scheduling for deterministic formula list targets
  is covered separately by
  `data-binding-graph-formula-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
- Public-update target-to-source scheduling for deterministic formula list
  targets is covered separately by
  `data-binding-graph-formula-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty scheduling for deterministic formula
  list targets is covered separately by
  `data-binding-graph-formula-list-fallback-bindable-list-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Remaining random target-dirty scheduling for formula list targets.
- `DataConverterNumberToList`, which is covered by the existing bindable-list
  and number-to-list contracts.
- Generated list item creation, generated item identity, item-level binding,
  list layout, virtualization, and `DataBindListItemConsumer` behavior.
- Artboard-owned `ArtboardComponentList` targets.
- Imported, owned, relative, parent, and nested view-model contexts.
- Real random generation, random call counts, secondary dependency
  invalidation, and full dirty-list scheduler parity.

## Completion Checks

- A formula list source bound to `BindablePropertyList.propertyValue` is
  admitted into the runtime data-bind graph.
- Rust reports the same source list size as C++ for the formula list bind.
- Explicit data-context advancement preserves the list target scalar.
- Later normal state-machine advancement applies the formula fallback scalar to
  the list target.
- Existing direct list and `DataConverterNumberToList` bindable-list probes
  continue to pass.
