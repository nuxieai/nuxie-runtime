# L12: content-box sizing on the immutable runtime

The baseline has **no ordinary `boxSizing` property**. It can nevertheless express
useful content-box cases by translating authored sizes into its existing
border-box dimensions. The point-padding candidate below works independently of
nonlegacy flex factors and border painting. This is a finite experiment, not
public admission or completion of L12.

Evidence: [candidate receipt](content-box-candidate-receipt.json),
[authored cases](content-box-candidate-cases.json), and
[Chrome/native gallery](../output/content-box-candidate-r2/render/gallery.html).

## Baseline source findings

The source audit compares ten complete runtime/schema/Taffy files byte for byte
with `6c7ac16617835b5f581784ff08a9e779bb52faf3`. It enumerates every property in
the `LayoutComponent` and `LayoutComponentStyle` inheritance chains. There is no
`boxSizing`, `box_sizing`, or content-box equivalent in the schema.

* `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs:393`
  initializes `YGStyle.taffy` from Taffy's default, changing only flex direction.
  Its `taffy_style` conversion at line601 clones that style and supplies dimensions,
  min/max, margins, insets, padding, border and gaps. It never changes box sizing.
* `vendor/taffy-0.12.1-rive-yoga-order/src/style/mod.rs:314` defines both internal
  box-sizing modes. `Style::DEFAULT` at line594 selects `BorderBox`. An internal
  engine capability does not establish an ordinary file property that selects it.
* `layout/layout_component_style.rs:236–262` applies ordinary border and padding
  edges; `layout/layout_sizing_style.rs:95–103` applies ordinary min/max dimensions.
  `layout_style_applier.rs:718–731` converts dimension percentages using the Yoga
  tagged path, while padding uses percent divided by100. These are different
  native floating-point operation orders.
* Taffy's `compute/flexbox.rs:178–197` adds padding/border to preferred and bounded
  sizes only when the internal box-sizing mode is ContentBox. The immutable file
  path leaves that branch inactive. `layout_component.rs:2174–2188` also normalizes
  automatic flex-child minima to zero; it does not activate CSS content-box sizing.

The reverted `set_css_content_box_occurrence` policy supplied the missing mode.
The historical L12 qualification cannot transfer to this target.

For the CSS contract, `box-sizing` initially computes to `content-box`, does not
inherit implicitly, and changes the interpretation of quantitative width,
height, min/max and flex-basis values. It does not change `auto` sizing. Padding
and border lie outside a content-box size. The module's explicit reset currently
chooses border-box, so authored `initial` and `unset` must still compute to
content-box rather than copying the reset. See the primary
[CSS Sizing specification](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/#box-sizing).

## Ordinary-file candidate and observed results

For fixed padding, translate a fixed content width by adding left+right padding,
and a fixed content height by adding top+bottom padding. Translate fixed min/max
the same way. Preserve auto preferred sizes and absent maxima. Padding remains
on the original semantic LayoutComponent; no wrappers or extra records are
required. All descendant percentages continue resolving inside its content box.

The experimental adapter selects a finite fixture by exact HTML/CSS identity,
then passes independently written equivalent CSS to the frozen public compiler.
It never reads Chrome measurements, source maps or viewport results to construct
the output. Point values are derived only from the authored dimensions/padding.
Every file is compiled once at390×160, then the same original and clone are
resized through240×160,390×200,768×120,240×160.

| Cases | Geometry | Pixels | Finding |
|---|---:|---:|---|
| Eight point-padding lowerings |64/64|64/64|Fixed asymmetric sizes, min/max, minimum winning over maximum, intrinsic auto height, responsive stretched auto width, reversed ordered siblings, font-relative arithmetic, and auto height with a fixed maximum work in this corpus.|
| Same-axis percentage addition |8/8|8/8|`width:50%; padding:0 5%` becomes ordinary `width:60%` with unchanged padding. A viable separate algebraic candidate, without whole-domain numeric qualification.|
| Two deliberately unlowered controls |0/16|0/16|Removing only box-sizing loses the extra outer extent and gives descendants the wrong percentage bases. This includes mixed percentage width and point padding.|
| Fractional outer-edge control |8/8|2/8|15.5px content height plus10px padding gives correct25.5px geometry, but retains the known baseline paint-coverage mismatch.|

Total:80/96 geometry,74/96 pixels,192/192 alternate-clear controls. The driver
exits1 and preserves all failures. Twelve repeated compilations reproduce exact
RIV bytes and maps. No runtime, renderer, schema, dependencies or root build
configuration changed. Public `box-sizing` declarations remain rejected.

The unlowered asymmetric control measures100×60 natively versus118×70 in Chrome;
its100% child becomes82px rather than100px wide. The mixed-unit control measures
120px rather than140px wide at240px viewport width. These demonstrate that merely
dropping the declaration is wrong; they do not rule out other file compositions.

## Visual inspection

All36 first-instance distinct viewport pairs were inspected directly in six
unscaled contact sheets. The placement script verifies all72 source image
placements by complete RGBA equality. Every later repeat/clone image is checked
against the matching inspected viewport image:120 image transfers cover60 pairs.

The point cases match the intended padding bands, inner percentage extents,
minimum/maximum clamps, sibling order and visible overflow. The automatic-width
case expands with the viewport; its fixed padding stays unchanged. The auto-height
maximum case deliberately overflows its parent and preserves the overflow in both
renderers. The unlowered controls visibly reduce the parent and child extents.
The fractional control has a thin native coverage boundary rather than Chrome's
opaque terminal row; full-image thresholds pass at390×200 only because the
fractional row occupies a smaller part of that image. No visual equivalence claim
is inferred from those two passing metric frames.

The initial r1 setup stopped before rendering because two fixtures shared the
same input identity. The r2 inputs add distinct inert CSS comments for finite
candidate selection. The initial compile error and all r2 failures remain in the
output directory; no tolerance was changed.

## Compiler integration plan

1. Add a computed box-sizing enum with reset BorderBox; parse content-box,
   border-box, inherit, initial and unset through the existing strict declaration,
   variables and cascade paths. Initial/unset are ContentBox. Keep non-inheriting
   defaults separate from explicit inheritance. Invalid and unsupported computed
   contexts must diagnose before bytes are returned, including overridden and
   unmatched declarations under the existing strict grammar.
2. Produce a separate lowered sizing view after computing authored values. For
   each axis, add its known point padding sum to fixed width/height and fixed
   min/max values. Auto dimensions/absent maxima stay unchanged. A zero minimum
   may retain its existing zero encoding because the native padding floor already
   supplies the same outer floor; verify either canonical choice explicitly.
   With a zero inset in an axis, its percentages need no adjustment.
3. **Do not mutate the authored computed Style used by descendants.** A parent
   with width100px and padding10px still passes a computed100px width to a child
   using `width:inherit`; only the parent's native outer width is120px. Retaining
   lowered120px as the inherited value would corrupt both mixed box-sizing trees
   and repeated content-box inheritance.
4. Use lowered sizing when checking `numeric::Bounds::child`; that code currently
   assumes all authored sizes denote border boxes, then subtracts padding. Feeding
   it unadjusted content sizes would understate descendant percentage bounds.
   Preserve the current bounded containing-width requirement for percentage
   padding and unknown intrinsic cases. Record outward-rounded addition and
   native scalar conversion error, rather than assuming finite authored scalars
   prove aggregate accuracy. Existing source-value limits must not silently become
   a new reduced language scope for derived outer dimensions.
5. Keep descriptor/numeric provenance tied to the actual lowered record values.
   The current flex size certificate assumes authored scalars match native fields;
   a content-to-border adjustment must be represented explicitly or remain
   unresolved in that private analyzer. Do not certify100px authored content as
   a120px native scalar with the old error bound.
6. Retain current padding restrictions for direct alignment wrappers, baseline
   groups and gaps. Baseline metric functions reject padding and read authored
   dimensions; they cannot be reused as content-box metrics without adjustment.
   Nonlegacy flex factors/basis and borders remain separately unqualified. Plain
   fixed-padding lowering does not require those features to be admitted first.
7. Add Rust/CLI/WASM/JS tests for exact output equivalence, deterministic output,
   diagnostics and no-output behavior. Cover mixed inherited border/content boxes,
   CSS-wide values, variables, point and zero-percent insets, nested automatic
   dimensions, explicit minima/maxima, crossing min/max, one-axis-only padding,
   declared bounds, near-limit addition and recursive percentage descendants.
   Expand the native/Chrome matrix to all four directions, known automatic-margin
   compositions, and original/clone resizing before public qualification. Preserve
   the fractional failure and the unlowered controls separately from positive gates.

## Remaining composition questions

General mixed `percentage + points` cannot fit a single existing dimension scalar
and unit field. For example,50% content width plus20px padding requires
`0.5*containingWidth +20`. Replacing it with a viewport-specific percentage or
pixel value would break same-file resizing and is outside the target contract.
That is a limitation of a single-field lowering, not proof of impossibility.

Same-axis percentage addition is promising, but preferred-size and padding
percentages use different binary32 operation orders; algebraic equivalence alone
does not bound their error over nested percentage chains. Vertical percentage
padding introduces containing **width** into a height expression and cannot be
folded into an ordinary percent-height coefficient. Point widths with percentage
padding likewise need a mixed expression.

Possible future wrapper/measurement compositions must reproduce both the
semantic owner's content box and its outer contribution to surrounding layout.
A paint-only expansion leaves sibling positions wrong. An intrinsic wrapper
whose percentage child resolves against that wrapper can introduce a cycle or
change the percentage containing block. Investigate those mechanisms with minimal
same-file resize cases before admitting them or declaring them impossible.
