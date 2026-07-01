# Default ViewModel Asset Source Handle Runtime Contract

Purpose: extend the default-context stable public source-handle family to
asset image sources without admitting artboard, trigger, list, or view-model
handles in this slice.

The existing default asset source mutation path can already resolve a root
`ViewModelPropertyAsset` or `ViewModelPropertyAssetImage.name` and mutate every
graph source node sharing that path. This slice exposes the resolved path as
an immutable `RuntimeDefaultViewModelAssetSourceHandle`, then applies the
handle through the same graph-owned asset source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving root `ViewModelPropertyAsset` and `ViewModelPropertyAssetImage`
  names on file view model `0` into
  `RuntimeDefaultViewModelAssetSourceHandle`.
- Mutating graph-owned default asset source nodes as raw asset indexes through
  the handle before ordinary state-machine advancement.
- C++ probe comparison through the existing default asset by-name mutation
  command and report surface.

Out of scope:

- Artboard, trigger, list, or view-model default source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Image loading, decoding, `RenderImage*` replacement, renderer hooks, file
  asset helper metadata mutation, or stable public asset object handles.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default asset source handle resolved from `image`
mutates the same graph-owned source path as the existing property-name API,
repeated same-value writes report no change, slash-path lookup stays
unresolved, and the C++ probe reports the same state-machine advance and
component update results.
