# Data Binding Graph ToString Color Converter Runtime Contract

## Purpose

Admit the next narrow `DataConverterToString` input kind:
default-context color sources feeding `BindablePropertyString.propertyValue`
targets.

This slice handles color-to-string conversion through the imported
`DataConverterToString.colorFormat` descriptor. It does not expand enum label
lookup or the wider string converter family.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceColor.propertyValue` sources.
- `DataConverterToString` as the direct converter on the data bind.
- Imported `DataConverterToString.colorFormat` bytes on the runtime graph
  converter descriptor.
- C++ color-to-string conversion semantics already pinned in `nuxie-binary`,
  including empty-format signed integer fallback and `%r/%g/%b/%a/%R/%G/%B/%A`
  marker expansion.
- Forward source-to-target writes before existing state-machine transition
  evaluation.
- C++ probe coverage through an observable string
  `TransitionViewModelCondition` consumer.
- Main-`ToTarget | TwoWay` target-dirty behavior for the direct
  color-to-string path is covered by
  `docs/prototypes/data-binding-graph-to-string-color-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- `DataConverterToString` input kinds other than color and the already admitted
  number/boolean/string/trigger/symbol-list-index paths.
- Enum display labels and missing-enum fallback.
- Public color or symbol-list source mutation APIs beyond existing default
  context graph binding.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- String graph source nodes can carry default-context color sources when
  `DataConverterToString` is present.
- `RuntimeDataBindGraphConverter::ToString` carries imported `colorFormat`
  bytes.
- `DataConverterToString` converts color source values before string target
  writes.
- A C++ probe-backed runtime test observes the converted value through a
  string transition condition.
- Existing no-converter string, number-to-string, boolean-to-string,
  string-to-string, trigger-to-string, symbol-list-index-to-string, and
  admitted `DataConverterToNumber` paths continue to pass.
