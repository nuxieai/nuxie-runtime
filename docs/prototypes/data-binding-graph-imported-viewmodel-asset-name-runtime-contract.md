# Imported ViewModel Asset Name Runtime Contract

Purpose: admit the imported root asset property-name mutation API after the
root number, boolean, string, color, enum, and symbol-list-index property-name
slices.

C++ exposes asset image lookup through
`ViewModelInstanceRuntime::propertyImage`, but the public runtime wrapper sets a
`RenderImage*` rather than the imported `propertyValue` index used by the
state-machine data-binding comparison. For this parity slice the observable
import-time fact is root-name mutation of a file-backed imported
`ViewModelInstance`: resolve a root asset property by name, require a
`ViewModelInstanceAssetImage`, and mutate its raw `propertyValue` before
binding. Any state machine later bound to the same imported instance observes
the new asset index.

Rust models the same fact with
`RuntimeImportedViewModelInstanceContext::set_asset_by_property_name(file,
name, value)`. The method resolves a root `ViewModelPropertyAssetImage` or
`ViewModelPropertyAsset` name against the context's view model, records an
asset override by the resolved source path, and lets the existing
imported-context bind path apply that value to every state machine bound
through the same context.

In scope:

- File-backed imported view-model instance contexts created with
  `RuntimeImportedViewModelInstanceContext::new`.
- Root `ViewModelPropertyAssetImage` and `ViewModelPropertyAsset` name lookup
  only.
- Mutating the imported raw asset index before binding or rebinding a state
  machine.
- Sharing the same mutated context across two authored state machines.
- C++ probe comparison through the existing asset binding report surface.

Out of scope:

- Artboard, trigger, list, and view-model property-name APIs.
- Nested, relative, parent, or slash-separated property paths.
- Image loading, decoding, `RenderImage*` replacement, renderer hooks, file
  asset helper metadata mutation, or stable public asset handles.
- Reverse target-to-source propagation, listener-owned data binding, broader
  update queues, nested artboard propagation, cloning, and runtime evaluation
  beyond applying the override during context binding.

Completion condition: a root imported asset property can be mutated by name on
one shared imported context, both state machines bound through that context
report the same asset source value as C++, and the existing imported asset
data-bind-index mutation plus owned asset-name tests continue to pass.
