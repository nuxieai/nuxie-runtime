# Data Binding Graph Operation ViewModel Converter Runtime Contract

## Purpose

Admit the first runtime graph `DataConverterOperationViewModel` path for
default-context number sources feeding number targets.

This slice verifies that graph-owned data binding can use a second imported
default view-model number as the operation operand before state-machine
evaluation.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Primary root-only `DataBindContext.sourcePathIds` of shape
  `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` primary sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterOperationViewModel` converters whose
  `sourcePathIds` resolve against the imported default root view-model
  instance.
- Secondary operation sources that resolve to `ViewModelInstanceNumber`.
- C++ forward arithmetic using the already-audited operation-value helper.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- Target-to-source propagation beyond the direct main-`ToSource | TwoWay`
  number case covered by
  `docs/prototypes/data-binding-graph-operation-viewmodel-target-to-source-runtime-contract.md`.
- Public-queue reverse conversion.
- Live dependency/dirt propagation when the secondary operation source changes.
- Recomputing the secondary operand for imported or owned context rebinding.
- Missing, non-number, relative-path, parent-path, or nested secondary
  operation sources.
- Dedicated grouped `DataConverterOperationViewModel` parity coverage beyond
  the existing generic group executor.
- Formula, interpolator, number-to-list, and scripted converters.
- Listener-owned data binding and nested artboard propagation.

## Completion Checks

- A default number source flows through `DataConverterOperationViewModel`.
- The converter resolves its secondary operand from the imported default root
  view-model instance.
- The converted number drives the same blend-state report as C++.
