# Data Binding Graph StringTrim Converter Runtime Contract

## Purpose

Add the first dedicated string-converter runtime graph path:
`DataConverterStringTrim` for default-context string sources feeding
`BindablePropertyString.propertyValue` targets.

This slice reuses the C++-modeled trim behavior in `nuxie-binary` and keeps the
runtime graph focused on applying the imported `trimType` descriptor at the
existing graph-owned source-to-target apply point.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources.
- Direct `DataConverterStringTrim` converters with imported `trimType`.
- Forward source-to-target conversion covered by a C++ probe through an
  existing string transition-condition consumer.
- Main-`ToTarget | TwoWay` target-dirty behavior for the direct string-trim
  path is covered by
  `docs/prototypes/data-binding-graph-string-trim-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- `DataConverterStringPad` and `DataConverterStringRemoveZeros`.
- Converter groups, formulas, operation converters, interpolators, and
  stateful converter advancement.
- Non-string sources feeding `DataConverterStringTrim`.
- Owned or external view-model string-trim contexts.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- The runtime graph stores the `DataConverterStringTrim.trimType` descriptor.
- Default-context string sources convert through the shared binary
  C++-parity trim helper before writing string bindable targets.
- A C++ probe-backed state-machine test observes the trimmed string through a
  transition condition.
- Existing admitted string-target converter probes continue to pass.
