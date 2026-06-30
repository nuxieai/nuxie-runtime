# State Machine Default ViewModel Boolean Binding Runtime Contract

## Scope

This slice adds the first non-number source-to-target data-binding path for the
state-machine runtime.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyBoolean.propertyValue`, resolves `sourcePathIds`
against the default root `ViewModelInstance`, and applies the resolved
`ViewModelInstanceBoolean` value to the cloned bindable before state-machine
transition evaluation.

The observable consumer for this slice is an already-supported
`TransitionViewModelCondition` boolean comparison.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyBoolean`;
3. `propertyKey` is `BindablePropertyBoolean.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceBoolean`.

On the next state-machine advance, Rust updates the matching cloned bindable
boolean from the resolved source before evaluating transitions.

## Out Of Scope

This slice does not implement public view-model APIs, external context binding,
source mutation APIs, relative/name-based paths, parent paths, nested paths,
nested artboard propagation, converters, data-bind dependency ordering,
target-to-source propagation, push observers, polling fallback lists, pending
add/remove handling, re-entry protection, listener-owned data binding,
`ListenerViewModelChange`, or non-boolean bindables beyond existing behavior.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceBoolean`
  overrides the imported `BindablePropertyBoolean.propertyValue` through a
  `DataBindContext`.
- A boolean `TransitionViewModelCondition` observes the propagated value after
  state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  default boolean source-to-target path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
