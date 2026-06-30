# Data Binding Graph StringPad Converter Runtime Contract

## Purpose

Add graph-owned runtime execution for `DataConverterStringPad` on
default-context string sources feeding `BindablePropertyString.propertyValue`
targets.

This completes the direct string-converter family for the current default
source-to-string target runtime graph lane: trim, remove-zeros, and pad.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources.
- Direct `DataConverterStringPad` converters with imported `length`, `text`,
  and `padType` descriptors.
- Forward source-to-target conversion covered by a C++ probe through an
  existing string transition-condition consumer.

## Out Of Scope

- Converter groups, formulas, operation converters, interpolators, and
  stateful converter advancement.
- Non-string sources feeding `DataConverterStringPad`.
- Owned or external view-model string-pad contexts.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- The runtime graph stores the `DataConverterStringPad.length`, `text`, and
  `padType` descriptors.
- Default-context string sources convert through the shared binary
  C++-parity pad helper before writing string bindable targets.
- A C++ probe-backed state-machine test observes the padded string through a
  transition condition.
- Existing admitted string-target converter probes continue to pass.
