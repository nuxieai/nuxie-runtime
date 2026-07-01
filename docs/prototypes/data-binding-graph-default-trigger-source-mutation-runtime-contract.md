# Data Binding Graph Default Trigger Source Mutation Runtime Contract

## Scope

This slice extends graph-owned source mutation for roadmap item `#12` to
default trigger sources and same-path observer propagation.

It is limited to a default-context `DataBindContext` whose source resolves to a
`ViewModelInstanceTrigger` and whose target is already represented by the finite
default `propertyValue` graph edge set. The mutation updates the
`RuntimeDataBindGraph` source nodes with a raw trigger count, marks the default
edges dirty when the default view-model context is bound, and lets the existing
graph apply path copy the source value to cloned bindable targets before
state-machine layer evaluation.

## Runtime Rule

`StateMachineInstance::set_default_view_model_trigger_source_for_data_bind`
resolves the trigger graph source path selected by state-machine data-bind
index and mutates every bound default-context trigger source node with the same
path.

The C++ probe action
`--runtime-set-default-view-model-source-trigger <stateMachineIndex> <dataBindIndex> <value>`
uses the same authored data-bind index, resolves the `DataBindContext`
`sourcePathIds` against the default `ViewModelInstance`, mutates the resolved
`ViewModelInstanceTrigger.propertyValue`, and then relies on normal
state-machine advancement to process the dirty data bind.

The value is a raw unsigned integer trigger count. This intentionally does not
fire triggers, dispatch listener actions, or change data-context trigger reset
semantics.

## Out Of Scope

This slice does not add external view-model contexts, public source handles,
remaining non-trigger source mutation observer families, trigger callback
targets, listener-owned trigger dispatch, `ListenerViewModelChange`,
callback-driven data binding, target-to-source propagation, converter
execution, full dirty-list scheduler parity, pending add/remove handling,
re-entry protection, relative paths, parent paths, nested paths, or nested
artboard data-context propagation.

Existing `StateMachineFireTrigger` and explicit data-context trigger reset
behavior remain unchanged.

It also does not replace direct cloned-bindable target mutation APIs; those are
still explicit target override seams until the graph owns target-to-source
propagation.

## Completion Checks

- The C++ probe can mutate a default `ViewModelInstanceTrigger` source by
  state-machine data-bind index.
- Rust mutates matching same-path `RuntimeDataBindGraph` source nodes rather
  than cloned bindable targets.
- Mutating after `bind_default_view_model_context` dirties the graph so the next
  state-machine advance observes the changed raw trigger count.
- A neighboring ordinary direct `ToTarget` trigger bind with the same source
  path reports the updated source and applies the updated target on the next
  state-machine advance.
- A C++ probe-backed test verifies the changed trigger through an existing
  transition-condition consumer.
