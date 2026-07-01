# Imported ViewModel Color Name Runtime Contract

Purpose: admit the imported root color property-name mutation API after the
root number, boolean, and string property-name slices.

C++ exposes this through `ViewModelInstanceRuntime::propertyColor` on a
file-backed imported `ViewModelInstance`. Mutating that color source changes
the imported instance before binding, so any state machine later bound to the
same imported instance observes the new value.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_color_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyColor.name`
against the context's view model, records a color override by the resolved
source path, and lets the existing imported-context bind path apply that value
to every state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyColor.name` lookup only.
- Mutating the context before binding or rebinding a state machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing color binding report surface.

Out of scope:

- Enum, symbol-list-index, asset, artboard, trigger, list, and view-model
  property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Mutating an already-bound state machine through a stable public source
  handle.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported color property can be mutated by name on
one shared imported context, both state machines bound through that context
report the same color source value as C++, and the existing imported color
data-bind-index mutation plus owned color-name tests continue to pass.
