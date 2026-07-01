# Default ViewModel List Source Handle Runtime Contract

Purpose: extend the default-context public source-handle family from trigger
sources to list sources without admitting list item identity, item traversal,
or broader object-handle APIs.

This slice resolves a root `ViewModelPropertyList.name` on file view model `0`
into a stable `RuntimeDefaultViewModelListSourceHandle`. Mutating through that
handle writes the same graph-owned default source path used by the existing
list property-name mutation API, replaces the modeled list item count, and lets
ordinary data-context/state-machine advancement report the changed list size.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyList` name lookup only.
- Public source-handle resolution for the root list source path.
- Mutating graph-owned default list source nodes by item count through that
  handle before ordinary data-context/state-machine advancement.
- C++ probe comparison against the default list by-name mutation command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, or view-model source-handle behavior beyond the existing committed
  APIs.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- List item identity, item view-model references, item-level traversal,
  generated item instancing beyond count parity, layout, virtualization, or
  stable public list item handles.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: resolving and mutating a default root list source by
handle produces the same data-context advance, state-machine advance, and list
binding reports as C++ by-name mutation, no-op repeat writes report unchanged,
and slash-path handle lookup remains unresolved.
