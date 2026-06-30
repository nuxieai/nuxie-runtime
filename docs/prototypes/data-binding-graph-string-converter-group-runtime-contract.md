# Data Binding Graph String Converter Group Runtime Contract

## Purpose

Add the first runtime graph `DataConverterGroup` path: default-context string
sources feeding `BindablePropertyString.propertyValue` targets through a direct
group of already-supported string converters.

This slice proves group composition and child ordering without admitting every
converter type, stateful converter behavior, or cross-type group source
admission.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceString.propertyValue` sources.
- Direct `DataConverterGroup` converters whose children resolve through
  imported `DataConverterGroupItem.converterId`.
- Group children that are already-supported runtime graph converters for this
  lane, specifically direct string trim/remove-zero/pad converters.
- Forward source-to-target conversion covered by a C++ probe through an
  existing string transition-condition consumer.

## Out Of Scope

- Groups that require non-string source admission for string targets, such as
  `DataConverterToString` followed by string converters.
- Number/boolean/trigger target groups.
- Formula, operation, range, rounder, interpolator, number-to-list, list, and
  scripted converters inside runtime graph groups.
- Stateful converter advancement and converter reset/dirty scheduling.
- Owned or external view-model converter-group contexts.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the existing graph-owned apply point.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- The runtime graph stores a group as an ordered list of child graph
  converters resolved through `rive-binary` group-item metadata.
- Group conversion feeds each child output into the next child in C++ order.
- Cyclic or unresolved child converters remain unsupported rather than
  recursing indefinitely.
- A C++ probe-backed state-machine test observes a trim-then-pad group through
  a string transition condition.
- Existing admitted string-target converter probes continue to pass.
