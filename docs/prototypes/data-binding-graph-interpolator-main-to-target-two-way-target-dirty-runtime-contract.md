# Data Binding Graph Interpolator Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterInterpolator` on main-`ToTarget | TwoWay` number binds.

Because this converter is stateful, the probe first warms C++'s interpolator
startup gate, then mutates the bindable number target. For this
state-machine bindable-property action path, C++ does not immediately run
`reverseConvert` or write the manually edited number target back to the source.
The manual target edit survives explicit `advancedDataContext()`, and the next
normal `StateMachineInstance::advance()` reapplies the direct interpolator's
source-to-target conversion state even when elapsed time is zero.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterInterpolator` on a `TwoWay` number data bind without
  the `ToSource` direction flag.
- Imported `DataConverterInterpolator.duration`.
- Warmed direct interpolator state before target mutation.
- Explicit `advancedDataContext()` preserving the manual target edit.
- Subsequent normal state-machine advancement overwriting the target from the
  direct interpolator source-to-target state even when elapsed time is zero.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `DataConverterInterpolator` inside `DataConverterGroup`.
- Main-`ToSource | TwoWay` target-to-source behavior for
  `DataConverterInterpolator`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Formula, number-to-list, generated-list, scripted, context-aware, relative,
  parent, nested, listener-owned, and nested-artboard converter scheduling.

## Completion Checks

- The probe warms direct interpolator state before mutating the target.
- Mutating the direct interpolator target on a main-`ToTarget | TwoWay` bind
  does not write the target value back to the source.
- Explicit data-context advancement preserves the manual target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target from warmed C++ direct interpolator source-to-target
  state, including when elapsed time is zero.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
