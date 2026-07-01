# Data Binding Graph OperationValue Main-To-Target Observer Runtime Contract

## Purpose

Pin the first same-path observer behavior for a state-machine
`advanceDataContext()` pass after a dirty direct `DataConverterOperationValue`
main-`ToTarget | TwoWay` number target edit.

C++ treats the bindable target edit as a delayed source-to-target dirty update
for this path. The explicit data-context advance preserves the edited target
on the mutating bind and does not reverse-convert it into the source. A
neighboring ordinary direct `ToTarget` number bind to the same source path must
therefore keep reporting the unchanged source and target values until normal
state-machine advancement runs source-to-target again.

## In Scope

- `StateMachineInstance::advanceDataContext()` reached through the C++ probe
  and mirrored by Rust's state-machine runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source nodes.
- Two `BindablePropertyNumber.propertyValue` targets sharing the same source
  path.
- A dirty mutating main-`ToTarget | TwoWay` direct number bind using
  `DataConverterOperationValue`.
- A neighboring ordinary direct `ToTarget` number bind to the same source path.
- Exact C++ probe reporting for both number binds after each explicit runtime
  action.

## Out Of Scope

- Public `updateDataBinds(true)` scheduling; that is covered by
  `docs/prototypes/data-binding-graph-operation-value-public-update-target-to-source-runtime-contract.md`.
- Grouped `DataConverterGroup<OperationValue>` observer scheduling.
- Other converter families, non-number targets, cross-type observers, multiple
  observers, imported/owned contexts, and relative, parent, nested, or
  listener-owned source paths.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this same-path two-bind fixture.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- The mutating OperationValue bind preserves the target edit through explicit
  `advanceDataContext()`.
- The same explicit data-context advance does not write the edited target back
  into the default number source.
- The same-path direct observer reports the unchanged source and its previous
  target value after the dirty data-context advance.
- The next normal state-machine advance reapplies source-to-target for the
  mutating bind using C++ `DataConverterOperationValue::convert` behavior.
- Both binds' exact source and target reports match the C++ probe after each
  explicit runtime action.
