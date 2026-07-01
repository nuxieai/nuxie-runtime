# Data Binding Graph ViewModel Main-To-Target Observer Runtime Contract

## Purpose

Pin direct view-model pointer behavior for a state-machine
`advanceDataContext()` pass after a dirty main-`ToTarget | TwoWay`
`BindablePropertyViewModel.propertyValue` target edit.

This is the state-machine bindable-property counterpart to the public
`updateDataBinds(true)` view-model observer slice. C++ does not treat the
explicit data-context advance like public target-to-source application here:
the edited target pointer is observable on the mutating bind, while the shared
default `ViewModelInstanceViewModel` source and a neighboring ordinary
`ToTarget` observer target stay unapplied for this fixture until normal
state-machine advancement runs source-to-target again.

## In Scope

- `StateMachineInstance::advanceDataContext()` reached through the C++ probe
  and mirrored by Rust's state-machine runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceViewModel.propertyValue` sources pointing at imported
  child view-model instances.
- Two `BindablePropertyViewModel.propertyValue` targets sharing the same
  source path.
- A dirty mutating main-`ToTarget | TwoWay` direct view-model bind.
- A neighboring ordinary direct `ToTarget` view-model bind to the same source
  path.
- Exact C++ probe reporting for source and target instance indexes after each
  explicit runtime action.

## Out Of Scope

- Public `updateDataBinds(true)` pointer observer behavior; that is covered by
  `docs/prototypes/data-binding-graph-viewmodel-public-update-observer-application-runtime-contract.md`.
- Imported, owned, relative, parent, nested, or listener-owned view-model
  contexts.
- View-model source relink APIs by data-bind index, property name, or name
  path.
- Multiple observers, cross-type observers, pointer comparator transition
  behavior, and nested view-model traversal.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct two-bind fixture.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- The mutating view-model bind preserves the edited target pointer through
  explicit `advanceDataContext()`.
- The same explicit data-context advance does not write the edited pointer
  into the default view-model pointer source.
- The same-path direct observer reports the unchanged source and no target
  pointer after the dirty data-context advance.
- The next normal state-machine advance reapplies source-to-target for the
  mutating and observer binds.
- Both binds' source and target instance indexes match the C++ probe after each
  explicit runtime action.
