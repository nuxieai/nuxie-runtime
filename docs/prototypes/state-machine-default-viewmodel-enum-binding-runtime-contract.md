# State Machine Default ViewModel Enum Binding Runtime Contract

## Scope

This slice adds default-context source-to-target binding for enum bindables in
the state-machine runtime.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyEnum.propertyValue`, resolves `sourcePathIds` against
the default root `ViewModelInstance`, and applies the resolved
`ViewModelInstanceEnum.propertyValue` to the cloned bindable before
state-machine transition evaluation.

The observable consumer for this slice is an already-supported
`TransitionViewModelCondition` enum comparison.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyEnum`;
3. `propertyKey` is `BindablePropertyEnum.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceEnum`.

On the next state-machine advance, Rust updates the matching cloned bindable
enum from the raw resolved source `propertyValue` before evaluating
transitions.

The source value is copied as the imported raw uint index. This intentionally
does not use the runtime enum-value helper that clamps invalid indexes to zero,
because C++ `DataBindContextValueEnum` syncs from
`ViewModelInstanceEnum::propertyValue()` and applies that raw value through
`CoreRegistry::setUint` for `BindablePropertyEnum.propertyValue`.

## Out Of Scope

This slice does not implement public view-model enum APIs, enum key/name
mutation, external context binding, source mutation APIs, relative/name-based
paths, parent paths, nested paths, `Solo` name mapping, converters,
data-bind dependency ordering, target-to-source propagation, push observers,
polling fallback lists, pending add/remove handling, re-entry protection,
listener-owned data binding, `ListenerViewModelChange`, or non-enum bindables
beyond existing behavior.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceEnum`
  overrides the imported `BindablePropertyEnum.propertyValue` through a
  `DataBindContext`.
- An enum `TransitionViewModelCondition` observes the propagated raw uint value
  after state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  default enum source-to-target path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
