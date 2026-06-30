# State Machine Default ViewModel Trigger Binding Runtime Contract

## Scope

This slice adds default-context source-to-target binding for trigger bindables
in the state-machine runtime, limited to the `propertyValue` observed by
existing transition-condition comparisons.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyTrigger.propertyValue`, resolves `sourcePathIds`
against the default root `ViewModelInstance`, and applies the resolved
`ViewModelInstanceTrigger.propertyValue` to the cloned bindable before
state-machine transition evaluation.

The observable consumer for this slice is an already-supported
component/view-model trigger `TransitionViewModelCondition` comparison.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyTrigger`;
3. `propertyKey` is `BindablePropertyTrigger.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceTrigger`.

On the next state-machine advance, Rust updates the matching cloned bindable
trigger from the raw resolved source `propertyValue` before evaluating
transitions.

## Out Of Scope

This slice does not implement public view-model trigger APIs, source mutation
APIs, trigger callback targets, listener-owned trigger dispatch,
`ListenerViewModelChange`, callback-driven data binding, external context
binding, relative/name-based paths, parent paths, nested paths, converters,
data-bind dependency ordering, target-to-source propagation, push observers,
polling fallback lists, pending add/remove handling, re-entry protection, or
nested artboard propagation.

Existing `StateMachineFireTrigger` and explicit data-context trigger reset
behavior remain unchanged.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceTrigger`
  overrides the imported `BindablePropertyTrigger.propertyValue` through a
  `DataBindContext`.
- A component/view-model trigger `TransitionViewModelCondition` observes the
  propagated raw uint value after state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  default trigger source-to-target property-value path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
