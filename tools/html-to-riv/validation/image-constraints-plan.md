# Image min/max constraints: immutable-target investigation plan

Status: source-backed historical plan, not blanket qualification. Initially read against compiler commit `f7ea75e9e6`, when public images rejected non-default min/max constraints. Subsequent source-derived point support is documented in `public-image-point-constraints-review.md`; responsive and automatic-minimum families below remain open. No runtime changes were made. The first independent native controls are now captured (see evidence below). The existing percentage-padding row limitation still applies; this plan does not remove it.

## What the source establishes

- `images.rs::plan` replaces both automatic intrinsic axes with asset width/height when alignment does not stretch. It emits no aspect ratio in that path. Merely removing the min/max rejection would independently clamp those two dimensions; it would not implement coupled intrinsic-image constraints.
- One automatic ratio axis sets ordinary `LayoutComponentStyle.aspectRatio` on the image's owner. Both authored definite axes emit no ratio. Both automatic stretched axes replace the cross axis with a responsive 100% dimension before intrinsic main measurement. These are distinct emission paths and require separate evidence.
- Padded images use an outer background/padding owner and a separate content/ratio owner. The latter currently has zero minima and absent maxima. For content-box point dimensions, it retains the exact authored content points. Clamping only the outer owner can consequently leave the inner image at the wrong size.
- `box_sizing.rs::lower_impl` translates point content-box dimensions and bounds by padding. Its image percentage-padding branch explicitly assumes only default image bounds reach it and changes a dimension to Auto without translating its bounds. Admitting bounds before replacing that assumption would be unsound.
- `numeric.rs::child_layout` clamps preferred upper bounds with max then min and subtracts padding. `Bounds::image` subsequently overwrites the ratio axis using the other axis, without retaining image-specific min/max arbitration. A correct min/max feature needs a sizing plan and matching bound propagation, not only wire fields. Keep `Bounds::image_outer` after inner image validation for content-derived outer sums.
- Native `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:182` and `:548` apply `maybe_apply_aspect_ratio` independently to min-size and max-size vectors, as well as preferred sizes. `geometry.rs:591` fills a missing dimension from the other dimension; it leaves vectors with both dimensions present unchanged. The native operation alone does not encode all CSS constraints on which transferred bounds may affect an authored definite dimension.
- The same native flex implementation treats `max <= min` as definite at `flexbox.rs:204`; its automatic-minimum measurement path is at `:818`. These are source facts, not image browser-equivalence results.
- Generated `layout_sizing_style_base.rs` defaults omitted min/max units to Undefined (0). `layout_style_applier.rs:717` maps Undefined and Auto to Taffy Auto. `layout_participant.rs:274` overrides omitted minima to explicit point zero for Fill axes. Therefore ordinary missing min fields are not universally equivalent to explicit zero min fields. Preserve a control for both. Existing public reset semantics and historical non-image tests do not prove their interchangeability for ratio owners.
- Native image fit is downstream of the owner's sizing. A plausible rectangle can still conceal the wrong content rectangle through clipping or object-fit. Capture both outer and actual Image bounds, plus a following tail sibling.

All runtime paths above are under `crates/nuxie-runtime/src/mechanical_port/source/` unless the vendored Taffy path is given. Do not modify those files.

## Browser semantics to discriminate

For intrinsic replaced elements with both dimensions automatic, constraints can change the paired dimension while preserving ratio, until conflicting bounds require distortion. Normalize a maximum below its minimum in favor of the minimum. Use the CSS2 replaced-element table as an independent conceptual oracle, but verify the module's flex-item contexts in pinned Chrome rather than applying that table blindly to every flex case. [CSS2 minimum and maximum widths](https://www.w3.org/TR/CSS2/visudet.html#min-max-widths)

Aspect-ratio bound transfer must respect authored definite sizes in its destination axis. A transferred minimum is limited by a destination preferred/maximum size; transferred maximums respect destination preferred/minimum sizes. This is a useful discriminator against native independent vector completion. [CSS Sizing 4, transferred min/max constraints](https://www.w3.org/TR/css-sizing-4/#aspect-ratio-size-transfers)

Explicit automatic flex minima are a separate case: replaced and non-replaced items use different combinations of content and transferred suggestions. Stretch is constrained by cross-axis min/max; flex base sizing and final min/max clamping are separate stages. Test in the actual reset and with explicit `min-width:auto`/`min-height:auto`, including a crowded parent. [Flexbox automatic minimum](https://www.w3.org/TR/css-flexbox-1/#min-size-auto), [Flexbox cross-axis alignment](https://www.w3.org/TR/css-flexbox-1/#align-items-property), [Flex layout algorithm](https://www.w3.org/TR/css-flexbox-1/#layout-algorithm)

## Ordinary wire fields

All min/max fields belong to existing LayoutComponentStyle via its LayoutSizingStyle base; no schema additions are needed. Values are floats; unit properties are unsigned integers. Units: 1 point, 2 percent, 3 automatic, 0 undefined. An absent maximum is the current compiler representation of `none`.

| CSS bound | Value key | Units key |
| --- | ---: | ---: |
| min-width | 502 | 627 |
| min-height | 503 | 628 |
| max-width | 500 | 629 |
| max-height | 501 | 630 |

`aspectRatio` is key 524; its value is width/height. Keys verified in generated `layout_sizing_style_base.rs:65` and `layout_component_style_base.rs:227`. Use schema-resolved names in public implementation; raw keys are only for private ordinary-file experiments.

## Candidate compositions, in diagnostic order

1. **Direct independent clamps for two definite dimensions, no padding.** Keep the existing no-aspect owner and ordinary bound fields. Because neither dimension is ratio-derived, constraints should clamp independently; verify points and responsive percentages, both box-sizing declarations, zero dimensions, min > max, and object-fit. Hypothesis pending native/Chrome evidence.
2. **Source-derived intrinsic constraint solution for both auto/start with constant bounds.** Resolve the replaced intrinsic constraint branches from encoded asset dimensions and authored computed point constraints, then emit ordinary point dimensions without a ratio. This is source evaluation, not a browser-baked layout: the inputs contain no viewport dimensions. Do not use it for stretch, percentages, indefinite parent measurements, flexible main sizing or any branch depending on containing size. Preserve decimal provenance and ratio arithmetic error; use numeric bounds and actual native paint comparisons. Conflicting two-axis constraints must be allowed to break the ratio when CSS does.
3. **Separate an authored definite axis from the ratio-derived axis.** An outer sizing owner fixes/clamps the authored axis without carrying aspect ratio. A nested ordinary ratio owner receives that resolved axis at 100% and derives the other axis. Put automatic-axis constraints on the owner that determines the final automatic extent. Test both possible packing directions. This aims to prevent transferred automatic-axis maxima from shrinking an authored fixed axis. It is not proven: parent intrinsic measurement can freeze a size before inner clamping, and the outer must contribute the final extent to siblings. Use nested read-only measurements to locate the first divergence.
4. **Inner content clamps under the established padding owner.** For constant content-box dimensions and point bounds, put authored content clamps on the unpadded owner and let the outer Hug plus padding produce the border box. For border-box bounds, the constraint belongs to the outer border box; the content must see its clamped residual, with the padding floor, without accidentally retaining the old authored inner points. Do not apply the same constraint to both levels without demonstrating it is mathematically identical.
5. **Responsive outer constraint then inner ratio.** Keep original-parent percentage bounds on an outer owner. Use its clamped dimension as the inner owner input; keep ratio and automatic-axis final clamps separated as needed. A percentage bound moved blindly to the inner owner acquires the wrong containing block. Percentage height constraints must retain parent height while vertical percentage padding uses parent width. Test non-square, padded parents and reverse resize order.
6. **Source-normalized transferred bounds.** Where all involved constraints are points and provenance proves the relevant inequalities, explicitly synthesize the appropriate opposite-axis bounds so native vector completion cannot invent the wrong transfer. If both axes must remain responsive with different bases, simple coefficient multiplication is insufficient. This alternative can supplement (2)/(3); it is not a license to bake viewport-specific branches.
7. **Original-containing-size carrier for unresolved responsive bounds.** Explore ordinary nested owners that retain the original width/height basis while exposing only the image's final extent to parent flow. A full-size wrapper that shifts the tail by the parent width is not a successful composition. This remains a research candidate for mixed bases and row intrinsic measurement; a failed direct clamp does not prove all file compositions impossible.

Do not promote a candidate based on geometry alone. Keep public diagnostics until its accepted context is explicit and its public API, bytes, lifecycle and visual evidence exist. Keep unsupported contexts in the corpus and preserve their precise rejection reason.

## Finite acceptance matrix

Use the same authored opaque 96×64 image (ratio 1.5), both row and column parents, `align-self:flex-start` unless stated, and the module reset. Use original/clone resize sequence 240×240 → 390×320 → 768×560 → 240×240; no recompilation. Each row below specifies a finite family, not a broad claim. Run all listed variants rather than considering one orientation representative.

| ID | Authored setup and variants | Discriminator / expected invariant |
| --- | --- | --- |
| I1 | auto/auto; max-width 48; max-height 32 | Single-axis intrinsic reduction transfers to paired axis. |
| I2 | auto/auto; min-width 192; min-height 128 | Single-axis intrinsic enlargement transfers to paired axis. |
| I3 | auto/auto; max-width 72 + max-height 40; maxima 60 + 48 | Both reductions, each axis separately decisive. |
| I4 | auto/auto; min-width 120 + min-height 100; minima 180 + 80 | Both enlargements, each axis separately decisive. |
| I5 | auto/auto; min-width 192 + max-height 32; max-width 48 + min-height 128 | Opposing constraints require ratio break, not clipping. |
| I6 | auto/auto; min-width 120 + max-width 48; min-height 96 + max-height 32 | Minimum wins contradictory same-axis bounds. |
| I7 | auto/auto; exact intrinsic bound 96/64; bound 0; fractional 47.99/31.99 | Equality, zero and branch-adjacent rounding. |
| D1 | width120/height80; each of four bounds alone crossing preferred value | Authored definite opposite dimension stays unchanged. |
| D2 | width120/height80; both mins, both maxes, min > max on each axis | Independent clamps, no ratio preservation requirement. |
| A1 | width120/heightauto; max-width60; min-width180 | Clamped authored width feeds automatic height. |
| A2 | width120/heightauto; max-height40; min-height120 | Automatic-axis clamp must not change authored width. |
| A3 | widthauto/height80; max-height40; min-height120 | Mirror of A1, verify row and column separately. |
| A4 | widthauto/height80; max-width60; min-width180 | Mirror of A2; detect unwanted transferred constraint. |
| A5 | A1–A4 with a same-axis min > max and an opposing-axis bound | Arbitration order, min wins, no accidental double clamp. |
| S1 | auto/auto + stretch; cross max120 then cross min300 | Clamped cross stretch feeds main measurement at each resize. |
| S2 | fixed main80/auto cross + stretch; cross max120/min300 | Genuine auto cross stretches; fixed main is retained. |
| S3 | auto main/fixed cross80 + default auto alignment; cross bounds40/120 | A fixed cross dimension must not accidentally become Fill. |
| S4 | auto/auto + stretch; main max100/min200, then both-axis conflict | Main clamps and cross stretch must follow browser's phase order. |
| P1 | D1, A1–A4, I1–I2 with padding 8px 10px | Both box-sizing values; compare border and content separately. |
| P2 | definite160×120; padding80px; max-width/max-height20 | Nonnegative content floor; outer padding may exceed maximum. |
| P3 | content-box width .03125; padding1000000px on one side; point min/max .03125/.0625 | Preserve exact inner points; detect cancellation; numeric-only if rendering huge is unsafe. |
| P4 | D1, A1–A4, I1 with padding5%; then 5% 10px | Both box-sizing values. Preserve existing row diagnostics where applicable; no automatic admission. |
| R1 | definite width50%/height40%; each max25%/min75% | Percentage bounds resolve at original parent axis and cross thresholds. |
| R2 | intrinsic auto/auto; max-width25%; max-height25%; paired max25% | Responsive ratio solution; parent aspect changes across resize. |
| R3 | width120/auto height; min-height25%/max-height25%; mirror | Fixed axis remains fixed; percentage auto-axis clamp original parent basis. |
| R4 | padding8px then5%; percentage bounds on both box-sizing values | Percentage-plus-points and mixed width/height bases; preserve unresolved cases. |
| R5 | R1–R4 in parent width100%, height240, padding20px | Distinguish parent's content basis from its border and viewport. |
| R6 | auto-height parent with percentage min/max height | Explicitly exercise indefinite basis; record Chrome and candidate diagnostics. |
| M1 | min-width:auto, min-height:auto, both auto; nonshrinking and crowded flex parent | Replaced automatic-minimum semantics distinct from reset min0. |
| M2 | repeat I1/A2/S1 with omitted ordinary minima versus explicit point0 in private file | Diagnose native transfer/default effects, never compare changed CSS oracle. |
| O1 | nested large percentage chain capped by finite max; larger min than max | Max may cap a preferred overflow, but larger min can still overflow. |
| O2 | narrow/tall and wide/short asset metadata, ratio-derived axis with finite bound | Test ratio and object-fit intermediate overflow, not only final rectangle. |
| O3 | individually finite content and percent padding, overflowing automatic outer sum | Preserve new outer-sum guard after constraint planning; no native render of huge geometry. |

For I/D/A/S rows use both box-sizing values even without padding to establish equivalence. P/R explicitly vary padding. A compact first wave is I1/I2/I5/D1/A1–A4/S1/M2, then enlarge by the matrix discriminators; do not launch a large unchanged native campaign repeatedly.

For expected numbers, derive source-only point arithmetic independently and compare to Chrome before using those numbers in assertions. Examples such as intrinsic 96×64 with max-width48 conceptually suggest 48×32, but flex-item layout and a candidate's ordinary-object measurement must still be observed. The entire table remains unqualified until evidence exists.

## Public implementation and acceptance gates

1. Preserve the authored style for inheritance, selectors and numeric provenance. Add an image-specific constraint plan shared by emission and `numeric.rs`. Avoid mutating a receiving child style into its parent's lowered border-box values.
2. Keep preferred-size generation, ratio transfer, constraint arbitration, padding conversion and outer stretch as explicit steps. Each needs a source-only regression that fails for a plausible wrong implementation (independent intrinsic clamps; forced ratio on two fixed axes; wrong percentage base; inner fixed point ignoring outer clamp).
3. Bounds must enclose native arithmetic over the full documented viewport domain, including intermediate ratio operations and object-fit scaling. A final max is not proof an earlier overflow is safe unless the immutable native evaluation order is evidenced. Preserve lower/witness evidence through min/max conflicts and do not turn unknown intrinsic measurements into finite claims solely because a maximum exists.
4. Public Rust/CLI/WASM/JS must agree on success and diagnostic cases. Check deterministic repeated files, discard maps/diagnostics before import, preserve asset bytes, and exact-byte regression for existing no-constraint images. New constraints should not reorder unchanged records unnecessarily.
5. Native geometry must include outer/content/image rectangles and tail placement; pixels include background, transparent image, crop and patterned image controls. Original and clone must reuse one file at all sizes. Retain lifecycle/resource cleanup controls and baseline runtime hashes.
6. Report geometry and pixel qualifications separately. Keep failures at original tolerance, visually inspect every distinct pair or verified identical transfer, and document a failed composition without calling the semantic feature impossible.
7. Update SUPPORT/BACKLOG/VALIDATION/progress only for the evidenced contexts. Record possible runtime changes in the separate enhancement discussion; do not implement them.

## First direct-control evidence (captured during this investigation)

Inspected `output/image-constraints-candidate-r1/combined-receipt.json`: 13 cases, 104 freshly rendered original/clone frames; 64 geometry passes, 64 pixel passes, 88 image-presence passes. These are private file mutations, not a new public API acceptance result. This plan's author inspected the machine receipt, not the visual sheets; root owns visual review.

Eight controls pass all eight geometry/pixel frames each: fixed-width/max-width ratio; fixed-height/max-height ratio; both-fixed max-width; both-fixed min-height; inconsistent width bounds; stretch max-width; both-fixed percentage max-width; fixed-width/auto-height min-height.

Five controls fail every geometry/pixel frame: intrinsic min-width, intrinsic max-width, padded border-box max-width, padded content-box max-width, fixed-width/auto-height max-height. Presence also fails for intrinsic min-width and fixed-width/auto-height max-height. In particular, the passing automatic-height **minimum** does not justify the failing automatic-height **maximum**: missing-native-bound completion differs between these paths. Keep both discriminators.

Root's next mutations test intrinsic width plus automatic-height ratio ownership and mirrored inner max-width for the two padded cases. These are narrower composition hypotheses than blanket min/max admission; remaining matrix rows still need investigation.
