# Default ViewModel Enum Source Handle Runtime Contract

Purpose: extend the default-context stable public source-handle family from
number/boolean/string/color to enum while keeping the remaining default handle
kinds out of this slice.

The existing default enum source mutation path can already resolve a root enum
view-model property name and mutate every graph source node sharing that path.
This slice exposes the resolved path as an immutable
`RuntimeDefaultViewModelEnumSourceHandle`, then applies the handle through the
same graph-owned enum source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving root `ViewModelPropertyEnum`, `ViewModelPropertyEnumCustom`, and
  `ViewModelPropertyEnumSystem` names on file view model `0` into
  `RuntimeDefaultViewModelEnumSourceHandle`.
- Mutating graph-owned default enum source nodes through the handle before
  ordinary state-machine advancement.
- C++ probe comparison through the existing default enum by-name mutation
  command and report surface.

Out of scope:

- Symbol-list-index, asset, artboard, trigger, list, or view-model default
  source handles.
- Nested, relative, parent, or slash-separated property paths.
- Imported or owned view-model contexts.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default enum source handle resolved from `choice`
mutates the same graph-owned source path as the existing property-name API,
repeated same-value writes report no change, slash-path lookup stays
unresolved, and the C++ probe reports the same state-machine advance and
component update results.
