# State Machine Bindable Number Blend Runtime Contract

Date: 2026-06-29

This document implements the first slice identified by
`state-machine-bindable-blend-source-audit.md`: static
`BindablePropertyNumber.propertyValue` reads for blend sources.

## Formal Goal

Support the C++ runtime path where state-machine-owned data binds create
per-instance bindable-property clones, and blend sources read the cloned
`BindablePropertyNumber.propertyValue` during state-machine advance.

The slice is complete when:

- `rive-binary` exposes the C++ data-bind target and latest bindable-property
  lookup needed by `rive-runtime` without making runtime reconstruct import
  stack state.
- `rive-runtime` collects imported `BindablePropertyNumber.propertyValue`
  values for state-machine-owned data binds whose targets are bindable numbers.
- Each `StateMachineInstance` owns per-instance bindable-number values seeded
  from the imported runtime model.
- `BlendState1DViewModel` can use the per-instance bindable number as its 1D
  blend value.
- `BlendAnimationDirect.blendSource=dataBindId` can use the per-instance
  bindable number as its direct blend mix source.
- C++ probe coverage compares state-machine advance reports and final component
  state for both bindable blend-source shapes.

## Scope Lock

This slice owns only imported static number values. It does not bind an
external view-model instance, evaluate `DataBind` update queues, run data
converters, propagate source-to-target or target-to-source changes, support
non-number bindable properties, or implement listener-owned data binding.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it read an imported `BindablePropertyNumber.propertyValue`?
2. Is the value reachable through a state-machine-owned `DataBind` target,
   matching C++ instance clone ownership?
3. Can it be verified with the existing C++ runtime probe without live data
   contexts, converters, listeners, nested artboards, or rendering?

If the answer is no to all three, defer it to later data-binding runtime work.

## Verification

Focused verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_bindable_blend_sources_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo test -p rive-runtime --test cpp_probe -- --nocapture
cargo check --workspace
make test
make cpp-compare
```
