# Data Binding Graph Formula Random Boolean Fallback Target-To-Source Call Count Runtime Contract

## Purpose

Pin Rust's host-supplied formula random-stream accounting for boolean
target-to-source fallback sources.

For boolean sources flowing through a direct `DataConverterFormula` random
token into number targets, C++ keeps the boolean source unchanged and reports
the fallback target value. Rust mirrors those C++ probe values while consuming
one hidden host random value during explicit or public reverse reapplication.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct graph-owned `DataConverterFormula` converters with
  `FormulaTokenFunction` output tokens.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- `StateMachineInstance::advanceDataContext()` for main-`ToSource | TwoWay`
  number binds.
- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds.
- Later normal state-machine advancement after each reverse path.
- `StateMachineInstance::data_bind_formula_random_call_count()` as the Rust
  observable for pulls from the host-supplied formula random stream.
- C++ probe comparisons for the same target-to-source fallback values around
  the call-count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Source-to-target non-number call counts, covered by
  `data-binding-graph-formula-random-non-number-fallback-call-count-runtime-contract.md`.
- Enum, color, string, and trigger target-to-source call counts.
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

- Setting a host random stream resets the boolean fallback call count to zero.
- The initial explicit data-context pass before target mutation leaves the
  call count at zero.
- Explicit target-to-source fallback consumes one hidden pull during the
  mutated data-context pass and reuses that count through later normal
  advances.
- Public target-to-source fallback consumes one hidden pull during
  `updateDataBinds(true)` and reuses that count through later normal advances.
- Random modes `0`, `1`, and `2` all preserve those counts.
- The same fixtures continue to match C++ probe binding values.
