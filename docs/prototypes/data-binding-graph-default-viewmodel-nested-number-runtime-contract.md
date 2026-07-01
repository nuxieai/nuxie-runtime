# Data Binding Graph Default ViewModel Nested Number Runtime Contract

## Purpose

Admit default-context nested number source binding through an absolute
`DataBindContext.sourcePathIds` path.

C++ `DataContext::getViewModelProperty(...)` starts at the bound root
`ViewModelInstance`, checks that the first path segment matches the root view
model id, then walks intermediate `ViewModelInstanceViewModel` values before
reading the final property. This slice proves the runtime graph follows that
same traversal for a path such as `[Root, child, amount]`.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceNumber.propertyValue` source feeding
  `BindablePropertyNumber.propertyValue`.
- C++ parity through the existing `BlendState1DViewModel` report surface.

## Out Of Scope

- Nested source mutation APIs.
- Nested boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, list, and view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested number fixture binds the child number source and
  matches C++.
- Rust observes a non-zero child default value through the nested source path.
- Existing owned nested number name-path coverage continues to pass.
