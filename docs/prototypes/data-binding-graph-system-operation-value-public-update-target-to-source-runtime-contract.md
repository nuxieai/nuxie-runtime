# Data Binding Graph System OperationValue Public Update Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` public `updateDataBinds(true)`
target-to-source behavior for main-`ToTarget | TwoWay` number binds.

These converters choose operation-value arithmetic from the authored data-bind
direction inside both `convert` and `reverseConvert`. For a main-`ToTarget |
TwoWay` public update, C++ dispatches through `reverseConvert`, and the system
converter then delegates to `DataConverterOperationValue::convert`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterSystemNormalizer` converters.
- Direct `DataConverterSystemDegsToRads` converters.
- Imported `operationType` and `operationValue`.
- C++ system-converter direction handling for public reverse conversion on a
  main-`ToTarget | TwoWay` bind.
- Immediate source-to-target reapplication after the system-converted source
  write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `ToSource`-only public-update behavior.
- Public-update coverage for system converter groups.
- Symbol-list-index inputs to system converters.
- Public-update coverage for operation-view-model, formula, interpolator,
  number-to-list, list, string, mixed groups, stateful groups, or scripted
  converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` `DataConverterSystemNormalizer` target is
  system-converted before writing the default view-model number source during
  public update.
- The same behavior is pinned for `DataConverterSystemDegsToRads`.
- The same public update reapplies source-to-target so the bindable target
  matches C++ after the source write.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing system-operation target-to-source and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
