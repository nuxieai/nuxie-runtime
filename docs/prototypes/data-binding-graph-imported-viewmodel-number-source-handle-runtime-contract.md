# Imported ViewModel Number Source Handle Runtime Contract

Purpose: admit the first stable public source handle without widening
`nuxie-runtime` into a general mutable object graph API.

The C++ runtime lets callers resolve a file-backed imported
`ViewModelInstanceRuntime::propertyNumber(name)` source and mutate its
`propertyValue`. Rust models the same runtime fact with an immutable handle
that stores the resolved imported context identity and data-bind source path.
Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` number override path, so every state
machine later bound through that context observes the same source value.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyNumber.name` into
  `RuntimeImportedViewModelNumberSourceHandle`.
- Resolving a slash-separated nested number property path into the same handle
  type, using the existing `ViewModelPropertyViewModel` path resolver.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- C++ probe comparison through `ViewModelInstanceRuntime::propertyNumber(name)`
  and the existing number binding report surface.

Out of scope:

- Handles for boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, list, and view-model sources.
- Public object handles that expose or mutate raw `propertyValue` object
  indexes directly.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, relative/parent path lookup, nested artboard propagation,
  cloning, layout, rendering, and animation advancement beyond applying the
  override during context binding.

Completion condition: root and nested imported number source handles resolved
from one shared imported context can mutate that context, a root handle cannot
mutate a different imported instance context, and both state machines bound
through the mutated context report the same number source value as C++.
