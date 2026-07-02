# Data Binding Graph Formula Random Non-Number Fallback Call Count Runtime Contract

## Purpose

Pin Rust's host-supplied formula random-stream accounting for direct
source-to-target non-number fallback sources.

For boolean, enum, color, string, and trigger sources flowing through a direct
`DataConverterFormula` random token into number targets, C++ reports the
early fallback value `0.0`. Rust mirrors that source-to-target behavior
without pulling from the supplied formula random stream.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue`,
  `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct graph-owned `DataConverterFormula` converters with
  `FormulaTokenFunction` output tokens.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- Source-to-target state-machine advancement through the existing
  `BlendState1DViewModel` fixture.
- `StateMachineInstance::data_bind_formula_random_call_count()` as the Rust
  observable for pulls from the host-supplied formula random stream.
- C++ probe comparisons for the same source-to-target fallback values around
  the call-count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Boolean, enum, color, string, and trigger target-to-source call counts.
- List fallback call counts, covered by
  `data-binding-graph-formula-random-list-fallback-call-count-runtime-contract.md`.
- Symbol-list-index random formula call counts, covered by the direct and
  grouped symbol-list-index call-count contracts.
- Number-source random formula call counts, covered by the direct and grouped
  number call-count contracts.
- Imported, owned, relative, parent, and nested view-model contexts.
- Formula converter groups for non-number fallback sources.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Setting a host random stream resets the Rust non-number fallback call count
  to zero.
- Boolean, enum, color, string, and trigger source-to-target fallback for
  random modes `0`, `1`, and `2` leaves the call count at zero.
- Repeated source-to-target advancement without a source mutation still leaves
  the call count at zero.
- The same fixtures continue to match C++ probe binding values.
