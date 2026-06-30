# Data Binding Graph ToString String Converter Runtime Contract

## Purpose

Admit the next narrow `DataConverterToString` input kind:
default-context string sources feeding `BindablePropertyString.propertyValue`
targets.

This slice handles C++ pass-through string conversion for converter-bearing
string binds. It does not expand into the wider string converter family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources with raw bytes preserved by
  `rive-binary`.
- `DataConverterToString` as the direct converter on the data bind.
- C++ string-to-string conversion semantics: preserve and forward the source
  bytes unchanged.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.

## Out Of Scope

- `DataConverterToString` input kinds other than string and the already
  admitted number/boolean paths.
- Color format markers, enum display labels, triggers, and symbol-list indexes.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Listener-owned data-binding contexts and nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context string sources when
  `DataConverterToString` is present.
- `DataConverterToString` preserves string source bytes before string target
  writes.
- A C++ probe-backed runtime test observes the converted value through a string
  transition condition.
- Existing no-converter string, number-to-string, boolean-to-string, and
  admitted `DataConverterToNumber` paths continue to pass.
