# Data Binding Graph String Converter Family Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for direct `DataConverterStringTrim`,
`DataConverterStringRemoveZeros`, and `DataConverterStringPad` on
main-`ToSource | TwoWay` string binds.

For this direction, explicit data-context advancement observes the edited
string target but C++ leaves the default view-model string source unchanged for
the direct string converter family. The edited target is preserved through that
explicit pass. The next normal state-machine advance refreshes
source-to-target in the reverse direction; these converters do not override
C++ `DataConverter::reverseConvert`, so the target receives the unchanged
source string through the base identity rather than the forward
trim/remove-zero/pad result.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` on main-`ToSource | TwoWay` string data binds.
- Explicit `advancedDataContext()` target-to-source scheduling that preserves
  the edited string target without mutating the string source.
- Delayed normal-advance source-to-target refresh through C++ base
  `reverseConvert` identity.
- Converter-shaped edits that would differ under forward conversion:
  whitespace trim, trailing-zero removal, and pad-to-length.
- Exact C++ probe reporting for each mutating string bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Direct `DataConverterToString` main-`ToSource | TwoWay` string-source
  behavior.
- Main-`ToTarget | TwoWay` target-dirty behavior, covered by the direct
  string converter-family target-dirty contracts.
- Public `updateDataBinds(true)` behavior, covered by
  `docs/prototypes/data-binding-graph-string-converter-family-public-update-target-to-source-runtime-contract.md`.
- Converter groups containing string converters.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Initial explicit data-context advancement matches C++ for each converter.
- Mutating the main-`ToSource | TwoWay` string target and explicitly advancing
  data context does not mutate the string source and preserves the edited
  target.
- The next normal state-machine advance refreshes the target from the
  unchanged string source through base reverse conversion.
- Each mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Previously admitted string converter-family forward, target-dirty, and
  public-update tests still pass.
