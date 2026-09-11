# A20 font underline metrics — Chrome qualification

Chrome 153.0.8010.12 is the reference. No Firefox gate was used.

## Change and accepted behavior

`from-font` preserves a present signed `post.underlineThickness` metric and
clamps the resolved thickness to at least 1 CSS pixel. Previously zero and
negative metrics incorrectly selected the auto fallback (2.4px at font-size
24px). A negative metric is not converted to its absolute value. `auto` remains
font-size/10 with the minimum clamp. A truly absent metric still selects the
auto fallback in code, but that branch is not yet browser-qualified.

The WASM success envelope now serializes typed fields directly. An intermediate
JSON Value widened f32 values, yielding 2.4000000953674316 in WASM metadata
against native JSON's 2.4. The regression checks exact numeric representation
and complete requirements equality; no tolerance was introduced.

## Reproduction and evidence

Run from the repository root after building the compiler, native-glyph probe,
Metal renderer-replay and WASM via the existing validation build commands:

```sh
NUXIE_HTML_REVIEW_DIR=output/playwright/html-to-riv/underline-font-metrics-fixed node tools/html-to-riv/validation/underline-font-metrics-control.mjs
cargo test -p nuxie-html-to-riv --features native-glyph-controls
node --test tools/html-to-riv/tests/javascript.test.mjs
```

The control derives checksum-valid Japanese fixture font variants with signed
metrics -200, -20, 0, 1 and 50 at UPM 1000. Each variant is exercised with auto
and from-font at 24px. Each of ten source scenes is compiled once at 390px and
the same Rive bytes are imported/resized/rendered at 240, 390 and 768px. Chrome
loads the exact font bytes; the harness asserts loaded status. Transparent glyphs
and red underlines with skip-ink:none isolate thickness and placement.

- 30/30 Chrome geometry and real native Metal pixel comparisons pass using
  unchanged shared thresholds. This is tolerance-based equivalence, not byte
  identity: fractional stripe antialiasing differences remain visible in diffs.
- All ten native/WASM outputs have equal Rive bytes, source maps and requirements.
- 90 public module tests pass, including eight underline tests.
- Six JavaScript tests pass, including accepted-corpus native/WASM parity and
  the fractional metric regression. The rebuilt WASM compiles successfully.
- Browser/native/diff sheets review-0.png through review-4.png were individually
  inspected: matching stripe placement and extent, thin minimum-clamped lines,
  thicker auto lines, no unintended glyph ink. Fractional edge rasterization
  differs slightly, within the existing pixel gate.

Raw requests, compiled scenes, manifests, native streams, bounds, PNGs, diffs and
results.json are in `output/playwright/html-to-riv/underline-font-metrics-fixed/`.
The earlier zero/-20 failures remain in `output/playwright/html-to-riv/underline-font-metrics/`.
The control is included in validation/run.sh's native-glyph section; this receipt
does not claim a fresh full run.sh pass.

Logs: /tmp/html-underline-metrics-module.log,
/tmp/html-underline-metrics-parity.log,
/tmp/html-underline-font-metrics-fixed.log and
/tmp/html-underline-metrics-wasm-fixed.log. Normal module all-target Clippy completes (dependency warnings remain), logged
in /tmp/html-underline-metrics-module-clippy.log. Strict all-target Clippy with
-D warnings stops in dependency code including nuxie-schema, nuxie-image-codec,
nuxie-script-signature and nuxie-ore-metal; see
/tmp/html-underline-metrics-final-clippy.log.

## Remaining work

A20 remains active. This isolated control does not qualify absent font metrics,
additional backends, DPR, affine transforms, group opacity, performance, or
existing vector-rendering failures. See precision-policy-review.md and
underline-transform-diagnosis.md for those separate results and reproducers.


## Absent post-table investigation

The optional `--absent-only` control removes the post directory entry, updates
SFNT search fields, and repairs the head/global checksums. The public Rust test
asserts that ttf-parser sees no underline metric (distinct from Some(0)), then
checks both auto and from-font produce the 2.4px fallback at font-size 24px.
All eight underline tests pass after this extension; log:
/tmp/html-underline-absent-test.log.

Chrome 153.0.8010.12 rejects this font before rendering with the exact diagnostic
`OTS parsing error: post: missing required table`. Consequently there is no
valid same-font browser fallback comparison for this sample. This is not a
passing visual case. Native compilation, native/WASM bytes/maps/requirements
for the auto sample and native rendering completed before Chrome rejected it;
the browser rejection stops the control before its from-font iteration.

The request embeds the exact derived font and is preserved with the native
artifacts and font-rejection.json under
`output/playwright/html-to-riv/underline-font-metrics-absent/`.
Reproduce with:

```sh
NUXIE_HTML_REVIEW_DIR=output/playwright/html-to-riv/underline-font-metrics-absent node tools/html-to-riv/validation/underline-font-metrics-control.mjs --absent-only
```

Expected outcome is a nonzero exit and recorded OTS rejection, not browser
substitution with a different font. Missing-metric browser qualification remains
unproven; this sample requires a browser-accepted font with unavailable metrics.
It does not block independent backend/DPR/performance or later authoring work.
