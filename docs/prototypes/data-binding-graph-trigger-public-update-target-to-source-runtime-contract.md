# Data Binding Graph Trigger Public Update Target-To-Source Runtime Contract

## Purpose

Admit the first public `updateDataBinds(true)` target-to-source path for
state-machine trigger data binds, including the direct `DataConverterTrigger`
reverse-conversion case.

This is a public-update slice only. Existing explicit `advancedDataContext()`
trigger target-to-source behavior and trigger reset behavior stay unchanged.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- Main-`ToTarget | TwoWay` trigger binds.
- Direct no-converter target-to-source public update behavior.
- Direct `DataConverterTrigger` public update behavior where C++ inherited
  `DataConverter::reverseConvert` passes the edited target value through to
  the source.
- Source-to-target reapplication during the same public update, including
  `DataConverterTrigger::convert` incrementing the source value before writing
  the bindable trigger target.
- Exact C++ probe reporting for trigger source and target values after each
  explicit runtime action.

## Out Of Scope

- Trigger listener dispatch, fire-trigger actions, audio/open-url/callback side
  effects, and event propagation.
- Explicit `advancedDataContext()` target-to-source behavior for direct
  `DataConverterTrigger`, covered separately by
  `docs/prototypes/data-binding-graph-trigger-converter-target-to-source-runtime-contract.md`.
- Trigger reset timing beyond preserving existing state-machine advance
  behavior.
- Trigger converter groups beyond the first
  `DataConverterGroup<DataConverterTrigger>` public-update slice covered by
  `docs/prototypes/data-binding-graph-trigger-converter-group-public-update-target-to-source-runtime-contract.md`.
- Same-path trigger observer scheduling, covered by
  `docs/prototypes/data-binding-graph-trigger-public-update-observer-preservation-runtime-contract.md`.
- Imported/owned contexts, relative/parent/nested lookup, pending add/remove
  behavior, re-entry protection, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- A mutated no-converter main-`ToTarget | TwoWay` trigger target can be
  consumed by public `updateDataBinds(true)`.
- A mutated `DataConverterTrigger` main-`ToTarget | TwoWay` trigger target is
  reverse-converted as pass-through into the source during public update.
- The same public update reapplies source-to-target so
  `DataConverterTrigger::convert` increments the source before writing the
  bindable target.
- The C++ probe emits trigger binding source/target rows, and Rust compares
  them for the admitted fixture.
