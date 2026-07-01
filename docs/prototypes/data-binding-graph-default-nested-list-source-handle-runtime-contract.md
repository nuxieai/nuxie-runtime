# Default Nested List Source Handle Runtime Contract

Purpose: extend nested default-context source handles to list sources without
admitting list item identity, item traversal, or broader object-handle APIs.

Default-context nested list source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, items]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelListSourceHandle` via a separate
`default_view_model_list_source_handle_by_property_name_path` API, then
mutates graph-owned default list source nodes by item count through the
existing source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/items` from file view model
  `0` into `RuntimeDefaultViewModelListSourceHandle`.
- Mutating graph-owned default list source nodes by item count through that
  handle before ordinary data-context/state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only `default_view_model_list_source_handle_by_property_name`
  semantics.
- View-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- List item identity, item view-model references, item-level traversal,
  generated item instancing beyond count parity, layout, virtualization, or
  stable public list item handles.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: a nested default list source handle resolved from
`child/items` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-list intermediate paths
remain unresolved, and the C++ probe reports the same data-context advance,
state-machine advance, and list binding results.
