# State Machine View-Model Number Condition Audit

Date: 2026-06-29

This document follows
`state-machine-bindable-number-mutation-runtime-contract.md`. The next
state-machine parity gap is `TransitionViewModelCondition` for
`BindablePropertyNumber` comparands.

## C++ Runtime Path

The relevant C++ files are:

- `src/animation/transition_viewmodel_condition.cpp`
- `include/rive/animation/transition_viewmodel_condition.hpp`
- `src/animation/transition_property_viewmodel_comparator.cpp`
- `src/importers/transition_viewmodel_condition_importer.cpp`

Import shape:

- `TransitionViewModelCondition` imports as a `TransitionCondition` child of
  the latest `StateTransition`.
- Its generated `leftComparatorId` and `rightComparatorId` fields are
  `runtime: false`; C++ does not read them from stored fields at runtime.
- `TransitionViewModelConditionImporter` owns the import-stack context.
- Each subsequent `TransitionComparator` child imports into the latest
  `TransitionViewModelConditionImporter`.
- The first comparator becomes the condition's left comparator; the second
  becomes the right comparator.
- `TransitionPropertyViewModelComparator` imports only when both a latest
  `TransitionViewModelConditionImporter` and latest `BindablePropertyImporter`
  exist. It stores that bindable property pointer.
- `TransitionValueNumberComparator` stores its authored numeric `value`.

Evaluation shape:

- `TransitionViewModelCondition::initialize()` intersects the left/right
  comparator kinds and builds a typed `ConditionComparison`.
- For `BindablePropertyNumber` versus `TransitionValueNumberComparator`, the
  left comparand asks
  `StateMachineInstance::bindablePropertyInstance(originalBindable)` and reads
  the cloned `BindablePropertyNumber::propertyValue()`.
- The normal transition operation table is reused: equal, not-equal,
  less-than-or-equal, greater-than-or-equal, less-than, and greater-than.
- `TransitionViewModelCondition::canEvaluate()` returns false if either
  comparator is a `TransitionPropertyViewModelComparator` and the state-machine
  instance has no `DataContext`.

That last gate matters: even a condition that only compares a cloned bindable
number to a literal number will not evaluate in C++ unless
`StateMachineInstance::dataContext()` is non-null.

## Current Rust State

Already available:

- State-machine-owned `DataBind` targets create per-instance bindable-number
  values in `StateMachineInstance`.
- Those values can now be explicitly mutated by data-bind index.
- `nuxie-binary` already accepts `TransitionViewModelCondition`,
  `TransitionPropertyViewModelComparator`, and
  `TransitionValueNumberComparator` with C++ import-stack parity.
- `RuntimeTransitionCondition` already evaluates regular bool, number, and
  trigger input conditions.

Missing before runtime admission:

- `nuxie-binary` does not project comparator children alongside each
  `TransitionViewModelCondition`; `RuntimeStateTransition.conditions` currently
  stores only the condition object.
- `nuxie-runtime` has no representation for view-model condition comparands.
- `StateMachineInstance` has no data-context-present gate, so it cannot mirror
  C++ `canEvaluate()` yet.
- `tools/cpp-probe` has no action that binds even a minimal data context to a
  state-machine instance.

## Selected Next Slice

The next implementation should be number-only
`TransitionViewModelCondition` support with an explicit data-context presence
gate.

Completion target:

- Extend the binary runtime projection so a `TransitionViewModelCondition`
  exposes its left and right comparator children in C++ import order.
- Support only this comparator shape:
  left `TransitionPropertyViewModelComparator` whose imported bindable property
  is `BindablePropertyNumber`, right `TransitionValueNumberComparator`.
- Add a narrow `StateMachineInstance` data-context-present flag or equivalent
  runtime seam. It should model only C++'s `dataContext() != nullptr` gate, not
  data-context path resolution.
- Extend `tools/cpp-probe` with an action that binds a minimal/empty data
  context to a state-machine instance for comparison.
- Evaluate the condition against the per-instance bindable-number value and the
  authored right-hand numeric value.
- Verify with C++ probe coverage for at least one static number value and one
  mutated number value.

## Scope Lock

This slice must not implement:

- Full `ViewModelInstance` public APIs.
- Data-bind path resolution.
- Source-to-target or target-to-source update queues.
- Data converters.
- `DataBindContextValue` caches.
- Non-number bindable properties.
- `TransitionSelfComparator`.
- `TransitionValueTriggerComparator`.
- Component/artboard property comparands.
- `ListenerViewModelChange`.
- Listener-owned data binding.
- Nested artboard data-context propagation.

Those belong to later state-machine slices or roadmap item `#12: Data Binding
Graph`.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the authored condition exactly number bindable-property versus numeric
   literal?
2. Is C++ comparing the same cloned `BindablePropertyNumber` instance Rust
   stores on `StateMachineInstance`?
3. Is the data-context work limited to the presence gate needed by
   `TransitionViewModelCondition::canEvaluate()`?

If not, defer it to the later data-binding runtime work.
