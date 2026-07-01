# Data Binding Graph String Converter Group Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for the already-supported string `DataConverterGroup` shape on
main-`ToSource | TwoWay` string binds.

The admitted group is `DataConverterStringTrim` followed by
`DataConverterStringPad`. For explicit data-context advancement, C++ observes
the edited string target but leaves the default view-model string source
unchanged. The edited target is preserved through that explicit pass. The next
normal state-machine advance refreshes source-to-target in the reverse
direction; `DataConverterGroup::reverseConvert` runs children from last to
first, and both direct string children use the base reverse identity, so the
target receives the unchanged source string rather than the forward
trim-then-pad result.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToSource | TwoWay` string data bind.
- Ordered group items resolving to `DataConverterStringTrim` then
  `DataConverterStringPad`.
- Explicit `advancedDataContext()` target-to-source scheduling that preserves
  the edited string target without mutating the string source.
- Delayed normal-advance source-to-target refresh through C++ group
  `reverseConvert` order and base reverse identity for the direct string
  children.
- A converter-shaped edit that would differ under forward trim-then-pad
  conversion.
- Exact C++ probe reporting for the mutating string bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Direct string converter-family main-`ToSource | TwoWay` behavior, covered by
  `docs/prototypes/data-binding-graph-string-converter-family-main-to-source-target-to-source-runtime-contract.md`.
- Public `updateDataBinds(true)` behavior for the admitted string group,
  covered by
  `docs/prototypes/data-binding-graph-string-converter-group-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for the admitted string group,
  covered by
  `docs/prototypes/data-binding-graph-string-converter-group-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Cross-type converter groups such as `DataConverterToString` followed by
  string converters.
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
  string converter group.
- Mutating the grouped main-`ToSource | TwoWay` string target and explicitly
  advancing data context does not mutate the string source and preserves the
  edited target.
- The next normal state-machine advance refreshes the target from the
  unchanged string source through group reverse conversion.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct string converter-family, string converter-group
  target-dirty, and string converter-group public-update tests still pass.
