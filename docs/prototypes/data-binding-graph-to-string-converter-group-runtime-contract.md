# Data Binding Graph ToString Converter Group Runtime Contract

## Purpose

Admit the smallest cross-type runtime graph `DataConverterGroup` path for
string targets: a default-context non-string source becomes a string only when
the first effective group converter is `DataConverterToString`, after which the
result may flow through already-supported direct string converters.

This slice proves source admission through a group without treating converter
groups as a general graph execution engine.

Main-`ToTarget | TwoWay` target-dirty behavior for this cross-type
number-to-string group path is covered separately by
`docs/prototypes/data-binding-graph-to-string-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- `ViewModelInstanceNumber.propertyValue` sources.
- A direct `DataConverterGroup` on the data bind whose first effective child is
  `DataConverterToString`.
- Group children that resolve through imported
  `DataConverterGroupItem.converterId`.
- The already-supported `DataConverterToString` number formatting path,
  followed by already-supported direct string converters such as
  `DataConverterStringPad`.
- Forward source-to-target conversion covered by a C++ probe through an
  existing string transition-condition consumer.

## Out Of Scope

- Non-number source admission through converter groups.
- Groups whose first effective converter is not `DataConverterToString`.
- Number, boolean, trigger, color, enum, asset, artboard, or list targets.
- Formula, operation, range, rounder, interpolator, number-to-list, list, and
  scripted converters inside runtime graph groups.
- Stateful converter advancement and converter reset/dirty scheduling.
- Owned or external view-model converter-group contexts.
- Reverse target-to-source propagation.
- Dirty/update queue parity beyond the linked state-machine target-dirty path.
- Relative, parent, and nested source path lookup.
- Nested artboard propagation.

## Completion Checks

- String graph source admission recognizes a direct converter group whose first
  effective child is `DataConverterToString`.
- The source admission rule still rejects ordinary non-string sources when no
  admitted direct converter or first-effective group converter exists.
- Group conversion feeds `DataConverterToString` output into the following
  supported string converter in C++ order.
- Cyclic or unresolved child converters remain unsupported rather than
  recursing indefinitely.
- A C++ probe-backed state-machine test observes a number-to-string-then-pad
  group through a string transition condition.
- Existing direct `DataConverterToString`, direct string-converter, and
  string-source converter-group probes continue to pass.
