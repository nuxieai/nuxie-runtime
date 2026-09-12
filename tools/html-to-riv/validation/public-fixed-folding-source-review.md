# Private fixed paint folding: independent source review

Reviewed `fixed_folding.rs` against the immutable renderer and existing typed binders without using native capture outcomes as premises. No concrete descriptor/order/source-binding error was found within the currently bound fully rounded rectangle grammar. This is a conditional source review, not pixel or GPU equivalence qualification.

## Rectangle arithmetic and its deliberate narrow scope

`quad` accepts one Rectangle per Shape, default origins/radii and identity-linear Shape worlds. Native `shapes/rectangle.rs:47–85` first generates vertices at `-origin*size` and `-origin*size+size`; the Rectangle's own x/y transform is applied afterward. `quad` instead calculates `center-size*0.5`, then adds size and Shape translation. These orders are not equivalent for arbitrary f32 rectangle fields. The current immutable `Derived` paint binder restricts these rectangles to width/height 32768 and centers ±16384 (replicas and line masks use +16384 on both axes). In this grammar the local corner calculations are exact small integers. Rounded box masks and binary membership masks have integer evaluated translations; their sums remain far within exact f32 integer precision. Therefore this reassociation does not establish a current case failure, but **the helper must not be generalized to arbitrary fractional/large rectangles without matching native vertex/transform order explicitly**. During review, the parent agent narrowed the comment and added an explicit descriptor guard requiring integer Shape translations with absolute value at most 65536. I inspected that guard. Together with the typed rounded grammar, every relevant integer sum is well below 2^24; the noted reassociation is now explicitly conditional in the source. Full Rectangle world matrices should be included separately in evaluator validation; Shape translations alone do not observe path transforms.

`descriptors` checks every requested Shape's four linear coefficients equal identity before using `quad`. Thus its axis-aligned intersection is not silently applied to rotated/skewed geometry. Finite quad checks precede intersection; surviving edges must be integers in [-16384,16384]. Empty and transparent replicas need no emitted rectangle, but malformed or nonfinite source quads still reject. Generic float min/max can change a zero sign, but output edges are geometric integer coordinates and do not claim a bitwise identity of path buffers.

## Clip and ordering semantics

Each replica's four initial clip source IDs are matched to the exact mask tuple in the candidate's independently bound paint trace, retaining its geometry owner. Additional line masks are intersected from actual records, not ideal CSS positions or inferred source order. The first rectangle is the actual large replica extent, so its existing [0,32768] restriction is retained rather than replaced by an ideal unbounded fill. Native clipping uses nested clip operations (`shapes/clipping_shape.rs:41–55`); a set intersection of axis-aligned filled rectangles describes their geometric overlap. Shared-edge antialias coverage, non-unit display transforms, GPU behavior and renderer pixel rules require separate evidence; this source argument alone does not prove raster equality under every display configuration.

`order` requires one complete acyclic chain over the entire replica set and exact After placement/ownership. Native `artboard.rs:1211–1234` inserts an After group after its target in the linked list, sets the draw entry to the list tail, and `draw_drawables:2375–2380` walks `prev`. Thus an A After(B) rule draws A before B. Walking the reader's A→B edges yields back-to-front paint order. `fixed_paint::Folded::new` reverses that descriptor list for insertion, matching ordinary reverse-order drawing. Transparent/inactive removal occurs only after complete order validation; it retains relative order and does not deduplicate same-owner active replicas. This matters for alpha blending.

The preserved artboard continues clipping at draw time. The fold does not cap descriptors to the constructor's viewport, so later artboard resizing can reveal previously cropped fixed paint. Native artboard clipping and background precede drawable traversal. This relies on the existing admitted identity artboard and no unmodeled prefix clip/effect/order semantics.

## Source ownership and prefix preservation

The input must be an immutable `Derived` with `PaintProof::Ordered`, with integral/original routes and ForegroundLayoutDrawable explicitly rejected. This is not an untrusted arbitrary-record parser: existing source/paint binders establish the complete suffix grammar and bind trace mask IDs. Full candidate record count must equal base+sizing+paint; box spans lie within that paint suffix. All non-artboard prefix SolidColors must already be hidden, preserving the original ordinary artboard background. The new output preserves the entire base+sizing prefix byte-for-byte through `fixed_paint` and replaces only the paint suffix.

`fixed_scalar::evaluate_derived` checks exact actual base bytes/count and the candidate's retained normalized source against the geometry binding. `Folded` retains the complete original graph bytes plus an owned geometry binding. `validate` requires both exact graph identity and `same_binding`, recomputes descriptors from actual fields, and passes those descriptors and prefix to the independent fixed-paint validator. Equal coordinates do not authorize swapping source owners or authored numeric provenance. Remaining prefix constraints can continue evaluating; this change does not yet optimize their import/update cost.

The rectangle-operation-order limitation is now explicit and guarded in the source. Separately, the scalar agent extended the certified-paint evaluator to include Rectangle matrices through the actual PathBase→Node transform hierarchy; origin shifts remain vertex coordinates, not world-transform seeds. Native confirmation of that extension is separate. No fixed_folding source code was changed by this reviewer.

All runtime paths above are relative to `crates/nuxie-runtime/src/mechanical_port/source/`.

## Reviewed file hashes

- `tools/html-to-riv/src/fixed_folding.rs`: `8a02fde49274cfb8922ddb03ce8e62b449da979412e7bf0d9b025566cdfc46b3` (after bounded-integer guard/comment repair)
- `tools/html-to-riv/src/fixed_paint.rs`: `777605dfec7fd3a1a8b4630620f8432c3de62471c4919525bf06f644758e7fb8`
- `tools/html-to-riv/src/wrapping_paint_binding.rs`: `989323fe6133208eb611710e555c37d761cf3dc003fba6d3fb12c8328721b27f`
- `tools/html-to-riv/src/wrapping_paint.rs`: `ac4c225f331f8f7532acd08e42e9836f1a354e955f391de79733f66aba8e4bc0`
- `crates/nuxie-runtime/src/mechanical_port/source/shapes/rectangle.rs`: `a7cc0d6c1698593dd3da3df3980fd1ae439cdb51187a5762c9241332df812bc3`
- `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs`: `6f218337cbf8daa1567f20244cadd9f0676cb1e6546ae0bc23751d02b404da01`
- `crates/nuxie-runtime/src/mechanical_port/source/shapes/clipping_shape.rs`: `466f8cb71255d1c082efdd41b27ee4f31b2f11cc3f4bdb5e512cf798539732d3`
