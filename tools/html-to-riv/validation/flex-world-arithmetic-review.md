# Direct flex local/world arithmetic

Read-only audit against immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`. All five source files in the receipt are byte-identical. Scope: ordinary direct LayoutComponent children, no effective insets/margins/gaps/padding, no origins, constraints, own transforms or animation. These exclusions must be proved from final records and ancestors; this document does not assert them for every compiled scene.

## Solved local locations

Pinned Taffy `src/compute/flexbox.rs:1678` sums outer target sizes in file order as f32 and subtracts that sum from the inner container size for free space. At `:1722`, alignment selects each child's initial offset. `src/compute/common/alignment.rs:59` distinguishes physical Start/End from FlexStart/FlexEnd under reversed flow; some combinations return exact zero while others use computed free space. Do not simplify all four compiler directions to one identical offset calculation.

Compiler ordinary row/column reverse file emission, native flow remains reversed, and end alignment compensates; reverse CSS directions retain logical file order and use native start alignment. This is compiler behavior, not a native default. The checker must compare actual flow/alignment/order to the expected mapping before using a simplified formula.

Taffy `flexbox.rs:2050` initializes main cursor from padding/border start (zero in scope); `:2062` traverses reversed native flow in reverse item order. At `:1967` for LTR, location main is the left-associated f32 sum:

`cursor + item.offset_main + margin_start + relative_inset`.

At `:1973`, cross location adds cross cursor, child offset, line offset, margin and relative inset. Zero operands may be eliminated only when actually proved zero. At `:2018`, the cursor advances by the nested sum `offset_main + margin_axis_sum + returned_size_main`; both the inner sum and cursor addition can round. The returned size matters: `:1934` calls child layout with the target dimensions, and `:1947` takes its returned size. Therefore flex target preservation is a separate premise, not established merely by passing a target to recursive layout.

The completed location/size is stored by `set_unrounded_layout` (`:2000`). Runtime `layout_component.rs:2073` disables Taffy rounding, then `:2292` copies location and dimensions into `Layout::new`. `Layout::new` (`:49`) and accessors (`:67`) copy f32 values without add/subtract. There is no pixel rounding correction at that handoff. Animation paths must be excluded: `Layout::lerp` at `:58` performs additional products/additions, and `apply_interpolation` starts at `:2793`.

## Layout translation and own transform

Runtime paths below are in `crates/nuxie-runtime/src/mechanical_port/source/`.

`layout_component.rs:901` begins with the copied `(left,top)`. For an immediate Artboard parent it additionally computes `(artboard_width * origin_x, artboard_height * origin_y)` and subtracts it (`:908-919`). With finite dimensions and proved zero artboard origin, these products/subtractions preserve numeric values; without that premise they need bounds. An ordinary LayoutComponent parent adds no such origin subtraction.

`build_own_transform_with` (`:944`) produces an identity or translation by stored Node x/y. Rotation/scale/pivot operations are skipped only when their defaults and flags establish the branch conditions. With x/y zero and no rotation/scale override, own matrix is identity. Imported file version changes ComposeTransform behavior (`:294`), so bind this conclusion to the actual imported file version. ComponentOrigin absence establishes local anchor zero (`:893`), but must be checked after any later baseline helper insertion.

## World composition: the extra rounding stage

`compose_world_transform` (`layout_component.rs:989`) reads the actual parent world matrix, constructs translation from layout location, then evaluates **`parent_world * base * own`**, left-associated (`:1013`). It does not directly copy parent translation into local geometry.

`math/mat2d.rs:255` implements matrix multiplication with explicit f32 `mul_add` for each product sum, then a separate translation addition:

`tx = a.xx.mul_add(b.tx, a.yx * b.ty) + a.tx`

`ty = a.xy.mul_add(b.tx, a.yy * b.ty) + a.ty`.

For finite pure translations with exact identity linear coefficients, the products/FMA reduce exactly to the child's local coordinate. The **final addition to parent translation rounds once per axis**. The second multiply by own identity preserves the composed translation (zero terms/addition), under the same finite/default premises. Thus one cannot set child world error equal to local error alone; propagate parent error plus local error plus this addition's outward rounding allowance. Across ancestors this repeats. Large parent offsets with tiny local offsets are concrete loss-of-significance controls.

If own translation is nonzero, the second multiply adds another rounded translation; if any linear coefficient is not exact identity, use the actual fused product-sum arithmetic rather than a generic one-add model. Nonfinite values invalidate the simplification (`0 * infinity` is not safely zero).

Point transformation (`math/mat2d.rs:331`) likewise uses FMA plus translation. A zero anchor returns the existing world translation exactly for finite pure translations. A rectangle corner at `(width,height)` adds size to translation and can introduce further rounding, so a geometry budget measuring edges or painted bounds must account for this beyond the origin-only world envelope. `shape_world_transform` returns the existing matrix (`layout_component.rs:1019`); this does not prove rasterization equivalence.

## Facts required before using the restricted formula

- Final native flow/alignment/file order and LTR context match the direction mapping; no helper participant changes sums or traversal.
- Main and cross target sizes survive recursive child layout. Start with leaf layout nodes, point cross dimensions, no automatic minimum ambiguity, aspect ratio, intrinsic measurement, padding/border floor, or unresolved min/max freeze. Fixed automatic minimum needs its own proof; see the defaults audit.
- No effective relative insets, margins, padding, borders or gaps, and no cross alignment/line offset requiring an unmodeled arithmetic term.
- Actual artboard origin and ancestor local transforms are known; every matrix on the chain is pure translation with exact identity linear coefficients. This is an inductive final-record/runtime-default proof, not the presence of a CSS width.
- No layout interpolation, animation, constraints, ComponentOrigin helpers or parent mutation modifies layout/transform after capture. Finalize after complete emission.
- Resolved values and intermediate sums are finite. Track both axes and include host viewport conversion, parent size error, layout arithmetic, parent world addition, and corner/descendant geometry as appropriate.

A first parent-first checker should store typed size and world envelopes per actual node, pass parent envelopes into the flex analyzer, and separately bind the analyzer's local/native-layout formula to this world addition. Avoid double-counting a parent-world addition already performed inside `flex_numeric::analyze`; verify its positions represent the same stage before adding an extra allowance. Missing target/cross/world facts stay unresolved. No source finding here establishes a zero-error whole scene.

## Controls

Four directions with actual native permutations; a single top-level parent and then a preceding fixed sibling; nested translations at large/small magnitudes; origin0 versus mutated origin; identity versus nonzero Node x/y; omitted versus explicit auto minima; a layout descendant that changes returned target size; fractional corner widths; clone/resize and no-animation checks. Preserve native/browser failures independently of any arithmetic certificate.

## Source receipt

| Source | SHA-256 | Equality to baseline |
| --- | --- | --- |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs` | `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs` | `6f218337cbf8daa1567f20244cadd9f0676cb1e6546ae0bc23751d02b404da01` | identical |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` | identical |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/common/alignment.rs` | `f082c4a5b0607c0083c5b774c66b4006bd7324dc80159674df1141c7a20f0547` | identical |
