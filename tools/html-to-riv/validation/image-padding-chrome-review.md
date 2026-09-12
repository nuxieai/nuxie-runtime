# Image padding: independent Chrome reference

The current [24-case corpus](public-image-padding-cases.json) contains **22 intended candidate compile outcomes and two retained diagnostic contexts**. These expectations describe the proposed scope, not completed native qualification. Pinned Chrome 153.0.8010.12 captured all 24 sources at 240×240, 390×320, 768×560 and 240×240: **96 measurements and PNGs, no errors**. The current authoritative browser receipt is [r2/receipt.json](../output/image-padding-chrome-r1/r2/receipt.json).

Every case has explicit `parent`, `image` and following 8×8 `tail` IDs. The only assets are the existing opaque 96×64 PNG, portrait 48×96 PNG and odd 37×23 PNG. Captures wait for image decoding and bind the authored HTML/CSS, reset, assets, browser requests and screenshots. They record full outer rectangles, computed style, Typed OM, ResizeObserver content/border boxes and a separately derived rectangle from outer size minus computed insets. The latter two content-size observations agree exactly in all 96 captures; computed-style decimal strings are preserved separately and are not treated as higher-precision geometry.

Selected observations at 240×240, in CSS pixels:

| Case | Image outer size | Image content size | Following tail |
| --- | --- | --- | --- |
| Auto/auto, border-box, padding 8 | 112×80 | 96×64 | y=80 |
| Auto/auto, content-box, padding 3 11 7 5 | 112×74 | 96×64 | y=74 |
| Width 120, border-box, padding 6 10 | 120×78.65625 | 100×66.65625 | y=78.65625 |
| Width 120, content-box, padding 6 10 | 140×92 | 120×80 | y=92 |
| Height 100, border-box, padding 6 10 | 152×100 | 132×88 | y=100 |
| Height 100, content-box, padding 6 10 | 170×112 | 150×100 | y=112 |
| Column auto/stretch, padding 8 10 | 200×136 | 180×120 | y=136 |
| Row auto/stretch, padding 8 10 | 176×120 | 156×104 | x=176 |
| Column fixed main height 80/stretch | 200×80 | 180×64 | y=80 |
| Row fixed main width 80/stretch | 80×120 | 60×104 | x=80 |
| Border-box width 50%, padding 8 10 | 100×69.328125 | 80×53.328125 | y=69.328125 |
| Border-box height 50%, padding 8 10 | 131×90 | 111×74 | y=90 |
| Padding exceeds fixed border-box width or both dimensions | 24×20 | 0×0 | y=20 |
| Contain and cover controls | 150×100 | 110×80 | y=100 |

These measurements distinguish a content-area natural ratio from a ratio applied to the padded outer box. Fixed main size plus cross-axis stretch intentionally does not preserve the natural ratio. In the two padding-floor controls, the image content collapses but its padding background and following sibling remain visible. Native validation must assess that intentional empty content without assuming positive image ink. The contain/cover controls have a content origin offset of (20,10), and the inspected screenshots retain background padding outside the fitted/clipped image.

The two retained contexts are valid CSS: percentage padding resolves against containing width (the 200-wide parent gives 10px vertical and 20px horizontal padding), and content-box percentage width plus fixed padding produces a responsive sum. They are outside this initial compiler composition. This is consistent with the definitions of [padding percentages](https://www.w3.org/TR/css-box-3/#padding-physical), [nonnegative inner sizing](https://www.w3.org/TR/css-sizing-3/#box-sizing), and [object-fit against the content box](https://www.w3.org/TR/css-images-3/#the-object-fit). These references inform the interpretation; no browser measurements are shipping dimensions.

All **26 distinct complete browser images** were directly inspected on eight unscaled sheets. Seventy remaining views have equal measured values and full RGBA after opaque-white extension to the larger canvas. This is browser-reference visual reuse only, not Chrome/native equivalence. The [sheet coverage](../output/image-padding-chrome-r1/r2/sheet-coverage.json) retains every source/target proof; the responsive odd-ratio image has separate 390-wide and 768-wide representatives, preserving fractional edges.

The original r1 corpus and all 96 captures remain unchanged. Its padding 8px 12px accidentally gave inset sums with the same 1.5 ratio as the opaque asset, weakening stretch/percentage discrimination. Nine existing declarations were strengthened to 8px 10px, with no new cases, then the whole revised corpus was recaptured under r2. [changes.json](../output/image-padding-chrome-r1/r2/changes.json) names them. Both historical compiler checks use the unchanged frozen `public-jpeg-build-r2` CLI: all 24 cases diagnose `unsupported-target-semantics` with no Rive output, matching each case's separate `priorExpected`. No production or runtime edits and no native renders were performed in this investigation.
