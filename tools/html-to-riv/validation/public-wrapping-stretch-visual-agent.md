# Public wrapping stretch visual review: sheets 09–17

Directly inspected the nine actual sheet PNGs using `view_image` with original detail. Each contains four full-size, unscaled Chrome/native/diff triples: 36 representative pairs reviewed. No visible missing or wrong paint, alignment error, reversal/order mismatch, or crop discrepancy was found. The diff panels contain faint filled residuals; the images are not asserted byte-identical.

Coverage manifest: `tools/html-to-riv/output/playwright/public-wrapping-stretch-r1/public-visual/coverage.json`; SHA-256 `4634054b4fd4a7c99324f7effad704702cf2d286db37f9f3852453c802be9561`.
Capture receipt SHA-256: `4ec69101193b49559c1bbc5c0eeb86b672dde7d9d9d9546642c5bbb0a27aa0b0`.

All nine sheet hashes and 108 constituent Chrome/native/diff image hashes were independently recomputed and matched coverage. Each reviewed member has an empty recorded pixel-failure list.

## Sheet 09

SHA-256: `9280c1bd745bd233bbdfaac8461ed41bbcd8262493c510953734a766f7ffa011`.

Narrow column crops match: blue/green blocks in four-line reverse-wrap cases and the right-edge purple strip in the reverse-direction case remain present. Relative box offsets and blank intervals agree.

Members directly inspected:

- `public-stretch-column-wrap-three-omitted`, frame 1.
- `public-stretch-column-reverse-four-normal`, frame 1.
- `public-stretch-column-reverse-four-stretch`, frame 1.
- `public-stretch-column-reverse-wrap-three-normal`, frame 1.

## Sheet 10

SHA-256: `9fe12ecae6edc8b33d6da0d2f8efbd0362c868bbc2efeb976dac54b8569d7532`.

Narrow reverse-column cases match, including purple at the right edge, the separated coral and yellow blocks, and the thin green strip at the crop. Normal/stretch twins have the same placements.

Members directly inspected:

- `public-stretch-column-reverse-wrap-three-stretch`, frame 1.
- `public-stretch-column-reverse-wrap-three-omitted`, frame 1.
- `public-stretch-column-reverse-reverse-four-normal`, frame 1.
- `public-stretch-column-reverse-reverse-four-stretch`, frame 1.

## Sheet 11

SHA-256: `38ee69278e42222b61bf3d4727ee820637f2b1074c96901c964283306aba7d60`.

Single-line alignment and overflow match. The narrow row keeps a blue segment to the right of coral; the reversed column keeps lower-left blue and right-hand yellow. The overflowing reversed row retains a top yellow strip and blue block, and column overflow retains yellow past the other blocks.

Members directly inspected:

- `public-stretch-row-wrap-single-normal`, frame 1.
- `public-stretch-column-reverse-reverse-single-stretch`, frame 1.
- `public-stretch-row-reverse-reverse-overflow-normal`, frame 1.
- `public-stretch-column-wrap-overflow-stretch`, frame 1.

## Sheet 12

SHA-256: `9b89916c9649b0db2333bf66cc79e2b75de350cd1dc5e1ccc730ed0796ad5ee2`.

At the 80px bottom crop, three-line normal, stretch and omitted cases each match their reference. The blue segment has the same visible height in these matched variants. Four-line reverse wrapping preserves the lower blue strip.

Members directly inspected:

- `public-stretch-row-wrap-three-normal`, frame 2.
- `public-stretch-row-wrap-three-stretch`, frame 2.
- `public-stretch-row-wrap-three-omitted`, frame 2.
- `public-stretch-row-reverse-four-normal`, frame 2.

## Sheet 13

SHA-256: `81e63da5b145af84da2678da05a85a3dd811ee82323c8239b4021926c00b9cc6`.

Reversed three-line layouts reproduce the offsets of blue/green and the very thin purple bottom edge. The four-line stretch case reproduces its green block and bottom-clipped blue block. No missing one-pixel sliver is visible.

Members directly inspected:

- `public-stretch-row-reverse-four-stretch`, frame 2.
- `public-stretch-row-reverse-wrap-three-normal`, frame 2.
- `public-stretch-row-reverse-wrap-three-stretch`, frame 2.
- `public-stretch-row-reverse-wrap-three-omitted`, frame 2.

## Sheet 14

SHA-256: `2fde0255c2e1c8962df2bb9fc0c8435f16c9c86c45989daaf270b5cd9788fdf4`.

The double-reversed four-line coral and yellow blocks align and clip equally. Column three-line cases reproduce the staggered blue, green and purple blocks with matching bottom crops.

Members directly inspected:

- `public-stretch-row-reverse-reverse-four-normal`, frame 2.
- `public-stretch-row-reverse-reverse-four-stretch`, frame 2.
- `public-stretch-column-wrap-three-normal`, frame 2.
- `public-stretch-column-wrap-three-stretch`, frame 2.

## Sheet 15

SHA-256: `60e7bfc6fa9de0d4a45313deb08b7c3814fe1b1bf455ac1679046a47ba0e92d5`.

Column normal/omitted and four-line reverse-wrap layouts preserve all visible colors, spacing and dimensions. The reverse-column three-line case reproduces the staggered blue/yellow bottom blocks and upper coral/purple positions.

Members directly inspected:

- `public-stretch-column-wrap-three-omitted`, frame 2.
- `public-stretch-column-reverse-four-normal`, frame 2.
- `public-stretch-column-reverse-four-stretch`, frame 2.
- `public-stretch-column-reverse-wrap-three-normal`, frame 2.

## Sheet 16

SHA-256: `942a44fdf50ff2614eb5310c346e397307e87d0e417f14d35679933f193873e5`.

Reverse-column stretch and omitted twins reproduce the same staggered box positions. Four-line double-reversed normal/stretch cases preserve horizontal order, the coral offset and neighboring yellow/green/blue edges.

Members directly inspected:

- `public-stretch-column-reverse-wrap-three-stretch`, frame 2.
- `public-stretch-column-reverse-wrap-three-omitted`, frame 2.
- `public-stretch-column-reverse-reverse-four-normal`, frame 2.
- `public-stretch-column-reverse-reverse-four-stretch`, frame 2.

## Sheet 17

SHA-256: `a8deb516864c9ff172085d2d4ca3cbe48ab6064cc62027f297e497a1c13745fd`.

Single-line bottom crops and overflowing layouts agree. The reversed-column single-line case retains only coral and a small yellow segment at the bottom; overflowing reversed row retains the same blue block/top yellow strip. The overflowing column yellow extends to the crop in both images.

Members directly inspected:

- `public-stretch-row-wrap-single-normal`, frame 2.
- `public-stretch-column-reverse-reverse-single-stretch`, frame 2.
- `public-stretch-row-reverse-reverse-overflow-normal`, frame 2.
- `public-stretch-column-wrap-overflow-stretch`, frame 2.

## Limits

This review covers sheets 09–17 only. It does not independently review sheets 00–08 or each clone/return frame; any transfer to duplicate frames needs the separate exact-repeat evidence. Source-labeled minimum/maximum fixtures show matching resulting box dimensions here; visual inspection alone does not prove the source-bound minimum-wins calculation. Transparent regions remain visibly clear, but invisible-owner mapping requires separate geometry checks.

Normal, explicit stretch and omitted variants are compared individually to their corresponding references. A follow-up identity check confirmed matching Chrome boxes and RIV/Chrome/native PNG hashes for all 64 normal/stretch twin frame pairs and all 32 normal/omitted twin frame pairs. Earlier notes mistakenly described contact-sheet row offsets as scene differences; those statements are corrected here. Fixed root layouts are cropped by changing viewports; this is not a test of responsive root sizing. No browser geometry was used as expected image data in this review, no new render was made, and no compiler source changed. Native command-stream grammar, full language coverage and practical performance are outside this visual review.

Twin identity evidence: `tools/html-to-riv/output/playwright/public-wrapping-stretch-r1/twin-identity.json`; SHA-256 `d0456302d8e0a8cf25a3c96c0913776455693be4b3d15785611964d3c7f6ce2f`. The additional omitted-twin check above compared the existing capture receipt fields without rerendering.
