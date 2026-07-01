# Imported ViewModel Asset Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to root asset
sources while preserving the documented unsupported nested asset path boundary.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyValue(name)` asset-image source and mutate
its `propertyValue`. Rust models the same runtime fact with an immutable
handle that stores the resolved imported context identity and data-bind source
path. Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` asset override path, so every state
machine later bound through that context observes the same asset index.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyAsset*` name into
  `RuntimeImportedViewModelAssetSourceHandle`.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through the root asset by-name mutation path and the
  existing asset binding report surface.

Out of scope:

- Slash-separated/nested asset handle lookup. This remains governed by
  `docs/prototypes/data-binding-graph-imported-viewmodel-nested-asset-name-path-unsupported-runtime-contract.md`.
- Handles for artboard, trigger, list, and view-model sources.
- Public asset object handles, asset metadata mutation, or raw `propertyValue`
  object indexes.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: a root imported asset source handle resolved from one
shared imported context can mutate that context, cannot mutate a different
imported instance context, slash-path handle lookup stays unresolved, and both
state machines bound through the mutated context report the same asset index
as C++.
