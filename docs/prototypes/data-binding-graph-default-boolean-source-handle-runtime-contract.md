# Default ViewModel Boolean Source Handle Runtime Contract

Purpose: extend the default-context stable public source-handle tracer from
number to boolean without admitting the whole default source-handle family.

The existing default boolean source mutation path can already resolve a root
`ViewModelPropertyBoolean.name` and mutate every graph source node sharing
that path. This slice exposes the resolved path as an immutable
`RuntimeDefaultViewModelBooleanSourceHandle`, then applies the handle through
the same graph-owned boolean source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a root `ViewModelPropertyBoolean.name` on file view model `0` into
  `RuntimeDefaultViewModelBooleanSourceHandle`.
- Mutating graph-owned default boolean source nodes through the handle before
  ordinary state-machine advancement.
- C++ probe comparison through the existing default boolean by-name mutation
  command and report surface.

Out of scope:

- String, color, enum, symbol-list-index, asset, artboard, trigger, list, or
  view-model default source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default boolean source handle resolved from
`enabled` mutates the same graph-owned source path as the existing
property-name API, repeated same-value writes report no change, slash-path
lookup stays unresolved, and the C++ probe reports the same state-machine
advance and component update results.
