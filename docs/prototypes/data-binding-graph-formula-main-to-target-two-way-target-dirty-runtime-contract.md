# Data Binding Graph Formula Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ dirty-path behavior for a deterministic `DataConverterFormula` on a
main-`ToTarget | TwoWay` number bind.

For this state-machine bindable-property action path, C++ does not immediately
run `DataConverterFormula::reverseConvert` or write the manually edited target
back to the source. The manual target edit survives explicit
`advancedDataContext()`, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion with `DataConverterFormula::convert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct deterministic `DataConverterFormula` on a `TwoWay` number data bind
  without the `ToSource` direction flag.
- Deterministic formula output-queue tokens already modeled by the forward
  formula contract: `FormulaTokenInput`, `FormulaTokenValue`, and
  `FormulaTokenOperation`.
- Initial source-to-target formula flushing through a normal state-machine
  advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` formula conversion, covered by
  `docs/prototypes/data-binding-graph-formula-target-to-source-runtime-contract.md`.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path; the direct deterministic
  formula public-update variant is covered by
  `docs/prototypes/data-binding-graph-formula-public-update-target-to-source-runtime-contract.md`.
- Exact dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Broader `DataConverterFormula::reverseConvert` behavior through public
  data-bind update queues.
- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior.
- Formula converter groups.
- Symbol-list-index, non-number, list, generated-list, scripted, and live
  context-aware formula paths.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the deterministic formula
  output from the unchanged source to the bindable number target.
- Mutating the formula-bound target on a main-`ToTarget | TwoWay` bind preserves
  the manual value through explicit data-context advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged source using deterministic formula `convert`.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
