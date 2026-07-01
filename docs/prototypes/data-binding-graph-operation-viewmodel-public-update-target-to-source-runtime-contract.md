# Data Binding Graph OperationViewModel Public Update Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterOperationViewModel` public `updateDataBinds(true)`
target-to-source behavior for a main-`ToTarget | TwoWay` number bind.

`DataConverterOperationViewModel::reverseConvert` resolves the converter's
secondary view-model number operand, then uses C++ operation-value reverse
arithmetic before writing the primary source. This public-update slice pins
that behavior for the default root context and the already imported secondary
operand.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Primary root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` primary source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterOperationViewModel` converters whose `sourcePathIds`
  resolve against the imported default root view-model instance.
- Secondary operation sources that resolve to `ViewModelInstanceNumber`.
- C++ `DataConverterOperationViewModel::reverseConvert` arithmetic using the
  resolved secondary operand.
- Immediate source-to-target reapplication after the reverse source write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for grouped `DataConverterOperationViewModel`.
- Secondary source mutation dependency dirtiness beyond the direct
  default-context number case covered by
  `docs/prototypes/data-binding-graph-operation-viewmodel-secondary-source-mutation-runtime-contract.md`.
- Recomputing the secondary operand for imported or owned context rebinding.
- Missing, non-number, relative-path, parent-path, or nested secondary
  operation sources.
- Public-update coverage for formula, interpolator, number-to-list, list,
  string, mixed groups, stateful groups, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` operation-view-model target is
  reverse-converted with the imported secondary number operand before writing
  the default view-model primary number source during public update.
- The same public update reapplies source-to-target so the bindable target
  matches C++ after the source write.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing operation-view-model target-to-source and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
