# Data Binding Graph Operation Pass-Through Runtime Contract

## Purpose

Admit the concrete `DataConverterOperation` base converter in the runtime
data-bind graph as inherited C++ `DataConverter` pass-through behavior.

`DataConverterOperationValue` and `DataConverterOperationViewModel` override
conversion with arithmetic behavior, but the concrete `DataConverterOperation`
base class does not. This slice pins the inherited base `convert` and
`reverseConvert` behavior for state-machine number data binds.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct imported `DataConverterOperation` on a main-`ToTarget | TwoWay`
  number data bind.
- C++ base `DataConverter::convert` pass-through during normal
  source-to-target application.
- C++ base `DataConverter::reverseConvert` pass-through during public
  `updateDataBinds(true)` target-to-source application.
- Exact C++ probe reporting for source and target values after each explicit
  runtime action.

## Out Of Scope

- Arithmetic `DataConverterOperationValue` and `DataConverterOperationViewModel`
  behavior, which have their own converter-specific contracts.
- Operation converter groups beyond already admitted concrete group slices.
- Non-number `DataConverterOperation` binds.
- Full C++ dirty-list scheduling, observer-list parity, persisting-list
  ordering, pending add/remove behavior, and re-entry protection.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A default number source flows through `DataConverterOperation` unchanged
  during normal source-to-target application.
- A mutated main-`ToTarget | TwoWay` number target flows through
  `DataConverterOperation` unchanged during public `updateDataBinds(true)`
  target-to-source application.
- The same public update reapplies the unchanged source-to-target value so the
  Rust source and target reports match C++ after each explicit action.
- Existing direct operation-value, operation-view-model, scripted
  pass-through, and public-update converter probes continue to pass.
