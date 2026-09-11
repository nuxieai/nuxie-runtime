# First flex item extension: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The full run passed 110 browser/native tests, 17 Rust tests, five JavaScript/WASM
integration tests, two gallery checks and the TypeScript check. Native Clippy
also passed with warnings denied for the module. The same accepted corpus was
compiled through the native and WASM clients and compared byte-for-byte.

Visual review inspected Chromium and Nuxie screenshots for every fixture below
at **240, 390 and 768 CSS px**, using contact sheets rendered at one image pixel
per CSS pixel. Reference: Chromium 153.0.8010.12, DPR 1. Native: Rust Metal.

* `flex-weighted-grow-shrink`
* `flex-column-basis-and-limits`
* `flex-auto-basis-from-width`
* `flex-percentage-basis`
* `flex-inflexible-explicit-basis`
* `flex-wrapping-limits`
* `flex-text-shrink-reflow`
* `flex-shorthand-and-cascade`
* `flex-image-participation`

The inspected output agrees in item distribution, spacing, wrap transitions,
min/max clamping, text line breaks and image stretching. Independent text/edge
rasterization remains covered by the existing bounded pixel tolerances; no
tolerance was increased for this extension. Each generated scene is resized
after import; layouts are not separately baked at each width.

Three exploratory failures remain in `deferred-cases.json`: sub-unit factors
with gaps, content-derived auto basis, and indefinite percentage basis with
overflow painting. All must currently produce compiler diagnostics. In addition,
unequal grow/shrink factors cannot be expressed by the existing Rive shared
weight and are rejected. See SUPPORT.md for the exact accepted subset.

This receipt records this increment's evidence. Future compiler/runtime changes
must rerun the gate; it is not a permanent approval of arbitrary Flexbox input.
