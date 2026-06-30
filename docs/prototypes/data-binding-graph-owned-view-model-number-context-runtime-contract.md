# Data Binding Graph Owned View-Model Number Context Runtime Contract

## Scope

This slice extends roadmap item `#12` beyond imported file-backed external
contexts by admitting the first owned runtime view-model instance path.

It is intentionally limited to one source kind: `ViewModelInstanceNumber`
resolved through the existing absolute `DataBindContext.sourcePathIds` path and
applied to an existing graph-owned `BindablePropertyNumber.propertyValue`
target. The purpose is to prove the runtime can create mutable view-model data
outside the imported instance table, bind it as the active state-machine data
context, and have the graph refresh source nodes from that owned data.

## Runtime Rule

Rust exposes a small owned context handle that is created from an imported
view-model definition by index. The handle starts with the finite number source
values the C++ runtime creates for a fresh `File::createViewModelInstance(...)`
instance. Tests may mutate a number property by view-model property index before
binding the handle to a state-machine instance.

Binding the owned context refreshes graph number source nodes from the owned
context and dirties the existing binding queue so the next state-machine advance
applies the value to cloned bindable targets.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating a `ViewModelInstanceNumberRuntime` property, and calling
`StateMachineInstance::bindViewModelInstance(...)`.

## Out Of Scope

This slice does not introduce a stable public application API for every
view-model property kind, imported-instance cloning, source handles for
boolean/string/color/enum/asset/artboard/trigger values, lists, symbols,
nested view-model values, converters, target-to-source propagation,
dirty-queue parity, observer subscriptions, pending add/remove behavior,
relative/parent/nested path lookup, listener-owned data binding, nested
artboard propagation, or trigger reset/report identity.

Imported file-backed context binding and default-context source mutation remain
unchanged.

## Completion Checks

- Rust can create an owned runtime view-model context for a view-model index.
- Rust can mutate an owned number value by view-model property index.
- Binding that owned context refreshes matching number graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-number paths stay unbound instead of writing stale
  values.
- A C++ probe-backed state-machine test verifies an owned number context through
  the existing `BlendState1DViewModel` consumer.
