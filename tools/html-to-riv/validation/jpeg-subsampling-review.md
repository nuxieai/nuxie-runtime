# Independent JPEG subsampling review

No blocking defect was found in the bounded extension from 4:4:4 to standard **4:2:2 (`2×1/1×1/1×1`) and 4:2:0 (`2×2/1×1/1×1`)**. The final source passes all **916 prepared independent checks**: 228 newly encoded JPEGs, 580 targeted mapping/entropy controls, and the existing 108 unchanged 4:4:4 files. This is source and entropy-admission evidence, not native/Chrome paint qualification or a universal corruption-detection claim.

[Final receipt](../output/jpeg-subsampling-review-r1/receipt.json), SHA-256 `dc65f0b1831a4c31803a2fee69ba76f715f90120b29f75fff59961e886aafa82`, binds commands, exact input files, old/final source snapshots, validator executable and results. The [reusable check](../output/jpeg-subsampling-review-r1/check.py) consumes the frozen files; it does not regenerate the old 4:4:4 corpus.

## Reviewed source

The final `src/jpeg_validation.rs` SHA-256 is `a637966ee9fedd8e5689bae1d655248e78dfffe4e8605cc2cf2e82eda78ca8f9`. `src/assets.rs` is unchanged at `5d6174ed6bcc3a06214f961f782656a3589c9a79b8f35da9b8eed4e4e6e05a69`. The old validator and asset source are frozen from `306be10735912e0368aa9cb03e3d1c3d70bb4ff8`.

`ComponentGrid` separates each component's real block columns/rows from its padded row stride and padded row count. For width `W`, height `H`, sampling factors `hᵢ/vᵢ`, and frame maxima `hMax/vMax`, the implementation uses:

```text
frame MCU columns = ceil(W / (8*hMax))
frame MCU rows    = ceil(H / (8*vMax))
real columns[i]   = ceil(W*h[i] / (8*hMax))
real rows[i]      = ceil(H*v[i] / (8*vMax))
padded stride[i]  = frame MCU columns*h[i]
padded rows[i]    = frame MCU rows*v[i]
```

Interleaved scans visit frame MCUs in raster order, then each selected component's `hᵢ×vᵢ` blocks, including edge padding. Single-component scans visit only that component's real blocks in raster order. Both use the same padded stride for the coefficient-presence array. In a 17×23 4:2:0 image, the real Y grid is 3×3 but its storage is 4×4; noninterleaved Y indices are `[0,1,2,4,5,6,8,9,10]`.

Restart intervals follow the scan's MCU definition: a complete frame MCU when interleaved, one real component block when noninterleaved. The check occurs before each next MCU, not between Y blocks belonging to one interleaved MCU. Predictor and EOB state reset at the marker; AC presence and previously delivered precision persist. Progressive AC scans remain single-component, so their EOB bounds and refinement traversal now use the real component block count. Final complete-precision checks still refer to actual component coefficients; padding is not treated as required visible AC content.

The existing Huffman/byte reader, scan grammar/precision checks, signed DC bounds, initial AC and refinement helpers are **byte-identical** to the old source. [The unchanged-core check](../output/jpeg-subsampling-review-r1/unchanged-core-check.json) records these comparisons. The extension changes the sampling screen, block grid and traversal, while retaining strict real-byte consumption, restart padding, EOB limits and coefficient overflow guards.

The primary [T.81 standard](https://www.w3.org/Graphics/JPEG/itu-t81.pdf), sections 4.8.2 and A.2, supports the interleaved/noninterleaved ordering and partial-MCU distinction. The unchanged portable decoder's `parser::update_component_sizes` stores padded component dimensions; its noninterleaved traversal in `decoder.rs` agrees with real grids for these three explicitly admitted sampling patterns. That agreement must not be extrapolated to other component-factor arrangements. Exact primary PDF and vendor-source hashes are in [resource and source bindings](../output/jpeg-subsampling-review-r1/resource-and-primary-bindings.json). The vendor parser, decoder and Huffman sources match immutable target `6c7ac16617835b5f581784ff08a9e779bb52faf3` byte-for-byte.

## Independent controls

All **228 generated files were rejected as `unsupported-image` by the old validator**, then accepted by the final source. [Generation and red receipt](../output/jpeg-subsampling-review-r1/generated-receipt.json) preserves exact PPM pixels, JPEG bytes, scan scripts, cjpeg 3.2.0 identity and commands. The finite corpus covers:

- Dimensions 1×1, 1×17, 17×1, 8×9, 9×8, 16×16, 17×23 and 33×65.
- Flat and varying-color pixels; baseline/progressive; 4:2:2 and 4:2:0; no restart, one-MCU restart and three-MCU restart.
- Separated sequential components, paired Y/chroma, paired chroma after single Y, split progressive DC groups, split AC bands followed by combined refinement, and DC refinement with changed component grouping.

The [580 targeted controls](../output/jpeg-subsampling-review-r1/mutations-manifest.json) derive from 48 odd-sized sources:

| Control | Expected and observed |
| --- | --- |
| Remove the last entropy byte from each of 210 scans, retaining subsequent markers | 210 rejected |
| Insert an extra entropy byte before each scan's next marker | 210 rejected |
| Change the first restart marker's sequence | 32 rejected |
| Change the declared restart interval | 32 rejected |
| Increase width 17→33, requiring an extra frame MCU column | 48 rejected |
| Increase width 17→25 where real Y blocks must be supplied by noninterleaved scans | 36 rejected |
| Increase width 17→25 for fully interleaved sequential Y data already present in the unchanged padded grid | 12 accepted |

The last distinction matters: a dimension-header edit is not automatically malformed. The valid 12 controls already contain the additional coded Y blocks. The other 36 have the same frame MCU grid but require additional real Y blocks in a noninterleaved scan. All 108 historical 4:4:4 files pass unchanged, with their original hashes verified before testing.

The independent executable includes the frozen validator plus the exact shared dimension helper and a minimal diagnostic adapter. It does not substitute an IDCT or decode pixels. The implementer's separate [24-test asset receipt](../output/public-jpeg-sampling-implementation-r1/receipt.json) covers the public asset path and eight full portable-decode fixtures; it is linked as supplementary evidence. One initial mutation-generator Path/string adapter error is preserved in the output; it occurred before mutation files were generated and did not require a compiler change.

## Resource bounds and limitations

Sampling factors are restricted to at most two, after the shared dimension guard: each axis ≤8,192 and decoded RGBA footprint ≤16 MiB. The new integer products and indices fit 32-bit `usize`. Every visited component index lies within its padded allocation. Across every legal integer width and its maximum legal height, the maximum presence-state sizes are:

| Sampling | Padded component blocks, summed | Presence masks | Portable progressive coefficient storage |
| --- | ---: | ---: | ---: |
| 4:4:4 | 199,386 | 1,595,088 bytes | 25,521,408 bytes |
| 4:2:2 | 134,904 | 1,079,232 bytes | 17,267,712 bytes |
| 4:2:0 | 101,184 | 809,472 bytes | 12,951,552 bytes |

The check script independently recomputes those maxima from the frozen formulas. The new patterns do not increase the previous maximum, but **16 MiB RGBA is not a total process-memory cap**. Decoder coefficients, output, encoded input and other buffers remain additional memory. No peak-allocation or worst-case CPU measurement is claimed.

The rest of the profile remains narrow: eight-bit, three-component Huffman baseline/progressive data, neutral permitted metadata, complete coefficients, at most 64 scans, and the existing resource/strict-consumption rules. Other sampling arrangements remain diagnostic. Root's native/Chrome work must separately qualify actual emitted files and preserve any chroma reconstruction differences. This review made no production, runtime, renderer, fixture or shared-status edits and launched no native render.
