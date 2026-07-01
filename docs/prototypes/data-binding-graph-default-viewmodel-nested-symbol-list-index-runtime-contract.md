# Data Binding Graph Default ViewModel Nested Symbol-List-Index Runtime Contract

## Purpose

Admit default-context nested symbol-list-index source binding through an
absolute `DataBindContext.sourcePathIds` path.

This is the symbol-list-index sibling of the nested scalar source slices: C++
`DataContext::getViewModelProperty(...)` walks the root
`ViewModelInstanceViewModel.propertyValue` reference to an imported child
`ViewModelInstance`, then reads the final child symbol-list-index property.
Rust mirrors that traversal in the runtime data-bind graph and feeds the
value through the already admitted `DataConverterToString` path.

## In Scope

- Default root view-model context binding through
  `StateMachineInstance::bind_default_view_model_context`.
- A root `ViewModelPropertyViewModel` named `child` that references a child
  view model.
- A default root `ViewModelInstanceViewModel` whose cached reference points to
  an imported child `ViewModelInstance`.
- A child `ViewModelInstanceSymbolListIndex.propertyValue` source feeding a
  string bindable through `DataConverterToString`.
- C++ parity through the existing transition-condition and component-update
  report surfaces.

## Out Of Scope

- Nested source mutation APIs.
- Nested asset, artboard, trigger, list, and view-model value kinds.
- Name-based, relative, and parent paths.
- Imported and owned runtime contexts beyond existing dedicated slices.
- Reverse propagation, broader update queues, listener-owned data binding,
  nested artboards, layout, and rendering.

## Completion Checks

- The default-context nested symbol-list-index fixture binds the child source
  and matches C++.
- Rust observes the child default symbol-list-index through the nested source
  path and existing `DataConverterToString` path.
- Existing owned nested symbol-list-index name-path coverage continues to pass.
