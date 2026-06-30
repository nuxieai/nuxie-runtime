# Data Binding Graph Node Table Runtime Contract

## Scope

This slice deepens the first `RuntimeDataBindGraph` implementation without
changing observable runtime behavior.

Default-context source-to-target bindings are now graph edges. Each edge points
at a source node carrying the resolved value and a target node carrying the
cloned bindable target identity. This prepares the graph for source mutation and
external context binding without introducing either behavior yet.

## Runtime Rule

When the default view-model context is bound, `RuntimeDataBindGraph` applies the
dirty default binding edge list in data-bind index order:

1. read the edge's source handle;
2. read the edge's target handle;
3. apply the source node value to the target node's cloned bindable target.

The supported value families remain exactly the finite `propertyValue` set from
the previous graph migration slice: number, boolean, string, color, enum,
asset, artboard, and trigger.

## Out Of Scope

This slice does not deduplicate nodes, expose public source handles, mutate
source node values, bind external contexts, run update queues, evaluate
converters, support reverse propagation, resolve relative/parent/nested paths,
or move the per-bindable import-time source metadata into a static graph
definition.

## Completion Checks

- `RuntimeDataBindGraphDefaultBinding` stores source and target handles instead
  of embedding the target identity and value directly.
- `RuntimeDataBindGraph` owns source and target node tables.
- Existing C++ probe-backed default view-model bind tests still pass for all
  supported value families.
- The roadmap records source-node mutation as the next natural implementation
  point for live data-binding work.
