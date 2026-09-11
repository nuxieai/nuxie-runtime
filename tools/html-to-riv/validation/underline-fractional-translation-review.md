# Fractional CSS translation and underlines

Status: native targeted fix and full native regression qualified except the
unchanged A09 failure. Vector
qualification remains incomplete.

The light/dark custom-property compositions exposed local RGB failures in a
one-pixel underline at240px. Browser note origin184.375px, native184.399994px;
Chrome painted one solid row199 while native spread coverage across199/200.

The original failures reproduced. A two-node reduction with184.4px top padding
fails at240; the otherwise identical184px control passes. Both are permanent
cases in validation/cases.json. Reduced command (about3s):

```sh
NUXIE_NATIVE_GLYPHS=1 npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'underline-(fractional-origin|integer-origin-control)'
```

Hypotheses were paint-space snapping, skip-ink clipping, and baseline rounding.
The runtime created underline paths before world translation, whereas strikes
already snapped at paint time. Snapping unit-scale axis-aligned underlines after
CSS translation fixes the original and reduced cases without changing shaping,
layout, skip-ink exclusions, authored source or tolerances. Original bounds remain
available for affine/scaled world transforms, which are not newly qualified.
The full and geometric fallback paths use the same adjusted stripe bounds.

Evidence:
- Original reproduction: output/playwright/html-to-riv/underline-fractional-repro/.
- Minimal before: output/playwright/html-to-riv/underline-fractional-minimal/.
- Fixed original/reduced/control: output/playwright/html-to-riv/underline-fractional-fixed/,12/12 native pass; sheets inspected.
- Compiler184/184, runtime decoration8/8 with pinned Inter, native/WASM publish parity pass.
- Vector focused run: output/playwright/html-to-riv/underline-fractional-vector/,3/12 pass. All four sheets inspected; text rasterization differences remain, including integer-origin control. This run has no pre-fix vector baseline and does not establish regression attribution.
- Full native run: output/playwright/html-to-riv/underline-fractional-full/:1,470/1,471 pass, existing A09 only. All1,461 scene pairs match explicitly reviewed baselines (see baseline-comparison.json).
- Precise row regression: validation/underline-row-control.mjs checks all6 reduced/control renders. Saved pre-fix images fail; fixed and full images pass. This catches blur even where average tolerances passed.
- DPR1/2/3 × skip-ink auto/all/none × widths240/390/768:27/27 pass. All54 images and request sources identical to reviewed underline-dpr-callback baseline. Evidence: output/playwright/html-to-riv/underline-fractional-dpr/.

Earlier A09 fractional-edge failure, broader transform/DPR/backend work and
remaining compiler backlog are not closed by this focused fix.
