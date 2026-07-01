# Owned ViewModel Number Source Handle Runtime Contract

Purpose: add the first stable public source handle for a Rust-owned runtime
view-model instance without admitting nested owned handles or a full owned
handle family.

This slice resolves a root `ViewModelPropertyNumber.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelNumberSourceHandle`. Mutating through that handle writes
the same owned-context number storage used by `set_number_by_property_index`
and `set_number_by_property_name`, then ordinary owned-context binding refreshes
matching graph source nodes before state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyNumber` name lookup only.
- Public source-handle resolution for the root owned number property index.
- Mutating owned number storage through that handle before binding the owned
  context to a state machine.
- C++ probe comparison against the existing owned-number runtime context
  command.

Out of scope:

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model owned source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or default view-model contexts.
- Persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned number source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-number binding path, no-op repeat writes report
unchanged, and slash-path handle lookup remains unresolved.
