# Data Binding Graph Default ViewModel Nested Artboard Runtime Contract

## Purpose

Admit default-context nested artboard source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the artboard sibling of the nested scalar and asset source slices:
C++ `DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child artboard property. Rust
mirrors that traversal in the runtime data-bind graph while preserving the raw
artboard index.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceArtboard.propertyValue` source feeding
  `BindablePropertyArtboard.propertyValue`.
- C++ parity through artboard binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested source kinds beyond the trigger sibling covered by
  `docs/prototypes/data-binding-graph-default-viewmodel-nested-trigger-runtime-contract.md`.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Nested-artboard runtime behavior, cloning, remapping, layout, rendering,
  reverse propagation, broader update queues, and listener-owned data binding.

## Completion Checks

- The default-context nested artboard fixture binds the child artboard source
  and matches C++.
- Rust observes the child default artboard index through the nested source
  path.
- Existing owned nested artboard name-path coverage continues to pass.
