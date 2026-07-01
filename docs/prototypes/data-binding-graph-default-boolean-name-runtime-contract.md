# Default ViewModel Boolean Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from number to boolean without admitting the whole default source family at
once.

This slice resolves a root `ViewModelPropertyBoolean.name` on file view model
`0`, mutates every graph source node with that root source path, and lets the
normal default-context dirty path update cloned bindable targets during
ordinary state-machine advancement.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyBoolean` name lookup only.
- Mutating graph-owned default boolean source nodes before ordinary
  state-machine advancement.
- C++ probe comparison through
  `ViewModelInstanceRuntime::propertyBoolean(name)->value(...)` when
  available, with a raw `ViewModelInstance::propertyValue(name)` fallback for
  the file-backed default instance.

Out of scope:

- Number behavior beyond the existing committed API, plus string, color, enum,
  symbol-list-index, asset, artboard, trigger, list, or view-model default
  source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Stable public source handles, target-to-source propagation, converter family
  expansion, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: mutating the default root boolean source by property
name produces the same state-machine advance and component update reports as
C++, and the existing data-bind-index default boolean mutation test continues
to pass.
