# Data Binding Graph Boolean To Number Converter Runtime Contract

## Scope

This slice admits the first cross-type runtime data-binding converter through
`RuntimeDataBindGraph`.

It is limited to stateless forward source-to-target conversion for
`DataConverterToNumber` when an existing graph-owned
`DataBindContext -> BindablePropertyNumber.propertyValue` edge resolves a
boolean view-model source.

## Runtime Rule

When a number target edge has a resolved `DataConverterToNumber` converter and
its source path resolves to a boolean source node, the graph converts `true` to
`1.0` and `false` to `0.0` before writing the cloned number bindable target.

No-converter number bindings keep the existing number source-to-target
behavior. Number-source bindings with `DataConverterToNumber` are accepted as
the C++ pass-through case. Other source kinds for `DataConverterToNumber`
remain out of scope for this runtime slice so string parsing, enum metadata,
color casting, symbol-list indexes, lists, and view-model values do not enter
the graph yet.

## Out Of Scope

This slice does not add string-to-number parsing, enum-to-number conversion,
color-to-number conversion, symbol-list index conversion, converter groups,
formulas, interpolation/smoothing, reverse conversion, target-to-source
propagation, dirty/update queues, converter lifecycle hooks, relative/parent
paths, nested paths, listener-owned data binding, or nested artboard
propagation.

## Completion Checks

- Number graph source nodes can carry either a number source or the admitted
  boolean source when `DataConverterToNumber` is present.
- `DataConverterToNumber` converts boolean source values before number target
  writes.
- A C++ probe-backed state-machine test verifies the converted number value
  through a `BlendState1DViewModel` consumer.
- Existing no-converter number binding and number source mutation tests
  continue to pass.
