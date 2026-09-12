# Content-owner corpus: unchanged public baseline

This is fresh evidence from the frozen public compiler, before applying any private content-owner composition. The original 28 admitted cases retain two known failures: eight large-coordinate child-width frames and six fractional-edge pixel frames. A separate six-case stretch corpus passes every current gate. No production code, runtime, renderer, source fixture, or comparison threshold changed.

| Corpus | Cases / frames | Geometry passes | Pixel passes | Clear checks | Repeat identities |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original audited admitted cases | 28 / 224 | 216 / 224 | 218 / 224 | 448 / 448 | 140 / 140 |
| Additional stretch controls | 6 / 48 | 48 / 48 | 48 / 48 | 96 / 96 | 30 / 30 |

Each ordinary Rive file is compiled once at 390×160. The unchanged probe imports it, creates an independent clone, and resizes both through 240×160 → 390×200 → 768×120 → 240×160. The driver compares every authored source ID against independent Chrome rectangles using the existing 0.1px geometry gate and unchanged pixel gates. Pinned Chrome is 153.0.8010.12 at device scale 1. Native rendering uses Rust Metal in effective `RasterOrdering` mode; its accepted CLI mode token is `clockwise-atomic`. Cyan and transparent clear controls check that ordinary white scene paint supplies the background.

The bound original result is `output/content-owner-public-r1/receipt.json`, with complete rows, source maps, commands, PNGs and geometry under `render/`. The command intentionally exits 1 with `failed-public-baseline`; every failed frame remains present. `stretch/receipt.json` and `stretch/render/receipt.json` record the separate successful controls and exit 0. The original 28-case receipts were not overwritten when adding stretch evidence.

## Retained failure sources

| Source case | Authored source ID | Failed frames | Observation |
| --- | --- | --- | --- |
| `content-large-rounded-outer` | `c` | 0–7 geometry | Native child width is 1,000,000px; Chrome rectangle width is 999,999.75px. Parent `p` passes. |
| `content-point-fractional-paint-control` | `p` | 0, 2, 3, 4, 6, 7 pixels | Existing mismatch-ratio gate fails at 240×160 and 768×120. Geometry passes all eight frames; the two 390×200 pixel frames pass the unchanged gate. |

The large child starts at x=1,000,000px, outside every captured viewport. Its eight passing pixel frames do not qualify its offscreen width. The other 26 original cases pass all geometry and pixel frames, including the new `owner-small-fractional-outer` source. Finite gate passes do not establish unrestricted fractional or large-coordinate agreement, nor exact Chrome/native color equality.

All six additional stretch cases preserve authored IDs `stage`, `p`, `a`, and `b`: automatic owner height in row and reversed-row parents with normal/space-evenly child distribution, plus automatic owner width in column and reversed-column parents. These controls establish the public behavior that a private wrapper must preserve. They are outside the original frozen 33-case corpus and are reported separately.

## Source and tool identity

The original source is `validation/content-owner-cases.json`, SHA-256 `733b761f3f15e184de513e756e38d891d8de5d00167f440ca142413be13dc33a`. The independent admission audit confirms all 28 expected successes and five intended diagnostics with no output. `selection.json` binds the exact admitted subset, the complete source snapshot, and that admission receipt. All 28 newly generated Rive files and source maps exactly match the independent admission artifacts. No source was rewritten to produce this baseline.

| Tool | SHA-256 |
| --- | --- |
| Frozen public compiler | `034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d` |
| Immutable baseline probe | `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a` |
| Immutable renderer replay | `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f` |

Compiler path: `output/public-value-build-r1/frozen/html-to-riv`. Native tools: `output/immutable-baseline-toolchain-r2/`. The runtime target remains `6c7ac16617835b5f581784ff08a9e779bb52faf3`. All 118 frozen input snapshots were verified, and the runtime identity guard passes. Tool, driver, pixel-gate and reset identities are bound in `tool-bindings.json` and the render receipts.

The independently captured original-source Chrome audit also matches all 224 public-run rectangles and full PNG files exactly; `independent-chrome-comparison.json` records that comparison. No browser measurement was used to construct shipping dimensions or alter the compiler. This review records native metrics and artifact identity; it does not claim a new completed direct visual inspection. Complete comparison galleries remain available for the separate composition review.
