# A20 skip-ink hard-clip difference

Pinned reference: Chromium 153.0.8010.12. This is a retained renderer mismatch,
not a reason to change tolerances or bake device pixels into compiler output.

The new Japanese fixture has kana, fullwidth slash/underscore and Latin
`agypqj`, with 24px text, 2px red underline and -6px offset. At 240px, Auto and
All skip-ink leave two faint native red pixels near the final Latin glyph.
The ordinary image gate passes; the added red-decoration support check detects
the sliver. None passes. Auto/All also pass at 390 and 768.

At native image (131,72) the RGBA is (245,164,169,255), and at (131,73) it is
(245,165,169,255); Chrome has the aliceblue background (240,248,255,255) there.
The recorded path contains a surviving interval x=122.894653..123.337524 in
text-local coordinates, translated by the parent's 8px inset. Filling that
fractional rectangle leaves antialiased red coverage.

Chrome's [TextPainter](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/paint/text_painter.cc)
computes outline intercepts, dilates them horizontally by thickness (capped at
13px for this version), and removes their rectangles from the underline.
[GraphicsContext::ClipOut](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/graphics/graphics_context.h)
uses non-antialiased difference clipping for floating-point rectangles.
The [implementation](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/graphics/graphics_context.cc)
passes that antialiasing choice through to the Skia canvas.

The runtime currently subtracts intervals geometrically and fills the remaining
rectangles with ordinary antialiasing. Those operations are not pixel-equivalent
to clipping, especially when a surviving interval is smaller than a pixel.
The render API now exposes an optional `clip_out_rect` operation, and recording/
text-stream replay preserve it. NativeMetalFrame now implements this operation and passes five exact native
controls; other backends still decline it. Runtime underlines now use hard clipping where supported, retaining an explicit
geometric fallback after restoring partial clip state. A correct general fix must preserve device transform, DPR and
host clipping state; rounding in compiler or text-local coordinates would fail
under scale/translation. Do not erase the reproducer or force small intervals to
zero with an arbitrary local-space threshold.

Reproducers live in cases.json (`underline-cjk-auto`, `underline-cjk-all`).
Artifacts: `output/playwright/html-to-riv/underline-edge-cjk/` and
`tools/html-to-riv/test-results-underline-edge-cjk/`. The exact 240px source,
manifest, Rive bytes, draw stream, bounds and PNGs are retained there.
Downloaded pinned source copies: `/tmp/underline-pinned-text_painter.cc`,
`/tmp/underline-graphics-context.h`, `/tmp/underline-graphics-context.cc`.
A20 remains active; hard clipping is an internal renderer dependency to resolve.

## Hard-clip transport preparation

`Renderer::clip_out_rect(Aabb) -> bool` subtracts a rectangle with non-antialiased
coverage under the current device transform. Unsupported implementations return
false without changing state. The glyph adapter forwards the request. Recording
keeps it as `clipOutRect rect=[left,top,right,bottom]`, using round-trip float
precision rather than the ordinary short path-coordinate formatter: values on
opposite sides of a half-pixel boundary must remain distinguishable.

The text stream parser rejects wrong arity, malformed numbers and nonfinite
coordinates. Replay reports `UnsupportedOperation("clipOutRect")` instead of
silently ignoring it. Binary SRIV and backends other than NativeMetalFrame remain unsupported via the
default false result. Runtime text now records full stripes and hard clips; no compiler requirements
or accepted CSS syntax changed.

Tests exercise save/transform/clip/restore ordering, precise recording→parse→replay
round-trip, malformed inputs, rejected nonfinite recording without stream changes,
and explicit failure through an unsupported renderer. The render API and stream
suites pass (56 nonempty test cases total), `/tmp/html-hard-clip-stream.log`.

For the native implementation, map exclusions through the actual current device
matrix and preserve the existing host clip. Do not approximate difference with
intersect or round text-local rectangles. Skia's raster clip implementation
[uses device-rectangle rounding for scale/translation and a path for other
transforms](https://skia.googlesource.com/skia/+/83739ee0da1e/src/core/SkRasterClip.cpp).
That source is supplemental architectural evidence, not the pinned Chrome 153
revision; numeric/tie behavior still needs pinned-source and rendered checks.

The native-glyph host example and renderer-replay also pass `cargo check` after
the API addition (`/tmp/html-hard-clip-host-check.log`,
`/tmp/html-hard-clip-replay-check.log`). No image rerun was warranted for this
transport-only step: the runtime does not yet call the new operation.

## Native Metal device mask

NativeMetalFrame now maps each exclusion through the current device matrix and
builds its complement inside the current clip bounds from integer-aligned device
rectangles. It applies that mask with the identity matrix, then restores the draw
matrix while keeping the new clip. Existing host clipping and save/restore remain
owned by the source clip stack. Memory/work is bounded by current clip height;
performance qualification for many exclusions remains pending.

Four mask tests cover half-pixel boundaries, scale/reflection, shear, invalid and
disjoint rectangles. The first run caught inconsistent float/double rounding at
0.49999997; the fix promotes before adding 0.5. The corresponding
[Skia floating-point helper](https://skia.googlesource.com/skia/+/b988ee43e367/include/private/base/SkFloatingPoint.h)
also uses double precision for this rounding boundary. Chrome 153 DEPS resolves
Skia to `9d07e5bad9e3e21da2426946e589daa647218271`; fetching SkRasterClip.cpp at
that revision returned HTTP 404. Exact pinned general-affine raster rules still
need qualification against Chrome, rather than assuming this scan converter is
universally identical to Skia.

Run `node tools/html-to-riv/validation/hard-clip-control.mjs` after
`cargo build -p renderer-replay --features native-metal`. Five native controls
(fractional, scaled, reflected, rotated, overlapping) pass exact RGBA comparison
at all 1024 pixels each. They also exercise an existing host intersection clip
and a restored blue strip drawn outside that clip. All five native images were
visually inspected. Their expected images are hand-specified native semantics,
not Chrome screenshots; this is not Chrome underline qualification.

Artifacts: `output/playwright/html-to-riv/hard-clip-native/` includes streams,
expected/native PNGs and results.json. Logs: `/tmp/html-hard-clip-mask-tests-final.log`
(4 passing), `/tmp/html-hard-clip-native-build-final.log` (native-metal build passes),
`/tmp/html-hard-clip-native-pixels.log` (5 exact matches). Earlier failing test/build
logs are superseded by these final receipts. Runtime underline wiring, other
backend/wrapper support, and Chrome text/pixel qualification remain pending.

Runtime integration and current Chrome results are recorded in
`underline-hard-clip-review.md`: native-glyph 90/90, vector 85/90, host-state
17/27 per profile. These supersede earlier notes that runtime wiring was pending.
