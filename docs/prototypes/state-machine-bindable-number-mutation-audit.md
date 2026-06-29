# State Machine Bindable Number Mutation Audit

Date: 2026-06-29

This document follows `state-machine-bindable-number-blend-runtime-contract.md`.
Static imported `BindablePropertyNumber.propertyValue` reads are now in place
for `BlendState1DViewModel` and `BlendAnimationDirect.blendSource=dataBindId`.
The next question is how far Rust should go before full data binding.

## C++ Runtime Path

`StateMachineInstance` constructs the instance-local bindable-property world in
`src/animation/state_machine_instance.cpp`:

- It clones every state-machine-owned `DataBind`.
- If a cloned data-bind targets a `BindableProperty`, it clones that target and
  stores `original BindableProperty* -> cloned BindableProperty*` in
  `m_bindablePropertyInstances`.
- It rewrites the cloned data-bind target to the cloned bindable property.
- It records a separate bindable-property-to-data-bind map for target-to-source
  binds (`m_bindableDataBindsToSource`) and source-to-target binds
  (`m_bindableDataBindsToTarget`).

`DataBindContainer` owns the update queue in
`src/data_bind/data_bind_container.cpp`:

- `addDataBind` enrolls push-capable target-to-source binds through
  `Core::notifyPropertyChanged`; non-push target-to-source binds go into a
  polling `m_persistingDataBinds` list.
- `updateDataBinds` drains persisting target-to-source binds, dirty
  target-to-source binds, then dirty source-to-target binds.
- Re-entrant add/remove/dirty calls are deferred into pending lists.
- `updateDataBind(dataBind, applyTargetToSource)` suppresses target-to-source
  application when `applyTargetToSource` is false, but still applies
  source-to-target dirt.

`StateMachineInstance::advance`, `tryChangeState`, event application, and
listener notification call `updateDataBinds(false)`. That means the normal
state-machine advance path updates state-machine bindable targets from their
bound sources, but it does not push state-machine target edits back into source
view-model values.

## Mutation Sources In C++

There are three different ways the cloned bindable number can change:

1. It can be initialized from the imported `BindablePropertyNumber`.
2. It can be written by a source-to-target data-bind after a data context has
   resolved a `ViewModelInstanceNumber` source.
3. It can be written directly by user/listener/runtime code that reaches the
   cloned bindable property, after which generated setters call
   `notifyPropertyChanged`.

Rust currently supports only the first path.

## Current Rust State

`rive-runtime` now collects imported state-machine-owned
`BindablePropertyNumber` targets and stores per-`StateMachineInstance`
`StateMachineBindableNumberInstance` values. Blend sources read those values by
authored bindable-property identity.

Missing:

- A public or probe-facing way to mutate a per-instance bindable number.
- A C++ probe action that mutates the cloned C++ bindable number so Rust can be
  compared against C++.
- Any `DataBindContainer` queue, data-context resolution, view-model instance
  source lookup, converter execution, or target-to-source propagation.

## Next Implementation Slice

The next slice should implement explicit mutable bindable-number overrides, not
full data binding.

Completion target:

- Extend `tools/cpp-probe` with a runtime state-machine action that sets a
  cloned `BindablePropertyNumber` by state-machine data-bind index.
  The C++ action can resolve:
  `stateMachine->dataBind(dataBindIndex)->target()` -> original
  `BindablePropertyNumber*`, then
  `stateMachineInstance->bindablePropertyInstance(original)` -> cloned
  instance property.
- Add a matching Rust `StateMachineInstance` mutation path for per-instance
  bindable numbers. It may be public as a narrow runtime API or test-facing at
  first, but it must mutate only the instance clone.
- Verify that existing `BlendState1DViewModel` and
  `BlendAnimationDirect.blendSource=dataBindId` sources observe the new value on
  the next state-machine advance.
- Compare C++ state-machine advance reports and final component state for both
  mutated blend-source shapes.

This should be enough to prove the instance-clone mutation boundary without
claiming support for live view-model binding.

## Scope Lock

This slice must not implement:

- Binding a `ViewModelInstance` or `DataContext` into a state machine.
- Resolving data-bind paths or manifest-backed names.
- `DataBindContextValue` caches.
- Source-to-target or target-to-source queues.
- Push observers, polling fallback lists, pending add/remove handling, or
  re-entry protection.
- Data converters.
- Non-number bindable properties.
- `ListenerViewModelChange`.
- View-model transition conditions.
- Transition-property data binds.
- Nested artboard data-context propagation.

Those belong to roadmap item `#12: Data Binding Graph`.

## Admission Rule

Before adding behavior to the next slice, ask:

1. Does it mutate only a per-state-machine-instance
   `BindablePropertyNumber` clone?
2. Can C++ reach the same clone through a state-machine-owned `DataBind`
   target?
3. Does an existing supported runtime consumer read the mutated number without
   requiring a data context, converter, listener, nested artboard, or renderer?

If not, stop and move the behavior into the later data-binding runtime plan.

## Follow-On Order

After explicit mutable numbers are probe-backed, the next state-machine slices
can stay small:

1. Add static/mutable `TransitionViewModelCondition` support for
   `BindablePropertyNumber` comparands.
2. Add the smallest source-to-target `ViewModelInstanceNumber` data-context
   binding path.
3. Generalize the data-bind queue only after a second concrete consumer needs
   it.
