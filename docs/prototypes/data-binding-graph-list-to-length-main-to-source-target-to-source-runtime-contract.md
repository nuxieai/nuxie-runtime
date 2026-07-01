# Data Binding Graph List To Length Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for direct `DataConverterListToLength` on
main-`ToSource | TwoWay` list-to-number binds.

For this direction, explicit data-context advancement first attempts
target-to-source. C++ has no `ViewModelInstanceList` source-write branch in
`DataBindContextValue::applyToSource`, so the edited number target does not
mutate the imported list source. The same dirty data-bind pass then applies
source-to-target in the reverse direction because the bind's main direction is
`ToSource`. `DataConverterListToLength` inherits the base `reverseConvert`,
which leaves the list value unconverted; the number target therefore receives
`DataValueNumber::defaultValue`, `0`.

Rust represents the imported list as a finite `ListLength` source fact, so this
slice maps the main-`ToSource` reverse source-to-target refresh to the same
default number result while preserving the unchanged list-length fact.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Imported `ViewModelInstanceList` sources represented by their imported item
  count.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterListToLength` on a main-`ToSource | TwoWay` number
  data bind.
- Explicit `advancedDataContext()` target-to-source scheduling.
- No source write for the list source during target-to-source.
- Reverse source-to-target refresh through C++ base `reverseConvert`, yielding
  the default number target value `0`.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- Main-`ToTarget | TwoWay` target-dirty behavior, covered by
  `docs/prototypes/data-binding-graph-list-to-length-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Main-`ToTarget | TwoWay` public update behavior, covered by
  `docs/prototypes/data-binding-graph-list-to-length-public-update-target-to-source-runtime-contract.md`.
- Public `updateDataBinds(true)` behavior for main-`ToSource | TwoWay`
  `DataConverterListToLength` binds.
- List targets and `BindablePropertyList` behavior.
- List mutation APIs and update-queue propagation from list edits.
- `DataConverterNumberToList` and generated runtime list items.
- Writing numeric target values back into list sources.
- Converter groups containing `DataConverterListToLength`.
- Imported and owned view-model contexts beyond the default root binding.
- Broader dirty/update queues, pending add/remove behavior, re-entry
  protection, relative/parent/nested lookup, listener-owned data binding,
  nested artboards, and render/layout behavior.

## Completion Checks

- Initial explicit data-context advancement writes C++'s reverse/default number
  value for the imported list source into the number target.
- Mutating the main-`ToSource | TwoWay` number target and explicitly advancing
  data context does not mutate the imported list source.
- The same explicit data-context pass refreshes the number target back to `0`
  through base reverse conversion.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Previously admitted `DataConverterListToLength` forward, target-dirty, and
  public-update tests still pass.
