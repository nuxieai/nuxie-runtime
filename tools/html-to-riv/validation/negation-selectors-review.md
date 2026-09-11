# S06 negation selectors

Status: qualified within the documented standalone HTML profile.

The restricted selector tokenizer accepts `:not()` with a strict list of
supported complex selectors, including nested negation and nth filters.
Specificity is the lexicographic maximum of argument specificities, with no
additional class unit. Actual matching uses the existing selectors engine.
Limits and intentional exclusions are documented in SUPPORT.md under S06.

Three public tests cover matching/nonmatching lists, universal arguments,
complex combinators, nested negation, nth filters in both directions of
composition, attributes, escaped/case-insensitive names, maximum specificity,
zero specificity, source order and invalid/unsupported branches. Excessive
nesting must fail with a concise diagnostic.

Six permanent cases cover lists, complex selectors, nested filters, specificity,
hidden source siblings, and a responsive plan-card composition using the
supported shape vocabulary. Early corpus validation caught an unclosed fixture
container and unsupported border shorthand in the new composition. Both fixture
errors were corrected; the original logs remain /tmp/html-not-module.log and
/tmp/html-not-module-fixed.log. Neither was a renderer failure.

Reproduce with standard module tests, native/WASM builds and full accepted-corpus
JavaScript publish parity, followed by:

```sh
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/negation-selectors" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'negation-selector-' --output tools/html-to-riv/test-results-negation-selectors
node tools/html-to-riv/validation/composition-review.mjs tools/html-to-riv/test-results-negation-selectors output/playwright/html-to-riv/negation-selectors
```

Validation: full compiler 130/130, native and WASM builds, and full accepted
native/WASM publish-artifact corpus parity pass. All 18 Chrome geometry/native
pixel checks pass at 240/390/768, resizing scenes compiled at 390. All six
comparison sheets were visually inspected: matching colors, selected heights,
rounded shapes, hidden-sibling positions and card composition widths agree.
Small edge rasterization differences remain within unchanged tolerances; this
is not a pixel-identity claim or a new full-corpus visual qualification.

Gallery and durable command logs: `output/playwright/html-to-riv/negation-selectors/`
(`gallery.html`, `module.log`, `native-build.log`, `wasm-build.log`, `parity.log`,
`pixels.log`). Initial fixture failures are preserved there as
`fixture-unclosed.log` and `fixture-border.log`. Numeric receipts and original
attachments: `tools/html-to-riv/test-results-negation-selectors/`.
