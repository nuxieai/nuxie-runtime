# S04 first, last and only-child selectors

Status: qualified within the standalone HTML profile.

The selector tokenizer accepts only these three nonfunctional pseudo-classes,
counting each toward class specificity. The existing selector engine uses source
DOM element siblings. All tags and hidden elements count; comments/whitespace do
not. No runtime capability or browser-baked positioning is introduced.

Three public compiler tests cover tag-independent counting, comments/whitespace,
hidden first/last siblings, only-child versus multiple children, combined first
and last, ASCII case and CSS escaping, specificity/source order/important,
sibling combinations and malformed or unsupported pseudo syntax.
Full compiler123/123 and native/WASM builds pass.

Five permanent fixtures pass15/15 Chrome geometry and actual native pixels at
240/390/768, resizing scenes compiled at390. All five sheets were visually
inspected: first/last colors, the nonmatch behind a hidden first sibling, only
child height, chained sibling styling and responsive card widths agree.
No tolerances changed. Gallery:
`output/playwright/html-to-riv/structural-selectors/gallery.html`.

Reproduce with the standard module tests, native/WASM builds and accepted-corpus
JavaScript parity test. Pixel command:

```sh
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/structural-selectors" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'structural-selector-' --output tools/html-to-riv/test-results-structural-selectors
```

Logs: `/tmp/html-structural-module.log`, `/tmp/html-structural-parity.log`,
`/tmp/html-structural-pixels.log`. Function forms, other pseudo-classes,
interactions and the existing broader language exclusions remain unsupported.

The complete accepted native/WASM publish-artifact parity test passes.
