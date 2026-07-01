# Default Nested Trigger Source Handle Runtime Contract

Purpose: extend nested default-context source handles to trigger sources
without admitting trigger dispatch, listener-owned events, or broader
object-handle APIs.

Default-context nested trigger source binding already resolves absolute
`DataBindContext.sourcePathIds` such as `[Root, child, fire]`. This slice
exposes the same resolved path through
`RuntimeDefaultViewModelTriggerSourceHandle` via a separate
`default_view_model_trigger_source_handle_by_property_name_path` API, then
mutates graph-owned default trigger source nodes as raw trigger counts through
the existing source-handle setter.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a generated child path such as `child/fire` from file view model
  `0` into `RuntimeDefaultViewModelTriggerSourceHandle`.
- Mutating graph-owned default trigger source nodes as raw trigger counts
  through that handle before ordinary state-machine advancement.
- Updating matching default trigger target mirrors when the mutated nested
  source path feeds a trigger bindable target.
- C++ probe comparison through the authored `DataBindContext.sourcePathIds`
  mutation path for the matching default-context data bind.

Out of scope:

- Changing root-only `default_view_model_trigger_source_handle_by_property_name`
  semantics.
- List or view-model nested default source handles.
- Relative or parent source lookup.
- Imported or owned view-model contexts beyond their already-admitted handles.
- Public trigger fire/dispatch APIs, listener-owned trigger dispatch,
  callback-driven data binding, stable trigger event handles, or event routing.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: a nested default trigger source handle resolved from
`child/fire` mutates the same graph-owned source path as C++'s
data-bind-index default source mutation for the matching nested data bind,
repeated same-value writes report no change, matching trigger target mirrors
remain in sync, non-trigger intermediate paths remain unresolved, and the C++
probe reports the same state-machine advance and component update results.
