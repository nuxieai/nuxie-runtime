# Data Binding Graph Owned View-Model Artboard Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceArtboard`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyArtboard.propertyValue` target. Owned artboard values are
stored as the same raw integer `propertyValue` used by imported file-backed
artboard sources.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates artboard slots for
`ViewModelPropertyArtboard` definitions using the same fresh-instance default
as C++ `File::createViewModelInstance(...)`. Tests may mutate an artboard by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating the fresh `ViewModelInstanceArtboard.propertyValue`, and binding the
owned instance to the state machine.

## Out Of Scope

This slice does not add owned trigger values, bindable artboard handles,
bound nested view-model instances, nested artboard creation, nested artboard
propagation, stable public source handles, lists, symbols, nested view-model
values, converters, target-to-source propagation, dirty-queue parity, observer
subscriptions, pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, or trigger reset/report identity.

## Completion Checks

- Rust can mutate an owned artboard value by view-model property index.
- Binding that owned context refreshes matching artboard graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-artboard paths stay unbound instead of writing
  stale values.
- A C++ probe-backed state-machine test verifies an owned artboard context
  through an existing transition-condition consumer.
