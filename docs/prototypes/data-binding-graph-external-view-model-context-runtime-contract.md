# Data Binding Graph External View-Model Context Runtime Contract

## Scope

This slice extends roadmap item `#12` by letting the runtime data-binding graph
bind an imported file-backed `ViewModelInstance` as the active state-machine
data context.

It is limited to the finite graph-owned `propertyValue` source-to-target edge
set already supported for default contexts. The graph keeps each source node's
`DataBindContext.sourcePathIds`, resolves those paths against the selected
imported `ViewModelInstance`, refreshes matching source-node values, marks
missing or type-incompatible sources unbound, and dirties the existing binding
queue so the next state-machine advance applies bound values to cloned
bindable targets.

## Runtime Rule

`StateMachineInstance::bind_view_model_instance_context(file, view_model_index,
instance_index)` binds a view-model instance that already exists in the
imported `RuntimeFile`.

The C++ probe action
`--runtime-bind-view-model-instance-state-machine-context <stateMachineIndex>
<viewModelIndex> <instanceIndex>` mirrors this by selecting
`file->viewModel(viewModelIndex)->instance(instanceIndex)` and calling
`StateMachineInstance::bindViewModelInstance(...)`.

This slice intentionally uses absolute/source-id path lookup through the
existing binary helper. It does not introduce arbitrary user-created live
view-model instances.

## Out Of Scope

This slice does not add public mutable view-model source handles, owned runtime
view-model data, converters, target-to-source propagation, update-queue parity,
observer subscriptions, pending add/remove behavior, listener-owned data
binding, callback-driven binding, relative or name-based path resolution beyond
existing imported helper behavior, parent context chains, nested artboard
context propagation, trigger callback dispatch, or external trigger reset/report
identity parity.

Default-context source mutation APIs remain unchanged.

## Completion Checks

- The graph stores source path metadata for the finite `propertyValue` source
  node set.
- Binding an imported view-model instance refreshes graph source values from
  that instance and marks the graph dirty.
- Unresolved or type-incompatible sources are represented as unbound graph
  sources instead of writing stale values during apply.
- C++ probe-backed state-machine tests verify non-default imported
  `ViewModelInstanceNumber`, `ViewModelInstanceBoolean`, and
  `ViewModelInstanceString`, `ViewModelInstanceColor`, and
  `ViewModelInstanceEnum` sources through existing `BlendState1DViewModel` and
  transition-condition consumers.
