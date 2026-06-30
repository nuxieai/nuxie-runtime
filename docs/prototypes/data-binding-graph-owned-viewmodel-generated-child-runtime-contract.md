# Data Binding Graph Owned ViewModel Generated Child Runtime Contract

## Purpose

Model the default nested view-model identity that C++ creates when
`File::createViewModelInstance(viewModel)` builds an owned root instance with a
`ViewModelPropertyViewModel` child.

This is the companion to the owned replacement slice. The replacement slice
models `replaceViewModelByName`; this slice models the pre-replacement child as
a non-null owned pointer so pointer conditions can distinguish it from `null`.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[viewModelIndex,
  propertyIndex]`.
- `ViewModelPropertyViewModel` properties whose referenced view model exists in
  the imported file.
- A Rust-owned pointer identity that is non-null and stable within the owned
  context.
- C++ probe coverage that binds an owned root instance without calling
  `replaceViewModelByName`.
- Pointer equality/inequality against null through an existing
  `TransitionViewModelCondition`.

## Out Of Scope

- Nested owned view-model paths beyond one root property.
- Equality between generated owned children from different contexts.
- Public APIs for generated child traversal or mutation.
- Default-context and imported-context live relinking.
- List bindables and list item propagation.
- Reverse target-to-source propagation.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Unmutated owned view-model pointer sources resolve as non-null graph values.
- Replacement by referenced imported instance still overwrites the generated
  owned pointer.
- Existing owned scalar, owned replacement, default, and external context probes
  continue to pass.
