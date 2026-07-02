# Data Binding Graph Formula List Fallback Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin main-`ToTarget | TwoWay` target-dirty behavior for deterministic
`DataConverterFormula` list fallback sources feeding number targets.

C++ keeps the imported `ViewModelInstanceList` source unchanged when a
formula-bound number target is manually edited. The manual target edit
survives explicit `advanceDataContext()`, and later normal state-machine
advancement reapplies source-to-target conversion, writing the formula list
fallback scalar to the target.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyNumber.propertyValue` targets consumed by
  `BlendState1DViewModel`.
- Direct `DataConverterFormula` with a single deterministic
  `FormulaTokenInput` output token.
- Main-`ToTarget | TwoWay` data-bind flags, without the `ToSource` direction
  flag.
- Initial source-to-target flushing through a normal state-machine advance.
- Mutating the bindable number target by data-bind index.
- Explicit `advanceDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Existing C++ probe number binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value remains the edited scalar after explicit data-context
    advancement,
  - target value becomes the formula fallback scalar on later normal advance.

## Out Of Scope

- Source-to-target list fallback behavior, covered by
  `data-binding-graph-formula-list-fallback-runtime-contract.md`.
- Explicit target-to-source behavior, covered by
  `data-binding-graph-formula-list-fallback-explicit-target-to-source-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior, covered by
  `data-binding-graph-formula-list-fallback-public-update-target-to-source-runtime-contract.md`.
- Random-function formula list fallback target-dirty behavior.
- `BindablePropertyList.propertyValue` targets, covered by the bindable-list
  formula fallback contracts.
- Generated list item creation, generated item identity, item-level binding,
  list layout, virtualization, and `DataBindListItemConsumer` behavior.
- Imported, owned, relative, parent, and nested view-model contexts.
- Real random generation, random call counts, secondary dependency
  invalidation, and full dirty-list scheduler parity.

## Completion Checks

- The initial normal state-machine advance applies the formula list fallback
  scalar to `BindablePropertyNumber.propertyValue`.
- A manual edit to the number target is preserved through explicit
  data-context advancement.
- A later normal state-machine advance reapplies the formula fallback scalar
  to the number target.
- Rust reports the same unchanged source list size as C++ throughout the
  sequence.
- Existing deterministic formula list source-to-target and reverse probes
  continue to pass.
