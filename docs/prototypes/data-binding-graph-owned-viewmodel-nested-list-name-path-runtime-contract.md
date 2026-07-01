# Data Binding Graph Owned ViewModel Nested List Name Path Runtime Contract

## Purpose

Admit generated owned nested list name-path binding as the next sibling after
the nested trigger path slice.

C++ can create an owned `ViewModelInstance`, resolve a generated child path,
look up `propertyList("child/items")`, add list item instances, and bind that
owned instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` stores only the list item count needed by the
existing bindable-list report surface, then resolves a slash-separated list
property path through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested list paths whose last segment is a direct list property `name` on the
  generated child.
- `RuntimeOwnedViewModelInstance::set_list_item_count_by_property_name_path`.
- Source-to-target graph binding for direct list data-bind paths longer than
  the root scalar shape, represented as `RuntimeDataBindGraphValue::List`.
- A C++ probe flag that resolves the list path with
  `ViewModelInstanceRuntime::propertyList`, adds enough blank item instances to
  set the observed size without introducing item identity cycles, and binds
  the owned context.

## Out Of Scope

- List item identity, cloned item instances, item-level view-model traversal,
  insertion/removal ordering, `instanceAt`, and stable item handles.
- `DataConverterNumberToList`, `DataConverterListToLength`, reverse
  conversion, and target-to-source list propagation for owned contexts.
- `ArtboardComponentList` item instancing, map-rule selection, layout,
  virtualization, scrolling, or rendering.
- Imported-intermediate nested list reads or writes, shared imported-instance
  mutation, stable public object handles, broader update queues,
  relative/parent/name-based lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- A generated owned nested list can have its item count set by name path before
  binding.
- The matching C++ probe reports the same bindable-list source item count and
  unchanged target `propertyValue` after resolving the list path, adding blank
  item instances, binding, and advancing the data context.
- Existing owned root/nested scalar probes and existing default-context list
  probes continue to pass.
