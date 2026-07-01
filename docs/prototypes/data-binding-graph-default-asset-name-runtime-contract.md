# Default ViewModel Asset Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from scalar and symbol-list-index sources to asset image sources without
admitting artboard, trigger, list, or view-model name APIs.

This slice resolves a root `ViewModelPropertyAsset` or
`ViewModelPropertyAssetImage.name` on file view model `0`, mutates every graph
source node with that root source path, and lets the normal default-context
dirty path update cloned bindable targets during ordinary state-machine
advancement.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyAsset` and `ViewModelPropertyAssetImage` name lookup
  only.
- Mutating graph-owned default asset source nodes as raw asset indices before
  ordinary state-machine advancement.
- C++ probe comparison through raw `ViewModelInstance::propertyValue(name)` /
  property-index lookup for the file-backed default instance.

Out of scope:

- Number, boolean, string, color, enum, and symbol-list-index behavior beyond
  the existing committed APIs, plus artboard, trigger, list, or view-model
  default source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Image loading, decoding, `RenderImage*` replacement, renderer hooks, file
  asset helper metadata mutation, or stable public asset handles.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: mutating the default root asset source by property name
produces the same state-machine advance and component update reports as C++,
and the existing data-bind-index default asset mutation test continues to pass.
