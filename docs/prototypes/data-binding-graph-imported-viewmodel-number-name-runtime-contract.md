# Imported ViewModel Number Name Runtime Contract

Purpose: admit the first imported scalar property-name mutation API after the
direct data-bind source-path mutation slices.

The C++ runtime exposes this through `ViewModelInstanceRuntime::propertyNumber`
on a file-backed imported `ViewModelInstance`. Mutating that number source
changes the imported instance before it is bound to a state machine, so another
state machine bound to the same imported instance observes the same value.

Rust models the same import-time/live-context fact with
`RuntimeImportedViewModelInstanceContext::set_number_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyNumber.name`
against the context's view model, records a number override by the resolved
source path, and lets the existing imported-context bind path apply that value
to any state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyNumber.name` lookup only.
- Mutating the context before binding or rebinding a state machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing number binding report surface.

Out of scope:

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Mutating an already-bound state machine through a stable public source
  handle.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported number property can be mutated by name on
one shared imported context, both state machines bound through that context
report the same number source value as C++, and the existing imported number
data-bind-index mutation plus owned number-name tests continue to pass.
