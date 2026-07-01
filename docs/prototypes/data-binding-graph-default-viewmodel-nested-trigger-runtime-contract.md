# Data Binding Graph Default ViewModel Nested Trigger Runtime Contract

## Purpose

Admit default-context nested trigger source binding through an absolute
`DataBindContext.sourcePathIds` path.

This is the trigger sibling of the nested scalar, asset, and artboard source
slices: C++ `DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child trigger property. Rust mirrors
that traversal in the runtime data-bind graph while preserving the raw trigger
counter value observed by data binding.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceTrigger.propertyValue` source feeding
  `BindablePropertyTrigger.propertyValue`.
- C++ parity through trigger binding reports and the existing transition
  condition consumer.

## Out Of Scope

- Nested source mutation APIs.
- Nested list and view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Listener/callback trigger dispatch, trigger reset/reapply variants beyond
  the existing admitted slices, reverse propagation, broader update queues,
  nested-artboard propagation, layout, and rendering.

## Completion Checks

- The default-context nested trigger fixture binds the child trigger source
  and matches C++.
- Rust observes the child default trigger value through the nested source
  path.
- Existing owned nested trigger name-path coverage continues to pass.
