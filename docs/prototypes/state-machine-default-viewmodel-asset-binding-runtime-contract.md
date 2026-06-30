# State Machine Default ViewModel Asset Binding Runtime Contract

## Scope

This slice adds default-context source-to-target binding for asset bindables in
the state-machine runtime, limited to the `propertyValue` observed by existing
transition-condition comparisons.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyAsset.propertyValue`, resolves `sourcePathIds` against
the default root `ViewModelInstance`, and applies the resolved
`ViewModelInstanceAssetImage.propertyValue` to the cloned bindable before
state-machine transition evaluation.

The observable consumer for this slice is an already-supported
`TransitionViewModelCondition` asset comparison.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyAsset`;
3. `propertyKey` is `BindablePropertyAsset.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceAssetImage`.

On the next state-machine advance, Rust updates the matching cloned bindable
asset from the raw resolved source `propertyValue` before evaluating
transitions.

## Out Of Scope

This slice does not implement public view-model asset APIs, render-image
mutation, `BindablePropertyAsset.imageValue` side effects, `Image.setAsset`
target binding, file asset resolution during runtime binding, external context
binding, source mutation APIs, relative/name-based paths, parent paths, nested
paths, converters, data-bind dependency ordering, target-to-source propagation,
push observers, polling fallback lists, pending add/remove handling, re-entry
protection, listener-owned data binding, `ListenerViewModelChange`, or
non-asset bindables beyond existing behavior.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceAssetImage`
  overrides the imported `BindablePropertyAsset.propertyValue` through a
  `DataBindContext`.
- An asset `TransitionViewModelCondition` observes the propagated raw uint value
  after state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  default asset source-to-target property-value path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
