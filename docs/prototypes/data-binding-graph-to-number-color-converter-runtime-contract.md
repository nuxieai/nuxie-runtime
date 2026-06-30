# Data Binding Graph Color To Number Converter Runtime Contract

## Scope

This slice admits the third cross-type `DataConverterToNumber` source kind
through `RuntimeDataBindGraph`.

It is limited to stateless forward source-to-target conversion when an existing
graph-owned `DataBindContext -> BindablePropertyNumber.propertyValue` edge has
`DataConverterToNumber` and resolves a color view-model source.

## Runtime Rule

When a number target edge has a resolved `DataConverterToNumber` converter and
its source path resolves to a color source node, the graph converts the raw
`u32` color through the C++ signed `int32_t` cast and then to `f32` before
writing the cloned number bindable target.

The previously admitted number pass-through, boolean-to-number, and
enum-to-number cases stay unchanged. Other `DataConverterToNumber` source kinds
remain out of scope for this runtime slice.

## Out Of Scope

This slice does not add string-to-number parsing, symbol-list index conversion,
list/view-model values, color channel interpretation, converter groups,
formulas, interpolation/smoothing, reverse conversion, target-to-source
propagation, dirty/update queues, converter lifecycle hooks, relative/parent
paths, nested paths, listener-owned data binding, or nested artboard
propagation.

## Completion Checks

- Number graph source nodes can carry color sources when
  `DataConverterToNumber` is present.
- `DataConverterToNumber` converts the raw color source value before number
  target writes.
- A C++ probe-backed state-machine test verifies the converted number value
  through a `BlendState1DViewModel` consumer.
- Existing no-converter number, boolean-to-number converter, enum-to-number
  converter, and color binding tests continue to pass.
