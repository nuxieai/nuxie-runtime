# Data Binding Graph Formula Random Group Always Public Update Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::always` public target-to-source scheduling for default-context
number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
inside `DataConverterGroup<OperationValue, Formula(random)>` has
`randomModeValue == 1` and consumes fresh random values across the initial
source-to-target pass, public target-to-source source write, same-update
source-to-target reapplication, and later state-machine advancement.

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
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source`.
- Same-update source-to-target reapplication after the public target-to-source
  source write.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped explicit target-to-source `RandomMode::always` scheduling is covered
  separately by
  `data-binding-graph-formula-random-group-always-target-to-source-runtime-contract.md`.
- Grouped target-dirty `RandomMode::always` scheduling.
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

- The initial source-to-target advance consumes the first supplied random
  value.
- Public target-to-source conversion after a target mutation consumes the next
  supplied random value for the source write.
- Same-update source-to-target reapplication consumes another supplied random
  value instead of reusing the source-write value.
- Later source-to-target advances keep consuming fresh supplied random values.
- Existing direct always, grouped default-mode, grouped always source-to-target,
  and grouped always explicit target-to-source random tests still pass.
