# Data Binding Graph Trigger Converter Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit the first trigger converter-group public update path:
`DataConverterGroup<DataConverterTrigger>` on a main-`ToTarget | TwoWay`
state-machine trigger bind.

This slice proves that the already admitted trigger public-update behavior also
flows through the graph-owned converter-group reverse/forward order for the
smallest trigger group shape.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- Main-`ToTarget | TwoWay` trigger binds.
- A direct `DataConverterGroup` containing one `DataConverterTrigger` child.
- Public `updateDataBinds(true)` target-to-source behavior where C++ reverses
  the group, uses `DataConverterTrigger` inherited base `reverseConvert` to
  write the edited target value to the source, then reapplies source-to-target
  through group forward conversion so `DataConverterTrigger::convert`
  increments the target.
- Exact C++ probe reporting for trigger binding source and target values after
  each explicit runtime action.

## Out Of Scope

- Explicit `advanceDataContext()` target-to-source behavior for trigger
  converter groups.
- Multi-child trigger groups and mixed trigger group compositions.
- Trigger source reset reapplication after later state-machine advancement.
- Imported or owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the existing public-update path.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- A mutated grouped main-`ToTarget | TwoWay` trigger target writes the edited
  trigger count into the default trigger source during public
  `updateDataBinds(true)`.
- The same public update reapplies source-to-target through the
  `DataConverterGroup<DataConverterTrigger>` forward path, incrementing the
  target like C++.
- Existing direct trigger public update, direct trigger converter public
  update, and trigger converter explicit target-to-source probes continue to
  pass.
