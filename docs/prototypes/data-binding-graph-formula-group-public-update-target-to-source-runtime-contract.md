# Data Binding Graph Formula Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for a
main-`ToTarget | TwoWay` number bind whose converter is a direct
`DataConverterGroup` containing a deterministic `DataConverterFormula`.

This slice pins the C++ composition path for a stateless mixed numeric group:
public update calls `DataConverterGroup::reverseConvert`, walks the children
from last to first, runs `DataConverterFormula::reverseConvert` through its
deterministic formula `convert` implementation, writes the default view-model
number source, then reapplies source-to-target through the same group.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` followed by
  deterministic `DataConverterFormula`.
- Formula output-queue tokens already modeled by the direct formula runtime
  contracts: `FormulaTokenInput`, `FormulaTokenValue`, and
  `FormulaTokenOperation`.
- Public-update reverse group order, applying formula reverse conversion before
  operation-value reverse conversion.
- Immediate source-to-target reapplication of the grouped converter after the
  reverse source write.
- Exact C++ probe reporting for the mutating grouped bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior.
- Other formula group compositions, including formula-first groups and groups
  with more than one formula child.
- Stateful formula/interpolator mixed groups beyond the already admitted
  interpolator-group slice.
- Number-to-list, generated-list, list, scripted, context-aware, relative,
  parent, nested, and listener-owned converter scheduling.
- Full C++ dirty-list scheduling, observer-list parity, persisting-list
  ordering, pending add/remove behavior, and re-entry protection.
- Non-number sources or targets.
- Imported and owned view-model contexts.
- Nested artboards and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` grouped formula target is reverse-converted
  in C++ reverse group order before writing the default view-model number
  source.
- The same public update reapplies source-to-target so the grouped bindable
  target matches C++ after the source write.
- The mutating grouped bind's exact source and target values match the C++
  probe after each explicit runtime action.
- Existing direct formula public-update, formula target-to-source, and
  operation-value group public-update tests still pass.
