# Data Binding Graph Default Context Migration Runtime Contract

## Scope

This slice introduces the first internal `RuntimeDataBindGraph` implementation
for roadmap item `#12`.

The migration is intentionally behavior-preserving. It moves the already-proven
finite default-context `propertyValue` source-to-target binds out of
`StateMachineInstance`'s per-type dirty/apply methods and into a graph-owned
default binding queue. It does not add new live data-binding features.

## Runtime Rule

When `StateMachineInstance::bind_default_view_model_context` succeeds, the
runtime data-binding graph marks its default-context binding queue dirty. Before
state-machine layer evaluation, the graph applies the sorted queue to cloned
bindable targets for the currently supported value families:

1. `BindablePropertyNumber.propertyValue`;
2. `BindablePropertyBoolean.propertyValue`;
3. `BindablePropertyString.propertyValue`;
4. `BindablePropertyColor.propertyValue`;
5. `BindablePropertyEnum.propertyValue`;
6. `BindablePropertyAsset.propertyValue`;
7. `BindablePropertyArtboard.propertyValue`;
8. `BindablePropertyTrigger.propertyValue`.

The source values and target identities remain the same ones built by the
existing import/runtime projection. The graph owns only runtime context state,
the single default-context dirty bit, and the ordered target-write queue for
this slice.

## Out Of Scope

This slice does not implement external contexts, public source mutation APIs,
target-to-source propagation, converters, observer registration, polling
fallback, pending add/remove, re-entry protection beyond the existing one-shot
dirty bit, list/symbol/view-model bindables, relative paths, parent paths,
nested paths, listener-owned data binding, nested artboard propagation, callback
driven data binding, or any new `nuxie-binary` helper.

The per-bindable default source metadata remains as construction input for the
first graph queue. Moving that metadata fully into a static graph definition is a
later cleanup once the runtime graph grows additional node families.

## Completion Checks

- `StateMachineInstance` no longer owns eight per-type default-context dirty
  flags or eight per-type default bind apply methods.
- `RuntimeDataBindGraph` owns data-context presence, default-context binding
  state, and the default source-to-target queue.
- Existing C++ probe-backed default view-model bind tests still pass for all
  eight supported value families.
- The roadmap and remaining-runtime audit record that the finite default bind
  set is now graph-routed, while later live data-binding behavior remains out of
  scope.
