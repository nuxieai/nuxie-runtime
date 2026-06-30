# Data Binding Graph External Trigger Identity Runtime Contract

## Scope

This slice closes the trigger identity gap left by the external and owned
view-model context slices.

`StateMachineFireTrigger`, trigger/self `TransitionViewModelCondition`, and
explicit `StateMachineInstance::advancedDataContext()` must operate on the
currently bound view-model trigger context. The default imported
`ViewModelInstanceTrigger` remains a separate reportable object when the active
context is a non-default imported instance or an owned runtime-created
instance.

## Runtime Rule

The runtime keeps two trigger views:

- the default imported trigger report view, matching the C++ probe's
  `file->viewModel(0)->instance(0)` inspection;
- the active bound trigger view, used by fire actions and trigger/self
  conditions.

Binding the default context makes those views share the same values. Binding a
non-default imported context or owned context refreshes only the active view.
Firing and advancing data context reset the active view. The default report view
is mutated only while the default imported context is the active context or when
the default source mutation API explicitly changes the default source.

Graph-owned trigger source nodes are also reset on explicit data-context
advance so raw `BindablePropertyTrigger.propertyValue` bindings do not retain a
stale active-context count.

## Out Of Scope

This slice does not add public trigger firing APIs, stable source handles,
listener-owned trigger dispatch, relative/parent/nested lookup, nested artboard
context propagation, observer queues, converter scheduling, or target-to-source
propagation.

## Completion Checks

- A C++ probe-backed test binds a non-default imported trigger context, fires a
  state-machine trigger action, and verifies the default imported trigger report
  is unchanged.
- Existing default-context trigger reset and default/imported/owned trigger
  source binding tests continue to pass.
- `advancedDataContext()` resets active trigger values and graph trigger source
  nodes without resetting the default report view for external or owned
  contexts.
