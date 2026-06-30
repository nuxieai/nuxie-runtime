# Data Binding Graph ToNumber Symbol List Index Converter Runtime Contract

## Purpose

Close the final currently identified `DataConverterToNumber` input-kind slice
for graph-owned default-context bindings: symbol-list-index sources feeding
`BindablePropertyNumber.propertyValue` targets.

This is an index conversion slice, not a list binding slice. It admits the
scalar `ViewModelInstanceSymbolListIndex.propertyValue` source kind to the
existing forward source-to-target converter path.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyNumber.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `DataConverterToNumber` as the direct converter on the data bind.
- C++ symbol-list-index conversion semantics: cast the raw index value to
  `f32`.
- Forward source-to-target writes before existing state-machine/blend-state
  evaluation, using the same graph target application path as prior
  `DataConverterToNumber` slices.
- C++ probe coverage through an observable `BlendState1DViewModel` consumer.

## Out Of Scope

- `BindablePropertyList` and `ViewModelInstanceList` binding.
- Symbol lookup by name or `SymbolType` at runtime.
- List item generation, `DataConverterNumberToList`, and list mutation APIs.
- Public symbol-list source mutation APIs.
- Owned runtime context symbol-list-index APIs.
- Converter groups, formulas, operation converters, interpolators, and
  stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Dedicated external-context probe coverage and listener-owned data-binding
  contexts.
- Nested artboard data-binding propagation.

## Completion Checks

- Runtime graph source nodes can carry default-context symbol-list-index
  sources for `DataConverterToNumber` number-target edges.
- `DataConverterToNumber` converts the raw index to a number before target
  writes.
- Unsupported source kinds for this converter continue to fail closed instead
  of becoming pass-through writes.
- A C++ probe-backed runtime test observes the converted value through a
  `BlendState1DViewModel` consumer.
- Existing no-converter number paths and previously admitted
  boolean/enum/color/string `DataConverterToNumber` paths continue to pass.
