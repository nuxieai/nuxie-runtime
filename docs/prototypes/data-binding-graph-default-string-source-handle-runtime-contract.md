# Default ViewModel String Source Handle Runtime Contract

Purpose: extend the default-context stable public source-handle family from
number/boolean to string while keeping the remaining default handle kinds out
of this slice.

The existing default string source mutation path can already resolve a root
`ViewModelPropertyString.name` and mutate every graph source node sharing that
path. This slice exposes the resolved path as an immutable
`RuntimeDefaultViewModelStringSourceHandle`, then applies the handle through
the same graph-owned string source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a root `ViewModelPropertyString.name` on file view model `0` into
  `RuntimeDefaultViewModelStringSourceHandle`.
- Mutating graph-owned default string source nodes through the handle before
  ordinary state-machine advancement.
- C++ probe comparison through the existing default string by-name mutation
  command and report surface.

Out of scope:

- Color, enum, symbol-list-index, asset, artboard, trigger, list, or
  view-model default source handles.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested string paths are covered separately by
  `docs/prototypes/data-binding-graph-default-nested-string-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or owned view-model contexts.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default string source handle resolved from
`label` mutates the same graph-owned source path as the existing property-name
API, repeated same-value writes report no change, root-name lookup stays
separate from slash-path lookup, and the C++ probe reports the same
state-machine advance and component update results.
