# S02 adjacent sibling selectors

Status: qualified within the standalone HTML authoring profile.

The selector tokenizer now admits + as a combinator; the existing selector engine
matches source-DOM element siblings. It contributes no specificity and introduces
no runtime capability. Accepted syntax and exclusions are in SUPPORT.md.

Three public tests compare compiled output against explicit target styles. They
cover whitespace/comment siblings, hidden intervening elements, different parents,
attribute/child chains, source-order/specificity/important behavior, and malformed
combinators. General siblings and pseudo-classes remain rejected. Full compiler
suite118/118 passes, including all accepted corpus imports.

Four permanent fixtures pass12/12 Chrome geometry and actual native-renderer
pixel comparisons at240/390/768, resizing the scene compiled at390. All four
comparison sheets were visually inspected: green/blue targets agree, the hidden
sibling affects selection while painting nothing, and card spacing/percentage
widths remain responsive. No thresholds changed. Native/WASM compiler builds
and complete accepted-scene artifact parity pass.

Gallery: `output/playwright/html-to-riv/adjacent-selectors/gallery.html`.
Logs: `/tmp/html-adjacent-module-final.log`, `/tmp/html-adjacent-parity.log`,
`/tmp/html-adjacent-pixels.log`.

Reproduce with the normal compiler tests/native/WASM builds and accepted-corpus
parity test. Pixel command:

```sh
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/adjacent-selectors" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'adjacent-selector-' --output tools/html-to-riv/test-results-adjacent-selectors
```

This is source-DOM styling, not runtime interaction. CSS Grid, scripting, editor
integration and the other documented exclusions remain outside this work.
