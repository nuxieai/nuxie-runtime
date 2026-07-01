# Data Binding Graph Interpolator Converter Group Main-To-Source Target-To-Source Runtime Contract

## Purpose

Admit main-`ToSource | TwoWay` target-to-source behavior for the first
stateful `DataConverterGroup` runtime graph shape.

The admitted group uses `DataConverterOperationValue` followed by
`DataConverterInterpolator`. C++ treats main `ToSource` as the group's main
converter direction, so an edited `BindablePropertyNumber.propertyValue` target
flows through forward group order before writing the default view-model number
source. Even when that grouped conversion leaves the source value numerically
unchanged, the same explicit data-context pass refreshes the visible target
through reverse group order with the existing group child state tree.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToSource | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` then
  `DataConverterInterpolator`.
- Imported `DataConverterInterpolator.duration`.
- Warmed group child interpolator state before the target mutation.
- Explicit `advanceDataContext()` target-to-source scheduling.
- Forward group order before the number source write.
- Same-pass source-to-target refresh through reverse group order even when the
  grouped source value is numerically unchanged.
- C++ `DataConverterInterpolator::reverseConvert` delegating to `convert`
  inside the group reverse pass.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Fresh/unwarmed main-`ToSource | TwoWay` interpolator-group behavior.
- Public `updateDataBinds(true)` behavior for this stateful group, covered by
  `docs/prototypes/data-binding-graph-interpolator-converter-group-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for this stateful group,
  covered by
  `docs/prototypes/data-binding-graph-interpolator-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Stateful groups whose interpolator child is not preceded by the admitted
  `DataConverterOperationValue` child.
- Resolved custom interpolator descriptors beyond the already admitted
  stateful smoothing path.
- Color interpolation targets.
- Formula, number-to-list, generated-list, scripted, context-aware, relative,
  parent, nested, listener-owned, and nested-artboard converter scheduling.
- Broader dirty queues for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  imported or owned view-model contexts.

## Completion Checks

- The probe warms the grouped interpolator state before mutating the target.
- Mutating the grouped main-`ToSource | TwoWay` number target and explicitly
  advancing data context follows C++ main-direction group conversion.
- The same explicit data-context pass refreshes the target through reverse
  group order even when the source value is unchanged.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct interpolator and grouped interpolator source-to-target,
  public-update, and target-dirty tests still pass.
