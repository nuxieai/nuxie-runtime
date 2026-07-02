# Data Binding Graph Formula Random List Fallback Call Count Runtime Contract

## Purpose

Pin Rust's host-supplied formula random-stream accounting for direct
default-context list fallback sources.

C++ list fallback formulas produce the observable fallback scalar for the list
input shape. Rust mirrors the existing C++ probe-matched list fallback
fixtures while pinning the host-supplied random stream side effect:
source-to-target and target-dirty list fallback do not pull from the stream,
bindable-list targets do not pull from the stream, and number-target reverse
reapplication consumes one hidden pull before landing on the same observable
fallback value.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children.
- Direct graph-owned `DataConverterFormula` converters with
  `FormulaTokenFunction` output tokens.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- `BindablePropertyNumber.propertyValue` targets used by
  `BlendState1DViewModel`.
- `BindablePropertyList.propertyValue` state-machine targets.
- Source-to-target, explicit target-to-source, public target-to-source, and
  main-`ToTarget | TwoWay` target-dirty scheduling covered by the existing
  list fallback C++ probe fixtures.
- `StateMachineInstance::data_bind_formula_random_call_count()` as the Rust
  observable for pulls from the host-supplied formula random stream.
- C++ probe comparisons for the same fixture values around the call-count
  assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Boolean, enum, color, string, and trigger random fallback call counts.
- Symbol-list-index random formula call counts, covered by the direct and
  grouped symbol-list-index call-count contracts.
- Number-source random formula call counts, covered by the direct and grouped
  number call-count contracts.
- Imported, owned, relative, parent, and nested view-model contexts.
- Formula converter groups for list fallback sources.
- Generated list items, list layout, virtualization, and
  `DataBindListItemConsumer` behavior.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Setting a host random stream resets the Rust list fallback call count to
  zero.
- Source-to-target list fallback for random modes `0`, `1`, and `2` leaves the
  call count at zero.
- Explicit target-to-source list fallback for number targets consumes one
  hidden pull during the mutated data-context pass and reuses that count
  through later normal advances.
- Public target-to-source list fallback for number targets consumes one hidden
  pull during `updateDataBinds(true)` and reuses that count through later
  normal advances.
- Explicit and public target-to-source list fallback for bindable-list targets
  leaves the call count at zero.
- Target-dirty list fallback for number and bindable-list targets leaves the
  call count at zero through preservation and later normal advancement.
- The same fixtures continue to match C++ probe binding values.
