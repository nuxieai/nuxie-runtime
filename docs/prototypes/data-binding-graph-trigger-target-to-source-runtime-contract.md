# Data Binding Graph Trigger Target-To-Source Runtime Contract

## Purpose

Extend graph-owned target-to-source runtime behavior to direct trigger binds.

This slice mirrors C++'s direct `ViewModelInstanceTrigger.propertyValue`
reverse path. A mutable `BindablePropertyTrigger.propertyValue` target writes a
raw trigger count back into its default view-model trigger source during
explicit data-context advancement, normal source-to-target bindings observe
that value, and the existing trigger-reset step then clears bound trigger
sources.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceTrigger.propertyValue` sources.
- Direct `BindablePropertyTrigger.propertyValue` targets.
- `DataBindContext.flags` handling for direct trigger binds:
  - default `ToTarget` binds continue to run source-to-target;
  - `ToSource | TwoWay` binds mark target-to-source dirty when the bindable
    trigger target is mutated;
  - explicit `advance_data_context` drains the target-to-source write before
    normal source-to-target application;
  - the existing trigger reset still runs after source-to-target application.
- C++ probe coverage using two binds to the same source: the first bind is
  mutated target-to-source, and the second bind converts the trigger source to
  string for an existing `TransitionViewModelCondition` consumer.

## Out Of Scope

- Pure `ToSource` without `TwoWay`.
- Reverse converter execution and converter groups in the target-to-source
  direction.
- Direct `DataConverterTrigger` main-to-source conversion is covered
  separately by
  `docs/prototypes/data-binding-graph-trigger-converter-target-to-source-runtime-contract.md`.
- Imported and owned view-model contexts.
- Trigger listener dispatch, callback/event side effects, and listener-owned
  data binding.
- Push observer lists, pending dirty queues, pending add/remove behavior, and
  re-entry protection beyond the explicit dirty bit needed for this direct path.
- Relative-path, parent-path, nested-path, nested-artboard, and render/layout
  behavior.

## Completion Checks

- Mutating the first two-way trigger target marks its graph binding
  target-to-source dirty.
- `advance_data_context` copies that target trigger count into the bound
  default source before source-to-target propagation.
- A second bind to the same source observes the updated trigger count before
  the trigger reset step and drives the same transition reports as C++.
- Existing source-to-target default-context trigger and trigger-converter
  behavior remains unchanged.
