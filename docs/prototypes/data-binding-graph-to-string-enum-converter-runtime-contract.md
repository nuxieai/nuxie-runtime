# Data Binding Graph ToString Enum Converter Runtime Contract

## Purpose

Pin the C++ runtime graph behavior for default-context enum sources feeding
`BindablePropertyString.propertyValue` targets through `DataConverterToString`.

The direct converter API can convert `DataValueEnum` to a display string when
enum metadata is available. This runtime graph path does not apply that display
label conversion today: C++ writes an empty string for this binding shape, so a
string transition condition looking for the enum label does not transition.
Rust therefore admits the graph input only as this C++ empty-string fallback.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceEnum.propertyValue` sources with resolvable imported
  `ViewModelPropertyEnumCustom` and `DataEnum` metadata.
- `DataConverterToString` as the direct converter on the data bind.
- C++ empty-string fallback behavior instead of enum display-label conversion.
- C++ probe coverage proving the binding does not produce a matching enum-label
  string transition.
- Main-`ToTarget | TwoWay` target-dirty behavior for this display-label
  unsupported enum-to-string path is covered by
  `docs/prototypes/data-binding-graph-to-string-enum-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- Enum display-label conversion for string-target graph bindings.
- Carrying enum display/key tables in runtime graph source values.
- Owned or external view-model enum-to-string contexts.
- Enum source mutation APIs for string-target enum converter binds.
- String trim/pad/remove-zero converters, converter groups, formulas,
  operation converters, interpolators, and stateful converter advancement.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- A C++ probe-backed runtime test covers a default-context enum source,
  `DataConverterToString`, and a string transition condition.
- Rust writes the same empty string fallback as C++ and does not take the
  string condition transition for the enum display label.
- Existing admitted `DataConverterToString` number/boolean/string/trigger,
  symbol-list-index, and color paths continue to pass.
