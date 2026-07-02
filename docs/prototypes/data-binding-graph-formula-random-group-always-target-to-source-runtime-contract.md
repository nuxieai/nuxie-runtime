# Data Binding Graph Formula Random Group Always Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::always` explicit target-to-source scheduling for
default-context number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
inside `DataConverterGroup<OperationValue, Formula(random)>` has
`randomModeValue == 1` and consumes fresh random values for the explicit
target-to-source source write, same-pass source-to-target reapplication, and
later state-machine advancement.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 1`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Explicit target-to-source scheduling through `advance_data_context`.
- Source-to-target reapplication after the target-to-source source write.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped public update target-to-source and grouped target-dirty
  `RandomMode::always` scheduling.
- Grouped `RandomMode::sourceChange` target-to-source/public-update/
  target-dirty scheduling.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- Random call-count parity outside the observed grouped bind.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit target-to-source conversion after a target mutation consumes the
  first supplied random value for the source write.
- Same-pass source-to-target reapplication consumes the next supplied random
  value instead of reusing the source-write value.
- Later source-to-target advances keep consuming fresh supplied random values.
- Existing direct always, grouped default-mode, and grouped always
  source-to-target random tests still pass.
