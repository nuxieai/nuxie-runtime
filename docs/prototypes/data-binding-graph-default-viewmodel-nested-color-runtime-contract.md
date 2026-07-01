# Data Binding Graph Default ViewModel Nested Color Runtime Contract

## Purpose

Admit default-context nested color source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the color sibling of the nested scalar source slices: C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child color property. Rust mirrors
that traversal in the runtime data-bind graph while preserving the raw color
value.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceColor.propertyValue` source feeding
  `BindablePropertyColor.propertyValue`.
- C++ parity through color binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested source kinds beyond the enum sibling covered by
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-enum-runtime-contract.md`.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested color fixture binds the child color source and
  matches C++.
- Rust observes the child default color through the nested source path.
- Existing owned nested color name-path coverage continues to pass.
