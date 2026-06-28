# Fixture And Comparison Harness

Ticket: `#3`

Reference runtime: `/Users/levi/dev/oss/rive-runtime`

## Findings

The C++ repo already has a useful unit-test asset corpus under:

```text
/Users/levi/dev/oss/rive-runtime/tests/unit_tests/assets
```

The starter Rust corpus should stay intentionally small and headless. The first goal is not visual parity; it is structural parity for file import, artboard lookup, object identity, hierarchy, dependency ordering, and transforms.

## Copied Starter Fixtures

```text
fixtures/
  minimal/
    two_artboards.riv
    long_name.riv
  graph/
    dependency_test.riv
    draw_rule_cycle.riv
    clipping_and_draw_order.riv
  animation/
    smi_test.riv
    state_machine_transition.riv
```

## Why These Fixtures

| Fixture | Use |
|---|---|
| `two_artboards.riv` | Basic file import, default artboard lookup, named artboard lookup. |
| `long_name.riv` | String decoding and object count sanity; C++ expects 7 loaded objects. |
| `dependency_test.riv` | First graph parity fixture: parent chain, dependency order, and world transform after `advance(0)`. |
| `draw_rule_cycle.riv` | Later draw-order regression; C++ verifies it advances and draws without hanging. |
| `clipping_and_draw_order.riv` | Later clipping and draw operation ordering. |
| `smi_test.riv` | State-machine input object import, nested input IDs, nested artboard reference IDs. |
| `state_machine_transition.riv` | Later state-machine stepping and property application. |

## C++ Reference Behaviors To Capture

The first comparison probe should mirror existing C++ tests:

- `two_artboards.riv`: default artboard name is `Two`; named artboard `One` exists.
- `long_name.riv`: default artboard object count is `7`.
- `dependency_test.riv`: artboard name is `Blue`; nodes `A`, `B`, `C`, shape `Rectangle`, and path `Rectangle Path` exist; parents match `Blue -> A -> B -> {C, Rectangle} -> Rectangle Path`; `B` has two dependents; graph order increases down the hierarchy; after `advance(0)`, `Rectangle` world translation is approximately `(39.203125, 29.535156)`.
- `smi_test.riv`: nested artboard named `artboard to nest component` has position `(100, 100)` and `artboardId == 1`; nested state machine/input objects resolve IDs `0`, `0`, `1`, `2`.
- `state_machine_transition.riv`: artboard has 3 animations and 1 state machine; state machine has 1 layer and 6 states; later, after stepping, stroke color transitions from black to white.

## Implemented V0 Comparison Output

`tools/cpp-probe` is now a small C++ probe executable that imports a `.riv` with `NoOpFactory`, returns a process success/failure signal for import-result checks, advances each source artboard by `0` on success, and emits strict JSON.

Proposed JSON:

```json
{
  "path": "fixtures/graph/dependency_test.riv",
  "artboardCount": 1,
  "artboards": [
    {
      "index": 0,
      "name": "Blue",
      "width": 500.0,
      "height": 500.0,
      "objectCount": 17,
      "objects": [
        {
          "localId": 0,
          "coreType": 1,
          "isComponent": true,
          "name": "Blue",
          "parentId": 0,
          "parentLocal": null,
          "graphOrder": 0,
          "worldTransform": [1, 0, 0, 1, 0, 0]
        }
      ],
      "components": []
    }
  ]
}
```

Rules:

- `localId` is the C++ artboard-local object index, not the serialized file index and not a pointer-derived value.
- `coreType` must be the generated Rive type key.
- `parent`, `graphOrder`, and `worldTransform` only appear for component/world-transform-compatible objects.
- `graphOrder` is the post-initialize graph order.
- `components` repeats the component subset for easier comparison.
- Use approximate float comparison with epsilon `0.0001`, matching the unit-test helper.

## Harness Location

```text
tools/cpp-probe/
crates/rive-binary/tests/cpp_import.rs
crates/rive-graph/tests/cpp_probe.rs
```

Build and run:

```sh
make cpp-probe
make cpp-binary-compare
make cpp-graph-compare
make cpp-compare
```

If the reference runtime has not been built yet, build the core archive without renderer dependencies from `/Users/levi/dev/oss/rive-runtime`:

```sh
RIVE_PREMAKE_ARGS="--file=premake5_v2.lua --with_rive_text --with_rive_layout" ./build/build_rive.sh
```

The binary comparison currently writes synthetic forward-compatibility `.riv` files, imports them through the C++ probe and `rive-binary`, and compares import success. The graph comparison checks artboard counts/names, compact artboard object counts, local type keys, component names, serialized parent IDs, and resolved parent local IDs.

## Source Anchors

- File test behaviors: `/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/file_test.cpp`
- C++ test loader: `/Users/levi/dev/oss/rive-runtime/tests/include/rive_file_reader.hpp`
- State-machine input fixture checks: `/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/state_machine_input_test.cpp`
- Draw-order hang regression: `/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/draw_order_test.cpp`
- Public artboard object accessor: `/Users/levi/dev/oss/rive-runtime/include/rive/artboard.hpp`
- Component graph metadata: `/Users/levi/dev/oss/rive-runtime/include/rive/component.hpp`

## Next Step

Use the C++ probe to extend ticket `#7`: compare `graphOrder` and then add explicit dependency edges beyond parent-child hierarchy.
