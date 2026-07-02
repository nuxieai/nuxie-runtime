# Data Binding Graph Formula Public Update Target-To-Source Runtime Contract

## Purpose

Admit direct deterministic `DataConverterFormula` public
`updateDataBinds(true)` target-to-source behavior for a main-`ToTarget |
TwoWay` number bind.

C++ `DataConverterFormula::reverseConvert` delegates to `convert`. This slice
pins that public-update path for a deterministic number formula whose output
queue uses the already admitted `FormulaTokenInput`, `FormulaTokenValue`, and
`FormulaTokenOperation` tokens.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct deterministic `DataConverterFormula` on a main-`ToTarget | TwoWay`
  number data bind.
- Deterministic formula output-queue tokens already modeled by the forward
  formula contract: `FormulaTokenInput`, `FormulaTokenValue`, and
  `FormulaTokenOperation`.
- C++ `DataConverterFormula::reverseConvert` delegating to formula `convert`.
- Immediate source-to-target reapplication after the formula-written source.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior.
- Formula converter groups.
- Symbol-list-index formula public-update reverse behavior is covered
  separately by
  `data-binding-graph-formula-symbol-list-index-public-update-target-to-source-runtime-contract.md`.
- Boolean-source formula fallback public-update reverse behavior is covered
  separately by
  `data-binding-graph-formula-boolean-fallback-public-update-target-to-source-runtime-contract.md`.
- Enum, color, string, and trigger formula fallback public-update reverse
  behavior is covered separately by
  `data-binding-graph-formula-remaining-fallbacks-public-update-target-to-source-runtime-contract.md`.
- List formula fallback public-update reverse behavior is covered separately by
  `data-binding-graph-formula-list-fallback-public-update-target-to-source-runtime-contract.md`.
- Other non-number sources or targets.
- Public-update coverage for interpolator, number-to-list, list, string,
  mixed groups, stateful groups, or scripted converters.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` formula target is passed through
  deterministic formula conversion before writing the default view-model
  number source during public update.
- The same public update reapplies source-to-target so the bindable target
  matches C++ after the source write.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing formula target-to-source and main-`ToTarget | TwoWay`
  state-machine dirty tests still pass.
