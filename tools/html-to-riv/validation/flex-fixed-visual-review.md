# Fixed flex controls: independent visual review

This review covers the three completed diagnostic runs in `output/flex-fixed-controls-r1`: `reductions-render`, `edge-render-r2`, and `fixed-render`. It does not change the compiler or any runtime code, qualify public flex grow/shrink, or claim that a file-level solution is impossible. The images show that several failures survive after flexible sizing is removed, and that identical browser/native geometry does not guarantee identical edge paint.

## Evidence and review method

All 28 case pairs were directly inspected as full images, placed without resampling in eight PNG sheets using Pillow. `visual-review.py` verifies each displayed rectangle against the entire source image after reopening the saved sheet: dimensions and every RGBA byte must agree. The sheets were viewed with original image detail. Names and placement coordinates are bound in `visual-evidence.json`; the fixed column sheet was widened and re-viewed so all source names remain legible.

The independent script verifies receipt hashes for every Chrome/native PNG, then compares every repeat image against the same-role frame 0 using full RGBA equality. Each case contains both original and cloned instances and four steps, all at the same viewport. There are 392 repeat-image transfers (28 cases × 7 remaining frames × 2 image roles) and 448 independently decoded clear-image checks (224 frames × cyan/transparent). All pass. Thus direct visual inspection covers 56 representative images, and explicit equality transfers cover the remaining 392 Chrome/native images. No duplicate case was omitted from direct inspection. These constant-viewport reductions isolate paint behavior; they do not establish responsive resizing behavior.

| Run | Cases / frames | Geometry gate passes | Pixel gate passes | Pixel gate failures |
| --- | ---: | ---: | ---: | ---: |
| reductions-render | 9 / 72 | 72 | 16 | 56 |
| edge-render-r2 | 11 / 88 | 88 | 24 | 64 |
| fixed-render | 8 / 64 | 64 | 0 | 64 |
| Total | 28 / 224 | 224 | 40 | 184 |

These counts preserve the existing gate and all expected failures. Passing pixel gates are not claims of exact pixel equality. For example, integer-height controls contain tiny tolerated native color differences away from the sampled center column.

## Direct visual observations

The reduction sheets show the original overlapping yellow, coral, teal and navy boxes and each removal, followed by isolated navy rectangles. The bottom-edge difference persists through `remove-a`, `remove-b`, `remove-fixed`, `remove-lead`, `single-box`, and `minimal-height`. Every one of those seven cases fails its pixel gate on all eight frames despite passing geometry. `minimal-no-height` and `minimal-no-paint` are white in both images and pass; removing visible paint removes the diagnostic, but does not provide a rendering solution.

The edge sheets show the integer-height controls as visually matching navy rectangles or a one-pixel line. Fractional rectangles show differences along their lower edge. The extremely thin 0.03125px native strip is very faint at full resolution; the independently sampled bytes and whole-image nonwhite count below substantiate the observation without relying on visual detectability alone.

The fixed sheets retain the same arrangement and major painted regions as their selected flex source scenes. Their failures persist with point-sized, non-flexing boxes. The two column controls built from Chrome sizes show small differences relative to native-size controls, so they must not be described as exact native replicas of the original flex scene.

## Exact transfer from original flex scenes

`fixed-bindings.json` identifies the original source receipt, request hash, Rive hash, case and frame for each control. The independent script rereads that source receipt, hashes the actual original request and Rive file, and verifies both source image hashes. It then compares full decoded control and original images, rather than trusting `fixed-image-comparison.json`.

All eight control Chrome images exactly equal their designated original flex Chrome image. All four native-dimension controls also exactly equal the original native image. Both row-direction Chrome-dimension controls equal the original native image as well. The two remaining native comparisons have these exact differences:

| Fixed control | Original source | Frame | Native differing RGBA channels |
| --- | --- | ---: | ---: |
| column-decimal-point-fixed-chrome | bridge-column-decimal-point | 1 | 206 |
| column-reverse-decimal-point-fixed-chrome | bridge-column-reverse-decimal-point | 1 | 175 |

Every other native comparison has zero differing channels. This independently reproduces the existing comparison summary: 8/8 Chrome and 6/8 native exact transfers. The four original flex cases are row subunit (frame 1), row-reverse subunit (frame 0), column decimal (frame 1), and column-reverse decimal (frame 1), all from `output/flex-proof-bridge-r2/render/receipt.json`. Full source/control PNG and RGBA hashes are recorded per comparison in `visual-evidence.json`.

## Exact edge samples and limits on interpretation

For all 11 edge controls, Chrome's measured box and the native probe report the same authored height. Each rectangle starts at (0,0), has width 100, and is painted `rgb(20,35,63)` over white. Samples below use x=50, away from the left and right edges; every displayed sample has alpha 255. “White” means RGB (255,255,255), and “paint” means RGB (20,35,63).

| Authored and measured height | Sample y | Chrome RGB | Native RGB | Chrome / native nonwhite pixels |
| ---: | ---: | --- | --- | ---: |
| 25 | 25 | white | white | 2500 / 2500 |
| 25.25 | 25 | white | (196,200,207) | 2500 / 2600 |
| 25.5 | 25 | paint | (137,145,159) | 2600 / 2600 |
| 25.75 | 25 | paint | (79,90,111) | 2600 / 2600 |
| 26 | 25 | paint | paint | 2600 / 2600 |
| 0.03125 | 0 | white | (248,248,249) | 0 / 100 |
| 0.25 | 0 | paint | (196,200,207) | 100 / 100 |
| 0.5 | 0 | paint | (138,145,159) | 100 / 100 |
| 0.75 | 0 | paint | (79,90,111) | 100 / 100 |
| 1 | 0 | paint | paint | 100 / 100 |
| 1.25 | 1 | white | (196,200,207) | 100 / 200 |

The height-25 and height-26 controls pass the unchanged pixel gate on all frames, as does height-1. Height 25.5 demonstrates the main point directly: equal geometry, but Chrome paints the sampled bottom row fully and native paints a partial-color row. Height 0.03125 has no Chrome ink at all and 100 faint native pixels. All eight fractional-height cases fail the existing gate.

Do not describe these observations as a universal Chrome “round to nearest pixel” rule. In particular, 0.25px paints one full Chrome row, while 25.25px paints 25 full rows and 1.25px paints one row; 0.03125px paints none. The native samples are consistent with fractional edge coverage in these axis-aligned rectangle controls. The evidence alone does not establish which browser painting rule produces every case, that all discrepancies are antialiasing, a complete quantization function, or that ordinary Rive compositions cannot reproduce the desired result. No tolerance was widened, and no pixel failure was excused on that basis.

The next useful experiment is a bounded ordinary-file composition targeting these exact fixed controls, with original and clone lifecycle checks and unchanged gates, while investigating Chrome's actual small-rectangle paint rule before generalizing.

## Bound artifacts

All paths below are relative to `output/flex-fixed-controls-r1` unless explicitly noted.

| Artifact | SHA-256 |
| --- | --- |
| `visual-review.py` | `bbfc62efdefd03ab0ba4f33877611d18c7b48251da3582b7bc37e9095a1d30fe` |
| `visual-evidence.json` | `591f77241cfe4d76c6b1ae59962b36bd60d0540538a6dc2cc1ec826b841b3a82` |
| `fixed-bindings.json` | `f4a60eaf2e13f163ff79f2cf6730170d335d49c08b6728a490d4d8f6c439653e` |
| `fixed-image-comparison.json` | `ca83cf77d6309c990c5c4130b0b020b302f582407ec52bc4e061a8d54061c282` |
| `reductions-render/receipt.json` | `1106f75e76eb01565865ad5aa71c17bba95ee3b5af2b4fd055a09ca191cb5367` |
| `visual-reductions-render-1.png` | `bdffe091fad84857175329ddae3c6df7605d7449bb30fef3225ca089dd586e0b` |
| `visual-reductions-render-2.png` | `182ccedc1771cd7c8d46625b62c0cc914ee02a65f58ee03fa73bedc008b8b111` |
| `visual-reductions-render-3.png` | `f212da2de4011a4d7c9ca8948df4d24fd35b408e924a42fe3cc063333bf7adbf` |
| `edge-render-r2/receipt.json` | `205aaeb02ded92015a5750cd5d8c84e839b33af3a9ba2e0ee4f649daac673a43` |
| `visual-edge-render-r2-1.png` | `385efc167939434b6aaed95afe97d8ded054dd54425ee55c0cf0710d04b556c6` |
| `visual-edge-render-r2-2.png` | `cb85c429f9b7e31d3a1c66f8f6a0de04bd0b02bcdb30afec5e5465e4bc31b8cf` |
| `visual-edge-render-r2-3.png` | `9c3ef69e7ebd8b953cd01ecea64721ac07cad47203dcd46f230dea8d51bc173d` |
| `fixed-render/receipt.json` | `691f9185aa24f0cf74260ec57de48d5e0aa696462af7d1b59ec9e8d29004ed28` |
| `visual-fixed-render-1.png` | `6ccd52ee144f8f4eace03883c981c818507439e9b660f3168a98f77c662576a5` |
| `visual-fixed-render-2.png` | `bcf8b68606ee3f713ca37ec74470b97c2ff1161a9a2c30e68e3de35111e3d05b` |

The source receipts bind Chrome 153.0.8010.12, the immutable native probe `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`, and the Rust Metal RasterOrdering renderer `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`. The controls use the already-built frozen compiler/candidate tools named in their receipts. This review verifies retained artifacts; it does not claim a fresh render or a new public compiler build.
