# Data Binding Graph String Binding Report Runtime Contract

## Purpose

Add exact C++ probe reporting for string data-bind source and target values so
future string-target dirty slices can compare the mutating bind directly instead
of inferring parity only through transition-condition side effects.

## In Scope

- `BindablePropertyString.propertyValue` targets owned by a state-machine
  instance.
- Default view-model `ViewModelInstanceString.propertyValue` sources reachable
  from `DataBindContext.sourcePathIds`.
- `StateMachineInstance` advance and `advancedDataContext()` probe snapshots.
- JSON `stringBindings` reports shaped like existing `numberBindings`: data
  bind index plus nullable source and target values.
- Rust probe deserialization and exact byte comparison for valid JSON string
  values emitted by the C++ probe.

## Out Of Scope

- New data-binding runtime behavior.
- Main-`ToTarget | TwoWay` string target-dirty semantics.
- String values that cannot be represented as valid JSON strings by the C++
  probe writer.
- Enum, asset, artboard, trigger, view-model, and list binding reports.
- Public runtime API design beyond test-only probe comparison helpers.

## Completion Checks

- C++ runtime advance reports include `stringBindings` beside existing
  `numberBindings`.
- Rust probe tests deserialize `stringBindings` with backward-compatible
  defaults.
- At least one default-context string bind compares exact source and target
  values against C++ after the context is bound.
- Focused C++ runtime probe tests and the full comparison gate pass.
