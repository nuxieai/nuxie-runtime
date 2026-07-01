# Imported ViewModel List Name Runtime Contract

Purpose: complete the imported root property-name mutation family for direct
list sources after the number, boolean, string, color, enum, symbol-list-index,
asset, artboard, and trigger name slices.

C++ exposes list lookup through `ViewModelInstanceRuntime::propertyList(name)`.
For this parity slice the observable runtime fact is the imported
`ViewModelInstanceList` item count before binding: resolve one root
`ViewModelPropertyList` by name on a file-backed imported `ViewModelInstance`,
clear the list runtime, add blank item instances until the requested count is
visible, and bind an authored state machine to the same imported instance.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_list_item_count_by_property_name`.
The method resolves a root `ViewModelPropertyList` name against the context's
view model, records a list override by the resolved source path, and lets the
existing imported-context bind path expose the item count through
`BindablePropertyList` source-size reports.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyList` name lookup only.
- Mutating the imported list item count before binding or rebinding a state
  machine.
- Sharing the same mutated context with an observing authored state machine.
- C++ probe comparison through existing bindable-list source-size and target
  reports.

Out of scope:

- Nested, relative, parent, or slash-separated list paths.
- List item identity, item view-model references, item-level traversal, map-rule
  selection, generated list item instancing, layout, or virtualization.
- Public stable list handles and cached `propertyValue` index exposure.
- Reverse target-to-source propagation, broader update queues, listener-owned
  data binding, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported list property can be mutated by name on
one shared imported context, the observing state machine bound through that
context reports the same list source size and advancement sequence as C++, and
the existing direct imported list mutation test continues to pass.
