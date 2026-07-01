# Data Binding Graph Scripted Pass-Through Runtime Contract

## Purpose

Admit the non-scripting `ScriptedDataConverter` path in the runtime data-bind
graph as inherited C++ `DataConverter` pass-through behavior.

The C++ probe build used by this repo does not execute converter scripts.
`ScriptedDataConverter` therefore inherits the base `convert` and
`reverseConvert` implementations, which return the input value unchanged. This
slice pins that behavior for state-machine number data binds without admitting
script execution, script assets, or custom converter logic.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct imported `ScriptedDataConverter` on a main-`ToTarget | TwoWay`
  number data bind.
- C++ base `DataConverter::convert` pass-through during normal
  source-to-target application.
- C++ base `DataConverter::reverseConvert` pass-through during public
  `updateDataBinds(true)` target-to-source application.
- Exact C++ probe reporting for source and target values after each explicit
  runtime action.

## Out Of Scope

- Running scripted converter code.
- Script asset loading, script virtual machines, and custom scripted runtime
  behavior.
- Scripted listener actions, scripted inputs, and scripted object dispatch.
- Non-number scripted converter binds.
- Scripted converters inside groups.
- Full C++ dirty-list scheduling, observer-list parity, persisting-list
  ordering, pending add/remove behavior, and re-entry protection.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A default number source flows through `ScriptedDataConverter` unchanged
  during normal source-to-target application.
- A mutated main-`ToTarget | TwoWay` number target flows through
  `ScriptedDataConverter` unchanged during public `updateDataBinds(true)`
  target-to-source application.
- The same public update reapplies the unchanged source-to-target value so the
  Rust source and target reports match C++ after each explicit action.
- Existing direct number and public-update converter probes continue to pass.
