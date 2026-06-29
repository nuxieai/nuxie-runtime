# State Machine ViewModel Pointer Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the asset-id
`TransitionViewModelCondition` slice. It closes the narrow
`BindablePropertyViewModel` pointer-comparison shape without pulling in the
live data-binding scheduler.

## Formal Goal

Implement `BindablePropertyViewModel` transition conditions in `rive-runtime`
for the case where both sides of a `TransitionViewModelCondition` are
`TransitionPropertyViewModelComparator` objects resolved to
`BindablePropertyViewModel` targets.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyViewModel` data-bind targets
  into per-instance runtime state.
- Represent the two pointer sources needed for this slice:
  - `null`, when the bindable has no live view-model instance pointer;
  - the current root data-context view-model instance, when the bindable target
    is driven by a `DataBindContext` whose `sourcePathIds` buffer has exactly
    one id.
- Apply C++'s `TransitionViewModelCondition::canEvaluate` data-context gate:
  view-model property comparands do not evaluate without a bound data context.
- Match C++ pointer comparison behavior: equality and inequality compare
  pointer identity, while ordered operations evaluate false.
- Compare no-context, root-equal, root-vs-null not-equal, null-equal, and
  ordered-operation cases against the C++ probe.

## Scope Lock

This slice owns only view-model pointer comparisons where both comparands are
state-machine-owned `BindablePropertyViewModel` targets.

It does not implement:

- trigger/self `TransitionViewModelCondition` behavior;
- `TransitionSelfComparator`;
- `TransitionValueTriggerComparator`;
- nested, relative, manifest-name, or multi-id data-context source paths;
- `DataBindContext::source()` resolution to `ViewModelInstanceViewModel`;
- `DataBindContextValueViewModel::apply` or push/poll data-bind scheduling;
- explicit public mutation APIs for bindable view-model references;
- component view-model comparands;
- listener-owned view-model dispatch, `ListenerViewModelChange`, scripting,
  rendering, animation keyframe callbacks, hit testing, or nested state-machine
  forwarding.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition comparing two `TransitionPropertyViewModelComparator`
   objects whose latest bindable properties are both
   `BindablePropertyViewModel`?
2. Can each side be modeled as either `null` or the current root data-context
   view-model pointer?
3. Can the behavior be compared directly against the C++ state-machine probe?

If not, defer it to a later trigger/self, data-binding, nested-context,
component-comparand, listener, or runtime-scheduler slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_viewmodel_pointer_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
