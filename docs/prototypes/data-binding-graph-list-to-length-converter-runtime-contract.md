# Data Binding Graph List To Length Converter Runtime Contract

## Purpose

Admit the first list-source converter path in the runtime data-binding graph:
`DataConverterListToLength` from a default view-model list source into a number
target.

This slice keeps list support read-only and finite by carrying only the
imported list size needed by the converter.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Imported `ViewModelInstanceList` sources and their attached
  `ViewModelInstanceListItem` children.
- `DataConverterListToLength` feeding `BindablePropertyNumber.propertyValue`.
- Imported-file and imported-instance context rebinding for list length through
  the existing graph source resolution path.
- Main-`ToTarget | TwoWay` state-machine target-dirty behavior is covered by
  `docs/prototypes/data-binding-graph-list-to-length-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source base-reverse/no-write
  behavior is covered by
  `docs/prototypes/data-binding-graph-list-to-length-public-update-target-to-source-runtime-contract.md`.
- Main-`ToSource | TwoWay` target-to-source no-write plus reverse/default
  number refresh behavior is covered by
  `docs/prototypes/data-binding-graph-list-to-length-main-to-source-target-to-source-runtime-contract.md`.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- List targets and `BindablePropertyList` behavior.
- List mutation APIs and update-queue propagation.
- `DataConverterNumberToList` and generated runtime list items.
- Reverse conversion beyond the linked public-update and main-`ToSource`
  base-reverse paths.
- Dirty/update queue parity beyond the linked state-machine target-dirty and
  public-update paths.
- Owned runtime view-model list contexts.
- Relative-path, parent-path, nested-path, and listener-owned behavior.

## Completion Checks

- A default `ViewModelInstanceList` source resolves its imported item count.
- `DataConverterListToLength` writes that count as a number target value.
- The converted number drives the same blend-state report as C++.
