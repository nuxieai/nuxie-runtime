# Data Binding Graph Owned ViewModel Context Runtime Contract

## Purpose

Add the first owned runtime context path for `ViewModelPropertyViewModel`:
an owned `RuntimeOwnedViewModelInstance` can carry a live replacement pointer
for a nested view model and feed a `BindablePropertyViewModel.propertyValue`
target through the existing data-binding graph.

This follows the raw generated-setter slice by covering the C++ relink path
(`ViewModelInstanceRuntime::replaceViewModelByName`) without taking on general
owned nested view-model traversal, lists, listener-owned data binding, or
reverse propagation.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[viewModelIndex,
  propertyIndex]`.
- `ViewModelPropertyViewModel` properties whose referenced view model exists in
  the imported file.
- Public Rust mutation by root property index and referenced instance index.
- A C++ probe flag that creates an owned view-model instance, wraps a
  referenced imported child instance, calls `replaceViewModelByName`, and binds
  the context to a state machine.
- C++ probe coverage through an existing view-model pointer transition
  condition.

## Out Of Scope

- Nested owned view-model paths beyond one root property.
- Unmutated generated nested owned child identity before
  `replaceViewModelByName`.
- Imported external context view-model pointer mutation.
- Default-context live relinking.
- List bindables and list item propagation.
- Reverse target-to-source propagation.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- `RuntimeOwnedViewModelInstance` records `ViewModelPropertyViewModel`
  properties and the imported instance IDs available for the referenced view
  model.
- Owned context resolution can produce `RuntimeDataBindGraphValue::ViewModel`.
- Binding an owned context applies the view-model pointer target before
  transition evaluation with C++ parity.
- Existing default, external, owned scalar, and raw view-model pointer probes
  continue to pass.
