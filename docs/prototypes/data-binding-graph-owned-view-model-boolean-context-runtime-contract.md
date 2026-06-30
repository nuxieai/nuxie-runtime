# Data Binding Graph Owned View-Model Boolean Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path admitted for
number sources to `ViewModelInstanceBoolean`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyBoolean.propertyValue` target. It proves that owned runtime
view-model data can carry a mutable boolean source and that binding the owned
context refreshes graph source nodes from that data.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates boolean slots for
`ViewModelPropertyBoolean` definitions using the same fresh-instance default as
C++ `File::createViewModelInstance(...)`. Tests may mutate a boolean by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating a `ViewModelInstanceBooleanRuntime` property, and binding the owned
instance to the state machine.

## Out Of Scope

This slice does not add owned string/color/enum/asset/artboard/trigger values,
stable public source handles, lists, symbols, nested view-model values,
converters, target-to-source propagation, dirty-queue parity, observer
subscriptions, pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, nested artboard propagation, or trigger
reset/report identity.

## Completion Checks

- Rust can mutate an owned boolean value by view-model property index.
- Binding that owned context refreshes matching boolean graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-boolean paths stay unbound instead of writing
  stale values.
- A C++ probe-backed state-machine test verifies an owned boolean context
  through an existing transition-condition consumer.
