# Immutable native padding investigation

Ordinary LayoutComponentStyle provides direct pixel and percentage padding on all four sides. The bounded border-box candidates pass160/160 geometry frames and158/160 pixel frames; two nested fractional leaf pixel failures are preserved. Four intentional content-box controls fail all32 geometry/pixel frames because the direct mapping retains border-box sizing. All384 clear checks pass. This is investigation under TARGET.md, not public admission or a change to SUPPORT/BACKLOG.

## Source semantics

The five audited runtime/vendor files exactly match immutable commit6c7ac16617835b5f581784ff08a9e779bb52faf3; `source-audit.json` records independently checked hashes.

- `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs:255` maps ordinary paddingLeft/Right/Top/Bottom and their unit fields into layout padding. Point unit1 maps pixels; Percent unit2 maps the authored percentage. Left/right map according to the runtime direction context; this corpus uses the compiler's horizontal LTR profile.
- `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs:625` copies those edges into Taffy; its length conversion at730 stores percentage/100 as ordinary LengthPercentage. This differs from width/height's separate rive_yoga_percent encoding and can have different rounding.
- `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:176` and448 resolve every padding side against parent_size.width, including top/bottom. The reference is the containing block's width, not the padded box's width or the flex main axis.
- The same function clamps preferred dimensions through min/max and floors outer sizes by padding+border. At479, inner size subtracts the content-box inset; child layout consequently receives content dimensions. `layout_component.rs:1110` also exposes width/height minus solved padding as inner dimensions.
- The ordinary bridge retains Taffy's default BorderBox (`vendor/.../style/mod.rs:594`) and does not assign a box-sizing override. The style algorithm has ContentBox branches, but this is not evidence of an ordinary encoded LayoutComponent property reaching that branch. No direct ordinary box-sizing field was found in this audited path.

## Native experiment

The adapter strips only tested padding and the intentional content-box declaration before compiling, then changes only the selected authored LayoutComponentStyle padding values/units in the ordinary file. It verifies the original decode/encode roundtrip. No records, parents, paint records or map identities are added/changed. Point padding is7/11/13/17px (top/right/bottom/left); percentage padding is2/3/4/5%. Definite cases also give the nested A box1/2/3/4px padding and nested percentage dimensions.

Six profiles run in row, row-reverse, column and column-reverse, with viewport400x200→200x120→100x80→400x200 on original and clone:

| Profile | Purpose | Geometry / pixels |
|---|---|---|
| px-definite | 50% sizes, fixed min/max, percentage child/grandchild, nested padding/alpha paint | 32/32,32/32 |
| px-floor | specified10x8 smaller than padding sums28x20 | 32/32,32/32 |
| px-intrinsic | auto width/height, start alignment, fixed children | 32/32,32/32 |
| percent-definite | width-based percentage sides with definite/minmax box and nested percentages | 32/32,30/32 |
| percent-intrinsic | auto box with percentage padding relative to definite root containing width | 32/32,32/32 |
| content-box-control | intentional direct BorderBox mapping against content-box CSS | 0/32,0/32 |

Concrete observations are preserved in `examples.json`. Pixel-floor resolves28x20 despite authored10x8. Pixel intrinsic row resolves98x40 (70x20 content plus28x20 padding); column resolves68x55. Percentage intrinsic row at viewportwidth400 resolves102x44:70x20 content plus32x24 padding. Thus top/bottom percentages use400px width, not200px viewport height or102px resulting box width.

At the first percentage-definite row viewport, P clamps to180x100, left/top padding resolves20/8, A width is74 (half of180−20−12), and leaf width is34 (half of74−4−2). Nested heights likewise resolve against content height; Chrome fractional quantization and native f32 may differ within the documented geometry tolerance, so these are geometry passes, not a universal bitwise geometry claim.

## Preserved failures and visual coverage

The two border-box pixel failures are `padding-column-reverse-percent-definite`, frames2/6: leaf local RGB error7.35. Chrome leaf is15x3.09375 at(9,26.8125); native geometry remains within the existing gate. The raw metrics/images remain failed, without weakened thresholds. Content-box controls visibly demonstrate a separate structural mismatch: first row Chrome P is208x120 while native remains180x100, and descendant percentage geometry differs accordingly. This excludes the direct unchanged border-box mapping for those content-box inputs, not all possible ordinary-file compositions.

Directly inspected all72 original frames0/1/2 pairs in18 full-frame contact sheets, visual-0.png through visual-68.png step4. Content-box size/descendant mismatches are clear; padding, intrinsic growth, asymmetry and reverse-flow placement otherwise follow the matching geometry. The tiny fractional leaf retains its local pixel failure; no other visible divergence found in passing pairs. The other120 pairs transfer by independent exact decoded-fullRGBA equality for Chrome/native to the directly inspected frames, recorded as240 image checks. All192 pairs therefore have complete visual evidence, with only72 claimed directly viewed. All24 files/maps reproduce exactly through the retained adapter/binary.

## Compiler implications, still unqualified

Native pixel/percentage padding is a viable border-box candidate. A future computed style can store physical side descriptors and emit value/units on the authored box, retaining paint on its outer rectangle. Keep default zero-padding byte output unchanged. Existing perpendicular alignment wrappers should retain authored padding on the inner authored box; their intrinsic size/bounds transfer and percentage containing-block behavior need their own tests. Baseline summaries currently derive sizes without these insets, so padded baseline participants/descendants must update that proof or diagnose rather than reuse unpadded metrics. Flex basis, min/max, root padding, safe auto margins, wrapping/cross-slot measurements and padding inside flexible helpers need joint qualification.

The intrinsic percentage cases here have a definite containing width; they do not settle cyclic percentage padding in an indefinite containing block. Negative padding remains invalid CSS. Pixel or em/rem resolution and percentage grammar need parser/cascade tests separately. Content-box could use compiler-owned dimension/bound expressions or an ordinary wrapper composition, but percentage sizes, padding floors, containing blocks and intrinsic contributions mean merely adding padding to a fixed width is not a general solution. No runtime change or public support claim is needed to retain this evidence.

Tracked bindings: [padding-native-receipt.json](padding-native-receipt.json). Retained evidence: `tools/html-to-riv/output/padding-native-r1/`.
