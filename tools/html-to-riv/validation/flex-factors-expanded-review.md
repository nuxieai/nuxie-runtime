# Expanded ordinary flex-factor controls

All24 new scenes pass192/192 Chrome geometry and pixel frames plus384 clear checks. The expansion demonstrates equal zero factors with positive point basis, equal factors with automatic basis derived from nested fixed content, and flex applied at ordered perpendicular cross-alignment wrappers. It remains bounded experimental validation on immutable baseline6c7ac16617835b5f581784ff08a9e779bb52faf3, not public routing or general flex qualification.

The six profiles run in all four main directions. Parent main size resolves200→100→50→200, repeated on the ordinary clone. Both authored flexible items have automatic preferred main size, explicit zero minimum, fixed40px cross extent, alpha fill and a nested alpha-filled descendant. A fixed40px nonshrinking sibling is present.

| Profile | Equal grow/shrink factor | A/B basis | Nested main extents | Alignment/order |
|---|---:|---|---|---|
| zero-point | 0 | 60px/40px | 10px/30px | start, DOM order |
| auto-basic | 1 | auto/auto | 60px/40px | start, DOM order |
| auto-partial | .25 | auto/auto | 60px/40px | start, DOM order |
| wrapper-point | 1 | 60px/40px | 10px/30px | A center/order2, B end/order−1, fixed order0 |
| wrapper-zero-point | 0 | 60px/40px | 10px/30px | same ordered wrappers |
| wrapper-auto | 1 | auto/auto | 60px/40px | same ordered wrappers |

The adapter uses the frozen public compiler for nonflex placeholders with authored main Auto. For each authored A/B, it reads the actual parentId: when the authored parent is a perpendicular alignment wrapper, it changes that outer LayoutComponent's fraction/main Fill/basis/units; otherwise it changes the authored component itself. The axis remains the original container's main axis. Point basis uses units1; auto basis uses units3. This matches the current proposed integration seam rather than applying flex to the inner perpendicular child. The run's compile logs record every authored-to-flex-participant pair. No source-map identity, authored paint, ordering or parent relationship is changed; no records are added by the adapter.

Zero-factor positive point basis retains A60/B40 while the container shrinks, including expected overflow. Equal auto-basis factor1 resolves A90/B70 at parent200 and A36/B24 at parent100, matching content-derived60/40 bases; factor.25 retains partial-factor leftover/overflow behavior. Ordered wrapper-auto row example keeps B at cross30 and A at cross15 while main sizes change: at parent200, B starts0 with width70 and A starts110 with width90; at parent100, B width24 and A starts64 with width36. Descendants follow the authored boxes and can overflow as in Chrome. All reverse directions and clones pass.

Directly inspected all72 original frames0/1/2 Chrome/native pairs in18 sheets (`visual-0.png` through `visual-68.png`, step4). No visible divergence found across shrink, zero-factor overflow, partial growth, reordered center/end alignment or nested alpha paint. The remaining120 pairs transfer by independent exact fullRGBA equality for Chrome/native to the directly reviewed frames (240 image checks):3/4/7→0,5→1,6→2. Thus all192 pairs have complete visual coverage; only72 are claimed directly viewed. All24 ordinary RIV files reproduce byte-for-byte and source maps reproduce as parsed JSON.

There are no failures in this new corpus. Earlier eight thin-alpha failures remain preserved in flex-factors-native-r1 and its tracked receipt; this expansion does not overturn them. Auto-basis evidence here covers fixed nested content in definite containers with automatic preferred main size. It does not establish arbitrary intrinsic ancestors, percentages in auto basis, automatic minimum sizes, main auto-margin/spacing-helper interaction, baseline summaries, nested flexible descendants, unrelated cross wrappers, or independent positive-basis grow/shrink. Public parser/routing integration requires its own regression/native validation. No public/runtime/renderer source changes or commit were made for this experiment.

Tracked bindings: [flex-factors-expanded-receipt.json](flex-factors-expanded-receipt.json). Retained local evidence: `tools/html-to-riv/output/flex-factors-expanded-r1/`.
