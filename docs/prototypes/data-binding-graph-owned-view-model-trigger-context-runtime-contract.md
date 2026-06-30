# Data Binding Graph Owned View-Model Trigger Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceTrigger`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyTrigger.propertyValue` target. Owned trigger values are stored
as the same raw integer trigger count used by imported file-backed trigger
sources.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates trigger slots for
`ViewModelPropertyTrigger` definitions using the same fresh-instance default as
C++ `File::createViewModelInstance(...)`. Tests may mutate a trigger count by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating the fresh `ViewModelInstanceTrigger.propertyValue`, and binding the
owned instance to the state machine.

## Out Of Scope

This slice does not add trigger firing APIs, trigger reset/report identity,
stable public source handles, lists, symbols, nested view-model values,
converters, target-to-source propagation, dirty-queue parity, observer
subscriptions, pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, or nested artboard propagation.

## Completion Checks

- Rust can mutate an owned trigger count by view-model property index.
- Binding that owned context refreshes matching trigger graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-trigger paths stay unbound instead of writing
  stale values.
- A C++ probe-backed state-machine test verifies an owned trigger context
  through an existing transition-condition consumer.
