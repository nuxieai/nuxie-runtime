# Data Binding Graph OperationValue Public Update Target-To-Source Runtime Contract

## Purpose

Admit the first public `updateDataBinds(true)` target-to-source slice for a
main-`ToTarget | TwoWay` number bind.

C++ state-machine bindable-property actions use a narrow dirty path that does
not immediately reverse-convert a main-`ToTarget | TwoWay` target edit during
`advancedDataContext()`. The public `DataBindContainer::updateDataBinds(true)`
path is different: when the same dirty bind is drained with
`applyTargetToSource=true`, C++ calls `reverseConvert` before writing the
view-model source, then reapplies source-to-target during the same update.
The converter-free direct-number variant is covered separately by
`docs/prototypes/data-binding-graph-number-public-update-target-to-source-runtime-contract.md`.
The grouped `DataConverterGroup<OperationValue>` variant is covered separately
by
`docs/prototypes/data-binding-graph-operation-value-group-public-update-target-to-source-runtime-contract.md`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterOperationValue` on a main-`ToTarget | TwoWay` data
  bind.
- C++ `DataConverterOperationValue::reverseConvert` for numeric targets.
- Immediate source-to-target reapplication after the reverse source write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for non-number targets.
- Public-update coverage for direct number binds without converters.
- Public-update coverage for converter groups, range mapper, rounder, system
  operation converters, operation-view-model, formula, interpolator,
  number-to-list, list, string, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- The C++ probe exposes a state-machine action that calls
  `updateDataBinds(true)` and reports the resulting state-machine binding
  snapshot.
- A mutated main-`ToTarget | TwoWay` operation-value target is reverse-converted
  before writing the default view-model number source.
- The same public update reapplies source-to-target so the bindable target
  remains equal to the edited value after reverse conversion.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing state-machine bindable-property target-dirty tests still preserve
  manual edits through `advancedDataContext()` and replay source-to-target only
  on the next normal state-machine advance.
