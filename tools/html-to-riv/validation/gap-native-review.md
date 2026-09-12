# Ordinary physical gap investigation

Ordinary immutable gap fields match all128 Chrome geometry frames in this16-scene matrix;124 pixel frames and all256 clear checks pass. The practical point/em cases and definite-percentage cases pass every pixel frame. Four intrinsic-percentage column/reverse pixel failures remain preserved. This is native investigation, not public compiler admission.

Existing spacing qualification covers justify-content helpers, not row-gap/column-gap. No completed native gap qualification was found in the current tracked validation reviews. The runtime and vendor files used for this audit exactly match immutable baseline6c7ac16617835b5f581784ff08a9e779bb52faf3; `source-audit.json` records hashes and exact source identity checks.

## Source mapping

`layout/layout_component_style.rs:245–248` maps gapHorizontal plus units to YGGutter::Column and gapVertical plus units to YGGutter::Row. For ordinary CSS horizontal writing this means column-gap→gapHorizontal, row-gap→gapVertical. Point unit1 is a physical length; Percent unit2 is an authored percentage. The mapping stays physical across row/column and reverse directions. `layout/layout_style_applier.rs` converts these to Taffy gap.width/gap.height and percentages via value/100.

Vendor `compute/flexbox.rs:476` resolves gaps against corresponding inner container dimensions; unlike percentage padding, vertical gap does not always use containing width. Its intrinsic-main branch at302–310 re-resolves the main percentage gap against the resulting inner main size after sizing the container. Pinned Chrome153.0.8010.12 exhibits that behavior in these controls too; do not assume an intrinsic percentage gap is always zero during final placement. Intrinsic contribution and final item positioning can differ.

## Cases and adapter scope

Four profiles run in all four main directions, with nested alpha-painted boxes and original/clone viewport400x200→200x120→100x80→400x200:

| Profile | Gap descriptors | Context | Geometry / pixels |
|---|---|---|---|
| point-definite | P row7px/column11px; A row3px/column5px | definite50% parent; nested intrinsic A | 32/32,32/32 |
| em-intrinsic-order | P .5em/.75em at font20px; A inherits computed10px/15px at font10px | auto parent; order2/−1/0 | 32/32,32/32 |
| percent-definite | P row10%/column5%; nested A point gaps | definite50% parent | 32/32,32/32 |
| percent-intrinsic-control | same percentage descriptors | auto parent, start-aligned | 32/32,28/32 |

The adapter compiles with gap declarations removed and writes only gapVertical/Horizontal values and units on the authored parent styles. It adds no records and retains source-map IDs, paints, parents, helper-free topology and order. For em/inherit, computed descriptors are supplied explicitly by the fixture; Chrome executes the CSS independently. This validates native descriptor behavior, not a gap parser/cascade implementation.

For point row layout, A width40 ends at40, B starts51, C starts92, proving column-gap11. Nested A children start at y0/y11 from8px child height plus row-gap3. In the em/order row case, parent intrinsic width120 includes90px children plus two15px gaps; A's intrinsic height26 includes two8px children plus inherited10px gap, despite its smaller local font.

Intrinsic row percentage control computes parent width90 but places B at44.5 and C at79: final4.5px gaps overflow the original intrinsic width. Intrinsic column control computes parent height54 and places B at24.390625 and C at49.78125 in Chrome; native positioning uses nearby binary32 values and passes the documented geometry gate. This is not an exact geometry identity claim. Examples and raw observations are retained.

## Failures, visual evidence and reproduction

The four failed pixel frames are column and column-reverse intrinsic-percentage controls at frames2/6 (100x80), failing mismatch ratio. They remain failed with unchanged thresholds; no unsupported/impossible conclusion follows from this candidate failure. Larger-view copies may pass a ratio threshold despite similar local fractional differences, so the small-view failures cannot be dismissed.

Directly inspected all48 original frames0/1/2 pairs in12 full-frame sheets, visual-0.png through visual-44.png step4. Point gaps, reverse placement, nested gaps, ordered intrinsic sizes and percentage overflow follow the matching geometry. Fractional intrinsic-column edge differences remain represented by the four pixel failures. No other visible divergence found in the passing pairs. The other80 pairs transfer independently through exact decoded-fullRGBA equality for Chrome and native to directly reviewed references (160 image checks), giving complete128-pair coverage. All16 RIV byte streams and parsed maps reproduce exactly.

## Practical integration boundaries

A compiler-owned physical pair descriptor can lower computed point gaps directly on the authored container's style, preserving zero default bytes. This should be staged behind computed parsing and existing numeric/semantic admission rather than silently expanding the whitelist. First/last intrinsic baseline summaries need gaps in descendant offsets and total size; invalidate unproved summaries until updated. Intrinsic numerical bounds and positioning must account for (participant_count−1) gap terms and finite percentage resolution.

Existing around/evenly and auto-margin helpers are actual extra native children. Applying native gap to that whole parent would also insert gaps around those synthetic participants, potentially duplicating authored inter-item gaps. Guard those helper combinations or prove a composition; the present helper-free matrix does not qualify them. Direct unpadded alignment wrappers can receive the authored parent's main gap as participants, but gap belongs only on the authored container's own child layout, not copied onto arbitrary synthetic wrappers. Wrapping line gaps, flex growth, padding, min/max, automatic margins, baseline groups, cyclic percentages and arbitrary nested intrinsic trees require joint checks.

This evidence establishes viable native candidates and retained limitations only. It does not claim all intrinsic percentage contexts are supported, grid support, generalized content sizing or a renderer limitation. No public/compiler/runtime source edits or commit were made.

Tracked bindings: [gap-native-receipt.json](gap-native-receipt.json). Retained local evidence: `tools/html-to-riv/output/gap-native-r1/`.
