# Ordinary derivative font experiment

This script is experimental asset preprocessing, not a public compiler feature or a browser-fidelity qualification. It modifies vector glyph outlines in a static TrueType font and produces an ordinary font asset; it neither invokes Chrome nor calculates text placement or line breaks. Root/runtime dependencies are unchanged.

The initial experiment uses full licensed Roboto and 16px only. FreeType's native normal target is a candidate, **not a claim that Chrome uses equivalent hinting**. Hinting is evaluated at integral pixels-per-em; coordinates are converted from 26.6 pixel units into source font units using fontTools `otRound`. This quantizes coordinates and does not support arbitrary effective size, device scale or transform.

Reproduce from the repository root in a new output directory:

```sh
python3 -m venv tools/html-to-riv/output/ordinary-font-derivative-r1/venv
tools/html-to-riv/output/ordinary-font-derivative-r1/venv/bin/pip install fonttools==4.59.1 freetype-py==2.5.1
tools/html-to-riv/output/ordinary-font-derivative-r1/venv/bin/python tools/html-to-riv/validation/derive-font-outlines.py tools/html-to-riv/fixtures/fonts/roboto-regular.ttf NEW_OUTPUT --sizes 16
```

Recorded artifacts are under `output/ordinary-font-derivative-r1`. `fonts/manifest.json` binds source, generator, settings, output hashes and invariants. `dependencies.json` also hashes the loaded bundled FreeType library (version 2.13.2). Source font license/provenance remain in `fixtures/fonts/`; generated experimental fonts retain license records and have suffixed family/full/PostScript/unique names. Identity output keeps original names.

## Controls and invariants

- `identity.ttf` is byte-identical to the original source font, not merely equivalent outlines.
- `unhinted-16.ttf` uses no-scale/no-hinting FreeType outline extraction. The size in its name is experiment bookkeeping; its contour extraction is size independent.
- `hinted-native-normal-16.ttf` uses native hint programs, `FT_LOAD_TARGET_NORMAL`, no auto-hinter and no bitmap extraction. Default FreeType interpreter properties apply.
- Every output is generated twice and required to be byte-identical. All 1294 glyph IDs remain; cmap, GSUB, GPOS, GDEF, hmtx and OS/2 tables are byte-identical. hhea ascent, descent, lineGap and advanceWidthMax and head unitsPerEm remain unchanged.
- Derived glyf outlines flatten composites. Existing per-glyph instructions and global fpgm/prep/cvt programs are removed. Glyph/head/maxp bounds and hhea geometric extrema are recalculated; hmtx advances and bearings are deliberately retained. Names and checksums change.
- The script rejects variable, CFF/CFF2 and color/SVG fonts. It is not a general font sanitizer or complete resource-boundary implementation.

## Observed transformation limitations

The identity changes zero glyph outlines. The hinted candidate changes 1275. The unhinted control changes six relative to fontTools source decomposition; `unhinted-differences.json` preserves the inspection:

Five transformed composites (en/em dash, less/greater-or-equal, horizontal bar) differ by less than 0.8 font units due to decomposition/rounding. Greek chi shifts 49 units: its source glyf xMin is 90 while hmtx left side bearing is 41, and FreeType normalizes the outline origin. The Agjp/AV/fi sample contours are unchanged in the unhinted control. These findings prevent claiming general geometry preservation from this control and require investigation before any broader admission. Native shaping must still be measured; preserved shaping tables alone are insufficient proof of runtime behavior.

The initial failed generator attempt recalculated hhea while asserting whole-table identity. The successful check explicitly preserves line/advance metrics while allowing required geometric extrema recalculation. `failed-initial-fonts` retains the partial identity artifact from that attempt; it is not a completed experiment or manifest.

Compare derived-font RIV scenes against **the original Roboto font in Chrome**, with single and double ordinary Fills as separate factors, using unchanged pixel tolerances. Do not overwrite these artifacts when generating a new candidate. No rendering result is claimed by this document.
