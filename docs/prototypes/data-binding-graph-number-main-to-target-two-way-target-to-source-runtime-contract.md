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
- Direct `DataConverterRounder` source-to-target conversion after a bindable
  target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-rounder-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct string-to-number `DataConverterToNumber` source-to-target conversion
  after a bindable target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-to-number-string-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct boolean, enum, color, and symbol-list-index `DataConverterToNumber`
  source-to-target conversion after a bindable target mutation dirties the bind
  is covered by
  `docs/prototypes/data-binding-graph-to-number-scalar-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct `DataConverterListToLength` source-to-target conversion after a
  bindable target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-list-to-length-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct `DataConverterInterpolator` source-to-target conversion after a
  bindable target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-interpolator-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct deterministic `DataConverterFormula` source-to-target conversion after
  a bindable target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-formula-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct system operation-value source-to-target conversion after a bindable
  target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-system-operation-value-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Direct operation-view-model source-to-target conversion after a bindable
  target mutation dirties the bind is covered by
  `docs/prototypes/data-binding-graph-operation-viewmodel-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Explicit `advancedDataContext()` preserving the manual target edit before
  the next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.
- The first same-path ordinary direct `ToTarget` observer for a dirty direct
  `DataConverterOperationValue` bind is covered by
  `docs/prototypes/data-binding-graph-operation-value-main-to-target-observer-runtime-contract.md`.

## Out Of Scope

- Main-`ToSource` converter dispatch, already covered by the preceding
  direction contract.
- Immediate target-to-source reverse conversion for main-`ToTarget` two-way
  binds; the C++ probe path does not do this.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path; the direct no-converter
  public-update variant is covered first by
  `docs/prototypes/data-binding-graph-number-public-update-target-to-source-runtime-contract.md`.
- Full C++ dirty-list scheduling for arbitrary neighboring ordinary
  `ToTarget` binds outside the admitted direct `DataConverterOperationValue`
  same-path observer fixture.
- Non-number target-to-source converter families.
- System-operation, operation-view-model, rounder, and `DataConverterToNumber`
  converter cases beyond the direct paths, interpolator cases beyond the
  direct warmed path, formula cases beyond the direct deterministic number
  path, string, number-to-list, list cases beyond direct list-to-length, or
  scripted converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- A `TwoWay` `DataConverterOperationValue` bind applies source-to-target using
  `convert` during state-machine advance, preserves the manual target edit
  through explicit data-context advancement, then overwrites the target from
  the unchanged source on the next normal state-machine advance.
- A neighboring ordinary direct `ToTarget` number observer for that
  `DataConverterOperationValue` fixture reports the same source and target
  values as C++ after each explicit runtime action.
- A `TwoWay` `DataConverterGroup<OperationValue>` bind applies
  source-to-target in forward group order at both normal state-machine advance
  points around a manual target edit.
- A `TwoWay` `DataConverterRangeMapper` bind applies source-to-target through
  `convert` at both normal state-machine advance points around a manual target
  edit.
- A `TwoWay` `DataConverterRounder` bind applies source-to-target through
  `convert` at both normal state-machine advance points around a manual target
  edit.
- A `TwoWay` string-to-number `DataConverterToNumber` bind applies
  source-to-target through `convert` at both normal state-machine advance
  points around a manual target edit.
- `TwoWay` boolean, enum, color, and symbol-list-index
  `DataConverterToNumber` binds apply source-to-target through `convert` at
  both normal state-machine advance points around a manual target edit.
- A `TwoWay` `DataConverterListToLength` bind applies source-to-target through
  the imported list length at both normal state-machine advance points around a
  manual target edit.
- A `TwoWay` direct `DataConverterInterpolator` bind reapplies warmed
  source-to-target converter state during the normal state-machine advance
  after explicit data-context advancement, including when elapsed time is zero.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
