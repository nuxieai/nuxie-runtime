# Data Binding Graph ToString Converter Group Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for the already-supported cross-type `DataConverterGroup`
shape on main-`ToSource | TwoWay` string binds.

The admitted group starts with `DataConverterToString` for a default-context
number source and then flows through `DataConverterStringPad`. Explicit
data-context advancement observes the edited string target but does not mutate
the number source because group conversion leaves a string value that does not
match the source type. The edited target is preserved through that explicit
pass. The next normal state-machine advance refreshes source-to-target in the
reverse direction; `DataConverterGroup::reverseConvert` runs children from
last to first, `DataConverterStringPad` contributes base reverse identity for
the number value, and `DataConverterToString` yields C++'s default empty
string fallback for the string target.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToSource | TwoWay` string data bind.
- Ordered group items resolving to `DataConverterToString` then
  `DataConverterStringPad`.
- Explicit `advancedDataContext()` target-to-source scheduling that preserves
  the edited string target without mutating the number source.
- Delayed normal-advance source-to-target refresh through C++ group
  `reverseConvert` order, including base reverse identity for the trailing
  string converter child.
- `DataConverterToString` non-string reverse fallback to the default empty
  string target.
- Exact C++ probe reporting for the mutating string bind's target value after
  each explicit runtime action.

## Out Of Scope

- Public `updateDataBinds(true)` behavior for this cross-type group, covered by
  `docs/prototypes/data-binding-graph-to-string-converter-group-public-update-target-to-source-runtime-contract.md`.
- Cross-type converter groups whose first child is not `DataConverterToString`.
- Non-number sources feeding `DataConverterToString` converter groups.
- Direct `DataConverterToString` main-`ToSource | TwoWay` non-string behavior,
  covered by
  `docs/prototypes/data-binding-graph-to-string-non-string-main-to-source-target-to-source-runtime-contract.md`.
- Pure string converter-group main-`ToSource | TwoWay` behavior, covered by
  `docs/prototypes/data-binding-graph-string-converter-group-main-to-source-target-to-source-runtime-contract.md`.
- Number-to-number, list, formula, operation, range, rounder, interpolator,
  number-to-list, and scripted converter groups.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Initial explicit data-context advancement matches C++ for the admitted
  number-to-string converter group.
- Mutating the grouped main-`ToSource | TwoWay` string target and explicitly
  advancing data context does not mutate the number source and preserves the
  edited target.
- The next normal state-machine advance refreshes the target to C++'s empty
  string fallback through group reverse conversion.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Existing direct `DataConverterToString`, direct string converter, pure string
  converter-group, and cross-type group public-update tests still pass.
