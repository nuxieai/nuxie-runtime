# Data Binding Graph RangeMapper Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for a
main-`ToTarget | TwoWay` number bind whose converter is a direct
`DataConverterGroup` containing `DataConverterRangeMapper`.

C++ drains this public update through `DataConverterGroup::reverseConvert`.
For the representative group in this slice, reverse conversion applies
`DataConverterOperationValue` first, then `DataConverterRangeMapper`, writes
the resulting view-model number source, and reapplies the same grouped
source-to-target conversion during the same update.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterRangeMapper` followed by
  `DataConverterOperationValue`.
- Public-update reverse group order, applying `OperationValue` reverse
  conversion before `RangeMapper` reverse conversion.
- Immediate source-to-target reapplication of the grouped converter after the
  reverse source write.
- Exact C++ probe reporting for the mutating grouped bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Neighboring ordinary `ToTarget` bind scheduling beyond the mutating-bind
  slice; the first same-path observer preservation/follow-up-advance behavior
  is covered by
  `docs/prototypes/data-binding-graph-public-update-observer-preservation-runtime-contract.md`.
- Full C++ dirty-list scheduling, observer-list parity, persisting-list
  ordering, pending add/remove behavior, and re-entry protection.
- Stateful `DataConverterInterpolator` children or mixed stateless/stateful
  groups.
- Resolved-interpolator range-mapper children beyond the already admitted
  primitive transform.
- Formula, number-to-list, generated-list, list, scripted, context-aware, and
  nested converter scheduling.
- Non-number sources or targets.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` grouped range-mapper target is
  reverse-converted in C++ reverse group order before writing the default
  view-model number source.
- The same public update reapplies source-to-target so the grouped bindable
  target matches C++ after the source write.
- The mutating grouped bind's exact source and target values match the C++
  probe after each explicit runtime action.
- Existing direct range-mapper public-update, direct range-mapper
  target-to-source, and range-mapper group main-`ToSource | TwoWay` tests still
  pass.
