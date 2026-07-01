# Data Binding Graph RangeMapper Public Update Target-To-Source Runtime Contract

## Purpose

Admit the direct `DataConverterRangeMapper` public `updateDataBinds(true)`
target-to-source slice for a main-`ToTarget | TwoWay` number bind.

The state-machine bindable-property dirty path preserves a manual
main-`ToTarget | TwoWay` target edit through `advancedDataContext()` and then
replays source-to-target during the next normal state-machine advance. This
public-update slice pins the separate C++ path where the same dirty bind is
drained with `applyTargetToSource=true`, so `DataConverterRangeMapper` runs
`reverseConvert`, writes the mapped source value, then reapplies
source-to-target in the same update.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterRangeMapper` on a main-`ToTarget | TwoWay` data bind.
- Imported `minInput`, `maxInput`, `minOutput`, `maxOutput`, `flags`, and
  `interpolationType` values already admitted by the range-mapper forward and
  reverse primitive contracts.
- C++ `DataConverterRangeMapper::reverseConvert` for numeric targets.
- Immediate source-to-target reapplication after the reverse source write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for range-mapper groups beyond the mutating-bind
  slice covered by
  `docs/prototypes/data-binding-graph-range-mapper-group-public-update-target-to-source-runtime-contract.md`.
- Public-update coverage for resolved-interpolator range mappers beyond the
  already admitted primitive transform.
- Public-update coverage for rounder, system-operation converters,
  operation-view-model, formula, interpolator, number-to-list, list, string,
  mixed groups, stateful groups, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` range-mapped target is reverse-converted
  before writing the default view-model number source.
- The same public update reapplies source-to-target so the bindable target
  remains equal to the edited value after reverse conversion.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing range-mapper target-to-source and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
