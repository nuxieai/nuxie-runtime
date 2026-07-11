# Data Binding Graph Runtime Contract

## Scope

This contract defines the runtime boundary for roadmap item `#12: Data Binding
Graph`.

The state-machine runtime now has C++ probe-backed coverage for a finite set of
default-context `propertyValue` source-to-target binds. Those shims are allowed
only as transitional compatibility code. New live data-binding behavior belongs
behind a runtime data-binding graph that owns bind instances, contexts, source
lookups, target writes, update scheduling, converters, and re-entry rules.

The graph starts as a runtime module over imported `RuntimeFile` and
`ArtboardGraph` facts. It does not move binary decoding, import-time schema
projection, or static dependency graph ownership into `nuxie-runtime`.

## Boundary

The data-binding graph owns these live runtime responsibilities:

- binding state-machine and artboard data binds to a concrete context;
- resolving `sourcePathIds`, including future relative, parent, and nested path
  forms;
- mapping a source value to one or more cloned bindable targets;
- mapping supported target mutations back to source values;
- tracking dirty sources, dirty targets, and pending updates;
- executing converters in the C++ order once admitted;
- handling observer push, polling fallback, pending add/remove during
  processing, and re-entry protection;
- exposing public source mutation APIs and external view-model context binding;
- carrying data contexts into nested artboard and listener-owned runtime slices
  once those slices are admitted.

The graph does not own these responsibilities:

- byte decoding, schema generation, import-stack behavior, or unknown-property
  handling in `nuxie-binary`;
- static object dependency edges, draw ordering, host registries, or other
  import-time facts owned by `nuxie-graph`;
- animation advancement, transition selection, hit testing, layout solving,
  text shaping, rendering, audio playback, or scripting execution;
- public scene callback side effects except where callback events later mutate
  a graph-owned data-binding source.

## Admission Rule

Before adding data-binding behavior, answer yes to at least one question:

1. Does it add a graph node, edge, context, source, target, converter, or queue?
2. Does it affect source-to-target or target-to-source propagation timing?
3. Does it define how an external view-model context is bound or mutated?
4. Does it define relative, parent, nested, or listener-owned source lookup?

If the answer is no to all four, the work belongs in a different runtime slice.
If the answer is yes, the implementation should be added through the graph
boundary rather than by adding another per-type dirty flag or apply method to
`StateMachineInstance`.

`nuxie-binary` must not grow live runtime data-binding helpers. It may expose
only import-time facts needed to construct the graph.

## First Implementation Path

1. Introduce an internal `RuntimeDataBindGraph` model with explicit nodes for
   bind instances, concrete contexts, source handles, target handles, converter
   handles, and queued updates.
2. Build the graph from the data-bind registrations already imported into
   `RuntimeFile` and the static host/dependency facts exposed by
   `ArtboardGraph`.
3. Move the existing finite default-context `propertyValue` propagation for
   number, boolean, string, color, enum, asset, artboard, and trigger sources
   behind the graph, preserving all current C++ probe results.
4. Add public source mutation and external-context binding APIs only after the
   graph owns the default-context path.
5. Admit converters, reverse propagation, relative paths, parent paths, nested
   paths, and listener-owned data binding one independently probed slice at a
   time.

## Out Of Scope For The First Graph Slice

The first implementation slice does not need to implement converters,
target-to-source propagation, external contexts, list/symbol/view-model
bindables, relative paths, parent paths, nested paths, listener actions, nested
artboards, callback-driven data binding, or public mutation APIs.

Those behaviors are in scope for item `#12` as a whole, but only after the graph
exists and owns the already-proven default-context source-to-target path.

## Completion Checks

Roadmap item `#12` is complete when:

- all finite default-context `propertyValue` binds currently handled by
  `StateMachineInstance` shims are routed through `RuntimeDataBindGraph`;
- public source mutation and external view-model context binding are covered by
  C++ probe-backed tests;
- dirty queues, pending update behavior, converter ordering, and re-entry rules
  are either C++ probe-backed or explicitly deferred with rationale;
- relative, parent, and nested path handling is covered or intentionally split
  into named nested-artboard/listener follow-up slices;
- no new live data-binding behavior is implemented in `nuxie-binary`;
- remaining data-binding runtime gaps are listed in the runtime audit rather
  than hidden in broad parity language.
