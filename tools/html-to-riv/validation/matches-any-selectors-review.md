# S07 is and S08 where selectors

Status: qualified within the documented standalone selector profile.

The shared selector-list parser supports is and where alongside not, preserving
strict lists for not and nth filters. Matching delegates to the selector engine.
Within the supported grammar, invalid branches in is/where are discarded before
specificity calculation. is uses maximum surviving argument specificity; where
uses zero. Unknown/profile-excluded tokens and resource violations remain
diagnostic errors. SUPPORT.md defines this intentional boundary explicitly.

Six public tests cover simple and complex arguments, nested is/where/not/nth,
CSS escapes, unmatched maximum-specificity branches, source order, universal
arguments, zero specificity, forgiving empty/invalid branches, discarded-ID
specificity, all-invalid lists, strict unsupported syntax and resource bounds.
The existing not rejection test now uses has; an old attribute-selector test
also needed its is rejection replaced by has. The first visual fixtures had an
unclosed source container, caught by the full accepted-corpus import test and
corrected before rendering. Original failed logs are retained.

Eight permanent fixtures cover is lists/complex arguments/specificity/forgiving
behavior, where zero specificity/nesting/forgiving behavior, and a responsive
plan-card composition mixing the selector functions.

Reproduce with standard module tests, native/WASM builds, accepted-corpus
JavaScript parity, then:

```sh
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/matches-any-selectors" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'matches-any-' --output tools/html-to-riv/test-results-matches-any-selectors
node tools/html-to-riv/validation/composition-review.mjs tools/html-to-riv/test-results-matches-any-selectors output/playwright/html-to-riv/matches-any-selectors
```

Validation completed: full compiler 136/136, native/WASM builds and full accepted
corpus publish-artifact parity pass. Eight fixtures pass 24/24 Chrome geometry
and actual native pixels at 240/390/768, resizing scenes compiled at 390.
All eight browser/native/diff sheets visually inspected: selected colors,
heights, rounded shapes, nested filtering and responsive card widths agree.
Minor edge rasterization differences remain within unchanged thresholds.
This is neither pixel identity nor a full-corpus visual requalification.

Gallery and durable logs: `output/playwright/html-to-riv/matches-any-selectors/`
with gallery.html, module.log, native-build.log, wasm-build.log, parity.log and
pixels.log. Earlier failures are retained as old-rejection-test.log and
fixture-unclosed.log. Numeric receipts and PNG attachments remain in
`tools/html-to-riv/test-results-matches-any-selectors/`.
