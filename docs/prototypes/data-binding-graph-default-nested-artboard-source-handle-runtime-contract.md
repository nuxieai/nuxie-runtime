# Default Nested Artboard Source Handle Runtime Contract

Purpose: extend nested default-context source handles to artboard sources
without admitting nested artboard runtime behavior or broader object-handle
APIs.

Default-context nested artboard source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, scene]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelArtboardSourceHandle` via a separate
`default_view_model_artboard_source_handle_by_property_name_path` API, then
mutates graph-owned default artboard source nodes as raw artboard indices
through the existing source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/scene` from file view model
  `0` into `RuntimeDefaultViewModelArtboardSourceHandle`.
- Mutating graph-owned default artboard source nodes as raw artboard indices
  through that handle before ordinary state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only
  `default_view_model_artboard_source_handle_by_property_name` semantics.
- Trigger, list, or view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Stable public handles for artboard instances, list items, trigger events, or
  view-model instance objects.
- Nested artboard instancing, remapping, draw propagation, host advancement,
  bound view-model instance replacement, renderer hooks, or layout behavior.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: a nested default artboard source handle resolved from
`child/scene` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-artboard intermediate paths
remain unresolved, and the C++ probe reports the same state-machine advance
and component update results.
