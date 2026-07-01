# Data Binding Graph Interpolator Main-To-Source Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterInterpolator` main-`ToSource | TwoWay`
target-to-source behavior for warmed default-context number binds.

C++ treats main `ToSource` as the converter's main direction, so an edited
`BindablePropertyNumber.propertyValue` target flows through
`DataConverterInterpolator::convert` before writing the default view-model
number source. Even when that converted value equals the existing source, the
same explicit data-context pass refreshes the visible target through
`DataConverterInterpolator::reverseConvert`, which delegates back to the same
stateful `convert` path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterInterpolator` on a main-`ToSource | TwoWay` number
  data bind.
- Imported `DataConverterInterpolator.duration`.
- Warmed direct interpolator state before the target mutation.
- Explicit `advanceDataContext()` target-to-source scheduling.
- Same-pass source-to-target refresh even when the interpolator-written source
  value is numerically unchanged.
- C++ `DataConverterInterpolator::reverseConvert` delegating to `convert`.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Fresh/unwarmed main-`ToSource | TwoWay` interpolator behavior.
- `DataConverterInterpolator` inside `DataConverterGroup`.
- Public `updateDataBinds(true)` behavior for direct interpolator binds,
  covered by
  `docs/prototypes/data-binding-graph-interpolator-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for direct interpolator binds,
  covered by
  `docs/prototypes/data-binding-graph-interpolator-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Resolved custom interpolator descriptors beyond the already admitted
  stateful smoothing path.
- Color interpolation targets.
- Formula, number-to-list, generated-list, scripted, context-aware, relative,
  parent, nested, listener-owned, and nested-artboard converter scheduling.
- Broader dirty queues for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  imported or owned view-model contexts.

## Completion Checks

- The probe warms direct interpolator state before mutating the target.
- Mutating the direct main-`ToSource | TwoWay` number target and explicitly
  advancing data context follows C++ main-direction interpolator conversion.
- The same explicit data-context pass refreshes the target through the direct
  interpolator reverse/convert path even when the source value is unchanged.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct interpolator source-to-target, public-update, target-dirty,
  and grouped interpolator tests still pass.
