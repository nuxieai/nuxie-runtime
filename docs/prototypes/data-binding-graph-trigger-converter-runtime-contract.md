# Data Binding Graph Trigger Converter Runtime Contract

## Scope

This slice admits the second stateless runtime data-binding converter through
`RuntimeDataBindGraph`.

It is limited to forward source-to-target conversion for `DataConverterTrigger`
on an existing graph-owned
`DataBindContext -> BindablePropertyTrigger.propertyValue` edge.

## Runtime Rule

When a graph source node is bound through a data bind whose resolved converter
is `DataConverterTrigger`, the graph converts the bound trigger value before
writing it to the cloned trigger bindable target.

The conversion matches the C++ modeled trigger converter: read the input as a
C++ integer-super value, increment it with `uint32_t` wrapping semantics, and
return that value as a trigger count. No-converter trigger bindings keep the
existing direct source-to-target behavior. Converter-bearing bindings whose
converter has not been admitted by a slice remain unapplied so unsupported
converter parity gaps stay visible.

## Out Of Scope

This slice does not add converter state, converter groups, formulas,
interpolation/smoothing, reverse conversion, target-to-source propagation,
trigger firing or listener dispatch semantics, dirty/update queues,
converter lifecycle hooks, relative/parent paths, nested paths, listener-owned
data binding, or nested artboard propagation.

## Completion Checks

- Trigger graph source nodes record whether their data bind has no converter, a
  supported trigger converter, or an unsupported converter.
- Default-context trigger source-to-target binding applies
  `DataConverterTrigger` before transition-condition evaluation.
- A C++ probe-backed state-machine test verifies the converted trigger count
  against `StateMachineInstance::advance`.
- Existing no-converter trigger binding, trigger mutation, and trigger reset
  tests continue to pass.
