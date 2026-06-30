# Data Binding Graph Rounder Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterRounder` execution in the runtime graph
target-to-source path for default-context number binds.

C++ target-to-source converter dispatch follows the binding's main direction:
main `ToSource` bindings call `convert`, while main `ToTarget` two-way bindings
call `reverseConvert`. The representative fixture in this contract is a
`ToSource | TwoWay` bind, so the numeric target is rounded with
`DataConverterRounder::convert` before writing the view-model source.
The main-`ToTarget | TwoWay` public update path, where C++ base
`reverseConvert` passes the target value through before rounder reapplication,
is covered by
`docs/prototypes/data-binding-graph-rounder-public-update-target-to-source-runtime-contract.md`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterRounder` on a `ToSource | TwoWay` data bind.
- Imported `DataConverterRounder.decimals`.
- C++ main-direction rounder math for numeric sources and targets.
- C++ probe coverage including exact source/target number binding reports for
  the mutating bind.

## Out Of Scope

- Broader public `DataBindContainer::updateDataBinds(true)` scheduler parity
  beyond the direct rounder dirty bind.
- Main-`ToTarget | TwoWay` rounder target-dirty behavior, covered by
  `docs/prototypes/data-binding-graph-rounder-main-to-target-two-way-target-dirty-runtime-contract.md`.
- `DataConverterGroup::reverseConvert` ordering.
- Formula, operation, range, system, interpolator, number-to-list, list, and
  scripted converters.
- Imported and owned view-model contexts.
- Pending dirty queues, pending add/remove behavior, observer-list parity, and
  re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated numeric target on a Rounder target-to-source bind is rounded
  according to C++ main-direction dispatch before writing the default
  view-model number source.
- The mutating bind's source and target values are compared directly against
  the C++ probe after each explicit runtime action.
- Existing direct number target-to-source and forward Rounder tests still pass.
