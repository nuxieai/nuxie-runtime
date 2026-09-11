# Compiler-owned derivative font outlines: candidate only

2026-09-11. Read-only investigation against the immutable baseline. No derivative font, implementation, build or new rendering result is claimed. The reported double-Fill result passes selected 32/48/64px cases but fails 12/16/24px; this does not identify the cause as hinting.

## What the baseline actually does

`crates/nuxie-runtime/src/mechanical_port/source/text/font_hb.rs:28` fixes `STANDARD_SCALE=2048`. `HbFont::get_path` (309–315) supplies font variation location and glyph ID, without the eventual CSS font size. `glyph_path` (540–555) calls `DrawSettings::unhinted(Size::new(2048), location)` with HarfBuzz path style. Existing embedded hint programs therefore do not grid-fit these ordinary outline paths. Merely adding or editing hint instructions in the embedded font cannot activate hinting in this path.

Shaping is separate: lines 475–534 shape at scale 2048, then multiply advances and offsets by `text_run.size/2048`. Font line metrics (409–428) likewise use rounded 2048-scale metrics. A successful outline modification need not bake glyph positions or replace ordinary Text reflow, but preserving advances requires explicit verification of this separate shaping path.

Read-only SFNT inspection of the actual fixtures found `fpgm`, `prep`, `cvt ` and `gasp` tables in both fonts. `roboto-a.ttf` has one simple glyph, with nonzero instructions. `roboto-regular.ttf` has 567 simple glyphs, all with nonzero instructions. Composite glyph instructions were not counted. Thus existing hint data is present and the baseline path ignores it; **the current evidence does not establish that pinned Chrome applies those instructions, or that doing so removes the observed pixel errors**. Coverage/gamma/stem darkening, subpixel phase, metric rounding and outline differences remain competing explanations.

## Feasible experiment within the boundary

A compiler could evaluate a font's hinted vector outline at one chosen effective pixels-per-em, convert those points back into font units, write an ordinary derived font, and embed that FontAsset. The baseline would then extract the modified contours unhinted. This preserves ordinary Text objects and responsive line breaking if character/glyph mappings and shaping data remain valid; the compiler does not choose line breaks or learn contours from Chrome screenshots.

Skrifa exposes a hinting instance configured by size, variation location and target, and its outline draw API can return adjusted metrics. Those APIs provide a candidate compiler-side mechanism, not evidence that a particular hinting target matches Chrome/macOS. Pin any newly selected compiler dependency independently of the immutable workspace. [Skrifa HintingInstance](https://docs.rs/skrifa/latest/skrifa/outline/struct.HintingInstance.html), [OutlineGlyph::draw](https://docs.rs/skrifa/latest/skrifa/struct.OutlineGlyph.html).

Constraints:

- One derivative is keyed by font hash, selected face/axes, target effective size and hinting mode. CSS size alone does not cover device scale, transforms, zoom or animated font size. Ordinary viewport reflow at unchanged size is a narrower viable target; fractional baseline/placement still needs testing.
- Preserve glyph IDs, cmap, GSUB/GPOS, advances and line metrics initially. Hinting may adjust phantom-point metrics; adopting those adjustments could change shaping and break lines. Compare metrics rather than assuming unchanged tables imply identical advances. Composite outlines also require deliberate handling. [OpenType glyf](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf).
- Re-encoding contours into integer font units introduces quantization. Validate round-trip baseline contours; changing unitsPerEm to gain precision entails rescaling all dependent tables, not just glyf.
- Regenerate valid glyf/loca/head/maxp bounds and checksums; ensure hmtx/bearings remain coherent. Remove or replace obsolete instructions in derived contours rather than leaving programs tied to old point numbering. Variable glyph deltas, composites, color glyphs and CFF are separate scope, not implicit support.
- Preserve the font's license and satisfy modification/naming conditions. Retain original and derived hashes plus deterministic generation settings. Do not mutate the source fixture.
- Vector outline preprocessing does not control the immutable renderer's coverage transfer. Even geometrically closer contours may still fail pixels. Double Fill must be an independent experimental factor, not silently combined into every derivative.

## Bounded validation before admission

Start with licensed static Roboto and full glyph IDs retained. Generate identity-roundtrip, unhinted-roundtrip and one hinted candidate at 12/16/24px; retain the original font control. Check existing glyph advances, kerning/ligature pairs, line metrics and ordinary original/clone reflow at widths straddling a wrap boundary. Include Agjp, AV/fi, thin stems, curves and counters, then integer and fractional x/y positions. Render each candidate with single Fill first and separately with the retained double-Fill treatment.

Compare against the unchanged pinned Chrome font/reference, existing local and aggregate gates, and full-image review. No Chrome-derived per-glyph contour fitting or per-string positioning. A narrower passing fixed-size vector-font profile would be useful evidence; a failing candidate only rejects that transformation/settings combination. It does not establish impossibility or justify runtime changes.
