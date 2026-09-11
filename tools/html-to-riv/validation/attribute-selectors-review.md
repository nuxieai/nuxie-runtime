# S01 attribute selectors

Status: accepted subset validated; explicit s modifier deferred to Chrome support.
Accepted syntax and intentional exclusions are in SUPPORT.md, “Attribute selectors”.

The compiler now tokenizes selector lists independently of declaration values.
Square-bracket contents are validated and preserved, commas inside strings do
not split selectors, and each attribute contributes one class specificity unit.
The existing scraper selector matcher applies operator/case semantics. Inert
ASCII data-* metadata is retained in the authoring DOM without runtime behavior.
No new runtime capability or browser-derived layout is introduced.

Three public compiler tests cover positive and negative matches for all operators,
missing/empty values, Unicode versus ASCII i-folding, escapes, brackets/commas in
values, combined selectors, specificity/source-order/important rules, and malformed
or deferred syntax. Full module115/115 passes. Native/WASM builds and the complete
accepted scene corpus artifact-parity test pass.

Five permanent fixtures pass15/15 at240/390/768, each resizing the scene compiled
at390. All five comparison sheets were visually inspected: expected tile colors,
empty-value nonmatches, percentage sizing, cascade precedence and responsive card
geometry agree. No tolerances changed. Output:
`output/playwright/html-to-riv/attribute-selectors-fixed/gallery.html`.

The first run passed12/15; the case/escape fixture failed because the browser
harness split a quoted comma during stylesheet scoping. The corrected harness
tracks strings, escapes and bracket/parenthesis depth, preserving per-selector
specificity. Initial artifacts remain in `output/playwright/html-to-riv/attribute-selectors`;
all three initial failing comparisons were inspected. The failures were harness
corruption, not a selector-matching discrepancy.

Pinned Chrome153.0.8010.12 accepts i but rejects s in CSS.supports(selector(...)).
Receipt: `attribute-selectors-fixed/case-flag-support.json`. The first local
implementation accepted s via the underlying matcher; final compiler intentionally
rejects it rather than claiming Chrome qualification. S01 remains partial for
this exact external reference dependency. Sibling/pseudo selectors are separate
backlog items.

Reproduce:

```sh
cargo test -p nuxie-html-to-riv --features native-glyph-controls
cargo build -p nuxie-html-to-riv --features native-glyph-controls --bin html-to-riv --example probe
bash tools/html-to-riv/validation/build-wasm.sh
node --test --test-name-pattern='accepted scene corpus' tools/html-to-riv/tests/javascript.test.mjs
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/attribute-selectors-fixed" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'attribute-selector-' --output tools/html-to-riv/test-results-attribute-selectors-fixed
```

Logs: `/tmp/html-attribute-selector-module-final.log`,
`/tmp/html-attribute-selector-parity-final.log`,
`/tmp/html-attribute-selector-pixels-final.log`. Initial test setup lacked viewport
sizes and was corrected before final testing; malformed-input tests also run
with valid viewports, so they exercise selector/attribute rejection directly.
