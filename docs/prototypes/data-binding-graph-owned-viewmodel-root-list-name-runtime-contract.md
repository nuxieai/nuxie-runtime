# Owned ViewModel Root List Name Runtime Contract

Purpose: cover the root owned list name API as the list counterpart to the
owned root scalar property-name completion slice.

`RuntimeOwnedViewModelInstance` already stores root `ViewModelPropertyList`
facts as item counts and exposes
`set_list_item_count_by_property_name_path`. This slice pins the root path
shape, such as `"items"`, against C++'s
`ViewModelInstanceRuntime::propertyList("items")` before binding the owned
context to a state machine.

In scope:

- Owned root view-model contexts created with
  `RuntimeOwnedViewModelInstance::new`.
- Root `ViewModelPropertyList.name` lookup through a single-segment property
  path.
- Mutating the stored list item count before binding.
- Binding the owned context to an authored state machine and comparing the
  existing `BindablePropertyList` source-size and target reports.

Out of scope:

- List item identity, item-level view-model traversal, generated item
  instances, map-rule selection, layout, or virtualization.
- Stable public list handles and cached `propertyValue` index exposure.
- Reverse target-to-source propagation, broader update queues, listener-owned
  data binding, nested artboard propagation, and runtime evaluation beyond
  applying the owned context during bind.

Completion condition: a root owned list property can be mutated by name before
binding, the state machine observes the same source list size and advancement
sequence as C++, and the existing nested owned list name-path test continues to
pass.
