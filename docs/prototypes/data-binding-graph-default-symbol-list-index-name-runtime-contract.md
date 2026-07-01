# Default ViewModel Symbol List Index Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from the scalar default source family to symbol-list-index without admitting
asset, artboard, trigger, list, or view-model name APIs.

This slice resolves a root `ViewModelPropertySymbolListIndex.name` on file
view model `0`, mutates every graph source node with that root source path,
and lets the normal default-context dirty path update cloned bindable targets
during ordinary state-machine advancement.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertySymbolListIndex` name lookup only.
- Mutating graph-owned default symbol-list-index source nodes before ordinary
  state-machine advancement.
- C++ probe comparison through raw
  `ViewModelInstance::propertyValue(name)` / property-index lookup for the
  file-backed default instance.

Out of scope:

- Number, boolean, string, color, and enum behavior beyond the existing
  committed APIs, plus asset, artboard, trigger, list, or view-model default
  source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Stable public source handles, target-to-source propagation, converter family
  expansion, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: mutating the default root symbol-list-index source by
property name produces the same state-machine advance and component update
reports as C++, and the existing data-bind-index default symbol-list-index
mutation test continues to pass.
