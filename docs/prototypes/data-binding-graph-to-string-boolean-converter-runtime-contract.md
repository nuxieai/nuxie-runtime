# Data Binding Graph ToString Boolean Converter Runtime Contract

## Purpose

Admit the next narrow `DataConverterToString` input kind:
default-context boolean sources feeding `BindablePropertyString.propertyValue`
targets.

This slice builds on the number-to-string converter path without expanding into
the full string converter family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceBoolean.propertyValue` sources.
- `DataConverterToString` as the direct converter on the data bind.
- C++ boolean-to-string conversion semantics: `true -> "1"` and `false ->
  "0"`.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.

## Out Of Scope

- `DataConverterToString` input kinds other than boolean and the already
  admitted number path.
- Color format markers, enum display labels, triggers, strings, and
  symbol-list indexes.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Listener-owned data-binding contexts and nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context boolean sources when
  `DataConverterToString` is present.
- `DataConverterToString` converts boolean source values before string target
  writes.
- A C++ probe-backed runtime test observes the converted value through a string
  transition condition.
- Existing no-converter string, number-to-string, and admitted
  `DataConverterToNumber` paths continue to pass.
