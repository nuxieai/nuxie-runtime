# Imported ViewModel String Name Runtime Contract

Purpose: admit the imported root string property-name mutation API after the
root number and boolean property-name slices.

C++ exposes this through `ViewModelInstanceRuntime::propertyString` on a
file-backed imported `ViewModelInstance`. Mutating that string source changes
the imported instance before binding, so any state machine later bound to the
same imported instance observes the new value.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_string_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyString.name`
against the context's view model, records a string override by the resolved
source path, and lets the existing imported-context bind path apply that value
to every state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyString.name` lookup only.
- Mutating the context before binding or rebinding a state machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing string binding report surface.

Out of scope:

- Color, enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Mutating an already-bound state machine through a stable public source
  handle.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported string property can be mutated by name on
one shared imported context, both state machines bound through that context
report the same string source value as C++, and the existing imported string
data-bind-index mutation plus owned string-name tests continue to pass.
