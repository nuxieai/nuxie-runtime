# Default Nested Boolean Source Handle Runtime Contract

Purpose: extend nested default-context source handles from number to boolean
without widening the full nested handle family.

Default-context nested boolean source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, enabled]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelBooleanSourceHandle` via a separate
`default_view_model_boolean_source_handle_by_property_name_path` API, then
mutates graph-owned default boolean source nodes through the existing
source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/enabled` from file view
  model `0` into `RuntimeDefaultViewModelBooleanSourceHandle`.
- Mutating graph-owned default boolean source nodes through that handle before
  ordinary state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only `default_view_model_boolean_source_handle_by_property_name`
  semantics.
- String, color, enum, symbol-list-index, asset, artboard, trigger, list, or
  view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a nested default boolean source handle resolved from
`child/enabled` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-boolean intermediate paths
remain unresolved, and the C++ probe reports the same state-machine advance
and component update results.
