# Image padding: ordinary-file layout plan

The useful composition is a padded authored owner containing an **unpadded content owner**, with the image ratio and content clip on the inner owner. The current R2 repair must keep **point sizes** on that inner owner but use **100%** for an authored percentage already resolved by the outer owner. This is a source-backed candidate for both box-sizing modes, intrinsic dimensions, fixed dimensions and responsive stretch. It is not native qualification. The implementation can remain inside the compiler; no runtime field or host sizing callback is needed.

This review used the pre-padding compiler frozen under [public-jpeg-build-r2](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/source-bindings.json), at repository checkpoint `244733b9dc0bdeebe48b6a3856a5ea1d7b5f7cb0`. That source inspection preceded the R1 padding implementation. Its missing percentage-axis case is corrected below; the historical source identities remain reproducible. The seven runtime/vendor files identified below were independently compared byte-for-byte with immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`; no runtime source was edited. No compiler or native render was run for this plan.

## R2 correction: the percentage applies to its original containing axis once

The original plan and finite R1 numeric review missed an admitted case: content-box percentage sizing with nonzero padding **only on the opposite axis**. Same-axis-zero padding leaves that percentage representable in the ordinary outer dimension. R1 incorrectly copied every non-auto content-box size, including a percentage, onto the inner owner and resolved the percentage a second time.

For `width:50%; padding:8px 0`, the outer width is 50% of its original containing width; the inner must fill that resolved width. For `height:50%; padding:0 10px`, the same rule applies vertically, subject to the existing definite-height admission guard. The repaired `images::padded_plan` copies only `Size::Pixels(_)` for content-box dimensions; percentages fall through to inner 100%. Exact point preservation, auto/ratio axis selection, and both-auto stretch remain unchanged.

This is a compiler bug and repair, not an immutable-runtime limitation. The [two retained public controls](public-image-padding-percent-cases.json) fail all 16 old R1 geometry/pixel/presence frames. Their old receipt is [preserved](../output/public-image-padding-percent-before-r1/receipt.json); current source reasoning and the earlier missed audit are recorded in [image-padding-numeric-review.md](image-padding-numeric-review.md). R2 native/Chrome, parity, regression and visual qualification belong to the active parent checkpoint and must not be inferred from this plan.

## The decisive distinction

CSS images use their natural ratio on **content dimensions**, regardless of `box-sizing`. Quantitative border-box sizes include padding and floor content at zero. Auto intrinsic sizing does not reinterpret the natural image dimensions as a specified border-box size. These are separate rules: [CSS Sizing 4, automatic aspect ratio](https://www.w3.org/TR/css-sizing-4/#aspect-ratio), [CSS Sizing 3, box-sizing](https://www.w3.org/TR/css-sizing-3/#box-sizing).

The immutable native owner does something different when it has padding and `aspectRatio`:

- [`YGStyle::default`](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs:393) uses Taffy's default; [`Style::DEFAULT`](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/style/mod.rs:594) selects `BorderBox`. The runtime adapter does not change box sizing.
- [`LayoutComponentStyle::apply_item_style`](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs:182) passes the ordinary ratio directly to [`set_aspect_ratio`](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs:507).
- [`generate_anonymous_flex_items`](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:530) applies the ratio to resolved dimensions; the padding adjustment is zero in BorderBox mode. [`maybe_apply_aspect_ratio`](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/geometry.rs:591) fills the missing dimension by dividing or multiplying the known dimension. The container path uses the same box-sizing distinction at flexbox.rs:175–197.

Thus placing the natural ratio on the padded outer owner is incorrect. A 96×64 image with border-box `width:120px;height:auto;padding:6px 10px` needs content width100 and ratio-derived content height66⅔, then vertical padding12. It does not need outer height80. The independently authored Chrome corpus already measures outer120×78.65625 and ResizeObserver content100×66.65625; the corresponding content-box case measures outer140×92 and content120×80. [Chrome characterization receipt](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/image-padding-chrome-r1/receipt.json), SHA-256 `463e1d0311bda9b9a4046dc74356b245cd8237d043fa6552cbbd01591f132851`.

`Image::measure_layout` does not repair the distinction: it returns each natural axis independently unless that axis is measured Exactly. Its later [`update_image_scale`](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/shapes/image.rs:457) fits decoded pixels into the assigned layout dimensions. `object-fit` and `object-position` operate within the CSS content box, whose clip must exclude padding. [CSS Images 3](https://www.w3.org/TR/css-images-3/#the-object-fit).

## Proposed internal interface and emitted structure

Keep the computed `Style` intact. Have `images::plan` return an image layout plan containing the ordinary outer sizing view, content-owner sizing, the optional ratio-derived axis, and paint metadata. These are emission facts, not inherited CSS values. `prepare_child` then emits:

```text
Authored LayoutComponent + LayoutComponentStyle
  source-map identity; flex participation; CSS outer dimensions
  physical padding; background; no image aspectRatio
  Unnamed LayoutComponent + LayoutComponentStyle
    zero padding; content dimensions; optional natural aspectRatio; clip=true
    Image + LayoutParticipant
      original encoded asset; fit; position; sampler; fills inner content
```

Use normal row/column packing matching the real parent axis for the single-child outer owner. This retains automatic cross stretch, as the existing content-owner fix established. Keep authored order and flex participation on the outer object. The image's own `flex-direction` must not accidentally select the ratio-derived axis. Preserve source-map identity on the outer object; identify the inner object only through read-only structural observation.

The existing seams are [images.rs plan/emit](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/images.rs), [compiler.rs prepare_child](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/compiler.rs:678), and [box_sizing.rs ContentOwner/lower](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/box_sizing.rs:21). The old image plan's intrinsic/100% mutations were qualified with zero own padding. Removing its padding rejection alone would confuse natural content dimensions with border-box sizes and make content-box stretch pass through an incorrect padding addition.

### Axis selection

Let `Pₓ=left+right`, `Pᵧ=top+bottom`, and natural ratio `r=naturalWidth/naturalHeight`. The formulas below describe CSS intent; they are not instructions to bake viewport-dependent values into the file.

| Authored situation | Outer sizing | Inner sizing and ratio |
| --- | --- | --- |
| Both axes auto, start alignment | Auto size includes the inner natural image plus padding | Explicit natural content width/height from validated asset metadata; no ratio dependency |
| Border-box definite axis, other auto and not stretched | Keep authored outer definite size; other outer axis auto | Controlled inner axis fills available outer content, floored at zero; derive the other inner axis using `r`; padding adds outside it |
| Content-box point axis, other auto and not stretched | Existing point translation can retain the explicit outer axis; other axis auto | Keep the exact authored point size on the inner controlled axis; derive the other inner axis using `r` |
| Both axes definite | Both outer dimensions follow box sizing | Keep content-box point axes on the inner; a percentage axis fills its already-resolved outer content. Border-box axes also fill outer content; do not apply ratio |
| Content-box percentage axis, zero padding sum on that axis | Apply the percentage against the original containing axis on the outer owner | Inner 100% consumes the resolved content axis; never copy the source percentage again. Opposite-axis point padding remains outside the inner |
| Both axes auto with qualified cross stretch | Set only the emitted outer cross axis to full containing content size as a border-box dimension | Inner cross axis fills outer content; ratio derives automatic main content size |
| Definite main axis, automatic stretched cross axis | Main dimension retains its sizing meaning; emitted outer cross dimension fills its line | Both inner dimensions resolve independently; no ratio should undo the cross stretch or change the fixed main size |

For a border-box controlled width `W`, the intended content width is `max(W-Pₓ,0)`; an automatic ratio-derived outer height is `contentWidth/r+Pᵧ`. Mirror the formula for controlled height. Both-auto/start means natural content size plus padding even with `box-sizing:border-box`.

Preserve the CSS flex phase distinction: an earlier definite cross size can inform an automatic main-size calculation, but late stretch of the contents does not generally recalculate the flex item's main size. Keep the existing bounded, legacy-flex qualification and test both axes; do not derive broader flex-basis/growth semantics from a successful stretch example. [CSS Flexbox, cross sizing](https://www.w3.org/TR/css-flexbox-1/#cross-sizing).

## Arithmetic and provenance requirements

1. Compute each padding side after font and variable resolution using the existing [Padding](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/padding.rs) and [numeric provenance](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/computed_provenance.rs). Inherited padding carries the parent's computed length, not a new multiplication by the child's font. Preserve escaped-number and decimal provenance. A coefficient rounded to zero is not proof of authored exact zero.
2. Synthetic full cross size belongs in both `Lowered.sizes` and its numeric view, not computed `Style`. Source CSS `auto` must remain `auto` for inheritance and diagnostics. A content-box stretch dimension must not become `100% + padding`.
3. For point content-box dimensions, retain the exact inner content owner instead of subtracting rounded outer additions back into content. This preserves the prior cancellation fix. Do not double-apply padding in either owner. This point-preservation rule must not include percentages: if the same-axis padding sum is zero, resolve the percentage on the outer owner and use 100% on the inner. Percentage-plus-nonzero-same-axis-padding remains diagnostic.
4. Move image ratio/fit bounds after the inner owner's bounds have been established. The current [Bounds::image](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/public-jpeg-build-r2/frozen/inputs/src/numeric.rs:28) assumes its dimensions describe image content. Validate the actual outer owner too; unknown parent dimensions must remain unknown. Removing a synthetic percentage from computed Style must not accidentally bypass the auto-parent bound rejection.
5. Retain native division/multiplication order for aspect ratio and fit. Asset dimensions are exact integers within the asset limits; their quotient is binary32 and is not an exact rational proof. Existing exponent guards check their modeled dimension/fit arithmetic over the viewport domain; they do not certify pixel identity or all ancestor/world arithmetic. Add no claim that every fractional scale passes the paint gate.
6. A zero content dimension is a real supported sizing result to test, not an invalid asset dimension. The inner clip must suppress image paint for padding-floor cases while the outer background and following sibling still paint and lay out correctly.

## Percentage padding: additional composition candidate

All four CSS percentage padding sides depend on the containing block's width. The native padding resolution uses `parent_size.width` in [compute_flexbox_layout](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:176), matching that dependency in the current horizontal writing mode. [CSS Box Model 3](https://www.w3.org/TR/css-box-3/#padding-physical).

The parent's first implementation retains existing point translation and leaves percentage padding diagnostic. That is an unresolved encoding/numeric qualification decision, not proof that percentage padding is impossible. A concrete next candidate for fixed **content-box point sizes** is to keep those points on the inner owner and leave the corresponding outer axis auto: the native outer owner then accumulates its own resolved padding around the content, without requiring a single mixed percentage-plus-points dimension. Disable synthetic outer cross stretch when the authored cross size was definite, even though the emission axis is now auto. Both-auto/start natural content can use the same approach. Test percentage padding against a definite parent, including a padded parent, before admission.

Content-box **percentage dimensions** plus nonzero padding on that **same axis** remain a different problem. Nonzero padding only on the opposite axis does not require a mixed field and is handled by the corrected outer-percentage/inner 100% plan above. For that unresolved same-axis sum, copying the percentage onto a new inner owner would change its containing block, and making the outer dimension auto can create a sizing cycle. Existing single-field lowering cannot express the required sum; this audit establishes no universal impossibility of a more elaborate ordinary composition. Do not silently reinterpret those percentages or resolve them using the compile viewport.

## Focused qualification matrix

Use the existing 96×64 patterned opaque asset and an alpha asset, an asymmetric visible background, and a following contrasting tail. Resize the same emitted file and clone through the established four viewports. The original 20 rows describe semantic coverage; rows 21–22 explicitly repair the percentage omission. The authoring agent's [24-source corpus](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/validation/public-image-padding-cases.json) is the concrete initial source set and can cover several rows together.

| # | Source context | Distinguishing assertion |
| --- | --- | --- |
| 1 | Both auto/start; border-box; asymmetric point padding | Natural content unchanged; outer adds all sides |
| 2 | Both auto/start; content-box; same padding | Same intrinsic result as row1 |
| 3 | Width120; height auto; border-box; padding6px10px | Content100×66⅔; outer120×78⅔ before Chrome quantization |
| 4 | Same with content-box | Content120×80; outer140×92 |
| 5 | Width auto; height80; border-box | Subtract vertical padding before ratio-derived width |
| 6 | Same with content-box | Preserve authored content height before deriving width |
| 7 | Both fixed; border-box; unequal side sums | Both content axes subtract their own padding; no ratio coercion |
| 8 | Both fixed; content-box | Exact inner point dimensions; outer adds padding |
| 9 | Column both-auto stretch; border-box | Cross content is container width minus image padding |
| 10 | Column both-auto stretch; content-box | Full outer cross size, not full cross plus padding |
| 11 | Row both-auto stretch; border-box | Mirror row9 using the definite parent height |
| 12 | Row both-auto stretch; content-box | Mirror row10; no percentage-height cycle |
| 13 | Row fixed main width; auto cross/stretch | Cross fill overrides ratio while main width remains fixed |
| 14 | Column fixed main height; auto cross/stretch | Main height remains fixed; cross fills independently |
| 15 | Border-box width 50%; height auto; point padding | Responsive content subtraction at every viewport |
| 16 | Border-box height 50%; width auto; definite parent height | Responsive mirrored ratio calculation |
| 17 | Declared border-box sizes below both padding sums | Zero content, padding floor, correct background/tail; no image ink |
| 18 | Fixed mismatched ratio; `object-fit:contain` | Letterbox background and object position belong to content rectangle |
| 19 | Same with `object-fit:cover` | Cropped image never paints into padding |
| 20 | Variable/fallback, em/rem and inherited padding with longhand override | Correct final side values and inherited font arithmetic |
| 21 | Content-box width 50%; height auto; `padding:8px 0` | Outer percentage applied once; inner fills it; ratio height and following tail remain correct |
| 22 | Content-box height 50%; width auto; `padding:0 10px` | Mirror row 21 under a definite containing height; preserve horizontal padding outside ratio-derived content |

Use padding whose horizontal/vertical sum ratio differs from the asset ratio; for example symmetric8px10px has sums20:16 rather than96:64. The revised [Chrome capture](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/image-padding-chrome-r1/r2/receipt.json) (SHA-256 `ff482dc3106975252f4780d722b9f7b5477df2cd043089859ca7783de9879806`) preserves the first capture and strengthens nine declarations accordingly: column stretch outer200×136/content180×120; row stretch outer176×120/content156×104. The fixed-main counterparts deliberately do not preserve the natural ratio. This remains browser characterization, not native qualification. Otherwise a wrong outer-ratio implementation can accidentally match stretch geometry. Add an alpha or `object-fit:none` variant to the paint rows without mistaking fit behavior for layout sizing.

The current source corpus's padding-floor case measures outer24×20 and content0×0 in Chrome. Observe content through ResizeObserver or an independent padding subtraction, not only the authored outer `getBoundingClientRect`. On native output inspect the inner owner/Image through existing read-only structure. Expected empty content needs an explicit absence check; retain any ordinary positive-ink gate failure rather than dropping the frame. Compare outer box, inner box, content offsets, tail and full pixels, then inspect all unique pairs. Preserve fractional failures and exact original/clone transfer evidence.

Retain diagnostic controls for unqualified image min/max interactions, automatic margins/alignment wrappers, baseline groups, flex redistribution, content-box percentage sizing with nonzero same-axis padding, and unbounded containing dimensions. Those pre-existing guards are not evidence against padding itself. Add resource controls that combine the largest admitted padding/font/percentage chains with extreme admitted asset ratios, and require CLI/raw ABI/JS parity and no output on diagnostics. Prove all formerly accepted zero-own-padding image files/maps remain exact; the new branch should not rewrite them.

## Source identities

Compiler hashes refer to the frozen pre-padding input files, keeping this plan reproducible while the implementation changes.

| Frozen compiler input | SHA-256 |
| --- | --- |
| images.rs | `bab8c742f2d0612196a86f0a20399350ab2964b89fd90ca30bfd536e38d1e1cb` |
| compiler.rs | `912b8a70352bd93d29f71d569b9262319da682a4c3480a449233c50d4cbca3a4` |
| box_sizing.rs | `3ebdbbee9f1821558ccad324cea593f2387857039f7a6ed692ff97cebf82754b` |
| padding.rs | `4429150da94bfdd59c412f2a983f47ace556a5434e57b00860cf6174db2c30ec` |
| numeric.rs | `d093c82e0f59e9978b0d073ef335ef8346893b9ddf7d4a2a257a3052cb510a8e` |
| computed_provenance.rs | `6f265a89a1f6d3b1c19c3e8ce32c17b87dc591059f9bb06ecdd93e1d035ad60c` |

| Immutable source checked against baseline | SHA-256 |
| --- | --- |
| layout/layout_style_applier.rs | `233d9061c9fdeca344f1ab4e35c25c1413a4a2e06fbc2880beaad26aa47cb9f0` |
| layout/layout_component_style.rs | `46b8b93aa0ee7cd97063716a64730e8174410de8d1fb44f1f8ad96310f5f1335` |
| layout/layout_participant.rs | `fceb9920fa6442d4ff9fc9c31086f9fcdd2def7ae0d445db69d5c899b2925c46` |
| shapes/image.rs | `f68470634b12c533c9263c5aba9af7ffdbc43cbe1abd9aeab9e9a3fa0acd4f62` |
| vendor Taffy style/mod.rs | `0d0cf04bcf4e83d94765deaae2a1aec3e9aee3cb81dadf0d5cf0b3a82ce39e15` |
| vendor Taffy geometry.rs | `15ceb9918eab3813e7c5f7e8c1488599c95772636ac33b6bdf824779796de7d5` |
| vendor Taffy compute/flexbox.rs | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
