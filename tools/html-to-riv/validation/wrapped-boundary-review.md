# Wrapped paint boundary evidence

The unchanged private Derived constructor was exercised on 32 additional recipes covering zero/thin extents, positive/negative off-artboard saturation, spanning rectangles, accumulated fractional placement and translucent overlap under both axes and reversals. All 32 construct twice with exactly matching ordinary scene/base/map/trace/proof artifacts. The same original and cloned scenes are resized without recompilation. This campaign retains 28 Chrome/native pixel failures; it does not qualify public wrapping.

## Results and failure preservation

The completed capture contains 256 geometry passes, 228 pixel passes and 28 pixel failures, with all 512 alternate-clear comparisons passing. All 20,608 independent native scalar/edge/mask checks and all 256 strict recorded command streams pass. Pinned Chrome is 153.0.8010.12 and the immutable renderer is Rust Metal RasterOrdering. Pixel gates and the capture driver are unchanged from the preceding 48-case campaign.

The 896 per-axis comparisons contain 92 raw edge differences. Of these, 64 disappear after independently intersecting both rounded intervals with the origin-zero artboard: these are intentional off-artboard saturation differences. All native saturated intersections equal their independently calculated unsaturated native intersections. The other 28 comparisons differ inside the viewport and correspond to the 28 failed pixel frames. No tolerance or expected outcome was changed to suppress them.

| Case family, each axis | Pixel failure frames | Observed discrepancy |
| --- | --- | --- |
| `thin-next-above` | 0–7 (8 per axis) | Chrome resolves the authored next-f32 value above 1/16 to exactly 1/16. Native retains the larger size and emits the one-pixel minimum line. |
| `accumulated-decimal` | 0, 1, 3, 4, 5, 7 (6 per axis) | Separately authored fractional layout operands accumulate differently after Chrome LayoutUnit conversion, producing a one-pixel displacement of the teal rectangle. |

The zero, exact thin threshold, next-below threshold, other admitted thin sizes, saturation/spanning and overlap cases do not acquire those pixel failures. This is evidence for the sampled cases, not arbitrary responsive layout equivalence. The complete per-case observed edge values and failure frames are preserved in `boundary-observation/case-summary.json`.

The primary-source audit in `wrapped-boundary-quantization-review.md` distinguishes native paint rounding from Chrome's earlier conversion of resolved layout lengths to 1/64 units. It records the source-derived predictions, observed confirmation and proposed compiler-side conversion work. Runtime changes are not needed or proposed by this evidence. Percentage used-value quantization and later layout arithmetic remain separate unresolved work.

## Validation boundary

The actual constructor binary is reused byte-for-byte from `wrapped-rounded-constructor-r1` (SHA-256 `69ba5be02217cc432133ce2fa6b8bc07f23bab55f0f0a152fe5dd773b94740d7`). Its frozen build/input/source receipts remain bound. Its preceding product build passed 419 Rust tests, 56 Node tests, native/WASM builds, TypeScript and the immutable source guard. Those results establish the unchanged product identity; they are not newly run tests for this validation-only corpus.

The observer recomputes selected scalar operations from actual native geometry, then checks every stream's masks, colors, draw order and save/restore behavior. Five modified-stream controls reject. Source maps and proof metadata are read only after ordinary `.riv` import and do not drive native layout or drawing. Stream analysis is not direct GPU-internal inspection.

All 96 distinct full-size Chrome/native/diff representatives across 26 sheets were directly inspected in saved root/agent notes. The notes explicitly confirm the failing extra thin lines and displaced teal rectangles. They also record matching sampled saturation and translucent overlap scenes. The other 160 frames are exact same-case pixel/geometry transfers, including clones and repeated sizes. Passing metrics and visual agreement are not claims of RGBA identity.

`wrapped-boundary-evidence.py` binds and replays the actual observations, insists the known failure set stays intact, and writes a compact evidence receipt only after constructor, baseline, source, stream, image and direct-review checks succeed. Public support counts remain unchanged. No runtime, renderer, schema, shared dependency or root build configuration was edited.
