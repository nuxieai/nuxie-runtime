# Data Binding Graph Trigger Converter Multi-Group Runtime Contract

## Purpose

Admit the first multi-child trigger converter-group shape:
`DataConverterGroup<DataConverterTrigger, DataConverterTrigger>` on default
state-machine trigger binds.

This slice proves that the one-child trigger group paths use ordered group
execution rather than a special-case single-child shortcut.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- A direct `DataConverterGroup` containing exactly two
  `DataConverterTrigger` children in authored group order.
- Public `updateDataBinds(true)` target-to-source behavior for
  main-`ToTarget | TwoWay` trigger binds. C++ reverses the group for
  target-to-source, each trigger `reverseConvert` passes the edited target
  through, then source-to-target reapplication runs group forward order so the
  two trigger converters increment the bindable target twice.
- Explicit `advanceDataContext()` target-to-source behavior for
  main-`ToSource | TwoWay` trigger binds. C++ runs group forward order in the
  main-to-source direction so the two trigger converters increment the edited
  target twice before writing the default trigger source.
- Exact C++ probe reporting for trigger binding source and target values after
  each explicit runtime action.

## Out Of Scope

- Mixed trigger group compositions.
- Trigger groups with more than two children.
- Stateful trigger-adjacent group children.
- Imported or owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the existing public-update and explicit
  data-context paths.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- A grouped two-trigger main-`ToTarget | TwoWay` public update writes the
  edited trigger count to the source, then reapplies source-to-target through
  both trigger converters, matching C++ source/target rows.
- A grouped two-trigger main-`ToSource | TwoWay` explicit data-context update
  writes the edited trigger count plus two to the source, matching C++
  source/target rows.
- Existing one-child trigger group public-update and explicit
  target-to-source probes continue to pass.
