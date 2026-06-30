# Data Binding Graph OperationValue Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit the first grouped public `updateDataBinds(true)` target-to-source slice
for a main-`ToTarget | TwoWay` number bind.

The direct public-update slice proves that C++ drains the same deferred
main-`ToTarget | TwoWay` dirty bind with `applyTargetToSource=true` by calling
`reverseConvert`, writing the view-model source, then reapplying
source-to-target in the same update. This grouped variant pins the matching
`DataConverterGroup::reverseConvert` order for ordered
`DataConverterOperationValue` children.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` data bind.
- Ordered group items resolving to `DataConverterOperationValue` children.
- C++ `DataConverterGroup::reverseConvert` child order for numeric targets,
  which applies children from last to first before writing the source.
- Immediate source-to-target reapplication after the grouped reverse source
  write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for direct number binds without converters.
- Public-update coverage for mixed-type groups, range mapper, rounder,
  system-operation converters, operation-view-model, formula, interpolator,
  number-to-list, list, string, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` operation-value group target is
  reverse-converted in C++ group reverse order before writing the default
  view-model number source.
- The same public update reapplies source-to-target in forward group order so
  the bindable target remains equal to the edited value after reverse
  conversion.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct public-update and state-machine bindable-property
  target-dirty tests still pass.
