# Data Binding Graph ViewModel Bind Source Runtime Contract

## Purpose

Admit the first graph-owned view-model bindable path:
default-context `ViewModelInstanceViewModel.propertyValue` sources feeding
`BindablePropertyViewModel.propertyValue` targets.

This slice proves that the runtime graph can carry view-model pointer identity
as a value without taking on list binding, nested source lookup, owned context
view-model mutation, or general data-context traversal.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyViewModel.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceViewModel.propertyValue` sources whose
  `ViewModelPropertyViewModel.viewModelReferenceId` and `propertyValue` resolve
  to an imported referenced view-model instance.
- Runtime graph source and target nodes carrying view-model pointer identity.
- Forward source-to-target propagation before state-machine transition
  evaluation.
- C++ probe coverage through an existing `TransitionViewModelCondition` pointer
  comparison.

## Out Of Scope

- List bindables and list item propagation.
- Public mutation APIs for view-model pointer sources.
- Owned runtime view-model contexts carrying nested view-model pointer values.
- Relative, parent, and nested source path lookup beyond the imported
  reference resolution already used by the binary layer.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Listener-owned data binding and nested artboard propagation.

## Completion Checks

- `RuntimeDataBindGraphValue` can represent view-model pointer identity.
- `RuntimeDataBindGraphTarget` can write that identity to
  `StateMachineBindableViewModelInstance`.
- Default-context graph construction discovers
  `BindablePropertyViewModel.propertyValue` data-bind sources.
- Pointer equality comparisons distinguish a graph-bound imported view-model
  instance from a null bindable.
- Existing root-context/null pointer comparison probes continue to pass.
