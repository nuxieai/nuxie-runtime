# Data Binding Graph Imported ViewModel Shared Relink Runtime Contract

## Purpose

Close the first gap between Rust's state-machine-local imported view-model
pointer relink overlay and C++'s mutation of the imported
`ViewModelInstance` itself.

C++ `ViewModelInstance::replaceViewModelByProperty` changes the imported
instance. If one state machine relinks a `ViewModelInstanceViewModel` source
and another state machine later binds the same imported view-model instance,
the second state machine observes the relinked source. Rust currently keeps
that relink inside one `RuntimeDataBindGraph`, so independent state-machine
instances do not share it.

## In Scope

- File-backed imported view-model contexts.
- `ViewModelInstanceViewModel` source relinks by state-machine data-bind index.
- Sharing one relinked imported source across two authored state-machine
  runtime instances bound to the same imported file instance.
- A narrow Rust handle/context that owns imported view-model relink overlays
  outside an individual `StateMachineInstance`.
- C++ probe comparison through existing view-model binding reports.

## Out Of Scope

- Mutating scalar, list, asset, artboard, trigger, or symbol-list-index values
  on imported instances.
- Sharing owned generated view-model mutations.
- Cross-file or cross-runtime global mutation state.
- Public stable object-handle API design beyond the minimum context object
  needed for this shared relink slice.
- Name-path relink sharing, relative/parent/nested lookup, reverse
  propagation, broader update queues, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- Relinking an imported view-model pointer source through one state machine is
  visible when a second state machine binds the same imported view-model
  instance.
- The existing state-machine-local imported relink and rebind probes continue
  to pass.
- The shared state remains explicitly scoped to imported view-model pointer
  relinks; other imported-instance mutations stay listed as follow-up work.
