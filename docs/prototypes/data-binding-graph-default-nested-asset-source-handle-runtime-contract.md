# Default Nested Asset Source Handle Runtime Contract

Purpose: extend nested default-context source handles to asset image sources
without admitting artboard, trigger, list, or view-model nested handles.

Default-context nested asset source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, image]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelAssetSourceHandle` via a separate
`default_view_model_asset_source_handle_by_property_name_path` API, then
mutates graph-owned default asset source nodes as raw asset indexes through
the existing source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/image` from file view model
  `0` into `RuntimeDefaultViewModelAssetSourceHandle`.
- Supporting the same asset property variants as root lookup:
  `ViewModelPropertyAsset` and `ViewModelPropertyAssetImage`.
- Mutating graph-owned default asset source nodes as raw asset indexes through
  that handle before ordinary state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only `default_view_model_asset_source_handle_by_property_name`
  semantics.
- Artboard, trigger, list, or view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Image loading, decoding, `RenderImage*` replacement, renderer hooks, file
  asset helper metadata mutation, or stable public asset object handles.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a nested default asset source handle resolved from
`child/image` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-asset intermediate paths
remain unresolved, and the C++ probe reports the same state-machine advance
and component update results.
