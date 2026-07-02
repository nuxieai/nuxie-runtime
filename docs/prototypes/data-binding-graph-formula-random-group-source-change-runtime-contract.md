# Data Binding Graph Formula Random Group Source Change Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::sourceChange` source-to-target behavior for default-context
number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
inside `DataConverterGroup<OperationValue, Formula(random)>` has
`randomModeValue == 2`, keeps using its cached random value until the bound
source changes, then clears that grouped formula cache and consumes a fresh
random value on the next formula evaluation.

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
- Source mutation through
  `set_default_view_model_number_source_for_data_bind`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped random call-count parity outside the observed grouped bind and
  formula `addDirt` random-cache behavior.
- Grouped target-to-source, public update, and target-dirty scheduling for
  non-default random modes.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Converter dependency invalidation for secondary source paths, including
  `DataConverterOperationViewModel` dependencies.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A grouped formula random bind accepts `randomModeValue == 2`.
- The first source-to-target advance consumes the first supplied random value.
- Mutating the bound default number source clears the nested formula random
  cache.
- The next source-to-target advance consumes the next supplied random value.
- A later source-to-target advance without another source mutation reuses the
  second value.
- Existing default-mode and always-mode grouped random formula tests still
  pass.
