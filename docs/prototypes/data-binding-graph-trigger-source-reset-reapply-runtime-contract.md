# Data Binding Graph Trigger Source Reset Reapply Runtime Contract

## Purpose

Admit the narrow C++ behavior where `advancedDataContext()` resets bound
`ViewModelInstanceTrigger.propertyValue` sources to `0`, and the following
state-machine advance reapplies that reset source value to trigger bindable
targets.

This slice closes the lifecycle edge exposed by direct trigger and direct
`DataConverterTrigger` target-to-source binds after explicit data-context
advancement.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources reset by
  `advance_data_context`.
- `BindablePropertyTrigger.propertyValue` targets.
- Main-`ToSource | TwoWay` trigger binds, both converter-free and direct
  `DataConverterTrigger`.
- The next state-machine advance applying the reset source value to the target.
- C++ parity for the source-to-target converter direction: direct
  `DataConverterTrigger` main-to-source binds use inherited base
  `reverseConvert` when applying source-to-target, so reset source `0` writes
  target `0`.
- Exact C++ probe reporting for trigger binding source and target values after
  the reset has been reapplied.

## Out Of Scope

- Resetting arbitrary bindable trigger targets without a bound trigger source.
- Public `updateDataBinds(true)` trigger reset behavior.
- Trigger converter groups.
- Imported or owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the dirty source-to-target reapply bit needed for
  this default-context path.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- A converter-free main-`ToSource | TwoWay` trigger bind writes the edited
  target to the source during `advance_data_context`, then the next
  state-machine advance reports both source and target reset to `0`.
- A direct `DataConverterTrigger` main-`ToSource | TwoWay` bind writes the
  converted edited target to the source during `advance_data_context`, then the
  next state-machine advance reports both source and target reset to `0`.
- Existing direct trigger, trigger converter, and public trigger update probes
  continue to pass.
