# Data Binding Graph Default ViewModel Nested Asset Runtime Contract

## Purpose

Admit default-context nested asset source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the asset sibling of the nested scalar source slices: C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child asset property. Rust mirrors
that traversal in the runtime data-bind graph while preserving the raw asset
index.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceAssetImage.propertyValue` source feeding
  `BindablePropertyAsset.propertyValue`.
- C++ parity through asset binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested artboard, trigger, list, and view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested asset fixture binds the child asset source and
  matches C++.
- Rust observes the child default asset index through the nested source path.
- Existing owned nested asset name-path coverage continues to pass.
