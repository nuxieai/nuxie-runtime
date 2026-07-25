# F-ED-03 — Exact generic property binding in Scene authoring

## Objective

Close the runtime-repository part of `RT-ED-005` by making Scene authoring
express the generic numeric and color `DataBind.propertyKey` contract already
implemented by pinned C++ and the Rust runtime.

This is an authoring-interface parity omission, not a new low-level runtime
mechanism. No frame-loop-owned runtime code may change.

The complete reported slice includes layout padding. Landing only a generic
binding for existing visual property tokens leaves P09-C01 partially open.

## Pinned C++ ownership contract

Pin: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

The transferred Editor localization used historical pin
`f4bb3025e263ad1a646ef6971358577a0aa6bfa2`. It remains provenance, not the
F-ED oracle: the relevant source set is not byte-identical because the later
pin adds property notifications, target observation, and explicit
source-first/target-first reconcile handling. Per `COR-01`, every assertion
below and every registered source hash is revalidated at `d788e8ec`.

- Serialized bind state is `propertyKey`, flags, and converter ID:
  `include/rive/generated/data_bind/data_bind_base.hpp:30-96`.
- The source path belongs to `DataBindContext`:
  `include/rive/generated/data_bind/data_bind_context_base.hpp:32-53`.
- Target identity is exact record adjacency. The file reader retains the most
  recent non-DataBind object and assigns it as the following DataBind's target:
  `src/file.cpp:299-325`.
- Import transfers a bind to its exact state-machine, converter, formula, or
  artboard container:
  `src/data_bind/data_bind.cpp:57-162`.
- A retained `DataBind` owns dirt, target, source cell, context value,
  converter, and container:
  `include/rive/data_bind/data_bind.hpp:20-58,123-135`.
- Source resolution and retained rebinding:
  `src/data_bind/data_bind_context.cpp:15-90`;
  `src/data_bind/data_bind.cpp:210-371`.
- Generic number application dispatches by `CoreRegistry::propertyFieldId`,
  calls `setDouble`, or applies C++ negative clamping and rounding before
  `setUint`:
  `src/data_bind/context/context_value_number.cpp:11-37`.
- Generic color application calls
  `CoreRegistry::setColor(target, propertyKey, value)`:
  `src/data_bind/context/context_value_color.cpp:11-20`.
- Source/target apply, reconcile, and direction ordering preserve that exact
  property key:
  `src/data_bind/data_bind.cpp:429-580`;
  `src/data_bind/data_bind_container.cpp:115-225`.
- Artboard clone retargets each cloned bind to the cloned object:
  `src/artboard.cpp:1038-1057`.

Canonical upstream behavior:
`tests/unit_tests/runtime/data_binding_test.cpp:39-111` binds number to
rectangle width, converted number to rotation, color to fill color, text, and
boolean, mutates the sources, and advances once.

## Existing faithful Rust execution

The Rust binary/graph/runtime layers already retain and execute arbitrary
property keys:

- binary target adjacency:
  `crates/nuxie-binary/src/lib.rs:5646-5695,5968-5976`;
- graph bind fields:
  `crates/nuxie-graph/src/lib.rs:1157-1169,1520-1540`;
- runtime occurrence ownership:
  `crates/nuxie-runtime/src/artboard_data_bind.rs:414-461`;
- arbitrary typed construction:
  `artboard_data_bind.rs:2993-3137`;
- exact dirty target/property application:
  `artboard_data_bind.rs:7137-7181`;
- generic numeric/color typed setters, including C++ clamp/rounding:
  `artboard_data_bind.rs:7817-7851`;
- target dirt/notification propagation:
  `crates/nuxie-runtime/src/artboard.rs:1986-1999,2345-2380`.

These modules are under the active FL lease and are read-only for F-ED-03.

## Scene omissions

Current Scene authoring is opacity-shaped:

- `NumberBindSpec` has no target property:
  `crates/nuxie/src/scene.rs:2557-2562`;
- public authoring exposes only opacity:
  `scene.rs:11578-11601`;
- validation hardcodes `WORLD_OPACITY`:
  `scene.rs:11623-11639`;
- collision identity is only `target`, not `(target, propertyKey)`:
  `scene.rs:11705-11717`;
- export always emits the opacity key:
  `scene.rs:21275-21320`;
- no general typed color-binding interface exists.

Layout padding has a separate owner omission:
generated `LayoutComponentStyle` records currently have no Scene semantic
target identity (`scene.rs:21357-21416`). A padding DataBind must immediately
follow that style record; assigning it to the component list would violate
C++ target adjacency.

## Exact repair

### A. Generic typed property binding

1. Replace the opacity-shaped internal bind with one C++-shaped property bind
   retaining:
   - target semantic identity;
   - typed `Prop<T>`;
   - typed source;
   - optional converter;
   - direction.
2. Expose public numeric and color bind methods using `Prop<f32>` and
   `Prop<u32>`. Direct binds accept an explicit direction without requiring a
   converter; `ToTarget` convenience methods remain available. Do not expose
   raw schema keys.
3. Keep current opacity methods as compatibility wrappers over
   `props::WORLD_OPACITY`.
4. Key collision/uniqueness by `(target, property.key)`, allowing two
   different bound properties on the same target while rejecting two owners
   for the same property.
5. Emit `DataBindPropertyKey(property.key)` immediately after the exact target
   record, together with converter, flags, and source path.

### B. Layout-style owner

1. Generate typed `LayoutComponentStyle` padding property tokens for stored,
   bindable Double keys:
   - padding left `512`;
   - padding right `513`;
   - padding top `514`;
   - padding bottom `515`.
2. Give every component-list style a stable Scene-side semantic target owned
   by that component-list occurrence.
3. Author numeric padding binds against that style target.
4. Export each padding DataBind immediately after the generated
   `LayoutComponentStyle` record.
5. Preserve target identity through Scene transaction rollback and export
   determinism.

This follows PORTING AF-1, AF-2, AF-4, AF-8 and RF-27. Do not create a second
authoring-only property application mechanism.

## TOUCH / DON'T TOUCH

`TOUCH`:

- `crates/nuxie/build.rs`;
- `crates/nuxie/src/scene.rs`;
- `crates/nuxie/tests/scene_authoring.rs`;
- `crates/nuxie/tests/data_converter_authoring.rs`;
- this specification;
- narrowly supported F-ED atlas/status evidence after green verification.

`DON'T TOUCH`:

- `crates/nuxie-graph/src/lib.rs`;
- every FL-reserved
  `crates/nuxie-runtime/src/{artboard,artboard_data_bind,components,constraints,draw,focus,lib,objects,retained_data_bind,text}.rs`;
- runtime `animation.rs`, `state_machine.rs`, and `state_machine/**`;
- frame-loop manifests/status and shared file-correspondence rows;
- Editor Next source;
- renderer code, thresholds, tolerances, timeout, memory caps, or corpus
  membership.

## Vertical TDD

Test only through public Scene/export/import/runtime interfaces, one
RED→GREEN behavior at a time:

1. Numeric property tracer: bind a number source to rectangle width; export
   and assert the DataBind immediately follows the rectangle with the exact
   width key and source path.
2. Same-target independence: bind width and another numeric property on one
   target; both export and execute. A second bind for the same
   `(target, propertyKey)` fails atomically.
3. Color tracer: bind one color source to multiple `SolidColor.colorValue`
   targets and prove exact export/import/runtime application.
4. Converter-bearing and converter-free direction variants preserve the
   independent optional converter ID and C++ direction flags. Exact-import
   tests cover direct `ToSource` plus source-first `TwoWay` reconciliation.
5. Layout owner: bind all four padding properties; assert each DataBind is
   adjacent to its exact `LayoutComponentStyle`, never the component list.
6. Relative/list-item source: two occurrences update independently through
   the same authored property bind.
7. Cold re-import executes the same number, color, converter, direction,
   padding, and occurrence-local results.
8. Invalid type/target/owner combinations and transaction rollback remain
   fail-closed.

## Acceptance

Focused:

```sh
cargo test -p nuxie --lib
cargo test -p nuxie --test scene_authoring
cargo test -p nuxie --test data_converter_authoring
cargo test -p nuxie-runtime --lib
```

Full:

```sh
make cpp-probe
make cpp-oracle-workspace-tests
make golden-compare
make scripted-golden-compare
make renderer-golden
make capi-smoke
make size-report
RIVE_CPP_PROBE="$PWD/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe" \
  cargo test --workspace
make lint-gate
cargo fmt --all -- --check
git diff --check
```

Required unchanged floors:

- ordinary and scripted: 317/317 entries, 647/647 segments, zero failures;
- C++ probe/workspace green;
- renderer corpus 1,468/1,468;
- both SDK variants below 9 MiB;
- no gate or error path loosened.

After green, update only the `RT-ED-005` atlas/status fields supported by the
runtime-repository evidence. Orchestrator verification and Editor consumption
remain pending. The immutable Editor checkpoint is
`27ef7d471c3034aba4a4b839d2c8150d3bcb40c3`.
