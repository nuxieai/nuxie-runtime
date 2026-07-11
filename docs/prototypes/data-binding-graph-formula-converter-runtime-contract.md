# Data Binding Graph Formula Converter Runtime Contract

## Purpose

Admit the first deterministic `DataConverterFormula` forward-conversion path in
the runtime data-binding graph.

This slice consumes the formula output-queue parity already modeled by
`nuxie-binary` and keeps runtime scope narrow: default-context number sources
feeding number targets can execute formula input/value/operation tokens without
adding randoms, functions, reverse propagation, or formula-owned source binding.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Formula output-queue tokens exposed by
  `RuntimeFile::data_converter_formula_tokens_for_object`.
- Deterministic `FormulaTokenInput`, `FormulaTokenValue`, and
  `FormulaTokenOperation` evaluation.
- C++ stack behavior for formula operation tokens, including the
  input-pass-through result when the stack does not collapse to exactly one
  value.
- C++ arithmetic behavior for formula operations `+`, `-`, `*`, `/`, and
  positive modulo.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Symbol-list-index and non-number formula inputs.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- Target-to-source formula propagation beyond the first direct deterministic
  number case covered by
  `docs/prototypes/data-binding-graph-formula-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` formula target-dirty behavior beyond the direct
  deterministic number case covered by
  `docs/prototypes/data-binding-graph-formula-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Public-queue `DataConverterFormula::reverseConvert` scheduling.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Number-to-list, generated-list, scripted, and live context-aware converters.
- External and owned contexts for this converter.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default number source flows through a direct imported formula before
  state-machine evaluation.
- Formula input/value/operation tokens are interpreted from the imported C++
  output queue, not from serialized infix order.
- The converted number drives the same blend-state report as C++ for each
  deterministic formula operation discriminant covered by the probe.
- Existing direct converter, converter-group, and stateful converter probes
  continue to pass.
