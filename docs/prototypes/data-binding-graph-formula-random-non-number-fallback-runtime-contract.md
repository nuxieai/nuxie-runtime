# Data Binding Graph Formula Random Non-Number Fallback Runtime Contract

## Purpose

Close the graph-owned non-number `DataConverterFormula` random-function
fallback path for source-to-target state-machine advancement.

C++ returns the formula fallback value `0.0` for represented non-number,
non-symbol-list-index sources before evaluating any formula tokens. That means
a `FunctionType::random` token is ignored for those sources, regardless of the
formula converter's `randomModeValue`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue`,
  `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Source-to-target state-machine advancement through the same
  `BlendState1DViewModel` number-target consumer used by the deterministic
  formula fallback probes.
- C++ probe coverage with a non-zero imported target default and supplied Rust
  random values, proving the target is overwritten with `0.0` before any
  random draw can affect the result.

## Out Of Scope

- List random fallback behavior is covered separately by
  `data-binding-graph-formula-list-fallback-runtime-contract.md`.
- Asset, artboard, view-model pointer, and other source kinds not admitted by
  this number-target graph path.
- Symbol-list-index random formula evaluation, which is covered by the
  symbol-list-index random contracts.
- Number-source random formula evaluation and scheduling, which is covered by
  the direct and grouped random contracts.
- Target-to-source reverse conversion for non-number formula sources.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- A real Rust random generator, C++ random seeding/queueing, or random
  call-count parity.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Boolean, enum, color, string, and trigger sources enter the direct formula
  converter path and write `0.0` to number targets.
- `FunctionType::random` output tokens are skipped for those source kinds.
- Random modes `0`, `1`, and `2` all preserve the same fallback behavior.
- Existing deterministic non-number, number-random, and symbol-list-index
  random formula probes continue to pass.
