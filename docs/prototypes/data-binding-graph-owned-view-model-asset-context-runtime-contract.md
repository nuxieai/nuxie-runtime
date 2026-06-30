# Data Binding Graph Owned View-Model Asset Context Runtime Contract

## Scope

This slice extends the owned runtime view-model context path to
`ViewModelInstanceAssetImage`.

It is limited to direct absolute `DataBindContext.sourcePathIds` lookup and
source-to-target propagation into an existing graph-owned
`BindablePropertyAsset.propertyValue` target. Owned asset values are stored as
the same raw integer `propertyValue` used by imported file-backed asset image
sources.

## Runtime Rule

`RuntimeOwnedViewModelInstance::new` creates asset slots for
`ViewModelPropertyAsset` and `ViewModelPropertyAssetImage` definitions using
the same fresh-instance default as C++ `File::createViewModelInstance(...)`.
Tests may mutate an asset by view-model property index before binding the owned
context through `StateMachineInstance::bind_owned_view_model_context`.

The C++ probe mirrors this by creating `file->createViewModelInstance(...)`,
mutating the fresh `ViewModelInstanceAssetImage.propertyValue`, and binding the
owned instance to the state machine.

## Out Of Scope

This slice does not add owned artboard/trigger values, render image handles,
asset resolver behavior, stable public source handles, lists, symbols, nested
view-model values, converters, target-to-source propagation, dirty-queue
parity, observer subscriptions, pending add/remove behavior,
relative/parent/nested path lookup, listener-owned data binding, nested
artboard propagation, or trigger reset/report identity.

## Completion Checks

- Rust can mutate an owned asset value by view-model property index.
- Binding that owned context refreshes matching asset graph source nodes and
  marks the graph dirty.
- Unresolved, missing, or non-asset paths stay unbound instead of writing stale
  values.
- A C++ probe-backed state-machine test verifies an owned asset context through
  an existing transition-condition consumer.
