# Data Binding Graph Default Asset Source Mutation Runtime Contract

## Scope

This slice extends graph-owned source mutation for roadmap item `#12` to
default asset-image sources and same-path observer propagation.

It is limited to a default-context `DataBindContext` whose source resolves to a
`ViewModelInstanceAssetImage` and whose target is already represented by the
finite default `propertyValue` graph edge set. The mutation updates the
`RuntimeDataBindGraph` source nodes with a raw asset property value, marks the
default edges dirty when the default view-model context is bound, and lets the
existing graph apply path copy the source value to cloned bindable targets
before state-machine layer evaluation.

## Runtime Rule

`StateMachineInstance::set_default_view_model_asset_source_for_data_bind`
resolves the asset graph source path selected by state-machine data-bind index
and mutates every bound default-context asset source node with the same path.

The C++ probe action
`--runtime-set-default-view-model-source-asset <stateMachineIndex> <dataBindIndex> <value>`
uses the same authored data-bind index, resolves the `DataBindContext`
`sourcePathIds` against the default `ViewModelInstance`, mutates the resolved
`ViewModelInstanceAssetImage.propertyValue`, and then relies on normal
state-machine advancement to process the dirty data bind.

The value is a raw unsigned integer property value. This intentionally does not
resolve file assets or apply render-image/imageValue side effects.

## Out Of Scope

This slice does not add external view-model contexts, public source handles,
remaining non-asset source mutation observer families, file asset resolution
during runtime binding, render-image mutation,
`BindablePropertyAsset.imageValue` side effects, `Image.setAsset` target
binding, target-to-source propagation, converter execution, full dirty-list
scheduler parity, pending add/remove handling, re-entry protection, relative
paths, parent paths, nested paths, listener-owned data binding, or nested
artboard data-context propagation.

It also does not replace direct cloned-bindable target mutation APIs; those are
still explicit target override seams until the graph owns target-to-source
propagation.

## Completion Checks

- The C++ probe can mutate a default `ViewModelInstanceAssetImage` source by
  state-machine data-bind index.
- Rust mutates matching same-path `RuntimeDataBindGraph` source nodes rather
  than cloned bindable targets.
- Mutating after `bind_default_view_model_context` dirties the graph so the next
  state-machine advance observes the changed raw asset value.
- A neighboring ordinary direct `ToTarget` asset bind with the same source path
  reports the updated source and applies the updated target on the next
  state-machine advance.
- A C++ probe-backed test verifies the changed asset through an existing
  transition-condition consumer.
