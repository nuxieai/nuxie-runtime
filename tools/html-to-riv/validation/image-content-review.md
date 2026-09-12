# Image content-box validation controls

`validation/image-content.mjs` changes validation observations and presence controls only. It never changes compiler output, native layout, image assets, renderer input, or existing `pixels.mjs` tolerances. `validation/public-image-native.mjs` uses this helper and snapshots it with the other validation sources. The original padding r1 receipt and captures are unchanged.

## Content observation and sampling

The browser evaluator waits for all images to decode, then observes their **border boxes** with ResizeObserver and records each entry's `contentRect`. Observing the border box ensures a padded image with zero content width and height still produces an observation. A completely zero outer box that never produces an observation times out explicitly after two seconds.

The recorded page coordinate is the unscrolled border origin plus its border inset plus `contentRect.x/y`. ResizeObserver defines those offsets as padding-left/top ([W3C content-rect definition](https://www.w3.org/TR/resize-observer/#content-rect)). The receipt records the raw content rectangle, derived content box, border insets, method and coordinate mapping. Device scale must be one. Transforms/zoom, compositing opacity/filter/blend, fragmented boxes, non-solid paint, nonzero image borders and clipping ancestry are explicitly unsupported test conditions.

Presence samples only **fully contained visible integer pixel cells**: starts are rounded up, ends down, and the region is intersected with the screenshot. This excludes a following sibling's fractional edge pixel from the image's content sample. Receipts record the resulting region and whether the original sampled box was clipped by the viewport. A nonempty content box with no fully contained visible pixel does not pass vacuously.

## Nonempty and blank expectations

For nonempty content, the helper resolves the flat solid background under the image from recorded computed colors: the image's own color over successive ancestor backgrounds, stopping at an opaque backdrop. Every contributing ancestor must cover the complete sampled region. The receipt records the chain, colors/alphas, covering boxes, final rounded RGB and source-over calculation. Missing opaque termination, a partial background, or unsupported paint fails the control explicitly. This is a flat authoring-profile calculation, not a general backdrop/overlap compositor.

The original presence rule remains unchanged: ink is a pixel whose mean RGB distance from the established background exceeds 32; reference ink must exist, native ink must retain at least half its count, and ink bounds must stay within the existing one-pixel tolerance. A nonempty reference with no distinguishable ink is an unsupported control, not evidence that a missing native image is acceptable. An added `id:content` region uses the same fully contained content pixels in the unchanged `comparePixels` implementation.

If either observed content dimension is exactly zero, the expectation is instead **blank-content**. This finite control requires the image's own opaque solid background and a nonempty fully contained sample of its outer box. The reference must contain no ink against that background; native unexpected ink fails using the same threshold of 32. Small color-conversion differences do not become presence failures merely because bytes differ. The ordinary RGB gates continue to constrain those differences. A translucent/transparent own background remains unsupported for this zero-content control.

Both modes require opaque screenshot samples. Unsupported conditions are recorded in `imagePresence[id].unsupported`, with `passed:false` and `failure:"unsupported-control"`. A shifted or missing nonempty image uses `missing-or-misaligned-content-ink`; unexpected native paint in the zero-content control uses `unexpected-ink-in-empty-content`.

## Existing evidence and gates are retained

`borderBoxImagePresence` retains the **exact former algorithm**, including its floor/ceil border-box sampling and white background. `imagePresence` contains the new content/blank control. The returned `pixelRegions` includes every original observed box plus additional valid content regions.

Whole-image mismatch/mean-error checks and original outer/sibling regional RGB checks run unchanged. New content regions can add failures; they cannot remove original failures. A corrected image-presence verdict therefore does not erase an existing tail-local pixel failure. Empty content does not add a meaningless zero-area RGB region; its explicit blank control is accompanied by the ordinary outer/whole/sibling gates.

Reuse now loads and binds the prior raw receipt (`native-receipt.json` for `render`, otherwise `LABEL-receipt.json`). It requires no prior errors, the same Chrome version and output label, and identical probe, renderer and reset hashes. Before copying a native PNG, its prior per-frame result must equal its unique raw receipt row. Existing Rive, stream, geometry, instance/step/viewport and native PNG hash checks remain in place, as does the `transferredNative` pointer/hash. A read-only check confirmed all 176 padding r1 result rows match their raw receipt and all three tool/reset bindings still match.

## Finite tests and source checkpoint

`node --test validation/image-content.test.mjs` passes **17/17 tests**. They cover fractional sibling contamination; sparse missing and shifted images; width-zero, height-zero and both-zero content; small native color rounding in blank controls; unexpected sparse ink; ambiguous/nonblank references; viewport-contained sampling; preservation of original pixel metrics/failures; a deterministic ResizeObserver adapter with padding/border offsets and zero-content delivery; prototype-like IDs; region-name collisions; transparent-own and translucent-own backgrounds; multiple solid ancestor layers; insufficient/gradient backgrounds; and the zero-content own-opaque requirement.

The ResizeObserver adapter test uses a controlled mock to check delivery options and coordinate mapping. It is not a Chrome capture. Actual Chrome content observations, reuse of exact native captures, resulting counts and visual review belong to the parent's new `render-content-r2` run. This task performed no browser/native render and rewrote no saved result.

`node --check validation/image-content.mjs` and `node --check validation/public-image-native.mjs` also pass. Stable source hashes:

| File | SHA-256 |
| --- | --- |
| `validation/image-content.mjs` | `e20c5e37113e3d5941697d2227213ae05f644835dc69c1d690d40a6f8d397aeb` |
| `validation/image-content.test.mjs` | `524209c70f426221ad3589de966eb406391d636da4c4499b4224439af4737068` |
| `validation/public-image-native.mjs` | `20bdc4e1c74744832238b43c48051d0a6f71cc6b4c9723d3c797549095b486e5` |
| `validation/pixels.mjs` (unchanged) | `c072f999ffb77cabdb636f2bbc231a16352c3700cd9341858e5d9532357e9821` |

No `src/`, `tests/`, dependencies, runtime, renderer, build configuration or shared progress document was changed by this task.
