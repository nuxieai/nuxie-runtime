# State Machine ViewModel Asset Condition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the enum
`TransitionViewModelCondition` slice. It closes the narrow asset-id comparison
shape that C++ supports for view-model bindable properties.

## Formal Goal

Implement `BindablePropertyAsset` transition conditions in `nuxie-runtime`: when
the left comparator is a `TransitionPropertyViewModelComparator` resolved to
`BindablePropertyAsset` and the right comparator is
`TransitionValueAssetComparator`, read the per-instance asset id and compare it
with C++'s asset operation behavior.

The goal is complete when the runtime slice can:

- Project state-machine-owned `BindablePropertyAsset` data-bind targets into
  per-instance runtime state.
- Read the imported `BindablePropertyAsset.propertyValue` default inherited
  from `BindablePropertyId`.
- Mutate that per-instance asset id through the data-bind-index testing seam.
- Evaluate asset bindables against `TransitionValueAssetComparator.value`.
- Match C++ operation behavior for asset comparisons: equality and inequality
  compare ids, while ordered operations evaluate false.
- Compare no-context, static-equal, mutated-equal, static-not-equal, and
  ordered-operation cases against the C++ probe.

## Scope Lock

This slice owns only asset bindables used as
`TransitionViewModelCondition` comparands.

It does not implement:

- view-model, trigger, artboard, or general id comparands;
- component asset comparands;
- asset loading, asset resolution, asset replacement APIs, or image/audio
  content behavior;
- live data-context APIs, data-bind queues, source/target observers,
  converters, delegation suppression, dirt propagation, or push/poll
  scheduling;
- nested artboard contexts or nested animation/state-machine forwarding;
- listener-owned view-model dispatch, keyframe callbacks, rendering, scripting,
  audio, or open-url behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the condition a `TransitionViewModelCondition` whose left comparator is a
   `BindablePropertyAsset` view-model property and whose right comparator is an
   asset id literal?
2. Can the value come from imported state-machine data-bind target metadata or
   explicit per-instance mutation?
3. Can it be compared through the existing C++ state-machine probe?

If not, defer it to a later data-binding, non-asset-comparand, component, or
runtime behavior slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe state_machine_viewmodel_asset_conditions_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
