# `ScriptedInterpolator` paired audit

Upstream owners: `src/scripted/scripted_interpolator.cpp` and the lazy clone
path in `src/animation/linear_animation_instance.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-runtime/src/scripted_interpolator.rs` owns per-keyframe table
  identity, clone-local ScriptInput/DataBind/converter state, ordered binding,
  fallback behavior, advancement, diagnostics, and teardown.
- `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` owns the
  shared `ScriptedObject::cloneProperties` binding lifecycle.
- `crates/nuxie-runtime/src/artboard.rs` supplies the exact occurrence-local
  Artboard DataContext and parent chain.
- `crates/nuxie-scripting/src/vm.rs` owns the Luau table and callbacks.
- `crates/nuxie/src/lib.rs` authenticates the referenced assets and performs
  the VM/runtime handoff at the source-prescribed lifecycle points.

Verdict: adapted and behaviorally equivalent under the fixed Rust scripting
boundary.

row_id: "B6-0323"; upstream: "src/scripted/scripted_interpolator.cpp"; verdict: ADAPTED;

The audit verified identity and linear fallbacks for `transform`; linear
fallback for `transformValue`; protected-call failure behavior; asset retention
across clones; one table per `(LinearAnimationInstance, keyframe)`; authored
ScriptInput cloning; occurrence-local DataBind and native-converter state;
complete parent-scoped DataContext resolution; live source refresh; stateful
converter advancement in clone order; and DataBind teardown before its target
table.

The audit found one real port omission. Rust cloned nested
`ScriptedDataConverter` definitions but the interpolator factory never
instantiated or attached their clone-specific Luau tables. The correction
reuses the already source-proven `ScriptedObject` binding walk: bind the outer
source, bind each converter occurrence, instantiate and hydrate a scripted
converter at its authored position, rebind its final inputs, then hydrate and
initialize the interpolator table. No fallback algorithm was added.

`cloned_interpolator_instantiates_its_scripted_data_converter_occurrence` is an
imported binary fixture with two real protocol assets. It proves the converter
generator and `init` precede `convert`, and that the converted value reaches
the cloned interpolator input. The shared binding-owner tests independently
bind nested scripted converters through complete parent DataContext chains,
prove distinct tables per occurrence, and prove stateful advancement. The
facade now drives that same retained implementation for interpolator clones.
Pinned upstream has no direct `ScriptedInterpolator` `TEST_CASE`, so the unit
test correspondence denominator is unchanged.
