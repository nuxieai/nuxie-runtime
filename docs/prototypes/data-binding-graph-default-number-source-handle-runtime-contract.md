# Default ViewModel Number Source Handle Runtime Contract

Purpose: add the first default-context stable public source handle without
widening the default source-handle family all at once.

The existing default number source mutation path can already resolve a root
`ViewModelPropertyNumber.name` and mutate every graph source node sharing that
path. This slice exposes the resolved path as an immutable
`RuntimeDefaultViewModelNumberSourceHandle`, then applies the handle through
the same graph-owned number source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a root `ViewModelPropertyNumber.name` on file view model `0` into
  `RuntimeDefaultViewModelNumberSourceHandle`.
- Mutating graph-owned default number source nodes through the handle before
  ordinary state-machine advancement.
- C++ probe comparison through the existing default number by-name mutation
  command and report surface.

Out of scope:

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model default source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default number source handle resolved from
`amount` mutates the same graph-owned source path as the existing
property-name API, repeated same-value writes report no change, slash-path
lookup stays unresolved, and the C++ probe reports the same state-machine
advance and component update results.
