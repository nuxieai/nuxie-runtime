# Data Binding Graph Formula Random Group Source-Change Public Update Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::sourceChange` public target-to-source scheduling for
default-context number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
inside `DataConverterGroup<OperationValue, Formula(random)>` has
`randomModeValue == 2`, warms its cached random value during the initial
source-to-target pass, reuses that value for the public target-to-source
source write, treats the changed source as a source change, clears the nested
formula cache, and consumes a fresh value for same-update source-to-target
reapplication.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 2`.
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
- Grouped explicit target-to-source `RandomMode::sourceChange` scheduling is
  covered separately by
  `data-binding-graph-formula-random-group-source-change-target-to-source-runtime-contract.md`.
- Grouped target-dirty `RandomMode::sourceChange` scheduling is covered
  separately by
  `data-binding-graph-formula-random-group-source-change-target-dirty-runtime-contract.md`.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- Secondary converter dependency invalidation beyond the observed grouped
  source write.
- Grouped public-update host random call-count parity is covered separately by
  `data-binding-graph-formula-random-group-public-update-call-count-runtime-contract.md`.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The initial source-to-target advance consumes the first supplied random
  value.
- Public target-to-source conversion after a target mutation reuses the warmed
  source-change random value for the source write.
- A changed source clears the nested source-change formula random cache.
- Same-update source-to-target reapplication consumes the next supplied random
  value instead of reusing the source-write value.
- Later source-to-target advances reuse that second value until another source
  change.
- Existing grouped default-mode, grouped always, grouped source-change
  source-to-target, and grouped source-change explicit target-to-source random
  tests still pass.
