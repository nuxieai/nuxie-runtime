# State Machine View-Model Number Condition Runtime Contract

Date: 2026-06-29

This document implements the first slice selected by
`state-machine-viewmodel-number-condition-audit.md`: number-only
`TransitionViewModelCondition` support.

## Formal Goal

Support C++ parity for a `TransitionViewModelCondition` whose left comparator
is a `TransitionPropertyViewModelComparator` bound to a
`BindablePropertyNumber` and whose right comparator is a
`TransitionValueNumberComparator`.

The slice is complete when:

- `nuxie-binary` exposes the two comparator children attached to each imported
  `TransitionViewModelCondition` in C++ import order.
- `nuxie-runtime` builds a number-only transition condition from the imported
  comparator pair.
- The condition reads the per-state-machine-instance bindable number clone,
  including explicit mutations made through the existing bindable-number
  override path.
- `StateMachineInstance` has a narrow data-context-present flag so the runtime
  can mirror C++ `TransitionViewModelCondition::canEvaluate()`.
- `tools/cpp-probe` can bind a minimal data context before state-machine
  advance, allowing C++ and Rust to compare the same condition path.
- C++ probe coverage compares state-machine advance reports and final component
  state for both static and mutated bindable-number condition values.

## Scope Lock

This slice does not implement live view-model APIs, data-bind path resolution,
source-to-target or target-to-source queues, data converters,
`DataBindContextValue`, non-number bindable properties,
`TransitionSelfComparator`, trigger comparators, component/artboard
comparands, `ListenerViewModelChange`, listener-owned data binding, or nested
artboard data-context propagation.

The data-context support in this slice is only a boolean presence gate. It
must not resolve paths or push values.

## Probe Contract

The C++ probe action for the presence gate is:

```sh
--runtime-bind-empty-state-machine-context <stateMachineIndex>
```

C++ creates a blank `ViewModelInstance`, wraps it in a `DataContext`, and binds
it to the selected `StateMachineInstance`. Rust sets the corresponding
data-context-present flag on its `StateMachineInstance`.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition exactly `BindablePropertyNumber` versus numeric literal?
2. Does it evaluate only when the state-machine instance has the context
   presence flag set?
3. Does it read only the existing per-instance bindable number value?

If not, defer it to a later data-binding or view-model condition slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_number_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe -- --nocapture
cargo check --workspace
make test
make cpp-compare
```
