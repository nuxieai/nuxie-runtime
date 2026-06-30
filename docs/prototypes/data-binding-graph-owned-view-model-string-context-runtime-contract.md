# Data Binding Graph Owned View-Model String Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceString`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyString.propertyValue` target. Owned string values are raw bytes
so they match the existing imported string path and avoid inventing a text
encoding policy in this slice.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates string slots for
`ViewModelPropertyString` definitions using the same fresh-instance default as
C++ `File::createViewModelInstance(...)`. Tests may mutate a string by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating a `ViewModelInstanceStringRuntime` property, and binding the owned
instance to the state machine.

## Out Of Scope

This slice does not add owned color/enum/asset/artboard/trigger values, stable
public source handles, lists, symbols, nested view-model values, converters,
target-to-source propagation, dirty-queue parity, observer subscriptions,
pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, nested artboard propagation, or trigger
reset/report identity.

## Completion Checks

- Rust can mutate an owned string value by view-model property index.
- Binding that owned context refreshes matching string graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-string paths stay unbound instead of writing
  stale values.
- A C++ probe-backed state-machine test verifies an owned string context
  through an existing transition-condition consumer.
