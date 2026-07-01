# Default Nested ViewModel Source Handle Runtime Contract

Purpose: extend nested default-context source handles to view-model pointer
sources without admitting broader object-handle or owned-instance APIs.

Default-context nested view-model pointer binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, grandchild]`. This
slice exposes the same resolved path through
`RuntimeDefaultViewModelViewModelSourceHandle` via a separate
`default_view_model_view_model_source_handle_by_property_name_path` API, then
relinks graph-owned default view-model pointer source nodes through the
existing source-handle relink setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/grandchild` from file view
  model `0` into `RuntimeDefaultViewModelViewModelSourceHandle`.
- Relinking graph-owned default view-model pointer source nodes to imported
  referenced instance indexes already recorded on the graph source.
- C++ probe comparison against the default view-model by-name path relink
  command for the matching nested path.

Out of scope:

- Changing root-only
  `default_view_model_view_model_source_handle_by_property_name` semantics.
- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, or list source-handle behavior beyond the existing committed APIs.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Public object handles, replacing generated owned child identities, arbitrary
  user-created runtime view-model instances, list item propagation, reverse
  target-to-source propagation, broader update queues, listener-owned data
  binding, and nested artboard propagation.

Completion condition: a nested default view-model pointer source handle
resolved from `child/grandchild` relinks the same graph-owned source path as
C++'s default source relink by name path for the matching nested property,
repeated same-value relinks report no change, root-name lookup remains separate
from slash-path lookup, missing nested paths remain unresolved, and the C++
probe reports the same data-context advance, state-machine advance, source
pointer, target pointer, and component update results.
