# Owned ViewModel String Source Handle Runtime Contract

Purpose: extend the owned runtime view-model source-handle family from boolean
sources to string sources without admitting nested owned handles or the
remaining owned handle kinds.

This slice resolves a root `ViewModelPropertyString.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelStringSourceHandle`. Mutating through that handle writes
the same owned-context raw string bytes used by `set_string_by_property_index`
and `set_string_by_property_name`, then ordinary owned-context binding
refreshes matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyString` name lookup only.
- Public source-handle resolution for the root owned string property index.
- Mutating owned string storage through that handle before binding the owned
  context to a state machine.
- C++ probe comparison against the existing owned-string runtime context
  command.

Out of scope:

- Number and boolean behavior beyond the existing committed APIs, plus color,
  enum, symbol-list-index, asset, artboard, trigger, list, or view-model owned
  source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned string source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-string binding path, no-op repeat writes report
unchanged, and slash-path handle lookup remains unresolved.
