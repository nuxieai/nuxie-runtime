# Imported ViewModel Artboard Name Runtime Contract

Purpose: admit the imported root artboard property-name mutation API after the
root number, boolean, string, color, enum, symbol-list-index, and asset
property-name slices.

C++ exposes artboard lookup through
`ViewModelInstanceRuntime::propertyArtboard`, but the public runtime wrapper
sets a `BindableArtboard` and optional bound view-model instance rather than
the imported `propertyValue` index used by the state-machine data-binding
comparison. For this parity slice the observable import-time fact is root-name
mutation of a file-backed imported `ViewModelInstance`: resolve a root artboard
property by name, require a `ViewModelInstanceArtboard`, and mutate its raw
`propertyValue` before binding. Any state machine later bound to the same
imported instance observes the new artboard index.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyArtboard` name
against the context's view model, records an artboard override by the resolved
source path, and lets the existing imported-context bind path apply that value
to every state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyArtboard` name lookup only.
- Mutating the imported raw artboard index before binding or rebinding a state
  machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing artboard binding report surface.

Out of scope:

- Trigger, list, and view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Nested artboard instancing, remapping, draw propagation, host advancement,
  bound view-model instance replacement, renderer hooks, or stable public
  artboard handles.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, cloning, and runtime evaluation beyond applying the override
  during context binding.

Completion condition: a root imported artboard property can be mutated by name
on one shared imported context, both state machines bound through that context
report the same artboard source value as C++, and the existing imported
artboard data-bind-index mutation plus owned artboard-name tests continue to
pass.
