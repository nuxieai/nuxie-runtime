# Data Binding Graph Pure ToSource Target Runtime Contract

## Purpose

Close the pure `ToSource` flag case for the already admitted direct
target-to-source runtime paths.

The previous direct reverse-binding slices used `ToSource | TwoWay` fixtures.
C++ treats pure `ToSource` as target-to-source-capable but not
source-to-target-capable. Rust already models that flag split in the shared
data-bind flag helpers; this slice adds probe coverage to keep that behavior
locked before moving on to list binding or broader queues.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct `ViewModelInstanceNumber.propertyValue` source feeding a
  `BindablePropertyNumber.propertyValue` target as the representative admitted
  value path.
- A first data bind with pure `ToSource` flags and no `TwoWay` flag.
- A second default `ToTarget` bind to the same source path, used to prove the
  pure `ToSource` write lands before normal source-to-target application.
- C++ probe coverage through the existing bindable-number target mutation and
  blend-state consumer reports.

## Out Of Scope

- Repeating the same flag-only fixture for every already admitted scalar,
  pointer, and trigger value kind.
- List target-to-source behavior.
- Reverse converter execution and converter groups in the target-to-source
  direction.
- Imported and owned contexts for pure `ToSource`.
- Pending dirty queues, pending add/remove behavior, observer-list parity, and
  re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A pure `ToSource` number bind does not require the `TwoWay` flag to mark and
  drain a target-to-source write.
- The first explicit `advance_data_context` does not need to push the source
  into the pure `ToSource` target.
- After target mutation, explicit `advance_data_context` writes the target
  value into the bound default source.
- A second `ToTarget` bind to the same source path observes the updated source
  value and drives the same blend-state reports as C++.
- Existing `ToSource | TwoWay` target-to-source tests still pass.
