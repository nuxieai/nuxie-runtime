# Data Binding Graph SymbolListIndex Target-To-Source Runtime Contract

## Purpose

Extend graph-owned target-to-source runtime behavior to direct
symbol-list-index sources.

C++ does not define a separate `BindablePropertySymbolListIndex` target. The
reverse path for `ViewModelInstanceSymbolListIndex.propertyValue` is routed
through uint-like targets such as `BindablePropertyInteger.propertyValue`.
This slice models that narrow path: a mutable integer bindable target writes a
raw symbol-list index back into its default view-model source, and a second
bind observes the updated source on the same explicit data-context advance.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- Direct `BindablePropertyInteger.propertyValue` targets for those sources.
- `DataBindContext.flags` handling for direct symbol-list-index binds:
  - default `ToTarget` binds continue to run source-to-target, writing the raw
    symbol-list index into the integer target;
  - `ToSource | TwoWay` binds mark target-to-source dirty when the bindable
    integer target is mutated;
  - explicit `advance_data_context` drains the target-to-source write before
    normal source-to-target application.
- C++ probe coverage using two binds to the same source: the first bind is
  mutated target-to-source, and the second bind converts the same
  symbol-list-index source to string for an existing
  `TransitionViewModelCondition` consumer.

## Out Of Scope

- A distinct `BindablePropertySymbolListIndex` type; C++ has no such runtime
  target in the current schema.
- General integer bindable source families beyond direct symbol-list-index
  sources.
- Target-to-source for trigger, view-model, and list bindables.
- Pure `ToSource` without `TwoWay`.
- Reverse converter execution and converter groups in the target-to-source
  direction.
- Imported and owned view-model contexts.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this direct path.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  render/layout behavior.

## Completion Checks

- Mutating the first two-way integer target marks its graph binding
  target-to-source dirty.
- `advance_data_context` copies that target integer into the bound default
  `ViewModelInstanceSymbolListIndex.propertyValue` source before
  source-to-target propagation.
- A second bind to the same source observes the updated symbol-list-index value
  and drives the same transition reports as C++.
- Existing source-to-target default-context symbol-list-index converter
  behavior remains unchanged.
