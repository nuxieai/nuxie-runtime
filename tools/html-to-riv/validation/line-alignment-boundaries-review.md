# Positional line alignment with overflowing and automatic parent cross sizes

All **192 geometry and pixel comparisons pass**, with **384 passing clear controls**, for 24 ordinary-file wrapper fixtures. All frames have visual coverage: 59 distinct Chrome/native pairs were directly inspected, and 133 remaining pairs are exact full-image decoded RGBA matches to reviewed counterparts. This establishes the bounded experimental composition; it does not admit new public compiler syntax or qualify all wrapping contexts.

## What the cases exercise

The matrix combines row/column axes, wrap/wrap-reverse, `align-content:flex-start/center/flex-end`, and two parent cross sizes: 20 px or auto. Three fixed-size items have main dimensions of 60, 60 and 40 px and cross dimensions of 10, 20 and 30 px. Their independent `align-self` values are flex-start, center and flex-end respectively. Parent main size is 50%; viewport sizes 400×400, 200×200, 120×120 and 400×400 exercise one, two, three and one lines in the same original and cloned scene. A green footer makes parent extent and overflowing paint visible.

The fixed 20 px cross size is smaller than some line contents and the multi-line extent. Start/center/end line positioning, wrap reversal, partially off-artboard content and footer overlap all match the Chrome references within the existing geometry/pixel tolerances. No tolerances changed.

The auto cases have different sizing contexts by axis. Row height:auto measures the combined line heights. Column width:auto stretches to its outer column container's available width; it is **not shrink-to-fit intrinsic width**. Fixture names ending in `intrinsic` are historical labels for the auto variant, not evidence of intrinsic-width support. The review and receipt use `auto` to describe that axis-neutral variant.

## Ordinary-file composition

Each authored item is enclosed in a transparent perpendicular layout wrapper. The wrapper copies the item's authored fixed main dimension and uses automatic cross size with native cross-axis stretch, so it occupies the line's full cross extent. Its internal main-axis positioning independently implements the item's start/center/end alignment; the parent selects line start/center/end through its existing combined alignment property. Wrap-reverse changes the wrapper's flex-relative start/end mapping.

The fixture adapter compiles transformed markup using the frozen public spacing compiler, then writes ordinary direction, wrap and alignment properties into the resulting file. The augmenter round-trips the raw bytes before editing and introduces no new property types. Three wrapper LayoutComponent/style pairs add six ordinary records. Numeric wrapper dimensions come from authored fixture constants, not browser measurements. Source maps omit synthetic wrappers and are used only to inspect authored geometry. No runtime changes, host CSS setters or recompilation on resize supply layout behavior.

This isolates the positional line/item alignment seam previously identified in the direct native mapping. It does not solve default `align-content:normal`/stretch or distributed line alignment; earlier failures remain separate evidence. It also does not transfer the foreground paint-order experiments' qualifications to these wrappers.

## Validation and provenance

Artifacts live under `tools/html-to-riv/output/line-alignment-boundaries-r1/`; tracked bindings are in [line-alignment-boundaries-receipt.json](line-alignment-boundaries-receipt.json).

- `render/receipt.json`: 24 scenes, 192 original/clone frames, 192 geometry passes, 192 pixel passes and 384 cyan/transparent clear-control passes. Chrome 153.0.8010.12; immutable rust-metal RasterOrdering renderer. Raw driver public labels are overridden by the experimental manifest.
- `reproductions.json` and `reproduced/`: the complete adapter was re-executed for all 24 exact requests; each output reproduces original RIV bytes and parsed source-map contents exactly.
- `visual-direct.json`, `visual-transfer.json`, and 15 sheets (`visual-0.png` through `visual-56.png`): 59 distinct pairs directly inspected without visible divergence; 133 exact full-image transfers independently rechecked for Chrome and native. Every receipt frame appears exactly once in this coverage.
- `experiment-manifest.json`: freezes fixture, adapter, source, binary, receipt, reproduction and visual hashes. Driver tool hashes and captured source snapshot hashes were verified. The underlying public compiler binary matches its existing frozen checkpoint.
- `build-provenance.json`: the augmenter and both source files are exact copies of `line-alignment-wrappers-r1`, verified against that experiment's manifest. The original build command was not preserved there. Root reconstructed and executed a build using the exact hashed sources and dependencies, documented in the original directory’s `rebuild-receipt.json`. The rebuilt binary differs in bytes, but `adapter-rebuilt.py` and `rebuilt-reproduction.json` prove that it reproduces all 24 boundary RIVs and parsed source maps exactly. No bit-identical binary rebuild is claimed.

## Remaining scope

This matrix has ordinary row/column main direction, fixed-size empty colored items and the three positional line alignments. It does not establish reverse main direction, auto/percentage item sizing, nested painted descendants, arbitrary overlap ordering, clipping/opacity, baseline groups, safe/logical alignment keywords, authored grow/shrink/basis or general resource boundaries. Public Rust/CLI/WASM/JavaScript admission remains a separate implementation and qualification task. No broad immutable-runtime impossibility is inferred from untested combinations.
