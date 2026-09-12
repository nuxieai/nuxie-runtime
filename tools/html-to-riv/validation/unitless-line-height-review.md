# A08: unitless line-height on the immutable runtime

An ordinary Text plus an unpainted LayoutComponent wrapper can produce the tested CSS line-box heights without precomputing line counts or browser positions. The source/font-derived candidate also matches baseline placement for the 24px responsive and 32px inherited-font cases. A 16px case retains a 0.5px baseline error, and existing text-paint differences remain. A08 is an investigated prerequisite, not public text or typography qualification.

## Capability and source boundary

CSS unitless line-height inherits its numeric factor and is multiplied by the receiving element's font size; inherited lengths retain their computed length. CSS leading is divided around the font metrics. These are distinct from glyph ink bounds. See [CSS 2.1 line-height](https://www.w3.org/TR/CSS2/visudet.html#line-height) and [CSS Inline Layout](https://drafts.csswg.org/css-inline-3/#line-height-property).

At immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`, TextStyle's ordinary `lineHeight` property is an absolute float (default -1), not an inherited factor. `text.rs::make_styled` copies it into each shaping run; `Text::break_lines` and `line_breaker.rs::compute_line_spacing` retain the first real font ascent, then use custom ascent/descent divided by the font's ascent ratio. `paragraphSpacing` adds inter-paragraph distance and cannot correct the first baseline or the last line-box boundary by itself. These observations do not require runtime mutations.

For homogeneous runs, let A/D be positive native font ascent/descent, H=A+D, L the used line-height, and N the runtime line count. The observed native intrinsic height is `N*L + A*(1-L/H)`, with first baseline A. Ordinary wrapper padding `top=B-A`, `bottom=L*A/H-B` then yields height N*L and first baseline B, independently of N, when both pads are nonnegative. This is derived from the immutable algorithm, not a universal CSS metric rule. Text's LayoutParticipant supplies width and intrinsic height through native layout, so wrapping responds to each artboard resize.

The experiment reads the unchanged Roboto font's hhea/head metrics. Its `rounded-leading` hypothesis uses `B=(L+round(A)-round(D))/2`; `half-leading` retains fractional A/D. Neither reads browser output. The first hypothesis is deliberately falsifiable and fails at 16px. The example consumes a small typed source plan corresponding to independent HTML/CSS fixtures; it is not a new HTML parser or public compiler path. The plan retains the parent multiplier and child font size separately. Font provisioning, licensing/admission and public text syntax remain separate work.

## Captured results

Eight ordinary files use the original full licensed Roboto, a single unchanged black Fill, no stroke, no derivative font and no hinting adjustment. Each is imported once, cloned independently and resized through 240×160 → 390×200 → 768×120 → 240×160. Chrome is 153.0.8010.12. Geometry/baseline checks use the existing 0.1px tolerance; text pixel checks reuse the existing `pixels.mjs` text path and local limits.

| Source / ordinary composition | Metric passes | Pixel passes |
| --- | ---: | ---: |
| 16px, factor 1.5, direct Text | 0/8 | 0/8 |
| 16px, factor 1.5, rounded-leading wrapper | 0/8 | 4/8 |
| 32px child inheriting factor 1.5 from 16px parent, direct | 0/8 | 0/8 |
| Same inherited factor, rounded-leading wrapper | 8/8 | 4/8 |
| 24px, factor 1.5, responsive wrapping, rounded-leading | 8/8 | 0/8 |
| Same responsive text, fractional half-leading | 0/8 | 0/8 |
| 32px child inheriting the parent's 24px length, direct | 0/8 | 0/8 |
| 32px, factor zero, direct | 0/8 | 0/8 |

The two-line 32px inherited-number case needs L=48px, not its parent's 24px. Direct native intrinsic text height is 87.6875px; the wrapper measures 96px, matching Chrome. Its native first baseline is 29.6875px relative to Text; wrapper top padding 5.3125px produces the observed Chrome baseline 35px relative to the line-box owner. This is layout geometry, not a claim that glyph outlines paint identically.

The 24px responsive wrapper changes through **4 → 2 → 1 → 4** lines, with matching browser breaks and owner heights **144 → 72 → 36 → 144px**. All original/clone metric observations pass. The fractional-metric variant has the same correct outer heights but a 0.203125px first-baseline offset. At 16px, the rounded hypothesis produces 17.5px while Chrome's measured baseline is 17px. No correction was selected from browser measurements; that failure stays open.

Chrome's Typed OM retains `1.5` on the inheriting 32px child while computed style reports `48px`. The length control retains `24px` in both. Every source screenshot equals its independently authored explicit used-length control. Baselines are initially derived from observed Range fragments and canvas font-bound ascent. A separate zero-size inline-block marker cross-check preserves complete PNGs and element boxes for seven nonzero cases and confirms those baseline observations. Markers alter the zero-line-height case from 0px to 22px, so its marker readings are explicitly unusable; the first failed marker run and revised receipt are retained. The zero-height native-versus-Chrome box mismatch is measured on the unmodified document. Its baseline and line-count comparisons are explicitly skipped because collapsed DOM ranges do not provide an unambiguous oracle.

The initial driver awaited `document.fonts.ready` but did not record that the face was requested before metric capture. A separate Chrome-only recapture explicitly calls `document.fonts.load`, verifies `fonts.check` and the requested face's loaded status, then measures and screenshots. All 64 font-ready metrics and complete RGBA images exactly match the original references. Native files, streams and images were unchanged, so their evidence and completed visual review transfer by exact identity. Both reference runs remain preserved; the stronger receipt is `output/unitless-line-height-reference-r2/receipt.json`.

The direct zero case has native intrinsic height 29.6875px while Chrome's element height is zero. Positive-padding candidates for this case and the inherited 24px-at-32px case reject before output because their required pads are negative. That is a limit of this specific construction, not an immutable-runtime impossibility. Mixed inline sizes, runs whose dominant metrics change between wrapped lines, multiple fonts, empty/trailing lines, reduced leading, vertical writing and general whitespace/wrap rules are unqualified.

All six unscaled review sheets were inspected: 24 complete Chrome/native pairs plus 40 exact original/clone/restore transfers cover all 64 frames. The direct cases are visibly vertically displaced; wrappers improve placement, while native stem/counter coverage still differs. Only 16/64 metric frames and 8/64 pixel frames pass, with just four frames passing both. A pixel pass therefore cannot replace the independent baseline check. No paint or hinting setting was retried or tuned.

## Next integration boundary

Preserve a typed computed value that distinguishes inherited numbers from lengths; resolve a number only against the receiving font size. Keep native intrinsic text metrics, CSS line-box geometry, paint bounds and synthetic wrapper provenance separate. This can be designed without admitting text publicly. Before shipping this wrapper, isolate the 16px baseline quantization discrepancy with a source-backed rule, then qualify a bounded single-font/single-style positive-leading profile with explicit newline and responsive-wrap behavior. Reduced leading needs a distinct ordinary composition or a diagnostic. Inline mixed fonts, empty lines and broader line-breaking behavior require their own controls. One distinct next hypothesis is asymmetric/integer leading, for example `round(A)+floor((L-round(A)-round(D))/2)`. It matches the tested integer-baseline observations algebraically but has not been source-verified or implemented; it must not become a hard-coded 16px correction. Existing Text origin/bounds properties or metric-only embedded-font variants are other distinct future candidates. Text-paint and asset admission gates must pass independently.

`unitless-line-height-receipt.json` binds sources, immutable library/tool hashes, generated files, source/CSS controls, native metrics, all PNGs, visual coverage and preserved failures. Detailed artifacts are in `output/unitless-line-height-r1`; reproduce with the new example and `unitless-line-height-native.mjs`. No public source/tests, runtime, renderer, dependencies or shared support documentation were edited.
