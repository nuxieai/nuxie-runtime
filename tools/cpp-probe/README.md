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

- `--fingerprint` prints the SHA-256 fingerprint of the probe's build inputs
  that `build.sh` embedded at compile time. The Rust comparison harnesses check
  it before spawning the probe and fail with "cpp-probe binary is stale — run
  make cpp-probe" when the binary no longer matches the sources.
- `--property-values` emits `CoreRegistry` getter-backed property values for file-level objects and artboard-local object slots.
- `--file-property-values` emits those getter-backed values only for file-level objects: file assets, view models, view-model properties/instances, data enums, and enum values.
- `--no-advance` skips the `Artboard::advance(0)` call before dumping artboard state, which is useful when comparing import-time member values instead of graph-updated values.
- `--runtime-random-reset` opts runtime state-machine probes into the counted deterministic `RandomProvider` shim and resets its call count.
- `--runtime-random-value value` queues one deterministic random value for the counted provider; repeat the flag to seed multiple draws. Without these runtime random flags, the probe preserves C++ `std::rand()` behavior.

Run the Rust comparison harnesses:

```sh
make cpp-binary-compare
make cpp-runtime-compare
make cpp-compare
```

## Text width ownership

`--text-measure-width-samples <new_text.riv> <Inter_18pt-Regular.ttf>` imports a
fresh file for each of three cases: no participant (`participant: 0`), fill
width (`1`), and fixed width (`2`). Participants use hug height. Every case has
text `Choose`, AutoHeight sizing, authored width 1, font size 40, line height
44, and an exact layout offer of 354. JSON reports the measured width and
height before `controlSize`. Use the assets from `RIVE_RUNTIME_REF`.

This is the independent oracle for `upstream_text_measure_width.rs`. Upstream
uses the offer only when a fill/fixed participant owns the text's width axis;
otherwise it caps measurement at authored width. AutoHeight's reported width
remains authored width in all three cases.
