# Data Binding Graph Interpolator Public Update Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterInterpolator` public `updateDataBinds(true)`
target-to-source behavior for a warmed main-`ToTarget | TwoWay` number bind.

C++ `DataConverterInterpolator::reverseConvert` delegates to `convert`, so the
public update path uses the same per-bind interpolator state as ordinary
source-to-target conversion. This slice pins the warmed direct-number case
only, after the C++ two-advance startup gate has elapsed.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterInterpolator` on a main-`ToTarget | TwoWay` number
  data bind.
- Imported `DataConverterInterpolator.duration`.
- Warmed direct interpolator state before the target mutation.
- C++ `DataConverterInterpolator::reverseConvert` delegating to `convert`.
- Immediate source-to-target reapplication after the interpolator-written
  source.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Fresh/unwarmed public-update interpolator behavior.
- `DataConverterInterpolator` inside `DataConverterGroup`.
- Resolved custom interpolator descriptors beyond the already admitted direct
  stateful smoothing path.
- Color interpolation targets.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Formula, number-to-list, generated-list, scripted, context-aware, relative,
  parent, nested, listener-owned, and nested-artboard converter scheduling.
- Imported and owned view-model contexts.

## Completion Checks

- The probe warms direct interpolator state before mutating the target.
- A mutated main-`ToTarget | TwoWay` interpolator target is processed through
  the warmed stateful `reverseConvert`/`convert` path before writing the
  default view-model number source during public update.
- The same public update reapplies source-to-target so the bindable target
  matches C++ after the source write.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct interpolator source-to-target and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
