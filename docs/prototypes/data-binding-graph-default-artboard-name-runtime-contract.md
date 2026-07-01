# Default ViewModel Artboard Name Runtime Contract

Purpose: extend the default-context root property-name source mutation seam
from scalar, symbol-list-index, and asset sources to artboard sources without
admitting trigger, list, or view-model name APIs.

This slice resolves a root `ViewModelPropertyArtboard.name` on file view model
`0`, mutates every graph source node with that root source path, and lets the
normal default-context dirty path update cloned bindable targets during
ordinary state-machine advancement.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyArtboard` name lookup only.
- Mutating graph-owned default artboard source nodes as raw artboard indices
  before ordinary state-machine advancement.
- C++ probe comparison through raw `ViewModelInstance::propertyValue(name)` /
  property-index lookup for the file-backed default instance.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, and asset behavior
  beyond the existing committed APIs, plus trigger, list, or view-model
  default source name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Nested artboard instancing, remapping, draw propagation, host advancement,
  bound view-model instance replacement, renderer hooks, or stable public
  artboard handles.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: mutating the default root artboard source by property
name produces the same state-machine advance and component update reports as
C++, and the existing data-bind-index default artboard mutation test continues
to pass.
