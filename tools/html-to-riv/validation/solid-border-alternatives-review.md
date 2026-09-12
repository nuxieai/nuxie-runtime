# P01 ordinary border alternatives

Three additional **ordinary-file composition candidates** preserve the passing square-border cases but do not resolve the fractional-edge pixel failures. This is a private finite-fixture experiment; it does not admit borders in the public compiler or qualify P01. The immutable runtime/schema/renderer/dependencies remain unchanged at `6c7ac16617835b5f581784ff08a9e779bb52faf3`.

The new useful result is a responsive **single compound ring** using existing NSlicedNode objects. It updates both outer and inner contours through ordinary layout and preserves the fixed border bands. A naive two-Rectangle Shape cannot do this because ordinary Shape control-size updates only its first parametric path. The NSlicedNode provides the missing ordinary-file mapping, so that earlier obstacle should no longer be treated as evidence against all responsive compound rings.

## Candidates and authored representation

All three retain the semantic owner's native four border fields, background, padding and content. Decorative siblings do not clip the semantic content. Absolute positioning remains relative to the owner's padding box, so negative insets compensate for the semantic border. Emitted subtrees remain contiguous and object references/source maps are remapped exactly as in `solid-border-candidate-review.md`.

| Candidate | Ordinary arrangement | Added records per nonzero border |
|---|---|---:|
| `clipped-double` | Absolute decorative LayoutComponent, all insets `-b`, both dimensions Auto/Fixed, clip=true. One miter Stroke of width `2b` follows its live border-box path; its own clip removes the outer half. | 4: layout, style, stroke, color |
| `inset-stroke` | Absolute decorative LayoutComponent, all insets `-b/2`, both dimensions Auto/Fixed, clip=false. One miter Stroke of width `b` follows the inset path; its centered stroke reaches the semantic outer boundary. | 4: layout, style, stroke, color |
| `sliced-ring` | Absolute decorative LayoutComponent with all insets `-b`, dimensions Auto/Fixed, clip=false. Its NSlicedNode contains an even-odd filled Shape with an outer and inner Rectangle and fixed-edge slice axes. | 12: layout/style, NSlicedNode, four axes, Shape, two Rectangles, fill/color |

For `sliced-ring`, the authored reference is a **4b × 4b square**, derived solely from the border width. The outer Rectangle covers `(0,0,4b,4b)`; the inner Rectangle covers `(b,b,2b,2b)`. Both have explicit zero origins. AxisX and AxisY each have nonnormalized offsets `b` and `3b`. The initial/serialized NSlicedNode width and height are all `4b`.

The native NSlicedNode is an intrinsically sizeable ordinary object. Its layout control-size callback updates its width/height, and `should_propagate_size_to_children=false` preserves both authored Rectangle dimensions. Slice mapping keeps the first and last segments fixed while stretching the middle segment. The semantic native border floor supplies final dimensions at least `2b`, including the zero-middle-size floor fixture. This is not a browser-baked layout: the authored square depends on `b`, not browser measurements, source viewport, captured geometry or requested render sizes. Both contours respond when the original and clone are resized without touching the file.

The equal initial width/height also avoids a specific baseline source caveat: the Y-axis slice analysis uses `size.x` as its reference size. An unequal reference rectangle cannot be assumed to preserve the requested Y bands. This candidate deliberately emits a square and does not change the runtime. Nonuniform side widths, elliptical radii and other slice configurations remain untested.

The executable freezes the previous source-backed public compiler library and wire writer, compiles each fixture's border-free CSS, verifies a decode/encode byte round trip, inserts the complete candidate graph, then emits the final ordinary RIV before import. Fixture metadata selects the candidate at compile time only. Source-map files serve read-only geometry joins; native import/render/resize receives only the RIV and standard viewport sizes. There are no runtime setters, host CSS logic, adapters, scripts, raster assets or runtime-policy sidecars.

## Results on unchanged gates

Pinned Chrome **153.0.8010.12**, DPR 1, the unchanged reset, immutable Rust Metal RasterOrdering, existing pixel gates and 0.1px authored-box geometry thresholds are retained. All files compile once at 390×160 and are observed at 240×160 → 390×200 → 768×120 → 240×160 on the same original and clone. Cyan/transparent clear controls remain enabled.

| Mode | Scenes / frames | Geometry passes | Pixel passes | Clear checks |
|---|---:|---:|---:|---:|
| Clipped doubled stroke | 9 / 72 | 72 | 52 | 144 / 144 |
| Inset stroke | 9 / 72 | 72 | 52 | 144 / 144 |
| Sliced compound ring | 9 / 72 | 72 | 52 | 144 / 144 |
| Total | 27 / 216 | 216 | 156 | 432 / 432 |

Every mode has the same bounded pass/fail classification:

- All eight geometry/pixel frames pass for integer background/content, translucent border over translucent background, visible content overflow, border floor and border-plus-padding floor.
- Fractional-origin translucent-border controls pass the metrics on all eight frames, but retain a visible and numerically measurable coverage difference. This is not an alpha-based admission workaround.
- Nested borders pass pixels at 240px (four original/clone/repeat frames) and fail at 390/768px, while geometry matches at all sizes.
- The minimal opaque fractional-origin border and the composed fractional-edge scene fail pixels at all eight frames, while geometry matches.

The previous strip candidate and all initial failures remain untouched. The new drivers use the existing public-baseline comparison infrastructure; their receipt label does not confer public admission on a private candidate executable. No tolerance or antialias exclusion changed.

## What the alternatives explain

The minimal failure is a 100×60 opaque border box at `(5.25,5.25)`, with a 3px border, no background and no children. At its 240×160 frame:

| Topology | Mismatched pixels | Mean channel error |
|---|---:|---:|
| Prior four strips | 624 | 0.5030598958 |
| Clipped doubled stroke | 616 | 0.4948828125 |
| Inset stroke | 616 | 0.4948893229 |
| Sliced compound ring | 616 | 0.4964843750 |

The seven above-threshold corner pixels in the prior translucent-strip control reduce to zero for the Stroke candidates and two for the sliced ring. Topology therefore affects a few corner/junction pixels, but **does not fix straight-edge coverage**. At pixel `(20,5)` every opaque native candidate, including the prior strips, produces RGBA `(191,89,115,255)`; Chrome produces `(170,34,68,255)`. For the translucent control, every native candidate produces `(223,172,185,255)` and Chrome produces `(212,144,161,255)` at that same location. These values are captured directly from the preserved full PNGs.

The prior ordinary background-only fractional-origin control also failed the pixel gate. Together, these observations show that changing border paint topology is insufficient for these concrete failures; they do not prove that every file composition or every fractional geometry is impossible. The runtime's ordinary coverage remains different from Chrome's hard pixel edge in these tested arrangements. A compiler should keep the affected profile unresolved/diagnosed until it has an evidence-backed paint contract, an admissible geometric subset across resizing, or another concrete ordinary-file candidate. Merely swapping strips for a Stroke or compound ring must not silently enable the failing inputs.

## Source, visual and lifecycle evidence

`output/solid-border-alternatives-r1/source-audit.json` verifies ten relevant runtime/schema source files byte-for-byte against the baseline. In particular:

- `layout_component.rs` implements ordinary layout paint, layout clipping and size propagation.
- `shapes/paint/stroke.rs` supplies the ordinary stroke path/paint; miter join and width are file properties.
- `shapes/shape.rs` selects only its first parametric path for direct control-size.
- `layout/n_sliced_node.rs` implements control-size, blocks descendant size propagation, installs the axis mapping and contains the Y-reference caveat.
- `math/n_slicer_helpers.rs` keeps even-indexed edge bands fixed and scales odd-indexed interior bands.
- AxisX/AxisY register their offsets on the imported NSlicedNode.

There are **48 distinct image pairs** in this matrix. Sixteen are exact full-RGBA matches to previously inspected candidate pairs. The other **32 pairs were directly inspected at original resolution** in 13 paired sheets; the generator verifies every pasted crop against the original full RGBA image. The coverage file binds all **216 frames**: 116 transfer directly to the previously reviewed evidence, 32 are new direct inspections, and 68 transfer to those new inspections. Every transfer compares actual complete RGBA arrays, not a thumbnail, region or metric. File/geometry/request/image identities remain separate even when their rendered pixels are equal.

The frozen final executable reproduces all **27 RIV files and source maps exactly**, including the earlier Stroke run. All repeated original/clone viewport PNGs are identical within their respective fixture and viewport. The native observer measured every authored DOM ID and asserted identity linear transforms; comparison includes genuine visible-overflow and floor controls. It does not certify non-layout helper geometry numerically or the complete numeric/resource domain.

Entry points relative to `tools/html-to-riv`:

- `validation/solid-border-alternatives-emitter.rs`, `-build.py`, `-cases.json`: all three executable border candidates, 27 border fixtures and four diagnostic primitive controls.
- `validation/solid-border-alternatives-receipt.json`: compact exact-hash receipt, run totals, source identity, topology comparison and reproduction.
- `validation/solid-border-alternatives-visual.py`: full-size sheet generation and exact transfer binding.
- `output/solid-border-alternatives-r1/render/gallery.html`: both Stroke matrices.
- `output/solid-border-alternatives-r1/r2/render/gallery.html`: sliced compound ring matrix.
- `output/solid-border-alternatives-r1/visual-coverage.json`: complete frame-to-review identity mapping.

Rebuild with `python3 tools/html-to-riv/validation/solid-border-alternatives-build.py FRESH_NAME`; use the resulting candidate and the fixtures with the unchanged `check-public-baseline.mjs` driver and fresh output paths. Original and r2 source snapshots, binaries, receipts and failures are retained. Public compiler/library source and shared documentation are unchanged.

## Representability and implementation implications

All candidates use constant extra geometry and linear extra records per border. Strokes use fewer records than the 16-record strip lowering. The sliced ring uses 12 records and requires native path deformation for the two contours; it creates no runtime image/asset dependency. No claim is made about GPU/CPU speed or tested peak memory. The helper graph, recursive remap, numeric bounds, added layout depth, NSlicedNode transform arithmetic and mutation/lifecycle resource boundaries still require qualification before production admission.

The single-ring result is a reusable ordinary-file representation worth retaining independently of the fractional-edge issue. It may support later border/radius compositions, but those possibilities need explicit fixtures and error analysis. For uniform square borders, the smaller inset-stroke graph is also a viable representation within the tested positive profile. Neither representation currently justifies broader public acceptance than the strips evidence.

Next work should treat the shared paint-coverage issue independently from the choice of border representation. Preserve the minimal fractional-origin opaque/background/alpha controls as the deciding tests; avoid spending further iterations on topologies that yield the same sampled straight-edge color. Any public border integration still needs CSS parsing and computed-width provenance/quantization, diagnostics for unresolved combinations, actual ancestor/resize admission checks, interaction/resource tests, and public Rust/CLI/WASM/JS parity. Keep P01 investigating rather than declaring arbitrary uniform borders qualified or impossible.


## Diagnostic primitive control: direct layout Fill versus Shape/Rectangle

A separate four-case control isolates the parent's minimal fixed-box failure: `<div id="p"></div>` with `#p{height:25.5px;background:#14233f}` at a 100×80 viewport. The positive control uses height 25px. Each is emitted using the ordinary LayoutComponent Fill and using a Shape with one Rectangle/Fill instead. The Rectangle begins with zero authored dimensions and receives the semantic owner's size through the baseline's ordinary control-size path; no browser rectangle or host setter supplies its geometry. The existing owner background Fill is reparented to the Shape, avoiding duplicate paint.

The Shape/Rectangle and direct-layout variants produce **identical full native PNGs** at both heights. At height 25.5, their layout geometry is 100×25.5 and pixel `(20,25)` is `(138,145,159,255)` in both native outputs, versus Chrome's `(20,35,63,255)`. Both fail the unchanged mismatch-ratio and mean-channel-error gates. At height 25, both pass and row 25 is white. The difference is therefore reproduced with either ordinary paint primitive; the experiment does not support blaming direct LayoutComponent background painting alone.

These diagnostic files are observed repeatedly at **the same 100×80 viewport**, on original and clone; they add no resize-domain claim. The four files give 32 geometry passes, 16 pixel passes and 64 clear passes across 32 frames. Both distinct image pairs were directly inspected at full size; exact full-RGBA identity binds all remaining frames and both primitive variants. All four RIV files/maps reproduce exactly with the frozen r3 executable. Main border matrices and all earlier failures remain unchanged.

`output/solid-border-alternatives-r1/r3/render/gallery.html`, `primitive.png` and `primitive-receipt.json` retain this control. `validation/solid-border-alternatives-primitive.py` verifies complete image identity, unchanged-size sheet crops and exact file reproduction. The primary receipt binds this supplemental receipt separately from the 27-scene responsive border matrix.
