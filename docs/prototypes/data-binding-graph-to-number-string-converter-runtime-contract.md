# Data Binding Graph ToNumber String Converter Runtime Contract

## Purpose

Close the next narrow runtime graph parity slice for
`DataConverterToNumber`: default-context string sources feeding
`BindablePropertyNumber.propertyValue` targets.

This slice exists to keep converter parity finite. It admits one additional
source kind to the graph-owned forward source-to-target path without expanding
into general converter lifecycle or data-binding scheduler behavior.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyNumber.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources.
- `DataConverterToNumber` as the direct converter on the data bind.
- Raw string bytes preserved by `rive-binary`.
- C++ `std::atof`-style numeric-prefix parsing as modeled by the binary
  crate's data-converter parity helpers.
- Forward source-to-target writes before existing state-machine/blend-state
  evaluation, using the same graph target application path as prior number,
  boolean-to-number, enum-to-number, and color-to-number slices.
- C++ probe coverage through an observable `BlendState1DViewModel` consumer.
- Main-`ToTarget | TwoWay` target-dirty behavior for this string-to-number
  path is covered by
  `docs/prototypes/data-binding-graph-to-number-string-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- Symbol-list index inputs to `DataConverterToNumber`.
- List, symbol, or view-model bindable value types.
- Converter groups, formulas, operation converters, interpolators, and
  stateful converter advancement.
- Reverse target-to-source propagation.
- Main-`ToTarget | TwoWay` target-dirty behavior for other
  `DataConverterToNumber` source kinds.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- External or listener-owned data-binding contexts.
- Nested artboard data-binding propagation.
- Public source-handle APIs beyond the already admitted source mutation paths.

## Completion Checks

- Runtime graph source nodes can carry default-context string sources for
  `DataConverterToNumber` number-target edges.
- Converted string values use the same parser model as binary-layer
  `DataConverterToNumber` conversion.
- Unsupported source kinds for this converter continue to fail closed instead
  of becoming pass-through writes.
- A C++ probe-backed runtime test observes the converted value through a
  `BlendState1DViewModel` consumer.
- Existing no-converter number paths and previously admitted
  boolean/enum/color `DataConverterToNumber` paths continue to pass.
