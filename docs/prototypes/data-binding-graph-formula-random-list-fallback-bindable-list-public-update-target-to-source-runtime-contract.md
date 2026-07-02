# Data Binding Graph Formula Random List Fallback Bindable-List Public Update Target-To-Source Runtime Contract

## Purpose

Pin public `updateDataBinds(true)` target-to-source behavior for
random-function `DataConverterFormula` list fallback sources feeding
`BindablePropertyList.propertyValue` targets.

C++ does not write an edited scalar list-target value back into a
`ViewModelInstanceList` source through this path. Like the deterministic
public-update path, it immediately reapplies the formula numeric fallback to
the list target during the same public update. The observable result is that
the source still reports the imported list size and the target scalar becomes
the formula fallback value for `randomModeValue` values `0`, `1`, and `2`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  list binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- `BindablePropertyList.propertyValue` state-machine targets.
- Direct `DataConverterFormula` with a `FormulaTokenFunction` output token.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- Mutating the bindable list target scalar by data-bind index before public
  target-to-source update.
- Existing C++ probe list binding reports:
  - source list size remains the imported list item count,
  - source number value remains absent,
  - target value becomes the formula fallback scalar during public update.

## Out Of Scope

- Deterministic public-update target-to-source behavior, covered by
  `data-binding-graph-formula-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
- Explicit target-to-source behavior for random-function formula list targets,
  covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
- Source-to-target random list-target fallback behavior, covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-target-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty scheduling for random-function formula
  list targets, covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-main-to-target-two-way-target-dirty-runtime-contract.md`.
- `DataConverterNumberToList`, which is covered by the existing bindable-list
  and number-to-list contracts.
- Generated list item creation, generated item identity, item-level binding,
  list layout, virtualization, and `DataBindListItemConsumer` behavior.
- Artboard-owned `ArtboardComponentList` targets.
- Imported, owned, relative, parent, and nested view-model contexts.
- Real random generation, secondary dependency invalidation, and full
  dirty-list scheduler parity. List fallback random call counts are covered by
  `data-binding-graph-formula-random-list-fallback-call-count-runtime-contract.md`.

## Completion Checks

- A random-function formula list source bound to
  `BindablePropertyList.propertyValue` can participate in a public
  target-to-source update.
- Rust reports the same unchanged source list size as C++ after
  `updateDataBinds(true)`.
- Rust applies the same formula fallback scalar to the list target as C++
  during `updateDataBinds(true)`.
- The edited scalar is not written into the source list.
- Random modes `0`, `1`, and `2` match C++ even when Rust is supplied non-zero
  random values.
- Existing deterministic formula list-target reverse and random formula
  list-target explicit reverse probes continue to pass.
