# Data Binding Graph Formula List Fallback Runtime Contract

## Purpose

Admit default-context list sources into the graph-owned
`DataConverterFormula` path for number targets.

C++ treats `ViewModelInstanceList` values as represented data-bind sources, but
`DataConverterFormula::convert` does not evaluate formula tokens for list
inputs. It returns the same early fallback number `0.0` used by other
non-number, non-symbol-list-index inputs.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children feeding `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenInput`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2` for the
  random-token cases.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer with
  a non-zero imported number target default.

## Out Of Scope

- Deterministic list-target `BindablePropertyList` behavior is covered
  separately by
  `data-binding-graph-formula-list-fallback-bindable-list-target-runtime-contract.md`.
- Formula random-function list-target `BindablePropertyList` behavior is
  covered separately by
  `data-binding-graph-formula-random-list-fallback-bindable-list-target-runtime-contract.md`.
- Generated list item creation, artboard component-list instancing, map-rule
  selection, layout, and virtualization.
- `DataConverterListToLength`, which is covered separately.
- `DataConverterNumberToList`, which is covered separately.
- Public-update target-to-source propagation for formula list sources is
  covered separately by
  `data-binding-graph-formula-list-fallback-public-update-target-to-source-runtime-contract.md`.
- Explicit main-`ToSource` target-to-source propagation for formula list
  sources is covered separately by
  `data-binding-graph-formula-list-fallback-explicit-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for deterministic formula
  list sources is covered separately by
  `data-binding-graph-formula-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Random-function target-to-source propagation for formula list sources is
  covered separately by
  `data-binding-graph-formula-random-list-fallback-target-to-source-runtime-contract.md`.
- Random-function target-dirty behavior for formula list sources is covered
  separately by
  `data-binding-graph-formula-random-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Broader target-to-source propagation.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- A real Rust random generator or C++ random seeding/queueing.
- Broader random call-count parity. Random-token zero-pull list fallback
  cases are covered by
  `data-binding-graph-formula-random-list-fallback-call-count-runtime-contract.md`.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default list source is admitted to the number-target formula graph as a
  list-valued source node instead of leaving the target at its imported default.
- `FormulaTokenInput` writes C++'s fallback `0.0` to the number target.
- `FunctionType::random` output tokens are skipped for the list source.
- Skipped random tokens do not consume the host-supplied Rust random stream.
- Random modes `0`, `1`, and `2` all preserve the same fallback behavior.
- Existing list-to-length, non-number formula fallback, and formula-random
  probes continue to pass.
