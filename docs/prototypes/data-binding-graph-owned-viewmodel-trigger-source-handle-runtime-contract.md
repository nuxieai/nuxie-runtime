# Owned ViewModel Trigger Source Handle Runtime Contract

Purpose: document the root owned trigger source handle without changing root
lookup semantics or admitting trigger dispatch, listener-owned events, or the
full owned nested handle family.

This slice resolves a root `ViewModelPropertyTrigger.name` on a
`RuntimeOwnedViewModelInstance` into a
`RuntimeOwnedViewModelTriggerSourceHandle`. Mutating through that handle writes
the same owned-context raw trigger count storage used by
`set_trigger_by_property_index` and `set_trigger_by_property_name`, then
ordinary owned-context binding refreshes matching graph source nodes before
state-machine advancement.

In scope:

- `RuntimeOwnedViewModelInstance::new` contexts for file view model `0`.
- Root `ViewModelPropertyTrigger` name lookup only.
- Public source-handle resolution for the root owned trigger property index.
- Mutating owned trigger count storage through that handle before binding the
  owned context to a state machine.
- C++ probe comparison against the existing owned-trigger runtime context
  command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, and artboard
  behavior beyond the existing committed APIs, plus list or view-model owned
  source-handle behavior beyond the existing committed APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested trigger paths are covered separately by
  `docs/prototypes/data-binding-graph-owned-viewmodel-nested-trigger-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or default view-model contexts.
- Public trigger fire/dispatch APIs, listener-owned trigger dispatch,
  persistent owned-context mutation after binding, reverse target-to-source
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

Completion condition: resolving and mutating a root owned trigger source by
handle produces the same state-machine advance and component update reports as
the existing C++ owned-trigger binding path, no-op repeat writes report
unchanged, and root-name lookup stays separate from slash-path lookup.
