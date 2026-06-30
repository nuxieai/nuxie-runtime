# Data Binding Graph Default Boolean Source Mutation Runtime Contract

## Scope

This slice extends graph-owned source mutation for roadmap item `#12` from
default number sources to default boolean sources.

It is limited to a default-context `DataBindContext` whose source resolves to a
`ViewModelInstanceBoolean` and whose target is already represented by the
finite default `propertyValue` graph edge set. The mutation updates the
`RuntimeDataBindGraph` source node, marks the default edge dirty when the
default view-model context is bound, and lets the existing graph apply path copy
the source value to the cloned bindable target before state-machine layer
evaluation.

## Runtime Rule

`StateMachineInstance::set_default_view_model_boolean_source_for_data_bind`
mutates only a boolean graph source node selected by state-machine data-bind
index.

The C++ probe action
`--runtime-set-default-view-model-source-bool <stateMachineIndex> <dataBindIndex> <value>`
uses the same authored data-bind index, resolves the `DataBindContext`
`sourcePathIds` against the default `ViewModelInstance`, mutates the resolved
`ViewModelInstanceBoolean.propertyValue`, and then relies on normal
state-machine advancement to process the dirty data bind.

## Out Of Scope

This slice does not add external view-model contexts, public source handles,
non-boolean source mutation, target-to-source propagation, converter execution,
observer/polling queue parity, pending add/remove handling, relative paths,
parent paths, nested paths, listener-owned data binding, or nested artboard
data-context propagation.

It also does not replace direct cloned-bindable target mutation APIs; those are
still explicit target override seams until the graph owns target-to-source
propagation.

## Completion Checks

- The C++ probe can mutate a default `ViewModelInstanceBoolean` source by
  state-machine data-bind index.
- Rust mutates the matching `RuntimeDataBindGraph` source node rather than a
  cloned bindable target.
- Mutating after `bind_default_view_model_context` dirties the graph so the next
  state-machine advance observes the changed source value.
- A C++ probe-backed test verifies the changed boolean through an existing
  transition-condition consumer.
