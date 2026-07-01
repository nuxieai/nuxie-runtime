# Data Binding Graph Default ViewModel Nested List Runtime Contract

## Purpose

Admit default-context nested list source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the list sibling of the nested scalar, asset, artboard, and trigger
source slices: C++ `DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child list property. Rust mirrors
that traversal in the runtime data-bind graph while preserving the imported
list item count observed by data binding.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceList` plus imported
  `ViewModelInstanceListItem` children feeding `BindablePropertyList`.
- C++ parity through list binding reports after explicit data-context
  advancement and state-machine advancement.

## Out Of Scope

- Nested source mutation APIs.
- Nested view-model value kind.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- List item identity, item-level traversal, generated item instances,
  reverse propagation, broader update queues, component-list instancing,
  layout, virtualization, nested-artboard propagation, and rendering.

## Completion Checks

- The default-context nested list fixture binds the child list source and
  matches C++.
- Rust observes the imported child list item count through the nested source
  path.
- Existing owned nested list name-path coverage continues to pass.
