# Data Binding Graph Default Enum Source Mutation Runtime Contract

## Scope

This slice extends graph-owned source mutation for roadmap item `#12` to
default enum sources.

It is limited to a default-context `DataBindContext` whose source resolves to a
`ViewModelInstanceEnum` and whose target is already represented by the finite
default `propertyValue` graph edge set. The mutation updates the
`RuntimeDataBindGraph` source nodes that share the selected source path with a
raw enum index, marks the default edges dirty when the default view-model
context is bound, and lets the existing graph apply path copy the source value
to cloned bindable targets before state-machine layer evaluation.

## Runtime Rule

`StateMachineInstance::set_default_view_model_enum_source_for_data_bind`
uses the state-machine data-bind index to find the authored source path, then
mutates every bound enum source node with that same path. This matches C++'s
runtime mutation of the underlying `ViewModelInstanceEnum.propertyValue`
rather than a single cloned data-bind edge.

The C++ probe action
`--runtime-set-default-view-model-source-enum <stateMachineIndex> <dataBindIndex> <value>`
uses the same authored data-bind index, resolves the `DataBindContext`
`sourcePathIds` against the default `ViewModelInstance`, mutates the resolved
`ViewModelInstanceEnum.propertyValue`, and then relies on normal state-machine
advancement to process the dirty data bind.

The value is a raw unsigned integer property value. This intentionally does not
introduce enum key/name mutation or runtime enum-value clamping.

## Out Of Scope

This slice does not add external view-model contexts, public source handles,
same-path propagation for non-number/non-boolean/non-string/non-color/non-enum
data-bind-index source mutation APIs, enum key/name APIs, target-to-source
propagation, converter execution, observer/polling queue parity beyond this
same-path enum witness, pending add/remove handling, relative paths, parent
paths, nested paths, listener-owned data binding, `Solo` name mapping, or
nested artboard data-context propagation.

It also does not replace direct cloned-bindable target mutation APIs; those are
still explicit target override seams until the graph owns target-to-source
propagation.

## Completion Checks

- The C++ probe can mutate a default `ViewModelInstanceEnum` source by
  state-machine data-bind index.
- Rust mutates all matching `RuntimeDataBindGraph` enum source nodes for the
  selected source path rather than only the first cloned data-bind edge.
- Mutating after `bind_default_view_model_context` dirties the graph so the next
  state-machine advance observes the changed raw enum value.
- A C++ probe-backed test verifies the changed enum through an existing
  transition-condition consumer.
- A second same-path enum bind reports the changed source and applies the
  changed target on the next state-machine advance.
