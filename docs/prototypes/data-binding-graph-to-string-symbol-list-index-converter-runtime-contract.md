# Data Binding Graph ToString Symbol List Index Converter Runtime Contract

## Purpose

Admit the next narrow `DataConverterToString` input kind:
default-context symbol-list-index sources feeding
`BindablePropertyString.propertyValue` targets.

This slice handles symbol-list-index-to-string conversion only. It does not
expand public symbol-list source APIs or the wider string converter family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `DataConverterToString` as the direct converter on the data bind.
- C++ symbol-list-index-to-string conversion semantics: convert the raw index
  value to decimal text.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.

## Out Of Scope

- `DataConverterToString` input kinds other than symbol-list index and the
  already admitted number/boolean/string/trigger paths.
- Public symbol-list source mutation APIs.
- Owned runtime context symbol-list-index APIs.
- Color format markers and enum display labels.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context symbol-list-index sources
  when `DataConverterToString` is present.
- `DataConverterToString` converts symbol-list-index source values before
  string target writes.
- A C++ probe-backed runtime test observes the converted value through a
  string transition condition.
- Existing no-converter string, number-to-string, boolean-to-string,
  string-to-string, trigger-to-string, and admitted `DataConverterToNumber`
  paths continue to pass.
