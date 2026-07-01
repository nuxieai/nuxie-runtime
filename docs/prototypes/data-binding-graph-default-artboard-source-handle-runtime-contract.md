# Default ViewModel Artboard Source Handle Runtime Contract

Purpose: extend the default-context public source-handle family from asset
sources to artboard sources without admitting nested artboard runtime behavior
or broader object-handle APIs.

This slice resolves a root `ViewModelPropertyArtboard.name` on file view model
`0` into a stable `RuntimeDefaultViewModelArtboardSourceHandle`. Mutating
through that handle writes the same graph-owned default source path used by
the existing artboard property-name mutation API, stores a raw artboard index,
and lets the normal default-context dirty path update cloned bindable targets
during ordinary state-machine advancement.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyArtboard` name lookup only.
- Public source-handle resolution for the root artboard source path.
- Mutating graph-owned default artboard source nodes as raw artboard indices
  through that handle before ordinary state-machine advancement.
- C++ probe comparison against the default artboard by-name mutation command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, trigger,
  list, or view-model source-handle behavior beyond the existing committed
  APIs.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Stable public handles for artboard instances, list items, trigger events, or
  view-model instance objects.
- Nested artboard instancing, remapping, draw propagation, host advancement,
  bound view-model instance replacement, renderer hooks, or layout behavior.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: resolving and mutating a default root artboard source by
handle produces the same state-machine advance and component update reports as
C++ by-name mutation, no-op repeat writes report unchanged, and slash-path
handle lookup remains unresolved.
