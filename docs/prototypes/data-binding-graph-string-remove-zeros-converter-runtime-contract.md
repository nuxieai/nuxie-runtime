# Data Binding Graph StringRemoveZeros Converter Runtime Contract

## Purpose

Add graph-owned runtime execution for `DataConverterStringRemoveZeros` on
default-context string sources feeding `BindablePropertyString.propertyValue`
targets.

This slice reuses the C++-modeled trailing-zero removal behavior in
`rive-binary` and keeps the runtime graph limited to direct source-to-target
conversion.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources.
- Direct `DataConverterStringRemoveZeros` converters.
- Forward source-to-target conversion covered by a C++ probe through an
  existing string transition-condition consumer.
- Main-`ToTarget | TwoWay` target-dirty behavior for the direct remove-zeros
  path is covered by
  `docs/prototypes/data-binding-graph-string-remove-zeros-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

- `DataConverterStringPad`.
- Converter groups, formulas, operation converters, interpolators, and
  stateful converter advancement.
- Non-string sources feeding `DataConverterStringRemoveZeros`.
- Owned or external view-model remove-zero contexts.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- The runtime graph recognizes direct `DataConverterStringRemoveZeros`
  converters.
- Default-context string sources convert through the shared binary
  C++-parity remove-zero helper before writing string bindable targets.
- A C++ probe-backed state-machine test observes the converted string through a
  transition condition.
- Existing admitted string-target converter probes continue to pass.
