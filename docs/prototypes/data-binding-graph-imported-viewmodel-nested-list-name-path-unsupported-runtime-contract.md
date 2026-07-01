# Data Binding Graph Imported ViewModel Nested List Name-Path Unsupported Runtime Contract

## Purpose

Pin the imported-context nested list property-name path boundary.

C++ probe mutation through
`--runtime-set-view-model-instance-source-list-by-name` uses
`ViewModelInstanceRuntime::propertyList(path)` for list values. Root list names
are usable, but the current C++ probe crashes when asked to mutate a nested
slash path such as `child/items` for this synthetic imported-context fixture.
Rust mirrors the safe boundary by refusing
`RuntimeImportedViewModelInstanceContext::
set_list_item_count_by_property_name_path` for slash paths.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyList` leaf.
- An explicit `false` result from
  `RuntimeImportedViewModelInstanceContext::
  set_list_item_count_by_property_name_path` for `child/items`.
- C++ probe baseline coverage showing the unmutated nested list source still
  binds with the expected item count.

## Out Of Scope

- Invoking the crashing C++ nested list-by-name mutation path.
- Adding a replacement C++ list mutation probe for nested list paths.
- Public list item identity, stable list item handles, list layout/virtualized
  instancing, reverse propagation, broader update queues,
  relative/parent/name-manifest lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- Rust returns `false` for `child/items` and keeps the original imported nested
  list source item count.
- The C++ baseline and Rust baseline list binding source and target reports
  stay equal for the bound observing state machine.
- The crash-prone C++ mutation action is documented as a boundary, not carried
  as a failing runtime test.
