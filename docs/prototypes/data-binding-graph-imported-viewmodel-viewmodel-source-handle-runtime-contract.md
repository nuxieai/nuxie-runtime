# Imported ViewModel ViewModel Source Handle Runtime Contract

Purpose: finish the imported source-handle family for view-model pointer
sources, including the nested property-name paths already admitted by the
imported view-model pointer relink slices.

The C++ runtime lets callers replace a file-backed imported
`ViewModelInstanceViewModel` source through
`ViewModelInstanceRuntime::replaceViewModel(path, value)`, where `path` may be
a root property name such as `current` or a slash-separated view-model pointer
path such as `child/grandchild`. Rust models that runtime fact with an
immutable handle that stores the resolved imported context identity and
data-bind source path. Applying the handle writes through the existing
`RuntimeImportedViewModelInstanceContext` view-model pointer override path, so
every state machine later bound through that context observes the same
referenced imported instance.

In scope:

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Resolving a root `ViewModelPropertyViewModel.name` into
  `RuntimeImportedViewModelViewModelSourceHandle`.
- Resolving a slash-separated path of `ViewModelPropertyViewModel.name`
  segments into `RuntimeImportedViewModelViewModelSourceHandle`.
- Rejecting handles whose imported view-model or instance identity does not
  match the target context.
- Rejecting invalid referenced instance indexes through the existing relink
  path.
- C++ probe comparison through
  `--runtime-relink-view-model-instance-source-viewmodel-by-name-path` and the
  existing view-model binding report surface.

Out of scope:

- Non-view-model leaf paths, relative paths, parent paths, or manifest-backed
  path lookup beyond the explicit slash-separated property-name path.
- Public view-model object handles or raw `propertyValue` object indexes.
- Handle invalidation, object lifetime tracking, or mutable `RuntimeFile`
  storage.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, layout, rendering, and
  animation advancement beyond applying the override during context binding.

Completion condition: root and nested imported view-model pointer source
handles can relink a shared imported context, cannot be applied to a different
imported context identity, repeated same-instance relinks report no change, and
state machines bound through the mutated context report the same view-model
source and target instance indexes as C++.
