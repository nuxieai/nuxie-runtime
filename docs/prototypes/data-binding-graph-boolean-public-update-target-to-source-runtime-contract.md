# Data Binding Graph Boolean Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the first
state-machine boolean data-bind family: direct boolean binds and direct
`DataConverterBooleanNegate`.

This slice closes the public-update counterpart of the existing explicit
`advancedDataContext()` direct boolean and BooleanNegate target-to-source
coverage.

## In Scope

- State-machine-owned `DataBindContext` objects resolved through the default
  root view-model context.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources.
- `BindablePropertyBoolean.propertyValue` targets.
- Main-`ToTarget | TwoWay` boolean binds.
- No-converter public target-to-source writes from the edited bindable target
  into the boolean source.
- Direct `DataConverterBooleanNegate` public target-to-source behavior where
  C++ symmetric reverse conversion negates the edited target before writing the
  source.
- Source-to-target reapplication during the same public update.
- Exact C++ probe reporting for boolean binding source and target values after
  each explicit runtime action.

## Out Of Scope

- Boolean converter groups.
- Boolean public-update behavior for imported or owned contexts.
- Broader dirty-list scheduler parity beyond the first same-path direct
  boolean observer slice covered by
  `docs/prototypes/data-binding-graph-boolean-public-update-observer-preservation-runtime-contract.md`,
  pending add/remove behavior, and re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` boolean target is written to the
  default boolean source during public `updateDataBinds(true)`.
- A mutated direct `DataConverterBooleanNegate` main-`ToTarget | TwoWay`
  boolean target is reverse-converted by negation before writing the source.
- The same public update reapplies source-to-target so Rust and C++ report the
  same boolean source and target values after each explicit action.
- Existing direct boolean, BooleanNegate, and public-update converter probes
  continue to pass.
