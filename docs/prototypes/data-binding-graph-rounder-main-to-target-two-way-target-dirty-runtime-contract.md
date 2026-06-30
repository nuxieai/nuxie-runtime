# Data Binding Graph Rounder Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct `DataConverterRounder`
on main-`ToTarget | TwoWay` number binds.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited target back to the source.
The manual target edit survives explicit `advancedDataContext()`, and the next
normal `StateMachineInstance::advance()` reapplies source-to-target conversion
through `DataConverterRounder::convert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterRounder` on a `TwoWay` number data bind without the
  `ToSource` direction flag.
- Imported `DataConverterRounder.decimals`.
- Initial source-to-target rounder flushing through a normal state-machine
  advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` rounder target-to-source behavior, covered by
  `docs/prototypes/data-binding-graph-rounder-target-to-source-runtime-contract.md`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- `DataConverterGroup::reverseConvert` ordering.
- Formula, operation, range, system, interpolator, number-to-list, list, and
  scripted converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the rounded source value to
  the bindable number target.
- Mutating the rounder-bound target on a main-`ToTarget | TwoWay` bind
  preserves the manual value through explicit data-context advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged source using C++ rounder forward arithmetic.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
