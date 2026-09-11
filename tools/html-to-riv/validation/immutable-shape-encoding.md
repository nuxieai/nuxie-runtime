# Ordinary Rive shape encoding on the immutable baseline

Read-only source investigation, 2026-09-11, in `html-css-immutable`. No runtime edits, binary experiment, GPU comparison or responsive-support qualification is claimed. The arrangements below are concrete candidates for the parent's bytes-only importer experiments. All referenced paths are relative to repository root and refer to this immutable worktree, not the archived CSS/radial branch.

The strongest existing representation is **a Fill directly parented to a LayoutComponent**. It already paints a live rounded layout rectangle; a separate Rectangle is unnecessary for ordinary responsive solid backgrounds. Borders need a separate decision: the existing layout border fields reserve space, while an ordinary Stroke is centered on its path. Neither is automatically a CSS inside border.

## Record dictionary

Object IDs below mean indices resolved by the artboard import context, not type IDs. `parentId` is property5; serialized record ordering and zero/default omission must follow the existing writer/importer. A property's wire type must come from the generated reader dispatch, not from the numeric property's name. Numeric object IDs are unsigned; geometry is float32; color37 is packed ARGB color; flags/bools must use their generated encodings.

| Record type | Ordinary properties / relationships |
|---|---|
| LayoutComponent **409** | parent5; width7, height8; styleId494 points to its LayoutComponentStyle; clip196; fractionalWidth706/fractionalHeight707. The width/fractional behavior depends on imported file-version compatibility; do not substitute the style's newer inherited width keys without checking that path. |
| LayoutComponentStyle **420** | parent5 should point to its layout owner; owner styleId494 points back to this record. widthUnits607/heightUnits608: Undefined0, Point1, Percent2, Auto3. border L/R/T/B504/505/506/507, units609/610/611/612; padding512–515, units617–620; margin508–511, units613–616. Position L/R/T/B516–519, units621–624; positionType597: Absolute2. corner link639, TL640, TR641, BL642, BR643. |
| Shape **3** | parent5; ordinary node x13/y14. It owns Paths and ShapePaints through parent relationships. |
| Rectangle **7** | parent5=Shape; width20/height21; originX123/originY124; x13/y14 inherited node coordinates. corner link164, TL31, TR161, BL162, BR163. Path flags128, isHole770 are inherited path properties. Use explicit origin0 for a top-left authored box; avoid relying on centered defaults. |
| Fill **20** | parent5=LayoutComponent, Shape, or ForegroundLayoutDrawable; fillRule40; visible41; blendMode747. Its SolidColor is a separate child record. FillRule NonZero0, EvenOdd1, Clockwise2 in render-api; use ordinary NonZero unless a tested compound-path representation specifically needs another rule. |
| SolidColor **18** | parent5=Fill or Stroke; colorValue37=ARGB. It installs the paint mutator on its parent paint. |
| Stroke **24** | parent5=LayoutComponent, Shape, or ForegroundLayoutDrawable; thickness47, cap48, join49, transformAffectsStroke50; visible41/blendMode747 inherited. SolidColor child supplies color. No existing “inside CSS border” property is established. |
| ForegroundLayoutDrawable **513** | parent5=LayoutComponent; Fill/Stroke children operate on the **parent layout's existing path**, not independently authored Rectangle geometry. Artboard special handling arranges it at the foreground of the layout subtree. |
| ClippingShape **42** | parent5 defines the subtree receiving clip; sourceId92 references a Node-compatible source whose descendant Shapes supply paths; fillRule93; isVisible94 controls source visibility. A LayoutComponent itself is Node-compatible but has no descendant Shape merely because it has a Fill: its generated background path is not gathered as a source Shape. Prefer layout clip196 for that path. |
| Plain Node **2** | Useful coordinate/draw grouping, x13/y14. Crucially it stops layout content-size propagation to free-content children, so inserting one changes whether a Rectangle receives responsive dimensions. |

Schema sources: `crates/nuxie-runtime/src/mechanical_port/source/generated/component_base.rs:33–35`; `generated/layout_component_base.rs:42–48`; `generated/layout/layout_component_style_base.rs` and `layout_sizing_style_base.rs:64–78`; `generated/shapes/rectangle_base.rs:53–58`; `generated/shapes/parametric_path_base.rs:34–38`; `generated/shapes/paint/{fill,stroke,solid_color,shape_paint}_base.rs`; `generated/foreground_layout_drawable_base.rs:16`; `generated/shapes/clipping_shape_base.rs:33–36`. Layout unit and absolute-position enum values come from `source/layout/layout_style_applier.rs:11–32` and its YGPositionType definition.

## A. Direct responsive background and circular corners

Candidate record graph (IDs illustrative):

```text
0 Artboard (existing root)
1 LayoutComponent409 {5:0, 494:2, 7:240, 8:120, 196:false}
2 LayoutComponentStyle420 {5:1, 607:1, 608:1,
                          639:false, 640:12, 641:20, 642:8, 643:16}
3 Fill20 {5:1, 40:0, 41:true}
4 SolidColor18 {5:3, 37:0xff336699}
```

Import relationships are supported by actual behavior: `shape_paint.rs:93–123` registers a paint with its parent ShapePaintContainer; `solid_color.rs:53–66` initializes the mutator from its parent paint. `layout_component.rs:1441` resolves styleId494 as a LayoutComponentStyle. Style parent links are needed for property-change callbacks (`layout_component_style.rs:34`), even when the owner also resolves styleId.

`layout_component.rs:1510–1555` rebuilds the path using `(0,0,layout.width(),layout.height())`, four style radii and the current world transform. `draw_proxy:1466–1502` draws registered paints; the foreground/end drawable restores saved clipping in `draw:1504–1509`. The path is therefore responsive to a successful ordinary layout resize, not a fixed Rectangle size from the compiler.

For a width-responsive experiment, keep the ordinary parent layout setup already demonstrated by the bytes-only compiler; change the width unit/value using the verified file-version encoding, rather than guessing whether 100% means width7=100 or fractionalWidth706=1. Test 240→390→768→240 without editing any runtime property beyond the ordinary artboard/layout resize entry point. The candidate above uses explicit Point dimensions precisely to avoid claiming percentage encoding before the parent verifies it.

Corner limits are concrete: layout paths have one scalar radius per corner, not independent x/y radii. RTL swaps TL/TR and BL/BR (`layout_component.rs:1520–1538`), whereas CSS physical corner names should not swap; compiler must compensate for direction when emitting these physical CSS corners. `Path::add_rounded_rect` (`shapes/path.rs:298–342`) individually clamps `abs(radius)` to `min(width,height)/2`. That differs from CSS's global overlap scaling for unequal corners. Safe first subset: nonnegative circular radii small enough that neither clamp/overlap rule activates. Arbitrary elliptical corners, percentage corners and oversized unequal corners need another lowering or explicit unsupported diagnostics.

## B. Direct Shape + Rectangle under a layout

```text
LayoutComponent409 L -> LayoutComponentStyle420 S
Shape3 Q {5:L, 13:0, 14:0}
Rectangle7 R {5:Q, 20:100, 21:60, 123:0, 124:0, 164:true, 31:10}
Fill20 F {5:Q, 40:0}
SolidColor18 C {5:F, 37:0xff336699}
```

Actual responsive attachment: `layout_component.rs:1888–1925` propagates content size through suitable non-layout children. `shape.rs:579–624` selects the **first parametric path** and forwards `control_size`; `parametric_path.rs:53–64` writes its width/height. A participating-in-layout Shape takes a separate host-scale path (`shape.rs:615–645`), so do not enable layout participation unintentionally. Nested LayoutComponent or a child with its own layout provider is skipped; a plain Node2 stops propagation.

This has two material consequences:

- A single ordinary Rectangle directly owned by a nonparticipating Shape can receive the layout's content size, preserving its authored circular corner radii while dimensions update. It is not necessarily the layout border box: padding/border affect the content size.
- Two Rectangle paths in one Shape do **not** both receive outer/inner sizes: only the first parametric path is controlled. A naive compound border ring made from two rectangles will resize incorrectly. A plain Node wrapper preserves fixed coordinates but also blocks automatic sizing. A scale-only solution scales border thickness and corner geometry, which is not CSS fixed-pixel border behavior.

`rectangle.rs:47–88` constructs corners from width/height and negative origin offsets. This means using origin0 is important for the proposed top-left layout attachment; origin0.5 requires positioning/compensation.

## C. Border candidates, with exact limitations

### Centered Stroke on live layout path

```text
LayoutComponent409 L (ordinary border/padding layout style)
ForegroundLayoutDrawable513 D {5:L}
Stroke24 B {5:D, 47:b, 50:true}
SolidColor18 BC {5:B, 37:borderARGB}
```

This is a responsive **ordinary outline**, not yet a CSS border. `foreground_layout_drawable.rs:70–108` retrieves the parent's current local/world path and draws paints on it; registration triggers path maintenance even when no background Fill exists. Centered thickness b extends b/2 outside the border-box boundary and consumes only b/2 inside. Merely reserving b with style border504–507 does not change the stroke path or align the stroke inside.

A bounded uniform-border candidate is a dedicated paint-only **absolute LayoutComponent wrapper**, rounded to the outer radius and clip196=true, carrying a Stroke of thickness2b. Clipping its outer half can leave an inside band b wide. Attach a no-clip content sibling outside this decorative wrapper to preserve CSS overflow:visible. Keep actual CSS border fields on the semantic layout owner for content spacing; the decorative wrapper must not add a second border to layout. Its size/position must match the owner border box, not accidentally the owner's content box; determine ordinary absolute inset containing-block semantics with a wire test before assuming all-zero offsets do that.

This is not yet proof: clipping a thick stroke can multiply AA coverage at the outer edge, joins/corners need comparison, and a wrapper's own clip must not affect semantic descendants. Nonuniform side widths and per-side colors cannot be represented by one stroke. Rounded inner corners for unequal borders can be elliptical and cannot be assumed to match the baseline scalar-radius path.

### Filled wrapper / side strips

For opaque rectangular boxes, four absolute ordinary layout strips with Fill+SolidColor can supply fixed-pixel border widths while the long axis responds to layout. Use top/bottom full-width strips and left/right strips excluding those heights (or another **nonoverlapping** partition) to avoid doubled translucent alpha. Ordinary absolute position597=2 plus point offsets516–519 and unit621–624 provide the existing wire mechanism; choosing the exact parent padding/border reference still requires a probe.

An outer border-colored Fill covered by an inner background-colored Fill is only valid when that inner background is opaque and intentionally covers the border paint. It is wrong for a transparent/no-background element, translucent backgrounds, or content behind the hole. Do not advertise it as a general border ring.

For a fixed resolved viewport, compiler-computed compound paths/ordinary ring shapes can represent a transparent hole. Responsive inner/outer geometry needs a demonstrated existing constraint/layout relation; parent scaling or controlling only the first Rectangle is not enough. This investigation did not establish such a relation.

## Draw order and clip ownership

The ordinary drawable graph is built from object records, then artboard layout proxies and foreground drawables are inserted. `artboard.rs:919–935` special-cases ForegroundLayoutDrawable by moving it near its parent; `:955–1025` inserts layout proxies at subtree boundaries. `sort_draw_order` finishes with `first_drawable = last_drawable` (`:1237`), and `draw_drawables:2375–2398` traverses **prev**, not next. Therefore raw record-order “first paints first” is false. Layout's late-inserted proxy paints its background before its children in this reverse traversal; its ordinary endpoint restores clipping after the subtree. Ordinary sibling draw order must be probed with overlapping colors when assigning record indices; do not blindly reuse DOM preorder.

Layout clip196 saves state, clips the layout's world rounded path, then paints; it clips its own background/paint and descendants, not just children. It is a border-box rounded clip, not a selectable CSS padding/content clip origin. Foreground strokes also lie inside that subtree clip when enabled.

ClippingShape42 has different scope: `clipping_shape.rs:232–266` walks its parent subtree to register affected drawables, then walks source92's descendant Shapes to gather paths. `:269–287` requires source92 to resolve to a Node-compatible object. A separate source Shape can be used as a mask for a wrapper subtree; isVisible94 controls source visibility. Verify visibility behavior, ordering and fill-rule before hiding source paints in the wire output. Do not point source92 to an otherwise paint-only layout and expect its generated layout path to become a Shape.

## Minimal bytes-only cases to run next

| Case | Candidate / decisive observation |
|---|---|
| background-point | A above at240×120; confirm Fill/SolidColor parent links, color, origin and corner ordering in ordinary recorded draw path. |
| background-resize | Verified ordinary percent/flex owner width; resize240→390→768→240. Background path width must follow layout, with circular radius12 unchanged. |
| background-rtl | Distinct four physical radii under RTL; confirm compiler swaps emitted style corners to counter runtime logical mirroring. |
| child-sizing-control | B above versus same Shape under Node2; confirm direct child receives content dimensions and Node wrapper preserves authored size. |
| compound-resize-negative | Shape with two Rectangle paths; verify only first is resized. Keep as a rejection/regression control for naive rings. |
| order-three-colors | Layout background, overlapping child, ForegroundLayoutDrawable Fill; inspect recorded paint order and final topmost color. |
| uniform-border-clip | Absolute decorative wrapper Stroke2b with clip196; translucent border and transparent center expose incorrect overlap/coverage immediately. Pair overflow-visible child extending beyond semantic owner. |
| four-side-strips | Square borders with widths4/8/12/16 and alpha0.5; resize and inspect corners for gaps or alpha doubling. |
| clip-shape-scope | Source Shape plus ClippingShape child of a wrapper; outside sibling must remain unclipped; hidden source must not paint. |
| corner-negative-controls | Oversized unequal circular and elliptical/percentage corners. Direct layout encoding must not silently claim CSS equivalence. |

Source-established capabilities are direct layout paint, four scalar layout radii, ordinary clipping, foreground paint, parametric child control-size and ordinary absolute positioning. All proposed border combinations and actual `.riv` import/property-version arrangements remain experiment candidates until the parent binds byte-only evidence against the immutable runtime. No CSS runtime policy, new Renderer method, shader extension or sidecar requirement is needed or permitted for these experiments.
