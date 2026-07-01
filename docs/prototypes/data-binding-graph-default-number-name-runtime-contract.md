# Default ViewModel Number Name Runtime Contract

Purpose: add the first default-context root property-name source mutation API
without widening the default source family all at once.

The existing default number source mutation path selects a source by authored
state-machine data-bind index. This slice admits the public name shape for one
root number property: resolve `ViewModelPropertyNumber.name` on file view model
`0`, mutate every graph source node with that root source path, and let the
normal default-context dirty path update cloned bindable targets.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyNumber` name lookup only.
- Mutating graph-owned default number source nodes before ordinary
  state-machine advancement.
- C++ probe comparison through
  `ViewModelInstanceRuntime::propertyNumber(name)->value(...)` when available,
  with a raw `ViewModelInstance::propertyValue(name)` fallback for the
  file-backed default instance.

Out of scope:

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model default source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Stable public source handles, target-to-source propagation, converter family
  expansion, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: mutating the default root number source by property name
produces the same state-machine advance and component update reports as C++,
and the existing data-bind-index default number mutation test continues to
pass.
