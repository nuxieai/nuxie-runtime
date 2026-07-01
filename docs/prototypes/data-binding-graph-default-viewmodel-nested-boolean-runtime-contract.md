# Data Binding Graph Default ViewModel Nested Boolean Runtime Contract

## Purpose

Admit default-context nested boolean source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the boolean sibling of the nested number slice: C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child boolean property. Rust mirrors
that traversal in the runtime data-bind graph.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceBoolean.propertyValue` source feeding
  `BindablePropertyBoolean.propertyValue`.
- C++ parity through boolean binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested boolean fixture binds the child boolean source and
  matches C++.
- Rust observes a true child default value through the nested source path.
- Existing owned nested boolean name-path coverage continues to pass.
