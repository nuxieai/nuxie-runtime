# Data Binding Graph OperationValue Group Target-To-Source Runtime Contract

## Purpose

Admit the first `DataConverterGroup` runtime graph path for target-to-source
data binding.

C++ target-to-source converter dispatch follows the binding's main direction.
For the representative `ToSource | TwoWay` fixture in this contract,
`DataConverterGroup::convert` runs children from first to last before writing
the view-model source. Main `ToTarget` two-way reverse group order remains a
separate target-to-source variant covered first by
`docs/prototypes/data-binding-graph-operation-value-group-public-update-target-to-source-runtime-contract.md`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a `ToSource | TwoWay` data bind.
- Ordered group items resolving to `DataConverterOperationValue` children.
- Main-direction group conversion by applying those child converters in item
  order.
- C++ probe coverage with a representative multiply-then-multiply group,
  including exact source/target number binding reports for the mutating bind.

## Out Of Scope

- Mixed-type converter groups.
- Stateful `DataConverterInterpolator` children.
- `DataConverterOperationViewModel`, system-operation converters, range mapper,
  formula, string, number-to-list, list, or scripted converters in reverse
  groups.
- Null/missing group item converter behavior beyond preserving existing
  imported child resolution.
- Imported and owned view-model contexts.
- Pending dirty queues, pending add/remove behavior, observer-list parity, and
  re-entry protection.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated numeric target on a grouped target-to-source bind is run through
  child converters according to C++ main-direction dispatch before writing the
  default view-model number source.
- The mutating bind's source and target values are compared directly against
  the C++ probe after each explicit runtime action.
- Existing direct number, BooleanNegate, and OperationValue target-to-source
  tests still pass.
