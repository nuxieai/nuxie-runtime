# Data Binding Graph Default ViewModel Nested String Runtime Contract

## Purpose

Admit default-context nested string source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the string sibling of the nested number and boolean slices: C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child string property. Rust mirrors
that traversal in the runtime data-bind graph while preserving raw string
bytes.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceString.propertyValue` source feeding
  `BindablePropertyString.propertyValue`.
- C++ parity through string binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested color, enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested string fixture binds the child string source and
  matches C++.
- Rust observes the child default bytes through the nested source path.
- Existing owned nested string name-path coverage continues to pass.
