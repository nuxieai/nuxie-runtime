# Default Nested Symbol-List-Index Source Handle Runtime Contract

Purpose: extend nested default-context source handles from the scalar source
kinds to symbol-list-index without widening asset/artboard/trigger/list/
view-model nested handles.

Default-context nested symbol-list-index source binding already resolves
absolute `DataBindContext.sourcePathIds` such as `[Root, child, symbol]`.
This slice exposes the same resolved path through
`RuntimeDefaultViewModelSymbolListIndexSourceHandle` via a separate
`default_view_model_symbol_list_index_source_handle_by_property_name_path`
API, then mutates graph-owned default symbol-list-index source nodes through
the existing source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/symbol` from file view model
  `0` into `RuntimeDefaultViewModelSymbolListIndexSourceHandle`.
- Mutating graph-owned default symbol-list-index source nodes through that
  handle before ordinary state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only
  `default_view_model_symbol_list_index_source_handle_by_property_name`
  semantics.
- Asset, artboard, trigger, list, or view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a nested default symbol-list-index source handle
resolved from `child/symbol` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-symbol-list-index
intermediate paths remain unresolved, and the C++ probe reports the same
state-machine advance and component update results.
