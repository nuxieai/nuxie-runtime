# State Machine Default ViewModel String Binding Runtime Contract

## Scope

This slice adds default-context source-to-target binding for string bindables in
the state-machine runtime.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyString.propertyValue`, resolves `sourcePathIds` against
the default root `ViewModelInstance`, and applies the resolved
`ViewModelInstanceString` value to the cloned bindable before state-machine
transition evaluation.

The observable consumer for this slice is an already-supported
`TransitionViewModelCondition` string comparison.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyString`;
3. `propertyKey` is `BindablePropertyString.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceString`.

On the next state-machine advance, Rust updates the matching cloned bindable
string bytes from the resolved source before evaluating transitions.

## Out Of Scope

This slice does not implement public view-model APIs, external context binding,
source mutation APIs, relative/name-based paths, parent paths, nested paths,
nested artboard propagation, converters, data-bind dependency ordering,
target-to-source propagation, push observers, polling fallback lists, pending
add/remove handling, re-entry protection, listener-owned data binding,
`ListenerViewModelChange`, or non-string bindables beyond existing behavior.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceString`
  overrides the imported `BindablePropertyString.propertyValue` through a
  `DataBindContext`.
- A string `TransitionViewModelCondition` observes the propagated value after
  state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  default string source-to-target path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
