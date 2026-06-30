# Data Binding Graph Owned View-Model Color Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceColor`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyColor.propertyValue` target. Owned color values are stored as
the same packed ARGB `u32` values used by the imported file-backed color path.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates color slots for
`ViewModelPropertyColor` definitions using the same fresh-instance default as
C++ `File::createViewModelInstance(...)`. Tests may mutate a color by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating a `ViewModelInstanceColorRuntime` property, and binding the owned
instance to the state machine.

## Out Of Scope

This slice does not add owned enum/asset/artboard/trigger values, stable public
source handles, lists, symbols, nested view-model values, converters,
target-to-source propagation, dirty-queue parity, observer subscriptions,
pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, nested artboard propagation, or trigger
reset/report identity.

## Completion Checks

- Rust can mutate an owned color value by view-model property index.
- Binding that owned context refreshes matching color graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-color paths stay unbound instead of writing stale
  values.
- A C++ probe-backed state-machine test verifies an owned color context through
  an existing transition-condition consumer.
