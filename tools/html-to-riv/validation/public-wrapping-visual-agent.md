# Public wrapping direct visual review: sheets 06–11

Directly inspected all six actual sheet PNGs with `view_image` at original detail. Each sheet contains four full-size, unscaled Chrome/native/diff triples: 24 representative pairs reviewed. This is visual inspection of the actual public CLI capture, not inferred inspection from passing counters.

Across these images I found no visible missing or wrong paint, positional mismatch, displaced edge band, transparency error, or inconsistent viewport crop. Diff panels contain faint filled residuals; they are not byte-identical images. The supplied per-member pixel-failure lists are empty.

Coverage manifest: `tools/html-to-riv/output/playwright/public-wrapping-r2/public-visual/coverage.json`; SHA-256 `70729dc1a41d7eaf88ce0d52fa2056ae181a7ab516d183afbcf4617306529b73`.
Capture receipt: `/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-wrapping-r2/receipt.json`; SHA-256 `994dee882ad8ab8efab4211c14b41ee0138825332142fbc98ac1f0999ff5b118`.

All six sheet hashes and their 72 constituent Chrome/native/diff file hashes were independently recomputed and matched the coverage manifest.

## Sheet 06

SHA-256: `7acee62d171c9333f2f234cdb289c6a78fc84af31cea39b0a62690abb520d9de`.

The narrow row layouts match in all four direction/wrap combinations. Yellow, coral, blue and green blocks meet the same edges; right-hand artboard cropping agrees. Reverse row cases preserve the short blue/yellow segments and tall coral edges.

Members (all directly inspected):

- `public-row-wrap-dyadic`, frame 1.
- `public-row-wrap-reverse-dyadic`, frame 1.
- `public-row-reverse-wrap-dyadic`, frame 1.
- `public-row-reverse-wrap-reverse-dyadic`, frame 1.

## Sheet 07

SHA-256: `dbe556f8e5159d1472212c9e32fe25b2f27f408d288d5c7d1f46d4271836810d`.

The narrow column layouts match. In column wrap-reverse, most paint lies beyond the right crop; the remaining blue strip and green block appear in both images. The reverse-column thin yellow edge and bottom coral strip also agree.

Members (all directly inspected):

- `public-column-wrap-dyadic`, frame 1.
- `public-column-wrap-reverse-dyadic`, frame 1.
- `public-column-reverse-wrap-dyadic`, frame 1.
- `public-column-reverse-wrap-reverse-dyadic`, frame 1.

## Sheet 08

SHA-256: `2ad41cba36d1e3a0cd44d513085c49200951422304ac95df1b809705eea56408`.

Integer row layouts agree at the 80px bottom crop. The wrap-reverse case retains only the blue/green strip; row-reverse wrap retains the coral segment; the double-reverse case retains the thin coral bottom edge. No missing or additional visible paint.

Members (all directly inspected):

- `public-row-wrap-integer`, frame 2.
- `public-row-wrap-reverse-integer`, frame 2.
- `public-row-reverse-wrap-integer`, frame 2.
- `public-row-reverse-wrap-reverse-integer`, frame 2.

## Sheet 09

SHA-256: `2ffb35cf6083847ec2262836664b815affeec8feea700a5fb2bbbf207fc8f188`.

Integer column layouts agree in position, width and bottom clipping. The reverse-column coral strips along the bottom, blue blocks, yellow intersections and green blocks remain present at corresponding positions.

Members (all directly inspected):

- `public-column-wrap-integer`, frame 2.
- `public-column-wrap-reverse-integer`, frame 2.
- `public-column-reverse-wrap-integer`, frame 2.
- `public-column-reverse-wrap-reverse-integer`, frame 2.

## Sheet 10

SHA-256: `b84ccf691e9b6a8b4a6cdefc8e9990602995b6b15669e17d9bf7738f2a0a21e0`.

Dyadic row layouts agree at the bottom crop, including the small gap before the yellow/coral line, blue/green offsets, and the lone cropped coral block in row-reverse wrap. No visible displaced edge bands.

Members (all directly inspected):

- `public-row-wrap-dyadic`, frame 2.
- `public-row-wrap-reverse-dyadic`, frame 2.
- `public-row-reverse-wrap-dyadic`, frame 2.
- `public-row-reverse-wrap-reverse-dyadic`, frame 2.

## Sheet 11

SHA-256: `d3de2614bc119308cc232cc95f661e797655c2cda74781d6bd4e4b242c43adb5`.

Dyadic column layouts agree at the bottom crop. The narrow coral strips in reverse-column cases, yellow block widths, and blue/green placements match. No visible opaque paint appears in the transparent areas.

Members (all directly inspected):

- `public-column-wrap-dyadic`, frame 2.
- `public-column-wrap-reverse-dyadic`, frame 2.
- `public-column-reverse-wrap-dyadic`, frame 2.
- `public-column-reverse-wrap-reverse-dyadic`, frame 2.

## Limits

This review covers these 24 representatives only. It does not independently review sheets 00–05 or each clone/repeated frame; any exact repeat transfer must be justified by the capture coverage evidence. Transparent leaf correctness here means the visible background remains clear at its region; pixels alone cannot prove that an invisible owner exists or is correctly mapped.

The fixed-size roots preserve their layout while the viewport clips them. These images do not qualify responsive wrapping, arbitrary HTML/CSS, or all viewport coordinates. The earlier private clip observer did not support this public graph shape; no independent native command-stream verification is claimed here. No compiler edits, new render, or runtime changes were performed for this review.
