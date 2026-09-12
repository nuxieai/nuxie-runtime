# Public wrapping remainder direct visual review: sheets 08–15

Directly inspected eight actual sheet PNGs using `view_image` at original detail: 32 full-size, unscaled Chrome/native/diff triples. Comparisons were made within each pair, accounting for the contact-sheet panel offsets. No visible missing/wrong paint, reversed order, displaced thin edge, centered-segment mismatch or crop discrepancy was found. Faint filled diff residuals remain; byte identity is not asserted.

Coverage manifest: `tools/html-to-riv/output/playwright/public-wrapping-remainder-r1/public-visual/coverage.json`; SHA-256 `22c2e6248e400b22b74fe6e0665250ddbcaa38f2272d357d5cebf27250bb3aa4`.
Capture receipt SHA-256: `d6a9bebb1b07fe35ba1af7d6a9975d4ce111122f5ca3096dab72996083bc75e3`.

Recomputed all eight sheet hashes and all 96 constituent Chrome/native/diff image hashes against coverage; all match. All reviewed member pixel-failure lists are empty.

## Sheet 08

SHA-256: `ebb304bc151d908c61df6122e449bd8dc6049bd891f6e77dc4a3e96cc9e6b37a`.

K2 row and K3 column cases retain the same yellow/coral/blue bands, narrow gaps and line order within each Chrome/native pair. No visible extra edge or missing band.

Members directly inspected:

- `public-remainder-k2-r1-row-wrap-single`, frame 1.
- `public-remainder-k2-r1-row-wrap-reverse-single`, frame 1.
- `public-remainder-k3-r1-column-wrap-single`, frame 1.
- `public-remainder-k3-r1-column-wrap-reverse-single`, frame 1.

## Sheet 09

SHA-256: `b3714677981cd0b4f85818d7abb99621302a6291b527e378521d42053a7d58c6`.

Reverse-main K3 rows and K4 columns reproduce block offsets, narrow white separators and reversed color order within each pair. The right-hand green/yellow edges remain aligned.

Members directly inspected:

- `public-remainder-k3-r2-row-reverse-wrap-single`, frame 1.
- `public-remainder-k3-r2-row-reverse-wrap-reverse-single`, frame 1.
- `public-remainder-k4-r1-column-reverse-wrap-single`, frame 1.
- `public-remainder-k4-r1-column-reverse-wrap-reverse-single`, frame 1.

## Sheet 10

SHA-256: `7e071cb78d91ec3f10f8e1cfac1d7720c92d33f8796400b81ed4d4992952076d`.

K4 rows and columns match the clipped shape extents and neighboring blue/green edges. Forward and reverse wrapping are assessed against their own references; no cross-fixture comparison is used.

Members directly inspected:

- `public-remainder-k4-r2-row-wrap-single`, frame 1.
- `public-remainder-k4-r2-row-wrap-reverse-single`, frame 1.
- `public-remainder-k4-r3-column-wrap-single`, frame 1.
- `public-remainder-k4-r3-column-wrap-reverse-single`, frame 1.

## Sheet 11

SHA-256: `6a10e5728da445bab6ff4f246cbcc930881b669871185664eee7bf2b38650ee6`.

K7 rows preserve all seven differently sized color bands and their gaps. Narrow column crops preserve the purple edge in the forward case and dark-teal/magenta/purple blocks in the reverse-wrap case.

Members directly inspected:

- `public-remainder-k7-r1-row-reverse-wrap-single`, frame 1.
- `public-remainder-k7-r1-row-reverse-wrap-reverse-single`, frame 1.
- `public-remainder-k7-r2-column-reverse-wrap-single`, frame 1.
- `public-remainder-k7-r2-column-reverse-wrap-reverse-single`, frame 1.

## Sheet 12

SHA-256: `b422c11216e6b85f45e73e30de3e7ead9cb19e7c88ca797ba24848f50d83ebe6`.

K7 remainder3 row stacks and remainder4 column crops show matching bands and separators. Right artboard boundaries truncate the same colors at the same visible positions.

Members directly inspected:

- `public-remainder-k7-r3-row-wrap-single`, frame 1.
- `public-remainder-k7-r3-row-wrap-reverse-single`, frame 1.
- `public-remainder-k7-r4-column-wrap-single`, frame 1.
- `public-remainder-k7-r4-column-wrap-reverse-single`, frame 1.

## Sheet 13

SHA-256: `21d5e423a0d01794199db82a78c01111040050c94215788f4785330ef25dd610`.

K7 remainder5 row stacks and remainder6 column crops retain all visible bands. No visibly displaced horizontal edge, center shift or incorrectly ordered block appears within a pair.

Members directly inspected:

- `public-remainder-k7-r5-row-reverse-wrap-single`, frame 1.
- `public-remainder-k7-r5-row-reverse-wrap-reverse-single`, frame 1.
- `public-remainder-k7-r6-column-reverse-wrap-single`, frame 1.
- `public-remainder-k7-r6-column-reverse-wrap-reverse-single`, frame 1.

## Sheet 14

SHA-256: `792cfce30f15ad34cae58efa8e475582225a11d8b9d9c0024413adf0a82fe8ad`.

Two-owner line cases reproduce the stepped edges and unequal centered extents, including thin yellow/coral columns and the blue/coral/yellow short lower segments. White gaps and visible endpoints agree.

Members directly inspected:

- `public-remainder-k2-r1-row-wrap-pairs`, frame 1.
- `public-remainder-k2-r1-column-reverse-wrap-reverse-pairs`, frame 1.
- `public-remainder-k3-r1-row-reverse-wrap-pairs`, frame 1.
- `public-remainder-k3-r2-column-wrap-reverse-pairs`, frame 1.

## Sheet 15

SHA-256: `25e584ae750b2f7298c68802fdb8e6359fd07e4e28addaaeba7b579a381b03c4`.

K4/K7 paired cases reproduce the narrow centered segments and stepped row edges. The K7 column crop retains the purple segment at the right edge; reversed K7 rows preserve dark-teal through yellow ordering and matching gaps.

Members directly inspected:

- `public-remainder-k4-r1-column-wrap-pairs`, frame 1.
- `public-remainder-k4-r3-row-reverse-wrap-reverse-pairs`, frame 1.
- `public-remainder-k7-r1-column-reverse-wrap-pairs`, frame 1.
- `public-remainder-k7-r6-row-wrap-reverse-pairs`, frame 1.

## Limits

This inspection covers these 32 representative pairs only, at the narrow 96×260 viewport shown on these sheets. Other sheets and exact transfers to clone/return frames require their separate review and repeat evidence. All 256 pixel gates pass according to the campaign, but that is not a claim that every frame was directly inspected here.

Geometry is not exact: the campaign separately records 552 nonzero scalar differences among 7,040 comparisons, with a maximum approximately 0.007815px. Such differences can be visually imperceptible and remain within the existing geometry gate. This review does not replace the source remainder-distribution proof or independently prove subpixel arithmetic from raster images.

This covers fixed roots, bounds, center offsets, wrap direction and artboard cropping in the given corpus; it does not qualify responsive roots or arbitrary CSS. No new rendering, source edit, runtime mutation, native stream grammar verification or performance test was performed.
