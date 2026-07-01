# Data Binding Graph Trigger Converter Group Target-To-Source Runtime Contract

## Purpose

Admit explicit `advanceDataContext()` target-to-source behavior for the first
state-machine trigger converter-group path:
`DataConverterGroup<DataConverterTrigger>` on a main-`ToSource | TwoWay`
trigger bind.

This slice is the explicit data-context counterpart to the already admitted
public `updateDataBinds(true)` trigger converter-group path.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- Main-`ToSource | TwoWay` trigger binds.
- A direct `DataConverterGroup` containing one `DataConverterTrigger` child.
- Explicit `advanceDataContext()` target-to-source behavior where C++ applies
  group forward order in the main-to-source direction, so
  `DataConverterTrigger::convert` increments the edited target value before
  writing the default trigger source.
- Exact C++ probe reporting for trigger binding source and target values after
  default-context binding and explicit data-context target-to-source actions.

## Out Of Scope

- Public `updateDataBinds(true)` trigger group behavior, which is covered by
  `docs/prototypes/data-binding-graph-trigger-converter-group-public-update-target-to-source-runtime-contract.md`.
- Multi-child trigger groups beyond the first two-child all-trigger group
  covered by
  `docs/prototypes/data-binding-graph-trigger-converter-multi-group-runtime-contract.md`,
  plus mixed trigger group compositions.
- Trigger source reset behavior beyond preserving the existing direct and
  direct-converter reset reapply probes.
- Imported or owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this direct
  group path.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- Mutating a grouped `DataConverterGroup<DataConverterTrigger>`
  main-`ToSource | TwoWay` trigger target marks its graph binding
  target-to-source dirty.
- `advance_data_context` applies group forward order in the main-to-source
  direction, incrementing the edited bindable target before writing the
  default trigger source.
- The same data-context advance leaves Rust and C++ with the same trigger
  binding source and target report.
- Existing direct trigger, direct trigger converter, trigger reset reapply, and
  public trigger group probes continue to pass.
