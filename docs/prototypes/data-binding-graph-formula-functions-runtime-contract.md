# Data Binding Graph Formula Functions Runtime Contract

## Purpose

Admit deterministic `FormulaTokenFunction` execution for graph-owned
`DataConverterFormula` number bindings, then prove those same function tokens
through the direct number target-to-source scheduling paths.

This slice extends the existing imported formula output-queue path from
input/value/operation tokens to C++ function tokens whose result is fully
deterministic. It relies on `rive-binary`'s C++ shunting-yard mirror so runtime
execution consumes each function token with the same argument count C++ records
while resolving `DataConverterFormula`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Formula output-queue descriptors exposed by
  `RuntimeFile::data_converter_formula_output_tokens_for_object`.
- Deterministic `FormulaTokenFunction` types `min`, `max`, `round`, `ceil`,
  `floor`, `sqrt`, `pow`, `exp`, `log`, `cosine`, `sine`, `tangent`,
  `acosine`, `asine`, `atangent`, and `atangent2`.
- C++ fallback value `0.0` for unknown non-random function discriminants.
- Direct `ToSource | TwoWay` number target mutation, using function-token
  formula conversion before writing the source.
- Direct main-`ToTarget | TwoWay` public `updateDataBinds(true)`, using C++
  `DataConverterFormula::reverseConvert` delegation to the same function-token
  formula evaluator before same-update source-to-target reapplication.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer and
  number binding reports.

## Out Of Scope

- `FunctionType::random`, random formula values, random cache state, and
  `randomModeValue`.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- Formula converter groups beyond already admitted deterministic
  input/value/operation group shapes.
- Main-`ToTarget | TwoWay` target-dirty scheduling for function-token
  formulas.
- Number-to-list, generated-list, scripted, and live context-aware converters.
- External and owned contexts for this converter/source combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The runtime graph builds formula descriptors from output tokens that preserve
  C++ function argument counts.
- Deterministic formula functions write the same number target values as C++.
- Unknown non-random function types write `0.0` like C++.
- Direct target-to-source and public update paths run function-token formula
  conversion before writing and reapplying the source, matching C++ reports.
- Random function tokens remain unsupported by the graph until a random-source
  state contract exists.
