# Data Binding Graph Owned View-Model Enum Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceEnum`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyEnum.propertyValue` target. Owned enum values are stored as
the same raw integer `propertyValue` used by imported file-backed enum sources.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates enum slots for
`ViewModelPropertyEnum`, `ViewModelPropertyEnumCustom`, and
`ViewModelPropertyEnumSystem` definitions using the same fresh-instance default
as C++ `File::createViewModelInstance(...)`. Tests may mutate an enum by
view-model property index before binding the owned context through
`StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating a `ViewModelInstanceEnumRuntime` property by value index, and binding
the owned instance to the state machine.

## Out Of Scope

This slice does not add owned asset/artboard/trigger values, enum-key public
APIs, stable public source handles, lists, symbols, nested view-model values,
converters, target-to-source propagation, dirty-queue parity, observer
subscriptions, pending add/remove behavior, relative/parent/nested path lookup,
listener-owned data binding, nested artboard propagation, or trigger
reset/report identity.

## Completion Checks

- Rust can mutate an owned enum value by view-model property index.
- Binding that owned context refreshes matching enum graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-enum paths stay unbound instead of writing stale
  values.
- A C++ probe-backed state-machine test verifies an owned enum context through
  an existing transition-condition consumer.
