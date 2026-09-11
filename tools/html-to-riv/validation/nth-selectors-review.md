# S05 nth child selectors

Status: qualified within the standalone HTML profile and documented resource limits.

Accepted syntax and exclusions are in SUPPORT.md under S05: An+B, integers,
odd/even, forward/reverse source element counting, optional filtered selector
lists and nested nth forms. Filter specificity uses its highest branch even
when unmatched. The selectors engine now has a compiler-owned parser adapter
that enables nth `of` lists; the restricted tokenizer still rejects unsupported
grammar before matching. No runtime capability or baked layout is introduced.

Four public compiler tests cover 14 formulas in both directions, filtered and
nested counting, combinators inside filters, unmatched maximum specificity,
malformed syntax, coefficient bounds and excessive nesting. An initial test
comparison accidentally gave its explicit expected selector lower specificity
than the base rule; that test setup was corrected. Deep error propagation also
revealed exponential diagnostic formatting: custom errors are now propagated
directly, with a regression assertion that nesting rejection stays concise.
The original test logs remain in /tmp/html-nth-tests*.log.

Validation completed:

- Full compiler: 127 passed, zero failures.
- Native compiler/probe and compiler WASM builds pass.
- Full accepted scene corpus has identical native/WASM publish artifacts.
- Six permanent nth-selector fixtures: 18/18 Chrome geometry and actual native
  pixel comparisons, compiling at 390 and resizing the same scene to 240/390/768.
- All six browser/native/diff sheets visually inspected: alternating selection,
  negative/reverse ranges, filtered counting, unmatched-filter specificity, hidden
  siblings and percentage-width rounded stacks agree. Small edge rasterization
  differences remain within unchanged thresholds; this is not pixel identity.

Gallery: `output/playwright/html-to-riv/nth-selectors/gallery.html`.
Durable command logs are in that directory as module.log, native-build.log,
wasm-build.log, parity.log and pixels.log. Pixel attachments and numeric receipts
are retained in `tools/html-to-riv/test-results-nth-selectors`.

Reproduce using the standard compiler tests, native/WASM builds and accepted
corpus JavaScript parity test, followed by:

```sh
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/nth-selectors" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'nth-selector-' --output tools/html-to-riv/test-results-nth-selectors
node tools/html-to-riv/validation/composition-review.mjs tools/html-to-riv/test-results-nth-selectors output/playwright/html-to-riv/nth-selectors
```

Broader renderer limitations and unqualified backlog items remain unchanged.
This targeted pixel run does not claim a new full-corpus visual qualification.
