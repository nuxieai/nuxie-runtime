# Data Binding Graph Interpolator Converter Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the first
stateful `DataConverterGroup` runtime graph shape.

The admitted group uses `DataConverterOperationValue` followed by
`DataConverterInterpolator` on a main-`ToTarget | TwoWay` number bind. C++
public update runs `DataConverterGroup::reverseConvert` from last child to
first; the interpolator child's `reverseConvert` delegates to its stateful
`convert`, then the operation-value child reverse-converts the interpolated
number before the default view-model source write. The same public update then
reapplies source-to-target through forward group order with the same child
state tree.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` then
  `DataConverterInterpolator`.
- Imported `DataConverterInterpolator.duration`.
- Warmed group child interpolator state before the target mutation.
- C++ reverse group child order for public target-to-source update.
- C++ `DataConverterInterpolator::reverseConvert` delegating to `convert`
  while preserving the group child state tree.
- Same-update source-to-target reapplication through forward group order.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Fresh/unwarmed public-update interpolator-group behavior.
- Main-`ToSource | TwoWay` and state-machine target-dirty behavior for
  stateful converter groups.
- Stateful groups whose interpolator child is not preceded by the admitted
  `DataConverterOperationValue` child.
- Resolved custom interpolator descriptors beyond the already admitted
  stateful smoothing path.
- Color interpolation targets.
- Formula, number-to-list, generated-list, scripted, context-aware, relative,
  parent, nested, listener-owned, and nested-artboard converter scheduling.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.

## Completion Checks

- The probe warms the grouped interpolator state before mutating the target.
- A mutated main-`ToTarget | TwoWay` grouped target is processed through C++
  reverse group order during public update.
- The interpolator child uses the same stateful reverse/convert path as the
  direct interpolator public-update slice.
- The same public update reapplies source-to-target through forward group order
  with the same child state tree.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct interpolator, grouped interpolator source-to-target, and
  stateless converter-group target-to-source tests still pass.
