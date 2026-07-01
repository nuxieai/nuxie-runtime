# Data Binding Graph System OperationValue Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit grouped `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` public `updateDataBinds(true)`
target-to-source behavior for main-`ToTarget | TwoWay` number binds.

System converters choose their operation-value direction from the owning data
bind. This slice proves that the runtime graph preserves that C++ rule when a
system converter is a child of `DataConverterGroup`, not only when it is the
top-level data-bind converter.

## In Scope

- Default root view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Main-`ToTarget | TwoWay` state-machine `DataBindContext` records targeting
  `BindablePropertyNumber.propertyValue`.
- A direct root `ViewModelInstanceNumber.propertyValue` source.
- A `DataConverterGroup` containing `DataConverterOperationValue` followed by
  either `DataConverterSystemNormalizer` or `DataConverterSystemDegsToRads`.
- Propagating the owning data-bind flags into grouped system-converter child
  construction.
- `StateMachineInstance::update_data_binds_apply_target_to_source`, matching
  C++ `StateMachineInstance::updateDataBinds(true)`.
- Exact C++ probe comparison for source, target, state-machine advance, and
  component update reports.

## Out Of Scope

- `ToSource`-only grouped system-converter public-update behavior.
- System converter groups with symbol-list-index inputs.
- Other system converter group compositions beyond the two-converter shape
  named above.
- Formula, interpolator, number-to-list, list, string, mixed stateful groups,
  scripted converters, and broader dirty-list scheduler parity.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  imported and owned runtime contexts, listener-owned data binding, nested
  artboards, layout, and rendering.

## Completion Checks

- A grouped `DataConverterSystemNormalizer` public update matches C++ after
  reverse conversion, source write, and immediate source-to-target
  reapplication.
- The same grouped behavior is pinned for `DataConverterSystemDegsToRads`.
- Grouped system-converter child construction receives the owning data-bind
  flags instead of falling back to an unsupported converter.
- Existing direct system-operation and neighboring converter-group public
  update tests continue to pass.
