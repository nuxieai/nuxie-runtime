# Data Binding Graph ToString Trigger Converter Runtime Contract

## Purpose

Admit the next narrow `DataConverterToString` input kind:
default-context trigger sources feeding `BindablePropertyString.propertyValue`
targets.

This slice handles trigger-count-to-string conversion only. It does not expand
trigger dispatch, listener handling, or the wider string converter family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `DataConverterToString` as the direct converter on the data bind.
- C++ trigger-to-string conversion semantics: convert the raw trigger count to
  decimal text.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.

## Out Of Scope

- `DataConverterToString` input kinds other than trigger and the already
  admitted number/boolean/string paths.
- Trigger dispatch side effects, listener-owned trigger handling, and callback
  target behavior.
- Color format markers, enum display labels, and symbol-list indexes.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context trigger sources when
  `DataConverterToString` is present.
- `DataConverterToString` converts trigger source counts before string target
  writes.
- A C++ probe-backed runtime test observes the converted value through a string
  transition condition.
- Existing no-converter string, number-to-string, boolean-to-string,
  string-to-string, and admitted `DataConverterToNumber` paths continue to
  pass.
