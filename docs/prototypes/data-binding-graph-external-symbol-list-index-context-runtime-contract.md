# Data Binding Graph External SymbolListIndex Context Runtime Contract

## Purpose

Pin imported file-backed `ViewModelInstanceSymbolListIndex` context binding for
the runtime data-binding graph.

The graph already resolves symbol-list-index values for default contexts and
converter execution. This slice makes imported external context parity explicit
with a C++ probe, so the map does not leave symbol-list-index as an untracked
gap in the file-backed context lane.

## In Scope

- `StateMachineInstance::bind_view_model_instance_context` over imported
  `RuntimeFile` view-model instances.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding an existing
  `DataConverterToString` target path.
- A fixture with a default root value and a distinct imported alternate value,
  proving external binding refreshes the graph source instead of falling back
  to default or target-initial state.
- C++ probe coverage through
  `StateMachineInstance::bindViewModelInstance(...)`.

## Out Of Scope

- Owned runtime symbol-list-index contexts.
- Default source mutation, already covered by the dedicated mutation contract.
- Symbol/list bindable target types.
- Stable public source handles.
- Reverse target-to-source propagation.
- Relative, parent, nested, and listener-owned data binding.

## Completion Checks

- Binding imported instance index `1` refreshes the symbol-list-index source
  from the alternate imported instance.
- The converted string target observes the alternate imported value on the next
  explicit state-machine advance.
- Existing default symbol-list-index converter and mutation probes continue to
  pass.
