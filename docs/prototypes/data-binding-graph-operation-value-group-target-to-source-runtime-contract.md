# Data Binding Graph OperationValue Group Target-To-Source Runtime Contract

## Purpose

Admit the first `DataConverterGroup::reverseConvert` runtime graph path for
target-to-source data binding.

C++ reverses converter groups by walking group items from the last child to the
first and calling each child's `reverseConvert`. This slice covers the smallest
numeric case built from already admitted reverse-capable children:
`DataConverterOperationValue` followed by another `DataConverterOperationValue`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a `ToSource | TwoWay` data bind.
- Ordered group items resolving to `DataConverterOperationValue` children.
- Reverse conversion by applying those child converters in reverse item order.
- C++ probe coverage with a representative multiply-then-multiply group, plus
  a second direct `ToTarget` bind to the same source observed through an
  existing blend-state consumer.

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
  child reverse converters from last to first before writing the default
  view-model number source.
- The same source path is marked dirty so a second ordinary `ToTarget` bind
  observes the reversed source value on the same explicit data-context advance.
- Existing direct number, BooleanNegate, and OperationValue target-to-source
  tests still pass.
