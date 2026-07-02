# Data Binding Graph Formula List Fallback Bindable-List Public Update Target-To-Source Runtime Contract

## Purpose

Pin public `updateDataBinds(true)` target-to-source behavior for deterministic
`DataConverterFormula` list fallback sources feeding
`BindablePropertyList.propertyValue` targets.

C++ does not write an edited scalar list-target value back into a
`ViewModelInstanceList` source through this path. Unlike explicit
`advanceDataContext()`, the public update path does immediately reapply the
formula numeric fallback to the list target during the same call. The
observable result is that the source still reports the imported list size and
the target scalar becomes the formula fallback value.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  list binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyList.propertyValue` state-machine targets.
- Direct `DataConverterFormula` with a single deterministic
  `FormulaTokenInput` output token.
- Mutating the bindable list target scalar by data-bind index before public
  target-to-source update.
- Existing C++ probe list binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value becomes the formula fallback scalar during public update.

## Out Of Scope

- Source-to-target list-target fallback behavior, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-target-runtime-contract.md`.
- Explicit target-to-source behavior, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
- Random-function formula list targets.
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

- A deterministic formula list source bound to
  `BindablePropertyList.propertyValue` can participate in a public
  target-to-source update.
- Rust reports the same unchanged source list size as C++ after
  `updateDataBinds(true)`.
- Rust applies the same formula fallback scalar to the list target as C++
  during `updateDataBinds(true)`.
- The edited scalar is not written into the source list.
- Existing deterministic formula list-target, explicit formula list-target
  target-to-source, random formula list-target, and direct bindable-list
  target-to-source probes continue to pass.
