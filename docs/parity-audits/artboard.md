# `Artboard` paired audit

Upstream owner: `src/artboard.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-runtime/src/artboard.rs` owns occurrence construction,
  retained objects, dependency ordering, property callbacks, dirty/update and
  advance settlement, nested/component-list recursion, data contexts, audio,
  selection, clone, and teardown.
- `crates/nuxie-runtime/src/draw.rs` owns renderer-late initialization and the
  `drawInternal`/`drawCanvases` execution boundary, including the exact
  `didChange` consumption point.
- `crates/nuxie/src/lib.rs` owns the public file-backed facade and preserves
  C++ construction/frame-loop order while retaining file context.

Verdict: adapted and behaviorally equivalent under the fixed Rust ownership,
Taffy, audio, and scripting ceilings.

The paired audit covered import and object attachment; clone/instance/drop;
component dependency construction and dirt callbacks; data-bind, joystick,
layout, and component update ordering; root and nested advance settlement;
default state-machine and scene selection; audio engine, volume, and sound
teardown; draw-canvas traversal, clipping, transforms, opacity, and
`didChange`; nested/component-list hosts; scripting hooks; and the public
factory-late initialization seam. The earlier FL-D7 closeout remains valid.
X1 semantic-geometry authority and X2 tree-wide host broadcast are additive
Rust host capabilities and do not skip or replace baseline Artboard behavior.

This pass found two genuine hand-port omissions that the earlier structural
audit had not closed:

- Pinned `Artboard::rootTransform(point)` applies a nested artboard's live
  rotation/scale and then recursively maps through its host. Rust had the
  matrices for update/draw but exposed no equivalent behavior. Because Rust
  owns mounted children by value instead of retaining an unsafe host pointer,
  each mounted occurrence now retains the exact host-space matrix supplied by
  its owner and combines it with the live self transform at query time.
  Top-level artboards still return the input point unchanged.
- Pinned `SolidColor::renderOpacityChanged()` calls `Artboard::changed()`,
  whose false-to-true edge immediately propagates through `parentArtboard()`.
  Rust set only the child's `did_change` bit. The child now publishes that same
  guarded edge at the value-ownership seam, and every enclosing Artboard
  consumes and republishes it before nested advance returns. Draw clears the
  change and deferred edge together at the pinned `drawInternal` point.

Evidence:

- `upstream_artboard_transform` runs five behavior cases green; the sixth
  translated silver body remains ignored only because that test surface lacks
  its SRIV comparator.
- `upstream_render_change` runs the complete initial frame plus all ten
  repeated advance/`didChange`/draw assertions green.
- The focused Artboard unit suite runs 245 tests green.
- Port-manifest, source-correspondence, and test-correspondence gates bind this
  audit to the pinned source and active converted tests.
