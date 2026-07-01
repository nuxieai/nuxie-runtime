# Imported ViewModel Enum Name Runtime Contract

Purpose: admit the imported root enum property-name mutation API after the
root number, boolean, string, and color property-name slices.

C++ exposes this through `ViewModelInstanceRuntime::propertyEnum` on a
file-backed imported `ViewModelInstance`. Mutating that enum source changes
the imported instance before binding, so any state machine later bound to the
same imported instance observes the new value index.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_enum_by_property_name(file,
name, value)`. The method resolves a root enum view-model property name
against the context's view model, records an enum override by the resolved
source path, and lets the existing imported-context bind path apply that value
to every state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyEnum`, `ViewModelPropertyEnumCustom`, and
  `ViewModelPropertyEnumSystem` name lookup only.
- Mutating the context before binding or rebinding a state machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing enum binding report surface.

Out of scope:

- Symbol-list-index, asset, artboard, trigger, list, and view-model
  property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Enum key or label lookup; this slice mutates by value index only.
- Mutating an already-bound state machine through a stable public source
  handle.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported enum property can be mutated by name on
one shared imported context, both state machines bound through that context
report the same enum source value as C++, and the existing imported enum
data-bind-index mutation plus owned enum-name tests continue to pass.
