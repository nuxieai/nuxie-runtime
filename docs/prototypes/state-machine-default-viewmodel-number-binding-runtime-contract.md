# State Machine Default ViewModel Number Binding Runtime Contract

## Scope

This slice introduces the first live source-to-target data-binding path needed
by the state-machine runtime.

Rust must match C++ when a state-machine-owned `DataBindContext` targets a
cloned `BindablePropertyNumber`, resolves its `sourcePathIds` against the
default root `ViewModelInstance`, and applies that `ViewModelInstanceNumber`
value to the cloned bindable during state-machine advancement.

The observable runtime consumer for this slice is an already-supported
state-machine bindable-number read, such as:

- `BlendState1DViewModel` reading the cloned number as its blend input.
- `TransitionViewModelCondition` reading the cloned number behind an existing
  data-context gate.

## C++ Shape

`StateMachineInstance::internalDataContext` calls
`DataBindContainer::bindDataBindsFromContext`, which lets `DataBindContext`
resolve its source from the bound `DataContext`.

`StateMachineInstance::advance` then calls `updateDataBinds(false)` before
advancing layers. For a source-to-target number bind, C++ applies
`DataBindContextValueNumber` to the cloned `BindablePropertyNumber` target even
when target-to-source propagation is disabled.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` has been called,
Rust may resolve only this exact source shape:

1. the state-machine data bind is `DataBindContext`;
2. the data-bind target is `BindablePropertyNumber`;
3. `propertyKey` is `BindablePropertyNumber.propertyValue`;
4. `sourcePathIds` resolves against `RuntimeFile::view_model_default_instance(0)`;
5. the resolved view-model instance value is `ViewModelInstanceNumber`.

On the next state-machine advance, Rust updates the matching cloned bindable
number from the resolved source before evaluating transitions or blend states.

## Out Of Scope

This slice does not implement public view-model instance APIs, external
context binding, source mutation APIs, relative/name-based paths, parent paths,
nested paths, nested artboard propagation, non-number bindables, converters,
data-bind dependency ordering, target-to-source propagation, push observers,
polling fallback lists, pending add/remove handling, re-entry protection,
listener-owned data binding, `ListenerViewModelChange`, or renderer/layout/text
side effects.

## Completion Checks

- C++ probe-backed fixtures prove that a default `ViewModelInstanceNumber`
  overrides the imported `BindablePropertyNumber.propertyValue` through a
  `DataBindContext`.
- At least one existing bindable-number state-machine consumer observes the
  propagated value after state-machine advancement.
- The remaining-work audit narrows the live data-binding entry to exclude this
  source-to-target number path.
- Focused runtime probes, workspace checks, and `make cpp-compare` pass.
