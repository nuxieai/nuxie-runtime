# Data Binding Graph Artboard Name-Based Source-Path Unsupported Runtime Contract

## Purpose

Pin the first artboard-owned `NameBased` data-bind source-path runtime boundary.

The binary/import model already tracks C++ `DataBindContext::resolvePath()` and
relative view-model property lookup when a `DataBindContext` has a live file
pointer. The admitted runtime path here is narrower: an artboard-owned
`DataBindContext` targeting `ArtboardComponentList`, bound through
`ArtboardInstance::bindViewModelInstance`. In this shape, the C++ runtime probe
reports the component-list target row but does not resolve the name-based list
source.

## In Scope

- Artboard-owned `DataBindContext` records targeting `ArtboardComponentList`.
- Default root view-model contexts bound through
  `bind_default_view_model_artboard_list_context`.
- `DataBindFlags::NameBased` source paths whose first id resolves through the
  file manifest to the `items` property name.
- C++ probe coverage showing the binding row remains present while source list
  size and source number stay absent and the empty target list size is still
  reported.
- Preservation of the existing state-machine-owned `NameBased` unsupported
  boundary.

## Out Of Scope

- Admitting file-backed relative name lookup for runtime source-to-target
  propagation.
- Name-based converter source paths.
- Multi-segment relative paths, parent paths, and nested artboard propagation.
- Artboard component-list item instancing, map-rule selection, layout, and
  virtualization.
- Reverse target-to-source list behavior and generated-list reverse
  converters.

## Completion Checks

- The artboard component-list name-based fixture reports the same unresolved
  source facts and empty target-list fact as C++.
- Rust keeps the artboard data-bind target row addressable by data-bind index.
- Binding the default artboard context returns `false` for this unsupported
  source path.
- The existing state-machine `NameBased` unsupported probe continues to report
  the unresolved cloned-data-bind behavior.
