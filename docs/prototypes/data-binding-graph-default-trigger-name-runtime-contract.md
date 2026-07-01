# Default ViewModel Trigger Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from scalar, symbol-list-index, asset, and artboard sources to trigger sources
without admitting list or view-model name APIs.

This slice resolves a root `ViewModelPropertyTrigger.name` on file view model
`0`, mutates every graph source node with that root source path, updates any
matching cloned default trigger mirror exactly like the existing data-bind-index
trigger source API, and lets ordinary state-machine advancement consume the
raw trigger count.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyTrigger` name lookup only.
- Mutating graph-owned default trigger source nodes as raw trigger counts.
- Updating matching default trigger target mirrors when the mutated source path
  feeds a trigger bindable target.
- C++ probe comparison through raw `ViewModelInstance::propertyValue(name)` /
  property-index lookup for the file-backed default instance.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, and artboard
  behavior beyond the existing committed APIs, plus list or view-model default
  source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Public trigger fire/dispatch APIs, listener-owned trigger dispatch,
  callback-driven data binding, stable public trigger handles, or event routing.
- Target-to-source propagation, converter family expansion, broader update
  queues, and nested artboard propagation.

Completion condition: mutating the default root trigger source by property name
produces the same state-machine advance and component update reports as C++,
and the existing data-bind-index default trigger mutation test continues to
pass.
