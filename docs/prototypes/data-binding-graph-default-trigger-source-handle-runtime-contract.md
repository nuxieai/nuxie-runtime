# Default ViewModel Trigger Source Handle Runtime Contract

Purpose: extend the default-context public source-handle family from artboard
sources to trigger sources without admitting trigger dispatch, listener-owned
events, or broader object-handle APIs.

This slice resolves a root `ViewModelPropertyTrigger.name` on file view model
`0` into a stable `RuntimeDefaultViewModelTriggerSourceHandle`. Mutating
through that handle writes the same graph-owned default source path used by
the existing trigger property-name mutation API, stores a raw trigger count,
updates any matching cloned default trigger mirror, and lets ordinary
state-machine advancement consume the value.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyTrigger` name lookup only.
- Public source-handle resolution for the root trigger source path.
- Mutating graph-owned default trigger source nodes as raw trigger counts
  through that handle before ordinary state-machine advancement.
- Updating matching default trigger target mirrors when the mutated source path
  feeds a trigger bindable target.
- C++ probe comparison against the default trigger by-name mutation command.

Out of scope:

- Number, boolean, string, color, enum, symbol-list-index, asset, artboard,
  list, or view-model source-handle behavior beyond the existing committed
  APIs.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested trigger paths are covered separately by
  `docs/prototypes/data-binding-graph-default-nested-trigger-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or owned view-model contexts.
- Public trigger fire/dispatch APIs, listener-owned trigger dispatch,
  callback-driven data binding, stable trigger event handles, or event routing.
- Target-to-source propagation, converter family expansion, broader update
  queues, listener-owned data binding, and nested artboard propagation.

Completion condition: resolving and mutating a default root trigger source by
handle produces the same state-machine advance and component update reports as
C++ by-name mutation, no-op repeat writes report unchanged, target mirrors
remain in sync, and root-name lookup stays separate from slash-path lookup.
