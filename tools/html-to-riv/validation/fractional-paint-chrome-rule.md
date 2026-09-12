# Pinned Chrome CSS rectangle paint rule

Source audit, 2026-09-12. This is source evidence, not a new render qualification. No runtime or compiler implementation changed, and no browser geometry was used to construct native output.

## Identity and scope

Chromium tag `153.0.8010.12` resolves to commit `971a7443b0c9b0a9b2860529b33331b76077ec62`, tree `f3b8516f22967cf75d95541bf324421ed6c0be4a`. Downloaded primary files, their exact URLs, byte lengths and SHA-256 hashes are preserved in `output/fractional-paint-chrome-source-r1/manifest.json`; the raw Gitiles response is `tag-commit.json`. These establish tagged source identity, not independent attestation of the installed Chrome executable build.

The relevant path is a regular CSS layout box with a solid background, no radius, border shape, background image, animation/worklet, scrolling background, fragmentation, forced colors or background transfer to the document view. The normal box painter uses its paint offset and fragment size. Positioning mode does not introduce a separate rounding rule here: an unpositioned or absolutely positioned regular box reaches the same painter after layout has supplied its offset. Root/body background propagation is explicitly outside this argument. [BoxFragmentPainter, lines 1382–1438 and 1927–1960](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/box_fragment_painter.cc#1382).

## Both background paths snap

The simple bottom-layer fast path builds a pixel-snapped color rectangle when the fill is not rounded, then fills it. The general color-background path also builds a pixel-snapped rectangle before filling. Thus inspecting only the general path would miss an optimization, but both paths agree for the scoped solid rectangle. No-radius handling must not be generalized to rounded backgrounds or shaped clipping. [BoxPainterBase, lines 995–1047, 1102–1110, 1254–1264 and 1460–1467](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/box_painter_base.cc#995).

`PhysicalRect` constructs the output from its rounded offset and separately snapped width/height. Each extent is snapped with its corresponding location as an argument; this is **not** independent rounding of the authored width and height. [PhysicalRect, lines 158–166 and 228–230](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/geometry/physical_rect.h#158). Offset rounding invokes `Round()` for each coordinate. [PhysicalOffset, lines 108–125](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/physical_offset.h#108).

## Exact ordinary-range formula

For each axis, let `l` and `s` be the **already computed LayoutUnit** paint location and size; these are signed integers in units of 1/64 px. Define `q` as truncation of `l` toward zero and `f=l-q`. Away from saturating arithmetic limits:

- `R(v) = floor(v + 1/2)`, including negative ties toward positive infinity.
- Painted start is `R(l)`.
- Candidate painted extent is `d = R(f+s)-R(f)`.
- If `d == 0` and `abs(s) > 1/16 px`, extent becomes `sign(s)`; otherwise it stays `d`.

For a positive ordinary box whose candidate extent is nonzero, this is equivalent to independently rounding the two paint edges: `[R(l), R(l+s))`. The thin-box exception prevents that edge-equivalence from being universal. The source uses the signed remainder and fractional origin to avoid unnecessary large-coordinate additions. LayoutUnit saturation and preceding layout quantization must not be replaced by floating-point algebra in an implementation. [LayoutUnit, lines 243, 294–297, 329–333, 473 and 805–817](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#294).

Examples derived from those integer operations, not newly observed pixels:

| Location | Size | Painted interval |
| --- | --- | --- |
| 64.5 | 45 | [65,110) |
| 64.25 | 45.5 | [64,110) |
| 64.75 | 45.5 | [65,110) |
| -0.5 | 1 | [0,1) |
| -1.5 | 1 | [-1,0) |
| 0 | 1/16 | empty |
| 0 | 5/64 | [0,1) despite equal rounded edges |

Consequently the reported Chrome coverage of rows 65 through 109 for y=64.5, height=45 is predicted by box-background snapping. This audit alone does not attribute every observed alpha or edge mismatch to this operation.

## Cumulative origin and transforms

Use the paint offset, not each element's parent-relative coordinate rounded independently. Child painting adds offsets before the background rectangle is formed. When a paint-offset translation node is introduced, Chromium normally splits the offset into an integer translation plus a retained fractional remainder. The split preserves snapping, subject to isolation/compositing exceptions. Therefore the future compiler cannot snap each ancestor separately and expect ordinary nested boxes to agree. [BoxFragmentPainter, lines 1161–1184](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/box_fragment_painter.cc#1161), [paint property builder, lines 736–783](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/paint_property_tree_builder.cc#736).

Snapping occurs in the current paint coordinate space. GraphicsContext forwards the resulting rectangle to its paint canvas; it does not perform another explicit device-edge rounding in these methods. At DPR 1, unit page scale and without an additional fractional transform, integer paint edges coincide with device-pixel edges. A CSS transform is not permission to apply the above formula to the final `getBoundingClientRect()`: local snapping precedes the transform, and nontranslation transforms may discard accumulated fractional offsets. Rotation, scaling, isolated/composited layers and nonunit DPR require their own validation. [GraphicsContext, lines 686–691 and 719–745](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/graphics/graphics_context.cc#686), [paint property builder, lines 703–734](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/paint_property_tree_builder.cc#703).

SVG rectangles and paths are a distinct reference: SVGShapePainter passes floating bounding boxes to rectangle drawing, or the original path to path drawing, without the CSS `ToPixelSnappedRect` step in that dispatch. This does not prove arbitrary SVG painting matches the native renderer; it proves that a CSS background and an SVG rectangle are not interchangeable Chrome baselines for fractional-edge behavior. [SVGShapePainter, lines 193–216](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/paint/svg_shape_painter.cc#193).

## Compiler implication

A faithful responsive file-level composition would need to derive paint-only snapped cumulative edges (including the thin-box rule) from the live unsnapped layout, while preserving layout and hit geometry. It must not round authored dimensions once, bake browser measurements, or snap every ancestor separately. This source result motivates testing an ordinary Rive composition capable of discontinuous rounding; it does **not** establish that such a composition is possible, impossible, safe within object limits, or qualified. Existing wrapping geometry successes remain geometry evidence; fractional pixel failures stay failures.
