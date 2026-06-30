# Data Binding Graph Enum Target-To-Source Runtime Contract

## Purpose

Extend graph-owned target-to-source runtime behavior to direct enum binds.

This slice mirrors the direct number, boolean, string, and color
target-to-source paths for a default-context enum bind. A mutable
`BindablePropertyEnum.propertyValue` target writes back into its
`ViewModelInstanceEnum.propertyValue` source, and a second source-to-target
bind observes that updated source on the same explicit data-context advance.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceEnum.propertyValue` sources.
- Direct `BindablePropertyEnum.propertyValue` targets.
- `DataBindContext.flags` handling for direct enum binds:
  - default `ToTarget` binds continue to run source-to-target;
  - `ToSource | TwoWay` binds mark target-to-source dirty when the bindable
    enum target is mutated;
  - explicit `advance_data_context` drains the target-to-source write before
    normal source-to-target application.
- C++ probe coverage using two binds to the same source: the first bind is
  mutated target-to-source, and the second bind feeds an existing
  `TransitionViewModelCondition` consumer so the source write is observable.

## Out Of Scope

- Target-to-source for asset, artboard, trigger, symbol-list-index, view-model,
  and list bindables.
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

- Mutating the first two-way enum target marks its graph binding
  target-to-source dirty.
- `advance_data_context` copies that target enum value into the bound default
  source before source-to-target propagation.
- A second bind to the same source observes the updated enum and drives the
  same transition reports as C++.
- Existing source-to-target default-context enum behavior remains unchanged.
