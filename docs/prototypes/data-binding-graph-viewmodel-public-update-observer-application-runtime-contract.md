# Data Binding Graph View Model Public Update Observer Application Runtime Contract

## Purpose

Admit direct view-model pointer public `updateDataBinds(true)`
target-to-source behavior and pin the first same-path observer scheduling case.

Unlike the scalar direct value lanes, C++ applies a neighboring same-path
view-model observer target during the same public update. This slice records
that pointer-specific ordering rather than forcing scalar preservation behavior
onto view-model pointers.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceViewModel.propertyValue` sources pointing at imported
  child view-model instances.
- `BindablePropertyViewModel.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct view-model bind.
- Public-update source writes from the edited view-model target pointer.
- Source-to-target reapplication of the mutating bind during the same public
  update.
- A neighboring ordinary direct `ToTarget` view-model bind to the same source
  path.
- Immediate public-update application of the observer target when the shared
  view-model pointer source changes.
- Exact C++ probe reporting for source and target instance indexes after each
  explicit runtime action.

## Out Of Scope

- Imported, owned, relative, parent, nested, or listener-owned view-model
  contexts.
- View-model source relink APIs by data-bind index, property name, or name
  path.
- Multiple observers, cross-type observers, pointer comparator transition
  behavior, and nested view-model traversal.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct view-model public-update path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` view-model target writes the edited
  child instance pointer into the default view-model pointer source during
  public `updateDataBinds(true)`.
- The same public update reapplies source-to-target for the mutating
  view-model bind so Rust and C++ report the same source and target instance
  indexes.
- A same-path direct observer reports the C++ source instance index and updates
  its target instance index during the same public update.
- Previously admitted explicit view-model target-to-source and direct
  value-family observer tests still pass.
