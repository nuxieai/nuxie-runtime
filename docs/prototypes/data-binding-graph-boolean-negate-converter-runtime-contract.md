# Data Binding Graph Boolean Negate Converter Runtime Contract

## Scope

This slice admits the first runtime data-binding converter through
`RuntimeDataBindGraph`.

It is limited to stateless forward source-to-target conversion for
`DataConverterBooleanNegate` on an existing graph-owned
`DataBindContext -> BindablePropertyBoolean.propertyValue` edge.

## Runtime Rule

When a graph source node is bound through a data bind whose resolved converter
is `DataConverterBooleanNegate`, the graph applies the converter before writing
the source value to the cloned boolean bindable target.

No-converter bindings keep the existing direct source-to-target behavior.
Converter-bearing bindings whose converter has not been admitted by a slice are
not treated as pass-through; they stay unapplied so unsupported converter parity
gaps remain visible.

## Out Of Scope

This slice does not add converter state, converter groups, formulas, random
sources, interpolation/smoothing, reverse conversion, target-to-source
propagation, dirty/update queues, converter lifecycle hooks, relative/parent
paths, listener-owned data binding, or converters for non-boolean target types.

## Completion Checks

- The graph source node records whether its data bind has no converter, a
  supported boolean-negate converter, or an unsupported converter.
- Default-context boolean source-to-target binding applies
  `DataConverterBooleanNegate` before transition-condition evaluation.
- A C++ probe-backed state-machine test verifies the converted boolean value
  against `StateMachineInstance::advance`.
- Existing no-converter boolean binding and mutation tests continue to pass.
