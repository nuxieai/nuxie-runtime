# Data Binding Graph Interpolator Converter Group Main-To-Target Target-Dirty Runtime Contract

## Purpose

Admit state-machine target-dirty behavior for the first stateful
`DataConverterGroup` runtime graph shape on main-`ToTarget | TwoWay` number
binds.

The admitted group uses `DataConverterOperationValue` followed by
`DataConverterInterpolator`. After the grouped interpolator state is warmed, a
manual edit to `BindablePropertyNumber.propertyValue` is preserved through
explicit data-context advancement. The next normal state-machine advance
reapplies the unchanged default view-model number source through forward group
order, using the existing group child state tree.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` then
  `DataConverterInterpolator`.
- Imported `DataConverterInterpolator.duration`.
- Warmed group child interpolator state before the target mutation.
- Explicit `advanceDataContext()` preserving a manually edited bindable target.
- Normal state-machine advance reapplying source-to-target through forward
  group order with the same child state tree.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public `updateDataBinds(true)` behavior for this stateful group, covered by
  `docs/prototypes/data-binding-graph-interpolator-converter-group-public-update-target-to-source-runtime-contract.md`.
- Fresh/unwarmed grouped-interpolator target edits.
- Main-`ToSource | TwoWay` target-to-source behavior for stateful groups.
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
- Mutating the grouped main-`ToTarget | TwoWay` number target and explicitly
  advancing data context preserves the manual target edit.
- The next normal state-machine advance reapplies the unchanged number source
  through forward `OperationValue -> Interpolator` group conversion.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct interpolator, grouped interpolator source-to-target,
  grouped interpolator public-update, and stateless converter-group
  target-dirty tests still pass.
