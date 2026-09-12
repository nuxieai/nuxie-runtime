# Additional intrinsic image aspect ratios

The frozen public compiler passes geometry and image-presence checks for all **12 scenes / 96 original-and-clone frames** in this bounded corpus. Pixel checks pass **80/96**. Two odd-ratio cases retain **16 local tail-edge failures**; these are partial visual results, not blanket image qualification. This corpus introduced no production, runtime, renderer or tolerance change. Parent-owned authoring-helper guards are described in the supplement below.

[Public cases](public-image-aspect-cases.json) use five new programmatically authored RGB PNGs: portrait 48×96, wide 160×40, square 41×41, odd 37×23 and small 3×5. The [generator](../fixtures/images/aspect-r1/generate.py) and [asset manifest](../fixtures/images/aspect-r1/manifest.json) preserve source construction, dimensions and encoded hashes. No HTML screenshot or browser measurement supplies asset pixels or shipping dimensions.

The [gallery](../output/public-image-aspect-r1/visual/gallery.html) exposes complete Chrome/native pairs, diffs and separate failure filters. The [native receipt](../output/public-image-aspect-r1/native-receipt.json), [measurement summary](../output/public-image-aspect-r1/measurement-summary.json) and [aggregate receipt](../output/public-image-aspect-r1/receipt.json) retain the underlying evidence.

## Measured contexts

Every scene contains an 8×8 following sibling. Rows below show the first 240×240 frame; its tail starts at the listed image main extent. The root responsive square also measures 195×195 at width 390 and 384×384 at width 768. All cases run at 240×240 → 390×320 → 768×560 → 240×240 on both original and cloned artboards from one compilation.

| Source context | Native image box | Chrome image box | Pixel frames passing |
| --- | --- | --- | ---: |
| Portrait root, start alignment | 48×96 | 48×96 | 8/8 |
| Portrait column, stretch | 120×240 | 120×240 | 8/8 |
| Portrait row, stretch | 60×120 | 60×120 | 8/8 |
| Wide column, fixed height, start | 320×80 | 320×80 | 8/8 |
| Wide row, fixed main width, cross stretch | 100×120 | 100×120 | 8/8 |
| Square root, responsive 50% width | 120×120 | 120×120 | 8/8 |
| Square row, start | 41×41 | 41×41 | 8/8 |
| Odd column, stretch | 200×124.324326 | 200×124.3125 | 0/8 |
| Odd row, fixed cross height | 96.521736×60 | 96.515625×60 | 0/8 |
| Odd column, fixed cross width | 93×57.81081 | 93×57.796875 | 8/8 |
| Small row, stretch | 72×120 | 72×120 | 8/8 |
| Small root, start | 3×5 | 3×5 | 8/8 |

The maximum absolute observed authored-box coordinate/extent difference is **0.013935px**, within the existing 0.1px geometry gate. The wide fixed-main case deliberately stretches its automatic cross size instead of applying the source ratio unconditionally. The portrait column overflows its fixed parent before the tail; larger viewports show that complete overflow. The following siblings therefore check actual used dimensions independently of painted image content.

The two failed odd-ratio cases pass whole-frame mismatch-ratio checks but exceed the unchanged local tail RGB gate: approximately **13.0602** for the column and **18.7315** for the row. The observed native tail boundaries have blended pixels where Chrome's tail boundary is hard. At column pixel (3,124), Chrome is RGB(23,33,43) and native is RGB(91,86,86). At row pixel (96,3), Chrome is RGB(245,187,37) and native is RGB(134,126,90). [Exact samples and crop coordinates](../output/public-image-aspect-r1/visual/failure-tail-samples.json), [20× pixel magnifications](../output/public-image-aspect-r1/visual/failure-tail-zooms.png). These observations do not establish which browser or native internal rounding stage causes the difference, nor a universal runtime limitation.

## Validation and direct review

Compilation reused `output/public-image-build-r2/frozen`, with CLI SHA256 `8f49ac9a7272f0770cc3ae67eb29755830219ff5af0716741b2f77735aa218df`. The new case/assets were frozen beside the requests by the parent-added corpus option; the old compiler snapshot remained unchanged. All 12 requests compiled successfully. Chrome was **153.0.8010.12**; the existing probe and Rust Metal renderer consumed ordinary emitted files on immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`. The run had zero process/browser errors.

Commands, from the compiler module directory:

```sh
python3 validation/public-image-compile.py output/public-image-aspect-r1 --build output/public-image-build-r2 --cases validation/public-image-aspect-cases.json
node validation/public-image-native.mjs output/public-image-aspect-r1 output/immutable-baseline-toolchain-r2/baseline-probe output/immutable-baseline-toolchain-r2/renderer-replay
python3 output/public-image-aspect-r1/visual-review.py
node output/public-image-aspect-r1/gallery-check.mjs
```

All **36 distinct full Chrome/native pairs** were directly inspected on **nine unscaled sheets**, plus the supplementary failure magnifications. The remaining **60 clone/restore frames** transfer visual review only after exact source, asset, viewport and complete RGBA equality; each retains its own native geometry and pixel gates. Checks also joined authored input and source IDs, encoded asset bytes in RIV and actual native recording decode payloads, Chrome natural dimensions, browser resource hashes and full PNG hashes. [Visual coverage](../output/public-image-aspect-r1/visual/coverage.json), [completed review receipt](../output/public-image-aspect-r1/visual/review-receipt.json).

The gallery check loaded all 72 representative PNGs, verified 147 local links and exercised the filters: 36 total representatives, zero geometry failures, six pixel failures and zero presence failures. [Gallery check](../output/public-image-aspect-r1/visual/gallery-check.json).

This extends measured intrinsic-sizing evidence beyond the previous 96:64 ratio. It does not qualify every ratio, image format, extreme numeric range, min/max, image padding, automatic margin, baseline, nonlegacy flex or sampling combination. Those diagnostic boundaries remain unchanged. The 16 failed frames stay visible; no public compiler admission changed as part of this corpus-only task.

## Supplemental transport parity and authoring bindings

The [new parity receipt](../output/public-image-aspect-r1/parity-r1/receipt.json) records **12 requests / 24 passing checks**: both raw WASM ABI and the frozen JavaScript wrapper reproduce each recorded CLI RIV hash and complete source map. The driver verified all **153 frozen build bindings**. This uses the committed `public-image-build-r2/frozen` compiler, WASM and wrapper; it does not validate the later JPEG candidate or rebuild current source. The parity driver compares against the earlier CLI outputs rather than invoking CLI again; independent supplemental checks verified all 12 stored request, RIV and map hashes before accepting those references.

```sh
node validation/public-image-parity.mjs output/public-image-aspect-r1 output/public-image-build-r2/frozen output/public-image-aspect-r1/parity-r1
```

The [supplemental receipt](../output/public-image-aspect-r1/supplement-r1/receipt.json) separately binds the current authoring compile helper, fixture generator/manifest, corpus, parity driver and parity artifacts. It preserves the existing compile/native/visual/aggregate receipts unchanged. The earlier review version and helper snapshot remain in the original source snapshots; new bindings do not retroactively claim that a later helper version produced the original images.

Read-only review of `--cases` found two path-handling gaps: unconstrained case names could write through the `frozen` symlink, and an absolute asset path inside the module could bypass `asset_root` during request hydration. The parent repaired the helper to require unique single-directory case names and relative asset paths without parent traversal before per-case writes. The corrected helper snapshot has SHA256 `bf300954b17e794c3bf6d07c86e385abe3283594d5f9687ccb64cf2de82eb354`; its [diff from the earlier snapshot](../output/public-image-aspect-r1/supplement-r1/compile-helper-from-original.diff) is retained. This correction was reviewed in source; no unsafe path experiment or compiler/native rerun was performed.

All 12 aspect case names and asset paths already meet those restrictions. Every current source asset equals its copied authoring snapshot and the encoded request bytes; generator and manifest hashes also match. [Supplemental authoring checks](../output/public-image-aspect-r1/supplement-r1/authoring-checks.json). The helper reads the committed compiler through its `frozen` symlink and places this corpus's new asset copies under the fresh output's `authoring-inputs`; it does not add the assets to the old compiler snapshot. Its authoring binding list covers cases and encoded assets, so this supplement explicitly supplies the separate helper/generator bindings.

Transport equivalence leaves the native result unchanged: **96/96 geometry, 80/96 pixels and 96/96 presence**, with all 16 failures retained. There were no native rerenders in this supplemental step.
