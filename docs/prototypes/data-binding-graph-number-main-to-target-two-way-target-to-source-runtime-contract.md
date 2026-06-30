# Data Binding Graph Number Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ dirty-queue behavior when a two-way number binding's main direction is
`ToTarget`.

C++ does not immediately run `converter->reverseConvert` for a
`ToTarget | TwoWay` bindable target mutation through the state-machine
bindable-property action path. The target edit is observable through explicit
`advancedDataContext()`, then the next normal `StateMachineInstance::advance()`
drains the dirty data bind with `applyTargetToSource=false` and writes the
source-derived value back to the target through `convert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- `TwoWay` number data binds without the `ToSource` direction flag.
- Initial source-to-target flushing through a normal state-machine advance,
  matching C++ `StateMachineInstance::advance()`.
- Direct `DataConverterOperationValue` source-to-target conversion after a
  bindable target mutation dirties the bind.
- `DataConverterGroup` source-to-target conversion through ordered
  `DataConverterOperationValue` children after a bindable target mutation
  dirties the bind.
- Direct `DataConverterRangeMapper` source-to-target conversion after a
  bindable target mutation dirties the bind.
- Direct deterministic `DataConverterFormula` source-to-target conversion after
  a bindable target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-formula-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct system operation-value source-to-target conversion after a bindable
  target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-system-operation-value-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Explicit `advancedDataContext()` preserving the manual target edit before
  the next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToSource` converter dispatch, already covered by the preceding
  direction contract.
- Immediate target-to-source reverse conversion for main-`ToTarget` two-way
  binds; the C++ probe path does not do this.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Non-number target-to-source converter families.
- System-operation converter cases beyond the direct number path, interpolator,
  formula cases beyond the direct deterministic number path, string,
  number-to-list, list, or scripted converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- A `TwoWay` `DataConverterOperationValue` bind applies source-to-target using
  `convert` during state-machine advance, preserves the manual target edit
  through explicit data-context advancement, then overwrites the target from
  the unchanged source on the next normal state-machine advance.
- A `TwoWay` `DataConverterGroup<OperationValue>` bind applies
  source-to-target in forward group order at both normal state-machine advance
  points around a manual target edit.
- A `TwoWay` `DataConverterRangeMapper` bind applies source-to-target through
  `convert` at both normal state-machine advance points around a manual target
  edit.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
