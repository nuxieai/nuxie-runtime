# Default ViewModel ViewModel Source Handle Runtime Contract

Purpose: complete the default-context public source-handle family with
view-model pointer relinking by root property-name handle.

This slice resolves a root `ViewModelPropertyViewModel.name` on file view
model `0` into a stable `RuntimeDefaultViewModelViewModelSourceHandle`.
Relinking through that handle writes the same graph-owned default source path
used by the existing view-model property-name relink API, selects an imported
referenced view-model instance by index, and lets ordinary data-context
advancement update `BindablePropertyViewModel.propertyValue` targets.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyViewModel` name lookup only.
- Public source-handle resolution for the root view-model pointer source path.
- Relinking graph-owned default view-model source nodes to imported referenced
  instance indexes already recorded on the graph source.
- C++ probe comparison against the default view-model by-name relink command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, or list source-handle behavior beyond the existing committed APIs.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Public object handles, replacing generated owned child identities, arbitrary
  user-created runtime view-model instances, list item propagation, reverse
  target-to-source propagation, broader update queues, listener-owned data
  binding, and nested artboard propagation.

Completion condition: resolving and relinking a default root view-model source
by handle produces the same data-context advance, state-machine advance,
source pointer, target pointer, and component update reports as C++ by-name
relink, no-op repeat relinks report unchanged, and slash-path handle lookup
remains unresolved.
