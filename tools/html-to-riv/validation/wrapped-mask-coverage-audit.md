# Wrapping paint mask coverage audit

Read-only source audit, 2026-09-12, immutable worktree. This is a conditional source argument and next implementation plan, not public admission or a new native capture. No runtime, renderer, or compiler source was changed for this audit.

## Main finding

Do not add an arbitrary two-unit mask margin yet. For the existing rectangular, clipped artboard and pinned non-MSAA Metal renderer, the renderer intersects axis-aligned rectangle clips **before** computing antialias coverage. The existing active mask can therefore preserve exactly the original artboard clip, including its boundary antialiasing, rather than multiplying a second half-coverage edge. This is a stronger and simpler route than estimating an AA margin. It must be bound to the actual generated graph and viewport domain before it is a certificate.

The current private composition accepts caller-supplied viewport intervals without a mask-domain check. Thus this finding is not already enforced merely by successful `compose_with_bounds` construction. The public compiler's `(0,16384]` viewport contract must not be confused with a check in the private API.

## Actual mask and native geometry

`src/wrapping_paint.rs`, `Graph::mask`, emits Shape under a scalar gate, then Rectangle with center `(16384,16384)` and dimensions `(32768,32768)`. Native parametric-path defaults have origin `(0.5,0.5)` (`generated/shapes/parametric_path_base.rs:27`). Rectangle construction (`shapes/rectangle.rs:66–105`) makes local corners `(-16384,-16384)` through `(16384,16384)`. The Rectangle translation makes shape coordinates exactly `[0,32768]²`.

Assuming the scalar certificate establishes identity linear transforms, zero inactive-axis translations, and exact gate `0` or `D=65536`, all rectangle corner arithmetic here is exact binary32: active bounds `[0,32768]²`; inactive bounds shifted by `D` on the selected physical cross axis. The leader inversion also needs its own binding: exact initial `D`, factor `-1`, offset=true, strength=1, and source-space/parent-zero premises produce `D-gate` exactly. A generic approximately normalized scalar is insufficient.

`ClippingShape::on_added_clean` collects the mask Shape, marks WORLD and CLIPPING path flags, and adds clip ownership under the ForegroundLayoutDrawable (`shapes/clipping_shape.rs:245–286`). `add_fill_paths` falls back to the raw world path when the mask has no fills (`:353–407`); the generated mask has no fills or effects. `ClippingShapeStart::draw` submits that world path using the renderer state, without installing the painted visible object's transform (`:39–55`). Native rectangle path construction emits straight Move/Line/Line/Line vertices and an optional repeat of the first corner (`shapes/path.rs:450–507`). No radii, multiple contours, skew, or deformation are present in the closed mask shape.

## The artboard clip and renderer intersection

Native `Artboard::default` explicitly sets clip=true (`artboard.rs:209–215`). The current closed base inspector permits no artboard clip override, no radii in its style, and requires top-left origin/default transforms (`src/wrapping_slots.rs:137–164`). Layout render paths use `[0,layout.width] × [0,layout.height]` and zero-radius `Path::add_rounded_rect` (`layout_component.rs:1511–1552`; `shapes/path.rs:298–357`). Artboard drawing saves renderer state, applies any artboard drawing transform, and establishes its local clip before descendants (`artboard.rs:2322–2346`). Descendant draw transforms must remain balanced, so the clipping proxy observes the same renderer matrix as that outer clip; bind this lifecycle premise or confirm it with the actual command stream.

The actual Rust renderer route is:

- `renderer/src/rive_renderer.cpp` port, `RendererContract::clipPath` (`rive_renderer_cpp.rs:1621–1640`): if the frame supports clip rectangles and `IsAABB` recognizes the raw path, call `clipRectImplSource`.
- `renderer/include/rive/renderer/rive_renderer.hpp` port, `IsAABB` (`rive_renderer_hpp.rs:294–331`): accepts first four Move/Line/Line/Line points forming an AABB, with every remaining point equal to the first. This matches both zero-radius artboard and mask paths.
- `render_context_cpp.rs:5218–5223`: rectangle clipping is enabled for every non-MSAA interlock mode, including the pinned Metal RasterOrdering/clockwise-atomic campaign. Do not extend that inference to MSAA without checking clip-plane support.
- `rive_renderer_cpp.rs:1188–1194`: equal current/existing clip matrices return immediately, avoiding inverse/multiply arithmetic.
- `clipRectImplSource` (`:1303–1344`): the combined rectangle is coordinatewise `max` of minima and `min` of maxima. It replaces the current clip rectangle and its inverse with the combined result; it does not accumulate independently antialiased copies.
- `gpu_cpp.rs:2844–2863`: a nonpositive combined width or height produces the Empty inverse matrix. Combined pixel bounds also intersect the existing bounds, preventing draws for a disjoint clip.

Therefore, if `0 < W,H <= 32768`, the active rectangle leaves `[0,W] × [0,H]` exactly unchanged by coordinatewise intersection. If the cross-axis viewport upper bound is below `D`, the inactive rectangle starting at `D` is disjoint. The compiler's intended `<=16384` domain satisfies both inequalities with ample separation. Using the same resulting rectangle and matrix recomputes the same clip inverse and AA coverage. There is **no extra boundary-coverage multiplication and no margin premise** for this route. Zero viewport endpoints may be included as an interval enclosure: they correspond to an already empty clip, not a claim about a drawable zero-sized CSS viewport.

Do not infer that every arbitrary enormous or singular host transform renders correctly. Under identical matrices and combined rectangles, however, the active mask adds no new transform-rounding calculation compared to the existing artboard clip. This equivalence is stronger than selecting a universal artboard-unit AA radius.

## Why this does not prove unbounded overflow or every renderer

The original ordinary artboard already clips outside its own bounds. Negative child coordinates and oversized descendants are observable only within that artboard in this output profile; the mask need only preserve those surviving fragments. This is not permission to change a previously unbounded-overflow contract into clipped output. If artboard clipping is disabled, current masks discard active paint outside `[0,32768]²`; e.g. a rectangle crossing x=-1 loses its negative-x portion. An inactive replica translated by D also becomes visible again if authored overflow paint reaches the translated mask. These are geometric counterexamples to universal mask coverage, not proof that every alternative composition is impossible.

If AABB recognition, equal matrices, or clip-rectangle support fails, separate path clips are used. The shader AA convention is device-space: `common_glsl` defines AA_RADIUS=0.5, and `draw_path_common_glsl` offsets ordinary fill vertices by `sign(...)*AA_RADIUS` (shader-build-authority source, around line748). The clip-rectangle coverage equation explicitly assigns coverage0.5 when a pixel center is on an edge (`common_glsl`, lines372–397). A hardcoded margin of 2 artboard units is not a universal device-space argument under arbitrary host scale. For a fallback full-overflow route, first bound all final painted geometry (including positioning error), then choose mask boundaries outside its device-space AA footprint and require the D-shifted inactive rectangle to remain disjoint. Such bounds depend on the host transform/profile or a stronger renderer equivalence argument. Do not mark that fallback proven from the current source audit.

## Concrete next implementation and tests

1. Add a private mask-domain proof that consumes the bound base scene and viewport intervals, checks the clipped zero-radius artboard contract and `viewport.upper <= 32768` on both axes, and records the fixed rectangle and D sentinel. Keep the intended public16384 limit independently intact. Reject wider private domains as unresolved mask coverage, rather than emitting a purportedly certified candidate.
2. Extend the generated scalar/record certificate to bind every mask Rectangle's center/dimensions/default origins/radii, its one Shape parent, each Shape's exact0/D scalar ancestor, identity transforms, raw-path-only ownership, and clip source/drawable references. Ensure the leader inverse is certified as carefully as membership signals.
3. For the pinned renderer validation, confirm the ordinary emitted artboard and mask raw paths match `IsAABB` and their clipping proxy command matrices equal the outer artboard matrix. A bytes-only command-stream observer is sufficient; it must not change rendering. A renderer source audit explains why those observations imply intersection-before-AA.
4. Run original/clone resize pixels with active paint touching x=0/y=0 and viewport right/bottom boundaries, fractional layout sizes, negative aligned overflow and oversized children, both physical axes, repeated leader/member clips, and the largest admitted viewport. Preserve two clear backgrounds. Include an inactive replica whose underlying geometry crosses the viewport and require zero contribution.
5. Preserve an intentionally wider private viewport rejection and a clip-disabled/off-artboard counterexample separately. Do not enlarge tolerances, infer arbitrary-host-transform qualification, or modify runtime clipping.

The important next change is **binding and testing the existing rectangle-intersection route**, not expanding every mask by an unexplained constant. This audit does not close numeric gate/positioning proofs or overall CSS paint-order admission.
