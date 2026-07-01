# Data Binding Graph Default ViewModel List Source Mutation Runtime Contract

## Purpose

Close the default view-model list source mutation gap after imported list.

Default source mutation already covers the scalar-ish `propertyValue` source
family. Lists need the same narrow runtime seam for their observable item
count, without admitting generated list item identity or list layout behavior.

## In Scope

- Default root view-model contexts bound with
  `StateMachineInstance::bind_default_view_model_context`.
- Direct `ViewModelInstanceList` source item-count mutation by state-machine
  data-bind index.
- C++ probe comparison through existing `BindablePropertyList` source-size and
  target-value reports after data-context advancement.

## Out Of Scope

- Imported or owned contexts; those are covered by their own contracts.
- List item identity, generated list item runtime instances, item-level
  traversal, map-rule selection, layout, and virtualization.
- Property-name APIs for list mutation.
- Reverse conversion for generated lists and broader update-queue parity.

## Completion Checks

- Mutating a default list source item count updates the bound graph source.
- The state-machine bindable-list report matches C++ after data-context
  advancement and ordinary advancement.
- Existing default and imported list probes continue to pass.
