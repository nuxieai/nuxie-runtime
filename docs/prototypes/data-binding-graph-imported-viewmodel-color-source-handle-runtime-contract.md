# Imported ViewModel Color Source Handle Runtime Contract

Purpose: extend the stable imported source-handle surface to color sources
without introducing a general mutable object handle API.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyColor(name)` source and mutate its
`propertyValue`. Rust models the same runtime fact with an immutable handle
that stores the resolved imported context identity and data-bind source path.
Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` color override path, so every state
machine later bound through that context observes the same color value.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyColor.name` into
  `RuntimeImportedViewModelColorSourceHandle`.
- Resolving a slash-separated nested color property path into the same handle
  type, using the existing `ViewModelPropertyViewModel` path resolver.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through
  `ViewModelInstanceRuntime::propertyColor(name)` and the existing color
  binding report surface.

Out of scope:

- Handles for enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model sources.
- Public object handles that expose or mutate raw `propertyValue` object
  indexes directly.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: root and nested imported color source handles resolved
from one shared imported context can mutate that context, a root handle cannot
mutate a different imported instance context, and both state machines bound
through the mutated context report the same color source value as C++.
