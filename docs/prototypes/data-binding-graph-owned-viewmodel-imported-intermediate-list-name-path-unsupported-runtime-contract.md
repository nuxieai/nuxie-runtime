# Data Binding Graph Owned ViewModel Imported-Intermediate List Name-Path Unsupported Runtime Contract

## Purpose

Pin the public list name-path mutation boundary for owned view-model contexts
after a generated child has been replaced with an imported child instance.

C++ can bind through the imported child's existing `ViewModelInstanceList`
item count, but an attempted public `child/items` list mutation after
replacing `child` with an imported instance leaves the observed graph source
unchanged. Rust keeps the same boundary:
`RuntimeOwnedViewModelInstance::set_list_item_count_by_property_name_path`
returns `false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested list source paths such as `child/items`.
- The attempted C++ public path shape that calls
  `ViewModelInstanceRuntime::propertyList("child/items")` after replacing the
  generated child with an imported child and appends blank item runtimes.
- Verifying that both C++ and Rust preserve the imported child's existing list
  item count after data-context advancement and state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- List item identity, item-level view-model traversal, generated item
  instances, list layout/virtualization, view-model pointer name-path mutation
  boundaries, stable public object handles, reverse propagation, broader
  update queues, relative/parent/name lookup, listener-owned data binding, and
  nested artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to append
  items through `child/items`, binds the owned context, and still behaves as
  if the imported child's original list item count is selected.
- Rust rejects the same mutation through
  `set_list_item_count_by_property_name_path("child/items", count)` after
  `child` is imported.
- The list binding source-size and target-size reports stay equal between C++
  and Rust.
