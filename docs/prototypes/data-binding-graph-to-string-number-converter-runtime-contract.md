# Data Binding Graph ToString Number Converter Runtime Contract

## Purpose

Start the next graph-owned converter family with the smallest observable
`DataConverterToString` slice: default-context number sources feeding
`BindablePropertyString.propertyValue` targets.

This slice admits only number-to-string conversion. It does not attempt to
complete every `DataConverterToString` input kind or the wider string converter
family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `DataConverterToString` as the direct converter on the data bind.
- Imported `DataConverterToString.flags` and `decimals` settings needed for
  C++ number formatting.
- C++ number-to-string formatting semantics already pinned in `nuxie-binary`,
  including rounding, trailing-zero removal, and comma insertion.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.
- Main-`ToTarget | TwoWay` target-dirty behavior for the direct number-to-string
  path is covered by
  `docs/prototypes/data-binding-graph-to-string-number-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- `DataConverterToString` input kinds other than number.
- Color format markers, enum display labels, booleans, triggers, strings, and
  symbol-list indexes.
- String trim/pad/remove-zero converters and converter groups.
- Formulas, operation converters, interpolators, and stateful converter
  advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Listener-owned data-binding contexts and nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context number sources when
  `DataConverterToString` is present.
- The converter descriptor stores the imported formatting settings needed after
  graph construction.
- Converted string values use the same number-formatting model as binary-layer
  `DataConverterToString` conversion.
- A C++ probe-backed runtime test observes the converted value through a string
  transition condition.
- Existing no-converter string and admitted `DataConverterToNumber` paths
  continue to pass.
