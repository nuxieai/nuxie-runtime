# Imported ViewModel Symbol List Index Name Runtime Contract

Purpose: admit the imported root symbol-list-index property-name mutation API
after the root number, boolean, string, color, and enum property-name slices.

C++ does not expose an obvious `ViewModelInstanceRuntime` convenience accessor
for symbol-list-index properties in the runtime surface audited here. The
observable behavior is still root-name mutation of a file-backed imported
`ViewModelInstance`: resolve the root `ViewModel::property(name)` to its
property index, read `ViewModelInstance::propertyValue(index)`, require a
`ViewModelInstanceSymbolListIndex`, and mutate its `propertyValue` before
binding. Any state machine later bound to the same imported instance observes
the new symbol index.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_symbol_list_index_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertySymbolListIndex`
name against the context's view model, records a symbol-list-index override by
the resolved source path, and lets the existing imported-context bind path apply
that value to every state machine bound through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertySymbolListIndex` name lookup only.
- Mutating the context before binding or rebinding a state machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing symbol-list-index binding report
  surface for integer targets.

Out of scope:

- Asset, artboard, trigger, list, and view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Symbol label/key lookup; this slice mutates by imported value index only.
- Mutating an already-bound state machine through a stable public source
  handle.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported symbol-list-index property can be mutated
by name on one shared imported context, both state machines bound through that
context report the same symbol-list-index source value as C++, and the existing
imported symbol-list-index data-bind-index mutation plus owned
symbol-list-index name tests continue to pass.
