# Imported ViewModel String Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to string sources,
including owned byte storage, without introducing a general mutable object
handle API.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyString(name)` source and mutate its
`propertyValue`. Rust models the same runtime fact with an immutable handle
that stores the resolved imported context identity and data-bind source path.
Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` string override path, so every state
machine later bound through that context observes the same source bytes.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyString.name` into
  `RuntimeImportedViewModelStringSourceHandle`.
- Resolving a slash-separated nested string property path into the same handle
  type, using the existing `ViewModelPropertyViewModel` path resolver.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through
  `ViewModelInstanceRuntime::propertyString(name)` and the existing string
  binding report surface.

Out of scope:

- Handles for color, enum, symbol-list-index, asset, artboard, trigger, list,
  and view-model sources.
- Public object handles that expose or mutate raw `propertyValue` object
  indexes directly.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: root and nested imported string source handles resolved
from one shared imported context can mutate that context, a root handle cannot
mutate a different imported instance context, and both state machines bound
through the mutated context report the same string source bytes as C++.
