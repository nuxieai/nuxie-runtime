# Data Binding Graph Formula Target-To-Source Runtime Contract

## Purpose

Admit the first deterministic `DataConverterFormula` target-to-source runtime
path.

C++ target-to-source converter dispatch follows the data bind's main direction.
For this main-`ToSource` formula bind, the target value is passed through
`DataConverterFormula::convert` before writing the view-model source. This does
not exercise public dirty-list scheduling or the full formula-owned dependency
system, but it does model the immediate same-bind formula refresh C++ performs
after the target-to-source write changes the source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a `ToSource | TwoWay` number data bind.
- Deterministic formula output-queue tokens already modeled by the forward
  formula contract: `FormulaTokenInput`, `FormulaTokenValue`, and
  `FormulaTokenOperation`.
- Main-direction formula conversion before writing the source.
- The immediate same-bind source-to-target refresh caused by the formula
  converter's C++ source dependency after the target-to-source write changes
  the source.
- Exact C++ probe reporting for the mutating formula bind and a second direct
  number bind to the same source path after normal state-machine advancement.

## Out Of Scope

- Main-`ToTarget | TwoWay` formula reverse scheduling.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity.
- Exact `advancedDataContext()` source-to-target scheduling for neighboring
  ordinary `ToTarget` observer binds.
- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula parent-source binding, source dependents, and add-dirt behavior
  beyond the immediate same-bind refresh listed above.
- Formula converter groups.
- Symbol-list-index target-to-source formula writes.
- Non-number sources or targets.
- Number-to-list, generated-list, scripted, and live context-aware converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- Mutating the formula-bound number target on a main-`ToSource` bind runs the
  target value through the deterministic formula before writing the C++ source
  value.
- The same formula-bound target is refreshed from that changed source during
  explicit data-context advancement, matching C++'s formula dependent update.
- A second direct number bind to the same source path observes that written
  source value after normal source-to-target application.
- The mutating and observing bind exact source/target reports match the C++
  probe after each explicit runtime action.
