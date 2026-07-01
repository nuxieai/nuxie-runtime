# Default ViewModel Symbol-List-Index Source Handle Runtime Contract

Purpose: extend the default-context stable public source-handle family from
the scalar source kinds to symbol-list-index while keeping asset/artboard/
trigger/list/view-model handles out of this slice.

The existing default symbol-list-index source mutation path can already
resolve a root `ViewModelPropertySymbolListIndex.name` and mutate every graph
source node sharing that path. This slice exposes the resolved path as an
immutable `RuntimeDefaultViewModelSymbolListIndexSourceHandle`, then applies
the handle through the same graph-owned symbol-list-index source mutation path.

In scope:

- Default view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Resolving a root `ViewModelPropertySymbolListIndex.name` on file view model
  `0` into `RuntimeDefaultViewModelSymbolListIndexSourceHandle`.
- Mutating graph-owned default symbol-list-index source nodes through the
  handle before ordinary state-machine advancement.
- C++ probe comparison through the existing default symbol-list-index by-name
  mutation command and report surface.

Out of scope:

- Asset, artboard, trigger, list, or view-model default source handles.
- Changing root-name lookup semantics to accept slash-separated property
  paths. Nested symbol-list-index paths are covered separately by
  `docs/prototypes/data-binding-graph-default-nested-symbol-list-index-source-handle-runtime-contract.md`.
- Relative or parent property paths.
- Imported or owned view-model contexts.
- Public target handles, reverse target-to-source propagation, converter
  family expansion, broader update queues, listener-owned data binding, and
  nested artboard propagation.

Completion condition: a root default symbol-list-index source handle resolved
from `symbol` mutates the same graph-owned source path as the existing
property-name API, repeated same-value writes report no change, root-name
lookup stays separate from slash-path lookup, and the C++ probe reports the
same state-machine advance and component update results.
