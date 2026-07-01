# Data Binding Graph Default ViewModel Nested ViewModel Runtime Contract

## Purpose

Admit default-context nested view-model pointer source binding through an
absolute `DataBindContext.sourcePathIds` path.

This closes the default-context nested value-kind sibling set. C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child view-model pointer property.
Rust mirrors that traversal in the runtime data-bind graph while preserving
the imported referenced-instance index observed by data binding.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A child `ViewModelPropertyViewModel` named `grandchild` that references a
  grandchild view model.
- Imported child and grandchild `ViewModelInstance` objects, with the child
  `ViewModelInstanceViewModel.propertyValue` selecting the referenced
  grandchild instance.
- A `BindablePropertyViewModel.propertyValue` target fed through the nested
  source path.
- C++ parity through view-model binding reports after explicit data-context
  advancement and state-machine advancement.

## Out Of Scope

- Nested source mutation APIs.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Stable public view-model handles, item-level traversal, reverse
  propagation, broader update queues, component-list instancing, layout,
  nested-artboard propagation, and rendering.

## Completion Checks

- The default-context nested view-model fixture binds the child view-model
  pointer source and matches C++.
- Rust observes the imported grandchild instance index through the nested
  source path.
- Existing owned nested view-model pointer coverage continues to pass.
