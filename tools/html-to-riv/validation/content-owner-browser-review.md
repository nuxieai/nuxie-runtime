# Independent content-owner browser reference

The **33-case corpus is ready for the private ordinary content-owner experiment**. The frozen public compiler accepts all 28 intended core cases and rejects the five separately labeled proposed cases without output. No admission expectation required correction; no source HTML/CSS was rewritten after the initial fixture generation. The five large-coordinate originals, one original fractional control and five prior diagnostic sources retain their exact authored HTML/CSS.

`content-owner-cases.json` has SHA-256 `733b761f3f15e184de513e756e38d891d8de5d00167f440ca142413be13dc33a`. The compiler used only for baseline admission is `output/public-value-build-r1/frozen/html-to-riv`, SHA-256 `034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d`. Exact requests, output/map hashes and diagnostics are preserved under `output/content-owner-browser-r1/baseline-admission-r1/`.

Pinned **Chrome 153.0.8010.12** captured every original document with the unchanged authoring reset and viewport sequence **240×160 → 390×200 → 768×120 → 240×160**. This matches the shared native driver's resize sequence; the separate public compile viewport was 390×160. There are **132 frames**, each with CSSOM values, CSS Typed OM values, ResizeObserver content/border sizes, bounding rectangles, offsets/client sizes and a viewport PNG. All 33 final frames exactly reproduce their initial measurements and PNG bytes.

The 28 core cases cover four directions, fixed point dimensions, min/max and conflicting clamps, zero-content padding floors, auto/intrinsic sizing, nested percentage descendants, authored inheritance, sibling order, overflow and translucent backgrounds. Auto sizing remains a falsifiable candidate constraint, not an assumption that the proposed composition works. The five owner percentage/preferred/bound/inset combinations remain **outside the initial point-owner profile** and publicly diagnostic. Browser measurements do not admit them into the compiler.

The large controls preserve distinct browser measurements rather than treating them as interchangeable. At the first viewport:

| Original case | Parent typed width | Parent rectangle width | Child ResizeObserver content width | Child rectangle width |
|---|---:|---:|---:|---:|
| large-add-side-first | 1000000 | 1000000.0625 | 1000000 | 1000000 |
| large-padding-cancellation | 0.03125 | 1000000.0625 | 0.03125 | 0 |
| large-rounded-outer | 999999.875 | 3000000 | 999999.8125 | 999999.75 |
| large-content-cancellation | 999999.9375 | 3000000 | 999999.9375 | 1000000 |
| large-full-source-limits | 1000000 | 3000000 | 1000000 | 1000000 |

These are observed values, not a conclusion about internal browser arithmetic. The latter four children begin at x=1000000. CSSOM and Typed OM text can serialize large values as `1e+06px`; the separate numeric Typed OM field retains the authored precision shown above. The actual native comparison must use the prescribed geometry gate without replacing reference dimensions with a preferred metric.

All 99 initial/resized captures were visually inspected through six unscaled contact sheets. Every source RGBA tile was verified byte-identical to its sheet crop; 33 restored frames transfer by exact PNG identity. The complete capture set has 78 distinct PNG hashes. Small controls visibly expose insets, direction, clamp results, inherited overflow and alpha overlap. Most large children are offscreen, so the dark parent strips cannot establish their child geometry. The five proposed diagnostic sources intentionally retain their original transparent paints and appear white; their geometry is still recorded. Fractional and large controls remain explicitly labeled, with no relaxed visual or geometry gates.

`output/content-owner-browser-r1/receipt.json` binds original browser data and its capture driver. `review-receipt.json` additionally binds admission evidence, contact sheets and this review. This work contains no candidate compiler output, native renderer run, production change or public support claim. The parent experiment and independent native baseline must compare the same original sources against these references before deciding whether an inner content owner improves L12.
