# Data Binding Graph Enum To Number Converter Runtime Contract

## Scope

This slice admits the second cross-type `DataConverterToNumber` source kind
through `RuntimeDataBindGraph`.

It is limited to stateless forward source-to-target conversion when an existing
graph-owned `DataBindContext -> BindablePropertyNumber.propertyValue` edge has
`DataConverterToNumber` and resolves an enum view-model source.

## Runtime Rule

When a number target edge has a resolved `DataConverterToNumber` converter and
its source path resolves to a `ViewModelInstanceEnum` source node, the graph
converts the raw enum `propertyValue` integer to `f32` before writing the
cloned number bindable target.

The previously admitted boolean-to-number and number pass-through cases stay
unchanged. Other `DataConverterToNumber` source kinds remain out of scope for
this runtime slice.

Main-`ToTarget | TwoWay` state-machine target-dirty behavior for this enum
path is covered by
`docs/prototypes/data-binding-graph-to-number-scalar-main-to-target-two-way-target-dirty-runtime-contract.md`.

## Out Of Scope

This slice does not add string-to-number parsing, color-to-number conversion,
symbol-list index conversion, list/view-model values, enum label/name mapping,
converter groups, formulas, interpolation/smoothing, reverse conversion,
target-to-source propagation, dirty/update queues beyond the linked
state-machine target-dirty path, converter lifecycle hooks, relative/parent
paths, nested paths, listener-owned data binding, or nested artboard
propagation.

## Completion Checks

- Number graph source nodes can carry enum sources when
  `DataConverterToNumber` is present.
- `DataConverterToNumber` converts the raw enum source value before number
  target writes.
- A C++ probe-backed state-machine test verifies the converted number value
  through a `BlendState1DViewModel` consumer.
- Existing no-converter number, boolean-to-number converter, and enum binding
  tests continue to pass.
