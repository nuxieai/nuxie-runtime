# Ordinary embedded image source audit

This is a source audit and read-only observer build for immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` (tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`). It establishes existing serialized image and layout mechanisms for a private experiment. It does not admit images to the public compiler or claim browser pixel parity. No image was decoded, rendered, or compared in this audit.

The [source receipt](../output/ordinary-image-source-r1/source-bindings.json) binds 36 complete repository source snapshots and verifies each against both the baseline Git object and current worktree. `git diff --name-only BASE -- Cargo.toml Cargo.lock crates vendor tools/renderer-replay` was empty. All 13 files in the existing [baseline toolchain manifest](../output/immutable-baseline-toolchain-r2/manifest.json) still match their recorded hashes. The baseline probe is `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`; RustMetal replay is `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`.

## Embedded bytes and actual decoding

1. `ImageAsset` type **105** is an ordinary global file asset. `FileAssetContents` type **106**, bytes property **212**, attaches to the latest file-asset importer. It does not depend on a host-provided URL loader. Signature **911** is a separate serialized field, not a MIME discriminator.
2. `FileAssetImporter::resolve` consumes those bytes and calls the asset's decoder when there is no successful host loader. With the baseline probe's `None` loader and `None` admission policy, a failed decode result is not propagated as an import error. Empty bytes likewise need not fail import. [Importer source](../../../crates/nuxie-runtime/src/mechanical_port/source/importers/file_asset_importer.rs), [contents source](../../../crates/nuxie-runtime/src/mechanical_port/source/assets/file_asset_contents.rs).
3. `ImageAsset::decode` calls the retained factory's `decode_image`, stores its optional result, and notifies image referencers. `ImageAsset::file_extension` returning `png` is unrelated to format dispatch. [Image asset source](../../../crates/nuxie-runtime/src/mechanical_port/source/assets/image_asset.rs).
4. `RecordingFactory::decode_image` reads only PNG/JPEG/WebP header dimensions, retains the exact encoded bytes, and records `decodeImage ... data=<hex>`. It returns success even when the header cannot be recognized, in which case dimensions are **0 × 0**. It does not enforce the full decoder's size limits. Successful recorded import and nonzero dimensions therefore do not prove complete payload validity. [Recording factory and header parsers](../../../crates/nuxie-render-api/src/lib.rs), lines 3846–3862 and 5077–5234.
5. The ordinary text-stream replay path is `RenderStream` resource replay → `NativeMetalFactory::decode_image` → mechanical `decodeImageHandle` → `RenderContext::decodeImageExecutable`. A replay decode failure is a `ReplayError::ImageDecode`, unlike the permissive import result. The Metal platform texture-decoder hook returns null. The enabled `rive-decoders` fallback is `MechanicalBitmapDecoder`, which calls both `preflight_encoded_image` and `decode_image_rgba` and requires matching dimensions. It installs no host override. [Stream replay](../../../crates/nuxie-render-stream/src/lib.rs), [Metal factory](../../../crates/nuxie-renderer/src/native_metal/mod.rs), [decoder installation](../../../crates/nuxie-renderer/src/native_metal/mechanical_render_context.rs), [mechanical decoder dispatch](../../../crates/nuxie-renderer/src/mechanical_port/source/renderer/src/render_context_cpp.rs).

The original toolchain command enabled `renderer-replay/native-metal`; that feature explicitly enables `nuxie-renderer/renderer-metal` and `rive-decoders`. No KTX2 feature or external image loader is selected by this path.

## Format, dimensions, color, and alpha boundary

The authoritative baseline software entry point is [nuxie-image-codec](../../../crates/nuxie-image-codec/src/lib.rs), not its separate unbounded or source-compatibility bitmap functions.

| Topic | Behavior reached by baseline RustMetal replay |
| --- | --- |
| Formats | Recognizes only PNG signature, JPEG SOI, and RIFF/WEBP. GIF, SVG, AVIF, HEIF, arbitrary ImageIO formats, and KTX2 are not admitted by this bounded entry point. A recognized signature alone does not prove acceptance. |
| Resource limits | Encoded bytes ≤ 64 MiB; each axis in 1…8192; tightly packed decoded `width × height × 4` ≤ 64 MiB. This is a decoded-buffer ceiling, not a bound on all decoder, GPU, or mip allocation. |
| Preflight | Uses PNG, JPEG, and WebP parsers to read dimensions before full decode. Full decoded dimensions and RGBA buffer length must match. RecordingFactory's lighter header parser is a distinct implementation. |
| macOS pixels | All three formats use `CGImageSourceCreateWithData`, then `CGImageSourceCreateImageAtIndex(source, 0, NULL)`, then a CoreGraphics bitmap draw. There is no fallback to the portable pixel decoder if ImageIO fails. |
| Color | Destination is **DeviceRGB**, created by `CGColorSpaceCreateDeviceRGB`, with 8 bits per component. There is no explicit named sRGB destination. ImageIO/CoreGraphics own source-profile conversion; ICC/browser parity cannot be concluded from this code. |
| Alpha | Alpha-bearing images use premultiplied-last, 32-bit big-endian RGBA. Opaque alpha-info modes use none-skip-last. The result is handed to the renderer as `rgbaPremul`; no separate unpremultiply step exists in this path. Existing source tests assert expected premultiplied PNG/WebP pixels and opaque JPEG alpha; those tests were read, not rerun here. |
| Orientation / pixel aspect | No EXIF-orientation lookup, orientation transform, pixel-aspect transform, or thumbnail-with-transform option is requested. Header dimensions remain encoded pixel dimensions. If ImageIO returns different dimensions, the common decoder rejects them. This is not evidence of CSS `image-orientation: from-image` parity. |
| Animation | One image at index zero is requested. No timing, disposal, frame-advance, or animation state is exported as an ordinary Image. Animated-file first-frame semantics remain unqualified. |
| GPU upload | `rgba32` becomes `MTLPixelFormat::RGBA8Unorm`; this path does not use an sRGB texture format. The mechanical renderer requests the full mip chain and generation of remaining mips. The Metal upload's `srgb` argument is unused, as in the pinned source. |

The Apple API declarations and comments were read from the installed SDK and copied into `output/ordinary-image-source-r1/apple-sdk-source/`. `CGImageSource.h` documents a separate thumbnail option for orientation/pixel-aspect transforms; the decoder does not call it. These headers do not expose Apple's actual decoder implementation. The source receipt records macOS **26.6.2 (25G83)** and the installed SDK identity. Orientation, profiled colors, high-bit-depth images, APNG/animated WebP, and format variants remain empirical questions for the parent experiment; this audit does not silently certify them.

The exact lockfile resolves **png 0.18.1**, **image-webp 0.2.4**, **moxcms 0.8.1**, and the baseline workspace's **jpeg-decoder 0.3.2** path patch `vendor/jpeg-decoder-0.3.2-rive-v9f` (`rive_v9f` feature). They participate in header preflight on macOS. Outside macOS the pixel path is different: PNG expands palette/transparency and strips 16-bit samples to eight bits; WebP reads one image; JPEG converts the decoder's RGB/grayscale/CMYK output to RGBA. Optional ICC conversion uses moxcms to sRGB, then channels are premultiplied with `(channel × alpha + 127) / 255`. Unsupported/invalid ICC transforms leave the prior samples unchanged. This is not a cross-platform pixel-equivalence claim.

Separate functions `decode_png_bitmap`, `decode_jpeg_bitmap`, `decode_webp_bitmap`, `decode_apple_bitmap`, and `_unbounded` entries have different contracts. In particular, the Apple compatibility function can bypass the ordinary format/size preflight, and the low-level WebP bitmap helper has its own first-fragment behavior. Those functions are not the callback installed by this Metal replay and cannot expand its claimed capability.

## Serialized image and layout fields

| Record / field | Serialized identifier and meaning |
| --- | --- |
| ImageAsset | Type **105**. Inherited authored width **208**, height **207**, initially zero. Inherited FileAsset `assetId` is **204**. |
| FileAssetContents | Type **106**. Exact encoded bytes **212** attach to the preceding current file-asset importer. |
| Image | Type **100**. Image `assetId` **206** is a **zero-based global FileAsset vector index**, resolved by BackboardImporter, not the FileAsset's authored id 204. Missing/out-of-range references can survive resolution and draw nothing. |
| Image origin | **380/381**, default **0.5/0.5**. Drawing translates by `−intrinsicSize × origin` after the world transform. |
| Image fit | **974**: 0 resize, 1 contain, 2 cover, 3 fit-width, 4 fit-height, 5 none, 6 scale-down, 7 fill. The implementation maps other values to fill after an eight-bit cast; experiments should use only the named values. |
| Image alignment | **975/976**, default **0/0**. Values −1/0/+1 mean start/center/end through `(alignment + 1) / 2`; offset uses remaining space after fit scaling. |
| LayoutParticipant | Type **1066**, child of Image. It connects the Image to the surrounding ordinary layout owner and exposes resolved left/top/width/height. |
| LayoutComponentStyle | Type **420**, aspect ratio **524**. Positive ratios flow into the layout engine; zero/nonpositive clears the ratio. An ordinary outer owner can carry the desired ratio. Presence of this field is not proof of all CSS replaced-element sizing rules. |
| LayoutComponent | Type **409**, clip **196**. Ordinary owner clipping is a candidate for cover content. Image's own draw routine does not clip to its layout rectangle. |
| Sampling | Asset filter **1073**: 0 bilinear / 1 nearest. Asset wrap X/Y **1074/1075**: 0 clamp / 1 repeat / 2 mirror. Image overrides **1076/1077/1078**: zero inherits; nonzero values subtract one before the asset enum mapping. Default is bilinear clamp. |

References: [generated Image](../../../crates/nuxie-runtime/src/mechanical_port/source/generated/shapes/image_base.rs), [ImageAsset](../../../crates/nuxie-runtime/src/mechanical_port/source/generated/assets/image_asset_base.rs), [DrawableAsset dimensions](../../../crates/nuxie-runtime/src/mechanical_port/source/generated/assets/drawable_asset_base.rs), [BackboardImporter asset index resolution](../../../crates/nuxie-runtime/src/mechanical_port/source/importers/backboard_importer.rs), [Image implementation](../../../crates/nuxie-runtime/src/mechanical_port/source/shapes/image.rs), and their complete bound snapshots.

`Image::width/height` prefer renderer image dimensions; authored asset dimensions are only a fallback when no renderer image exists. Thus an invalid 0×0 RecordingFactory result suppresses even valid authored metadata. The runtime does not implement browser density correction here. `measure_layout` returns an exact constraint when that axis is `Exactly`, otherwise its intrinsic axis size; it does not infer the unconstrained axis from aspect ratio or itself clamp an `AtMost` constraint. Surrounding layout can still impose constraints, which must be observed independently.

`control_size` receives a resolved size and updates fit scale. Contain uses the minimum axis ratio; cover the maximum; fit-width/height one ratio; none uses one; scale-down is `min(widthRatio, heightRatio, 1)`; fill/resize use independent ratios. `computed_width/height` are intrinsic dimensions multiplied by authored scale and layout scale. They are paint dimensions, **not necessarily the participant's CSS-like box**. The world matrix also includes participant translation, fit offset, and surrounding transforms. `local_bounds` remains the intrinsic-origin rectangle.

Files at version **7.2 or later** keep layout scale separate from authored node scale. Older files overwrite node scale during fitting. The compiler's current wire header is **7.3**, so the existing path retains authored scale. A clone copies the serialized Image base and the same asset handle, then receives its own layout sizing; the observer intentionally measures both occurrences through a resize-and-restore sequence.

## Read-only observer

[ordinary-image-probe.rs](ordinary-image-probe.rs) builds directly with the existing immutable baseline rlibs. No Cargo resolution, runtime source rebuild, decoder substitution, or runtime configuration change was performed. The [build receipt](../output/ordinary-image-source-r1/probe-build/receipt.json) binds its exact rustc command, source snapshot, binary, log, 115-file baseline rlib inventory, and native libhydrogen archive. The inventory records all available rlibs; it does not claim they are all linked.

Binary: `output/ordinary-image-source-r1/probe-build/ordinary-image-probe`, SHA-256 **`fee749c105222d50648ac57ae82294461ced8cb593061ccfc40343aff68739e1`**.

```text
ordinary-image-probe SCENE.riv NEW_OUTPUT 240x160 390x200 768x120 240x160
```

It imports bytes with `File::import(..., None, None, None)`, settles, clones once, then calls only ordinary `set_size`/`update_pass` before reading each Image. It writes `image-metrics.json`, exact input `scene.riv`, and `import.stream`. Observations include intrinsic/computed dimensions, origin/alignment, authored/render scale, local bounds, world matrix, file-asset index, asset metadata, participant resolved rectangle, and recorded asset/image occurrence identities. Identity strings are process-local and comparable only within the run. The recorded-image identity proves a retained RecordingFactory occurrence, **not a successful full pixel decode**. No drawing or native replay occurs in this observer; the parent's native renders are the separate decode/paint authority.

## Bounded first experiment

Use a static, orientation-neutral image with distinct edge/quadrant colors and a transparent region; embed its exact bytes, author matching asset dimensions, and retain the original bytes as the browser source. Start with declared two-axis boxes and fill/contain/cover/none/scale-down; inspect alignment and clipping independently of the owner rectangle. Add responsive outer-owner sizing, then a ratio-bearing outer owner plus Image participant to test the `width:100%; height:auto` equivalent. Keep content, padding, border, and cover clip rectangles separately observable.

Host APIs such as `set_render_image`, `set_render_image_occurrence`, `RuntimeImageAssetOwners`, `register_image_dimensions`, and a custom file asset loader can replace or mutate owner state after import. They are not necessary for embedded bytes and are not used by this observer. Authored dimension fields and ordinary fit/layout records are serializable; runtime host calls, metadata sidecars, and precomputed per-viewport scale updates are not evidence that a file alone implements responsive image behavior.

Remaining qualification is explicit: actual PNG/JPEG/WebP variants; exact decoded colors and transparent-edge filtering; orientation/profile handling; natural-ratio sizing under min/max/flex constraints; cover clipping with padding/border; original/clone resize stability; bad/truncated/oversized asset diagnostics. This audit leaves every such empirical outcome to the private experiments and retains the existing runtime unchanged.
