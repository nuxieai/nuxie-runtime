# Independent review of compiler-owned JPEG admission

The reviewed validator has no identified blocking defect within its declared **8-bit, three-component, 4:4:4, Huffman baseline/progressive profile**. On the final source, 108 independently encoded valid JPEGs pass, and all 1,196 targeted malformed entropy controls reject. This is source and entropy-admission evidence. It does not qualify the public image compiler, transports, or Chrome/native rendering.

Final evidence: [r2 receipt](../output/public-image-jpeg-review-r2/receipt.json), SHA-256 `0dde583ca98d9eaced6e3cff84d4e8780a17a73695ab99d2a60e981ced1d233a`. The receipt binds the exact source snapshots, standalone audit executable, original input hashes, every result, and unchanged baseline decoder sources. The initial pre-hardening results remain in [r1 acceptance](../output/public-image-jpeg-review-r1/receipt.json) and [r1 malformed controls](../output/public-image-jpeg-review-r1/mutations-receipt.json).

## Reviewed source and boundaries

The final compiler sources are:

| Source | SHA-256 |
| --- | --- |
| `src/jpeg_validation.rs` | `173a906f46cdc063eb79a93726569bb53d65ef447338b95bfd46fdacb53a012a` |
| `src/assets.rs` | `5d6174ed6bcc3a06214f961f782656a3589c9a79b8f35da9b8eed4e4e6e05a69` |

[`decode_jpeg`](../src/assets.rs#L442) executes the complete container screen, strict entropy validator, and the existing portable JPEG decoder, in that order. It checks dimensions and RGB24 format before and after full decoding, exact decoded length, and absence of exposed EXIF/XMP/ICC metadata. The container screen walks every scan boundary, so rejected APP markers cannot hide between progressive scans. The neutral JFIF case admits a single pre-frame JFIF 1.1/1.2 header with equal nonzero density, no density unit, and no thumbnail. It does not depend solely on decoder metadata accessors.

The unchanged decoder is `vendor/jpeg-decoder-0.3.2-rive-v9f`; `decoder.rs` and `huffman.rs` were compared byte-for-byte against immutable target `6c7ac16617835b5f581784ff08a9e779bb52faf3` and copied into the final evidence. No runtime or renderer mutation was made for this review. The original decoder bit reader can synthesize zero bits after a marker; the new compiler screen prevents the specifically tested missing-entropy cases from reaching that recovery path.

This is deliberately narrower than general JPEG support: subsampling, other component counts/precision, arithmetic/lossless modes, other metadata, more than 64 scans, incomplete final coefficient precision, and nonconforming padding remain rejected. These restrictions must stay visible in public support documentation. A complete renderable progressive preview is not sufficient: this admission rule requires every coefficient band to finish at precision zero.

## Algorithm findings

- **Block and MCU counts:** with all three sampling factors exactly 1×1, both an interleaved MCU and a noninterleaved scan advance over the same `ceil(width/8) × ceil(height/8)` grid. Interleaved scans consume one block per selected component. Component identity, uniqueness, scan shape, and AC single-component restrictions are checked before traversal.
- **Huffman and byte consumption:** canonical code generation rejects oversubscription and the reserved all-one code. Symbols and magnitude bits come from actual bytes only. FF requires stuffed zero inside entropy; early markers reject. The final partial byte must contain only one padding bits, with no full extra entropy bytes. The matching JPEG requirements are T.81 B.1.1.2, C.2, and F.1.2.3.
- **Progressive state:** initial scans cannot overwrite previously initialized bands; refinement requires exactly the previous precision and decreases it by one. AC follows component DC. The nonzero bitset controls correction-bit consumption only; it is not a decoded image. AC refinement consumes the new sign before traversing prior nonzero coefficients, counts only zero coefficients in a zero run, and consumes exactly sixteen zeros for ZRL. EOBRUN covers the current block plus its declared following blocks, and existing nonzero coefficients still consume refinement bits during an EOB run.
- **Restart boundaries:** intervals count MCUs, restart numbering resets for each scan, and each restart requires valid padding and the expected RST sequence. Pending EOB runs may not cross a restart or scan end. Predictor reset is explicit. Coefficient presence and progressive precision persist across restarts.
- **Coefficient storage:** the final hardening decodes signed DC differences and rejects accumulation/scaling outside i16 storage. Initial AC magnitudes reserve all remaining refinement bits within i16. This prevents a shifted nonzero coefficient from wrapping to zero, which would otherwise invalidate the Boolean state abstraction. DC refinement sets lower bits without changing first-scan predictor accounting; newly introduced AC coefficients have bounded one-bit magnitude. Pixel reconstruction and IDCT remain delegated to the unchanged decoder.

The progressive traversal agrees with the primary [libjpeg-turbo 3.1.0 progressive decoder](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/3.1.0/src/jdphuff.c), including progression state, EOB-run accounting, and correction bits. T.81 is available in the [W3C-hosted standard](https://www.w3.org/Graphics/JPEG/itu-t81.pdf). Exact downloaded source/PDF bytes and hashes are retained in [primary-source bindings](../output/public-image-jpeg-review-r1/primary-sources.json). These comparisons support this bounded structural review; they are not proof that every corrupt JPEG is detected.

## Independent checks

The 108 generated controls use libjpeg-turbo `cjpeg` 3.2.0 with explicit 1×1 sampling for all three components. Exact commands, PPM source pixels, scan scripts, JPEG bytes, executable identity, and hashes are preserved. Coverage includes:

- Eight dimensions: 1×1, 1×9, 9×1, 7×17, 8×8, 9×9, 17×23, and 33×65.
- Flat and varying-color source pixels; baseline and progressive encoding; no restart interval, one-MCU restart, and three-MCU restart.
- Additional sequential single-component and paired-component scans, independently grouped progressive DC scans, and split AC bands followed by a combined refinement band.

There are 576 actual scans across those controls. Removing the last entropy byte in each scan produces **576/576 rejections**; inserting an extra zero entropy byte before its next marker produces **576/576 rejections**. Changing the first restart sequence where present produces **44/44 rejections**. All later markers and unrelated source bytes are retained, so these are not merely tests for absent EOI. The resulting total is **1,196/1,196 targeted invalid inputs rejected**, with no panic.

The final standalone harness includes the exact frozen validator, the exact shared dimension-check function extracted from frozen `assets.rs`, and a minimal diagnostic adapter. It does not call the complete asset pipeline. An initial harness build failed because the final validator had moved dimension validation into its parent module; this adapter-only failure is preserved in [the build note](../output/public-image-jpeg-review-r2/initial-harness-build-failure.txt). After including that exact helper, the same existing inputs passed on the final source. No production change was made by the reviewer.

The implementer separately reports `cargo test --lib assets:: --offline` passing 20 tests (11 asset and 9 JPEG), including full decode, synthetic refinement/restart/EOB cases, overflow guards, and truncation of all ten scans in the original progressive fixture. That report is supplementary; this review's independently reproducible counts are the 108 positive and 1,196 negative standalone checks above.

## Resource scope and remaining qualification

The dimension bounds are axis ≤8,192 and RGBA footprint ≤16 MiB per image. Across that domain, the maximum padded block count is 66,462 (for example 8,050×521), requiring **1,595,088 bytes** for all three presence masks, plus small fixed state. The strict screen completes before portable pixel decoding. Inputs are additionally bounded by 256 supplied assets and 16 MiB aggregate encoded bytes; aliases count before deduplication.

The 16 MiB RGBA bound is **not a total process-memory cap**. The unchanged progressive decoder may allocate roughly 25.5 MB of i16 coefficient storage at the worst padded dimensions, plus output and other buffers. This is bounded by the admitted dimensions, but the reviewer has not measured peak native/WASM memory or worst-case CPU time. No universal corruption, fuzzing, or denial-of-service proof is claimed.

Public Rust/CLI/WASM/JS integration, malformed-image rejection through each public transport, and native/Chrome qualification of actual emitted PNG/JPEG/WebP files remain separate checkpoint work. This source review does not transfer private image-render evidence to newly emitted public scenes. The reviewer is stopping after this bounded package, per the goal-review request.
