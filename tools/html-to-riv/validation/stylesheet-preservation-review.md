# Stylesheet preservation: full native regression

The browser harness now scopes CSS by mutating selectorText on its original
constructed stylesheet and adopting it. This avoids losing pending var()
shorthand components through CSSOM declaration serialization. Direct control:
custom-property-shorthand-control.mjs; its original/serialized/preserved receipt
is in output/playwright/html-to-riv/custom-properties-flex/.

Full native glyph run: 1,341 passed, 1 failed (1,332 scene/width comparisons plus
10 pixel-comparator tests). Chrome153.0.8010.12, real rust-metal renderer,
240/390/768 widths with scenes compiled at390. No tolerance changes.

All1,332 scene comparisons have identical HTML/CSS and byte-identical browser
and native PNGs versus explicit prior reviewed runs. compare-review-images.py
records the chosen baseline and SHA256 values for every case. Baseline priority
includes the old line-box full run, later OpenSans fixes, public ellipsis,
selectors and custom-property reviews. This proves no image regression in this
corpus, not qualification of every backend, DPR or unresolved feature.

The sole failure is em-layout-cascade at240. Visually re-inspected browser,
native and diff images: exact geometry (23.75px blue-box origin and97.5px orange
origin), differing fractional-edge raster coverage.449 pixels differ, ratio
0.0058463542 above the unchanged0.005 limit. It is byte-identical to the previous
failure. A09 remains partial; this is not a passing suite.

Evidence: output/playwright/html-to-riv/stylesheet-preserved-full/ contains
review.json, gallery.html, tests.log, inputs.json, inputs-unchanged.json and
baseline-comparison.json. Inputs verify the compiler, probe, renderer and
fixture/harness files remained unchanged during the run. Source and image
identity are reported separately from pass/fail status.
