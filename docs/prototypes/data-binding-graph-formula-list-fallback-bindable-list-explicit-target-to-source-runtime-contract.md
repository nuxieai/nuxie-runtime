# Data Binding Graph Formula List Fallback Bindable-List Explicit Target-To-Source Runtime Contract

## Purpose

Pin explicit target-to-source behavior for deterministic
`DataConverterFormula` list fallback sources feeding
`BindablePropertyList.propertyValue` targets.

C++ does not write an edited scalar list-target value back into a
`ViewModelInstanceList` source through this path, and it also does not
immediately reapply the formula numeric fallback to the list target during the
same explicit data-context advancement. The observable result is that the
source still reports the imported list size and the edited list-target scalar
is preserved.

## In Scope

- `StateMachineInstance::advanceDataContext()` for
  main-`ToSource | TwoWay` list binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyList.propertyValue` state-machine targets.
- Direct `DataConverterFormula` with a single deterministic
  `FormulaTokenInput` output token.
- Mutating the bindable list target scalar by data-bind index before explicit
  data-context advancement.
- Existing C++ probe list binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value remains the edited scalar after explicit data-context
    advancement.

## Out Of Scope

- Source-to-target list-target fallback behavior, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-target-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior for deterministic
  formula list targets, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
- Explicit target-to-source behavior for random-function formula list targets,
  covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior for
  random-function formula list targets, covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
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

- A deterministic formula list source bound to
  `BindablePropertyList.propertyValue` can participate in an explicit
  target-to-source pass.
- Rust reports the same unchanged source list size as C++ after explicit
  data-context advancement.
- Rust preserves the same edited list-target scalar as C++ after explicit
  data-context advancement.
- The formula numeric fallback is not reapplied to the list target during the
  same explicit target-to-source pass.
- Existing deterministic formula list-target, random formula list-target, and
  direct bindable-list target-to-source probes continue to pass.
