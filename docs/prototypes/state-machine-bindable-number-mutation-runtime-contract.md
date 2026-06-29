# State Machine Bindable Number Mutation Runtime Contract

Date: 2026-06-29

This document implements the next slice selected by
`state-machine-bindable-number-mutation-audit.md`: explicit mutation of
per-state-machine-instance `BindablePropertyNumber` clones.

## Formal Goal

Support a narrow C++-verified mutation path for cloned bindable numbers without
implementing full data binding.

The slice is complete when:

- `tools/cpp-probe` can mutate a cloned state-machine
  `BindablePropertyNumber` by resolving a state-machine data-bind target to its
  per-instance clone.
- `StateMachineInstance` exposes a matching Rust mutation path for
  per-instance bindable numbers.
- Existing supported consumers, `BlendState1DViewModel` and
  `BlendAnimationDirect.blendSource=dataBindId`, observe the mutated value on
  the next state-machine advance.
- C++ probe coverage compares state-machine advance reports and final
  component state for both mutated bindable blend-source shapes.

## Scope Lock

This slice mutates only the Rust equivalent of C++'s cloned
`BindablePropertyNumber` stored on a `StateMachineInstance`.

It does not bind a `ViewModelInstance`, resolve a `DataContext`, evaluate
`DataBindContextValue`, run converters, propagate source-to-target or
target-to-source queues, implement observers or polling fallback lists, support
non-number bindable properties, handle listener-owned data binding, or mutate
shared imported source objects.

## Probe Contract

The C++ probe action is:

```sh
--runtime-set-state-machine-bindable-number <stateMachineIndex> <dataBindIndex> <value>
```

C++ resolves `stateMachine->dataBind(dataBindIndex)->target()` to the authored
bindable property, then asks `StateMachineInstance::bindablePropertyInstance`
for the clone and writes `BindablePropertyNumber::propertyValue`.

Rust resolves the same `dataBindIndex` against the bindable-number aliases
recorded from state-machine-owned data binds and mutates only that
`StateMachineInstance`.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it mutate only an existing cloned `BindablePropertyNumber` value?
2. Can C++ verify the same mutation through the probe action above?
3. Does an already-supported runtime consumer observe the value without live
   data contexts, converters, listeners, nested artboards, or rendering?

If not, defer it to the data-binding graph or a later state-machine consumer
slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe state_machine_mutable_bindable_blend_sources_match_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo test -p rive-runtime --test cpp_probe -- --nocapture
cargo check --workspace
make test
make cpp-compare
```
