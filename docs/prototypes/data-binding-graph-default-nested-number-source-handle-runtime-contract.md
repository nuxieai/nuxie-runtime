# Default Nested Number Source Handle Runtime Contract

Purpose: admit the first nested default-context source handle by extending the
default number handle API from root names to slash-separated generated child
paths, without widening the full nested handle family.

Default-context nested number source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, amount]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelNumberSourceHandle` via a separate
`default_view_model_number_source_handle_by_property_name_path` API, then
mutates graph-owned default number source nodes through the existing
source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/amount` from file view model
  `0` into `RuntimeDefaultViewModelNumberSourceHandle`.
- Mutating graph-owned default number source nodes through that handle before
  ordinary state-machine advancement.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind. C++ public
  `ViewModelInstanceRuntime::propertyNumber("child/amount")` does not provide
  the parity surface for this shape.

Out of scope:

- Changing root-only `default_view_model_number_source_handle_by_property_name`
  semantics.
- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a nested default number source handle resolved from
`child/amount` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, non-number intermediate paths
remain unresolved, and the C++ probe reports the same state-machine advance
and component update results.
