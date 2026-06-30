# Data Binding Graph Default Artboard Source Mutation Runtime Contract

## Scope

This slice extends graph-owned source mutation for roadmap item `#12` to
default artboard sources.

It is limited to a default-context `DataBindContext` whose source resolves to a
`ViewModelInstanceArtboard` and whose target is already represented by the
finite default `propertyValue` graph edge set. The mutation updates the
`RuntimeDataBindGraph` source node with a raw artboard property value, marks the
default edge dirty when the default view-model context is bound, and lets the
existing graph apply path copy the source value to the cloned bindable target
before state-machine layer evaluation.

## Runtime Rule

`StateMachineInstance::set_default_view_model_artboard_source_for_data_bind`
mutates only an artboard graph source node selected by state-machine data-bind
index.

The C++ probe action
`--runtime-set-default-view-model-source-artboard <stateMachineIndex> <dataBindIndex> <value>`
uses the same authored data-bind index, resolves the `DataBindContext`
`sourcePathIds` against the default `ViewModelInstance`, mutates the resolved
`ViewModelInstanceArtboard.propertyValue`, and then relies on normal
state-machine advancement to process the dirty data bind.

The value is a raw unsigned integer property value. This intentionally does not
perform nested artboard remapping or `ArtboardReferencer` target updates.

## Out Of Scope

This slice does not add external view-model contexts, public source handles,
non-artboard source mutation, nested artboard remapping,
`ArtboardReferencer::updateArtboard` target binding, bound view-model
propagation, literal `TransitionValueArtboardComparator` support,
target-to-source propagation, converter execution, observer/polling queue
parity, pending add/remove handling, relative paths, parent paths, nested paths,
listener-owned data binding, or nested artboard data-context propagation.

It also does not replace direct cloned-bindable target mutation APIs; those are
still explicit target override seams until the graph owns target-to-source
propagation.

## Completion Checks

- The C++ probe can mutate a default `ViewModelInstanceArtboard` source by
  state-machine data-bind index.
- Rust mutates the matching `RuntimeDataBindGraph` source node rather than a
  cloned bindable target.
- Mutating after `bind_default_view_model_context` dirties the graph so the next
  state-machine advance observes the changed raw artboard value.
- A C++ probe-backed test verifies the changed artboard through an existing
  transition-condition consumer.
