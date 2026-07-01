# Imported ViewModel Trigger Name Runtime Contract

Purpose: admit the imported root trigger property-name mutation API after the
root number, boolean, string, color, enum, symbol-list-index, asset, and
artboard property-name slices.

C++ exposes trigger lookup through `ViewModelInstanceRuntime::propertyTrigger`,
but the public runtime wrapper fires/increments the trigger rather than setting
the imported `propertyValue` count used by the state-machine data-binding
comparison. For this parity slice the observable import-time fact is root-name
mutation of a file-backed imported `ViewModelInstance`: resolve a root trigger
property by name, require a `ViewModelInstanceTrigger`, and mutate its raw
`propertyValue` before binding. A state machine later bound to the same
imported instance observes the trigger count before ordinary advancement can
consume or reset it.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyTrigger` name
against the context's view model, records a trigger override by the resolved
source path, and lets the existing imported-context bind path apply that value
to state machines bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyTrigger` name lookup only.
- Mutating the imported raw trigger count before binding or rebinding a state
  machine.
- Sharing the same mutated context with an observing authored state machine.
- C++ probe comparison through ordinary state-machine advance reports, matching
  the existing trigger shared-mutation slice.

Out of scope:

- List and view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Public trigger fire/dispatch APIs, listener-owned trigger dispatch,
  callback-driven data binding, stable public trigger handles, or event routing.
- Reverse target-to-source propagation, broader update queues, nested artboard
  propagation, cloning, and runtime evaluation beyond applying the override
  during context binding.

Completion condition: a root imported trigger property can be mutated by name
on one shared imported context, the observing state machine bound through that
context reports the same advancement as C++, and the existing imported trigger
data-bind-index mutation plus owned trigger-name tests continue to pass.
