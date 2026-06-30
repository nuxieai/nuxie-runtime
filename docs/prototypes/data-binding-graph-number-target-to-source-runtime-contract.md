# Data Binding Graph Number Target-To-Source Runtime Contract

## Purpose

Add the first graph-owned target-to-source runtime path for data binding.

This slice covers a default-context number bind where a mutable
`BindablePropertyNumber.propertyValue` target writes back into its
`ViewModelInstanceNumber.propertyValue` source, then a second source-to-target
bind can observe that updated source during the same explicit data-context
advance.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceNumber.propertyValue` sources.
- Direct `BindablePropertyNumber.propertyValue` targets.
- `DataBind.flags` handling for direct number binds:
  - default `ToTarget` binds continue to run source-to-target;
  - `ToSource | TwoWay` binds mark target-to-source dirty when the bindable
    number target is mutated;
  - explicit `advance_data_context` drains the target-to-source write before
    normal source-to-target application.
- C++ probe coverage using two binds to the same source: the first bind is
  mutated target-to-source, and the second bind feeds an existing
  `BlendState1DViewModel` consumer so the source write is observable.

## Out Of Scope

- Target-to-source for non-number bindables.
- Pure `ToSource` without `TwoWay`.
- Reverse converter execution and converter groups in the target-to-source
  direction.
- Imported and owned view-model contexts.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this scalar
  path.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  render/layout behavior.

## Completion Checks

- Mutating the first two-way number target marks its graph binding
  target-to-source dirty.
- `advance_data_context` copies that target value into the bound default
  source before source-to-target propagation.
- A second bind to the same source observes the updated number and drives the
  same blend-state reports as C++.
- Existing source-to-target default-context number behavior remains unchanged.
