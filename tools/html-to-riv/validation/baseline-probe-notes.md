# Immutable baseline observation probe

`baseline-probe.rs` depends only on `std`, `nuxie-runtime` and `nuxie-render-api`. It is intended to be linked against the unchanged baseline dependency graph by the root task. No build or execution is claimed by this source handoff.

Suggested CLI:

```sh
baseline-probe scene.riv fresh-output 240x320 390x320 768x320 240x320
```

The probe imports the ordinary file once, settles its original artboard, creates one ordinary instance clone, then applies the requested resize sequence to the original and clone. Instance 0 is the imported default artboard and instance 1 is its clone. The example records eight frames. It does not parse runtime requirements or source maps, install CSS policies, replace fonts, or use custom renderer adapters.

Each frame records all runtime objects exposing both LayoutComponent and WorldTransformComponent, keyed solely by runtime object-table index. `width`/`height` are local layout dimensions; `worldMatrix` is the observed six-component transform. Translation is not asserted to equal a browser border box, and rotated/skewed objects require transforming their corners for any axis-aligned comparison. Objects without these interfaces are outside the layout report, not treated as missing compiler source nodes.

`frame-N.stream` is a cumulative ordinary RecordingFactory stream; replay it with frame index N. `frames.json` records the corresponding instance, step and viewport. The output includes the exact imported `scene.riv` for external hashing. Fractional dimensions go unchanged to artboard sizing; only the recording canvas rounds upward to integer pixels. Probe dimensions are bounded to (0,16384] to avoid accidental huge replay allocations. A fresh output directory is required; failures can leave partial evidence, and no completion manifest is written until all frames succeed.

This measures baseline behavior. It does not prove Chrome equivalence, independent object identity across cloned tables, successful native rasterization, or CSS responsiveness. The external test harness should bind file/binary/source hashes and compare these observations against independent references.
