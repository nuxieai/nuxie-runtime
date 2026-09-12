# Source-proven folding of fixed wrapping scenes

The current fixed public subset can eliminate substantial helper work. Its parent and direct empty children have fixed normalized dimensions, a fixed top-left parent origin, no responsive lengths, no padding/gaps/margins, and no text/intrinsic sizing. Artboard resize changes clipping, not the authored layout. That is a valid constant-folding opportunity; the current thousands of rounding records and quadratic paint replicas are not inherently necessary for this subset.

This proposes a new qualified compiler path. It does not change the current output contract or establish support without the checks below.

## What can fold

The fixed source determines stable CSS order, line membership, per-line cross sizes (including source-derived stretch allocation), main/cross reversals, each visible origin, each visible fixed extent, and paint order. The retained source—not a Chrome capture, saved DOM geometry, native probe, or compile viewport—is the input. A fixed container can overflow a smaller artboard; its coordinates must stay unchanged across resize and clone.

The most valuable first target is paint only. `paint_box.rs` emits 2,310 ordinary records per distinct rounded geometry. `wrapping_paint.rs` also builds line-selection masks and candidate replicas, including an O(N²) candidate enumeration. When layout and membership are certified constant, each actual solid rectangle can instead be one ordinary Shape, Rectangle, Fill, SolidColor group, emitted in the already-proven effective paint order. Exact wire object count depends on any retained parent/paint bookkeeping, but four records per active solid rectangle is the primitive target. Existing integral/original-paint certificates remain an even cheaper branch when applicable.

Keep the original unpainted LayoutComponent owners and current position graph in this first experiment. Their object IDs and source-map geometry remain exactly unchanged. Replace only the paint suffix and preserve the existing original-paint hiding rule. This isolates the optimization and can remove the largest cost without simultaneously proving new layout placement semantics.

Later, constant-fold visible positioning too: keep authored LayoutComponent owners for geometry/source identity, but land them using ordinary constant Node targets and the same strength-one TransformConstraint/ComponentOrigin mechanism already qualified in `wrapping.rs`. A source-bound exact physical target per owner can replace line boundary detection and forward/backward max carries. Retaining slots initially avoids depending on unqualified absolute-position inset semantics; merely writing `LayoutComponent.x/y` is not enough because layout may overwrite it. There is no need to delete source owners or replace them with paint Shapes.

## Preserve geometry separately from paint

The source map must keep reporting actual unsnapped authored boxes, including fractional/zero extents and off-artboard positions. Snapped paint is a separate generated Shape/Rectangle. Derive painted edge pairs using the same qualified signed rounding and thin-box rule as the current compiler, not by rounding only width/height. Preserve alpha, logical paint order, and transparent omission semantics.

For paint rectangles, set Rectangle center to `(left+right)/2,(top+bottom)/2` and extent to `right-left,bottom-top` under a root-space Shape; the center/extent representation and its exactness need direct ordinary-record validation. Do not bake clipping at the compile viewport. The immutable artboard clips the fixed rectangle at each runtime viewport. If retaining the current saturation at ±16384, bind its equivalence for every supported positive viewport up to 16384, including edge ties and the raw-size-dependent thin rule. A zero-area painted result may be omitted only while preserving its source geometry owner.

## Proof boundary

A new private certificate should own the normalized parent/leaf source metadata, stable order, direction, content/self alignment, and source record identity. It must prove viewport independence before offering folded constants. Reject percentages, automatic preferred sizes, responsive ancestors, assets/intrinsic content, unknown transforms, animation/drivers, or any arithmetic operation whose exact behavior is not covered. Future responsive paths continue using dynamic composition.

There are two different contracts to check:

1. Browser semantics: source LayoutUnit calculations, line packing, pinned remainder allocation, and paint-edge rules generate the intended CSS result.
2. Optimization equivalence: folded output reproduces the already-qualified original scene's machine geometry/paint for this source across the supported viewport domain.

Do not substitute real-number ideal coordinates for actual f32 graph outputs and call it a semantics-preserving optimization. An independent constant evaluator of the restricted emitted scalar/transform expressions can establish the latter contract, provided it reads actual fields/defaults and follows immutable operation order. Its input is source-built records; evaluating those expressions is compile-time folding, not importing measurements from a browser or renderer. Alternatively prove each admitted arithmetic operation exact and singleton-valued. If the two contracts disagree at a rounding boundary, retain the old route and report the failed folding premise; any intentional visual correction needs its own explicit change and qualification.

The final reader must independently bind each constant target/rectangle against the certificate and actual record fields, not trust an emitter trace or compare against re-emission alone. Mutations of source dimensions, inactive min/max bounds, IDs/ownership, hidden original colors, paint ordering, center/extent bits, or a same-ID certificate from another source must fail. Existing preserved-prefix and closed-record grammar controls remain applicable.

## Cheapest decisive experiment

Add a private paint-only folding constructor for three fixed leaves with a fractional painted edge and reverse flow. Retain the existing layout and scalar graph byte-for-byte; preserve all source-map IDs; change only the paint suffix. Construct twice from exact authored requests using the frozen compiler. Compare old/new actual source-map geometry, Chrome pixels, strict native paint commands, and source-bound constant evaluations at multiple artboard sizes, including clipping and repeated original/clone sizes. Include alpha overlap so incorrect static paint order is visible. Do not add runtime native-probe results to the constructor.

Then broaden to normal/stretch, both main and cross reversals, signed off-artboard edges, thin/zero extents, odd LayoutUnits, exact/near rounding ties, many owners, and fully transparent rectangles. Retain mismatches. Once geometry/stream/pixel equivalence and direct visual inspection pass, benchmark record count, bytes, import, clone, resize, and draw under the same frozen native protocol. This experiment tests the largest saving with the smallest new proof surface.

Separate later milestones are position-graph folding and eliminating now-unused slots/helpers. Each should preserve owner/source identity where possible and provide an explicit source-map migration if object numbering ever changes. Neither static rasterization nor recompile-on-resize is part of this proposal.
