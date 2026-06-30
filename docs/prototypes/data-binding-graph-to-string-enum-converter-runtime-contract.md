# Data Binding Graph ToString Enum Converter Runtime Contract

## Purpose

Pin the C++ runtime graph behavior for default-context enum sources feeding
`BindablePropertyString.propertyValue` targets through `DataConverterToString`.

The direct converter API can convert `DataValueEnum` to a display string when
enum metadata is available. This runtime graph path does not admit that
conversion today: C++ does not transition when a default view-model enum source
is bound through `DataConverterToString` into a string transition condition.
Rust therefore keeps this graph input unsupported for parity.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceEnum.propertyValue` sources with resolvable imported
  `ViewModelPropertyEnumCustom` and `DataEnum` metadata.
- `DataConverterToString` as the direct converter on the data bind.
- C++ probe coverage proving the binding does not produce a matching string
  transition.

## Out Of Scope

- Admitting enum sources to string-target graph bindings.
- Carrying enum display/key tables in runtime graph source values.
- Owned or external view-model enum-to-string contexts.
- Enum source mutation APIs for string-target enum converter binds.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- A C++ probe-backed runtime test covers a default-context enum source,
  `DataConverterToString`, and a string transition condition.
- Rust does not admit the enum source into the string-target graph binding and
  matches C++ by not taking the string condition transition.
- Existing admitted `DataConverterToString` number/boolean/string/trigger,
  symbol-list-index, and color paths continue to pass.
