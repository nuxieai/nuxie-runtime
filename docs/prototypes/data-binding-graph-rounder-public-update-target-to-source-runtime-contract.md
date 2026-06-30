# Data Binding Graph Rounder Public Update Target-To-Source Runtime Contract

## Purpose

Admit the direct `DataConverterRounder` public `updateDataBinds(true)`
target-to-source slice for a main-`ToTarget | TwoWay` number bind.

`DataConverterRounder` only overrides C++ `convert`; it inherits the base
`DataConverter::reverseConvert`, which returns the input value unchanged. This
public-update slice pins that behavior: the edited target value is written to
the source unchanged, then source-to-target reapplication runs rounder
`convert` during the same update.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterRounder` on a main-`ToTarget | TwoWay` data bind.
- Imported `DataConverterRounder.decimals`.
- C++ base `DataConverter::reverseConvert` pass-through for numeric targets.
- Immediate source-to-target reapplication through rounder `convert`.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for rounder groups.
- Public-update coverage for non-number targets.
- Public-update coverage for system-operation converters,
  operation-view-model, formula, interpolator, number-to-list, list, string,
  mixed groups, stateful groups, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` rounder target writes the edited target
  value unchanged into the default view-model number source during public
  update.
- The same public update reapplies source-to-target through rounder `convert`,
  so the bindable target becomes the rounded source value.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing rounder target-to-source and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
