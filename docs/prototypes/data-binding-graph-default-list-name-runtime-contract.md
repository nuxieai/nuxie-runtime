# Default ViewModel List Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from scalar, symbol-list-index, asset, artboard, and trigger sources to list
sources without admitting view-model name APIs.

This slice resolves a root `ViewModelPropertyList.name` on file view model `0`,
mutates every graph source node with that root source path by replacing the
modeled list item count, and lets ordinary data-context/state-machine
advancement report the changed list size.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyList` name lookup only.
- Mutating graph-owned default list source nodes by item count.
- C++ probe comparison through `ViewModelInstanceRuntime::propertyList(name)`
  with raw `ViewModelInstance::propertyValue(name)` / property-index fallback
  for the file-backed default instance.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard, and
  trigger behavior beyond the existing committed APIs, plus view-model default
  source name APIs.
- Nested, relative, parent, or slash-separated list paths.
- List item identity, item view-model references, item-level traversal,
  generated item instancing beyond count parity, layout, virtualization, or
  stable public list handles.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: mutating the default root list source by property name
produces the same data-context advance, state-machine advance, and list binding
reports as C++, and the existing data-bind-index default list mutation test
continues to pass.
