# C++ Probe

`rive_cpp_probe` is a small parity oracle for the Rust port. It imports a `.riv`
file through the C++ runtime, reports import success through its process exit
status, and emits strict JSON for graph-relevant runtime state on success:

- artboard count, names, dimensions, and object arena size
- per-object core type and component marker
- component name, serialized parent id, resolved parent local id, graph order
- world transform for `WorldTransformComponent` objects

Build the reference runtime from its `tests` project first, then build the
probe:

```sh
cd /Users/levi/dev/oss/rive-runtime/tests
../build/build_rive.sh

cd /Users/levi/dev/rive-rust
make cpp-probe
```

Run it directly:

```sh
tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe --file fixtures/graph/dependency_test.riv
```

Useful probe switches:

- `--property-values` emits `CoreRegistry` getter-backed property values for file-level objects and artboard-local object slots.
- `--file-property-values` emits those getter-backed values only for file-level objects: file assets, view models, view-model properties/instances, data enums, and enum values.
- `--no-advance` skips the `Artboard::advance(0)` call before dumping artboard state, which is useful when comparing import-time member values instead of graph-updated values.

Run the Rust comparison harnesses:

```sh
make cpp-binary-compare
make cpp-graph-compare
make cpp-compare
```
