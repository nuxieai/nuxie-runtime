# Default ViewModel ViewModel Name Relink Runtime Contract

Purpose: complete the default-context root property-name source mutation family
with view-model pointer relinking by root property name.

This slice resolves a root `ViewModelPropertyViewModel.name` on file view model
`0`, relinks every graph source node with that root source path to the
requested referenced view-model instance index, and lets ordinary data-context
advancement update the `BindablePropertyViewModel.propertyValue` targets.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Root `ViewModelPropertyViewModel` name lookup only.
- Relinking graph-owned default view-model source nodes to imported referenced
  instance index values already recorded on the graph source.
- C++ probe comparison through
  `ViewModelInstanceRuntime::replaceViewModel(name, referencedRuntime)`.

Out of scope:

- The raw generated `propertyValue` setter behavior, which remains covered by
  `docs/prototypes/data-binding-graph-viewmodel-source-mutation-runtime-contract.md`.
- Nested, relative, parent, or slash-separated view-model paths.
- Public object handles, replacing generated owned child identities, imported
  external context relinking beyond the existing name-path API, list item
  propagation, reverse target-to-source propagation, broader update queues,
  listener-owned data binding, and nested artboard propagation.

Completion condition: relinking the default root view-model source by property
name produces the same data-context advance, state-machine advance, source
pointer, target pointer, and component update reports as C++, and the existing
data-bind-index default view-model relink and raw setter tests continue to
pass.
