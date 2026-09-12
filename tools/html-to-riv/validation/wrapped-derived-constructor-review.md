# Private Derived constructor review

This is a validation-only construction audit, not public HTML/CSS compiler admission. The reviewed constructor is `output/wrapped-derived-constructor-r2/candidate`, SHA-256 `3ecbb860ce3ad1ed1f1c0f8a734fc5b538b586c4fda8bd95a06d2fd386a06ee6`.

The frozen bridge invokes the actual `wrapping_domains::resolve` and `wrapping_composition::compose_with_bounds` with both viewport domains `[0,16384]`. It serializes `candidate.records()` directly. There are no post-proof record mutations, browser-derived geometry, or runtime policy setters. The optional ordinary white Artboard paint is part of the checked base before composition. Diagnostic proof/map/trace files are observation artifacts, not runtime requirements.

Verified all 83 source/snapshot bindings, all 52 effective crate inputs, and 48 accepted deterministic constructions with five artifacts each. Each copied artifact matches its first and repeat construction. Schema/JPEG sources and the root manifest have no changes from immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`. Effective copied dependency files match their snapshots; the schema manifest only expands identical inherited edition/license/repository values and establishes its copied workspace. Compiler manifest changes relocate those identical dependencies and add the validation binary. This does not qualify the separate renderer build or public compiler interface.

The authored Chrome cases and recipes correspond in dimensions, percentage bases, explicit min/max constraints, colors, parent axis/reversal and visible alignment. Chrome wrappers use logical align-self plus physical child alignment; the resulting visible cross position is the line origin plus the physical alignment fraction times `(line size - visible size)`. Native wrappers can retain uniform alignment while the checked ordinary constraint composition produces that same visible target. Only the parent and visible children are mapped; hidden wrapper geometry is not claimed equivalent. Main-axis visible overflow is intentional.

## Corrected paint-order hypothesis

An initial review incorrectly predicted a reverse-main paint-order failure from a general DOM-order assumption. The actual pinned Chrome captures contradict that prediction: in `derived-row-reverse1-wrap1-level0-equal`, frame 0, the yellow rectangle paints over the orange overlap in Chrome and native. The hypothesis is withdrawn; it is not a compiler defect finding.

Pinned Chromium **153.0.8010.12** source explains the result for these ordinary, unpositioned boxes. [`flex_layout_algorithm.cc`](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/layout/flex/flex_layout_algorithm.cc) applies reversals before final placement at line 1299. Lines 1621–1630 reverse lines for wrap-reverse and item indices for reverse-direction. Lines 1824/1874 traverse those orders and line 2030 adds the resulting fragments. [`box_fragment_painter.cc`](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/paint/box_fragment_painter.cc), lines 1160–1192, paints ordinary child fragments in that traversal order. This supports the physical-order composition for the pinned Chrome cases; it is not a general claim about every CSS stacking context or other browsers.

Decoded source SHA-256, independently fetched from the version tag:

- Flex layout: `e99678c91c38568d0d430707ed3ba47efafc352bbdd13be8aa170283041706f1`
- Box painter: `ade9646e2689f23a2c00c91e1df3efbcf377a8488feb244b0edbbeec008aa224`

## Direct visual review

`output/playwright/wrapped-derived-r2/visual/inspection-agent-a.json` binds direct inspection of sheets 00–11 to `coverage.json`: 48 representative rows, covering row-axis one-line and two-line sizes across reversal and alignment combinations. The shapes, line placements and visible overlap order match by eye. Faint filled differences and thin horizontal responsive-visible boundary differences remain visible in the difference panels. All 48 inspected representatives retain their existing passing metric labels; that does not claim pixel identity.

No direct review of sheets 12–35 or repeat/clone frames is claimed here. The complete campaign's **48 pixel-failure frames remain failures**. This inspection neither widens tolerances nor qualifies the whole campaign or public module.
