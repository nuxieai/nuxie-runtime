# Data Binding Graph Trigger Converter Target-To-Source Runtime Contract

## Purpose

Admit explicit `advanceDataContext()` target-to-source behavior for the first
state-machine trigger converter path: direct `DataConverterTrigger` on a
main-`ToSource | TwoWay` trigger bind.

This slice is the explicit data-context counterpart to the already admitted
public `updateDataBinds(true)` trigger converter path.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- Main-`ToSource | TwoWay` trigger binds.
- Direct `DataConverterTrigger` target-to-source behavior where C++
  main-to-source conversion increments the edited target value before writing
  the trigger source.
- Exact C++ probe reporting for trigger binding source and target values after
  default-context binding and explicit data-context target-to-source actions.

## Out Of Scope

- Public `updateDataBinds(true)` trigger behavior, which is covered by
  `docs/prototypes/data-binding-graph-trigger-public-update-target-to-source-runtime-contract.md`.
- Trigger converter groups.
- Bindable trigger reset after later state-machine advancement.
- Imported or owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this direct
  path.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- Mutating a direct `DataConverterTrigger` main-`ToSource | TwoWay` trigger
  target marks its graph binding target-to-source dirty.
- `advance_data_context` applies `DataConverterTrigger::convert` in the
  main-to-source direction before writing the default trigger source.
- The same data-context advance leaves Rust and C++ with the same trigger
  binding source and target report.
- Existing direct trigger target-to-source and public trigger update probes
  continue to pass.
