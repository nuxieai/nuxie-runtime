# S03 general sibling selectors

Status: validated within the standalone HTML authoring profile.

The selector tokenizer admits ~; the existing selector matcher handles preceding
source-DOM element siblings. No runtime capability or browser-baked layout is
introduced. Accepted semantics and exclusions are in SUPPORT.md.

Public compiler tests cover later-only matching, hidden left operands, intervening
elements/comments/whitespace, parent boundaries, attribute and child chains,
combinations with +, specificity/source order/important, and malformed syntax.
Four permanent fixtures compare nonadjacent siblings, hidden operands, chained
styles and responsive stacked cards. All12 comparisons at240/390/768 pass;
each scene is compiled at390 and then resized. All four comparison sheets were
visually inspected: matching colors, percentage widths, gaps and radii agree.
No tolerances changed.

Full compiler120/120 and native/WASM builds pass. Full accepted scene artifact
parity is checked with the usual JavaScript corpus test. Gallery:
`output/playwright/html-to-riv/general-siblings/gallery.html`.

Reproduce:

```sh
cargo test -p nuxie-html-to-riv --features native-glyph-controls
cargo build -p nuxie-html-to-riv --features native-glyph-controls --bin html-to-riv --example probe
bash tools/html-to-riv/validation/build-wasm.sh
node --test --test-name-pattern='accepted scene corpus' tools/html-to-riv/tests/javascript.test.mjs
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/general-siblings" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'general-sibling-' --output tools/html-to-riv/test-results-general-siblings
```

Logs: `/tmp/html-general-sibling-module.log`, `/tmp/html-general-sibling-parity.log`,
`/tmp/html-general-sibling-pixels.log`. CSS Grid, scripting, interactions and editor
integration remain excluded. Pseudo-classes are subsequent backlog items.

The full accepted native/WASM publish-artifact parity test passes.
