# Current admission on the unchanged runtime

The initial public compiler is implemented; a qualified release is not yet claimed. “Admitted” below means accepted by the current compiler, not that every combination has completed browser/native qualification. All earlier backlog items remain subject to immutable-target requalification.

## Admitted public profile

| Area | Current admission |
|---|---|
| Input | Exactly `html`, `css`, finite positive `width`/`height`. Rust/CLI use unversioned CompileInput; JS adds `languageVersion: "nuxie-html-immutable-v1"`. |
| HTML | Nested `div`, `section`, `article`, `main`, `header`, `footer`, `aside`, `nav`. Attributes: `id`, `class`, `style` only. Whitespace text/comments do not paint. At least one box is required. |
| Identity | Explicit nonempty unique IDs, or generated `node/…` identities. Source map includes DOM element path and ordinary Rive object ID; it is not required for loading. |
| Layout | Column boxes under the explicit reset. Optional declarations `display:flex` and `flex-direction:column`, or `inherit` for either under the current fixed flex/column parent invariant. Width/height accept `auto`, nonnegative `px`, `em`, `rem`, `%`, unitless zero, `inherit`, `initial` and `unset`. Inherited sizes retain computed percentage/auto descriptors; initial/unset sizes become auto. Auto width lowers to parent-width stretch; auto height uses ordinary content sizing. |
| Font-size context | Nonnegative px/em/rem/percent, unitless zero and inherit/initial/unset. Used only for computing lengths; no text admission. Root font-size is explicitly16px. |
| Context restriction | Percentage height directly inside an auto-height authored parent is rejected pending a baseline encoding proof. Root percentage sizes use the artboard dimensions. |
| Solid colors | `color` and `background-color`: named sRGB colors, transparent, 3/4/6/8-digit hex, rgb/rgba and hsl/hsla forms accepted by the color parser. Channels lower to 8-bit ARGB. `color` inherits; it currently supplies currentColor rather than rendered text. |
| Background shorthand | `background` accepts one supported solid color, `currentColor`, `none`, `inherit`, `initial` or `unset`. It shares cascade ordering with background-color while remaining a separate shorthand operation for future constituent resets. Images/layers and mixed constituent syntax are rejected. |
| Color keywords | `currentColor`, `inherit`, `initial`, `unset` resolve for color/background-color. Background currentColor uses the final computed foreground regardless of declaration order. An inherited currentColor background resolves against the descendant's own final foreground; literal inherited colors remain literal. This is covered by validation/public-inheritance-receipt.json, including the preserved pre-fix failure. Width/height also admit inherit/initial/unset. Other layout properties require their own admission and proof. |
| Selectors/cascade | Type/class/ID/universal selectors; descendant, child and sibling combinators; supported attribute selectors; first/last/only-child; nth-child/nth-last-child including supported `of` selector lists; :not/:is/:where. Specificity, source order, inline style and !important participate. Dynamic states and pseudo-elements are not admitted. Selector parser support is not blanket native qualification. |
| Output | Ordinary LayoutComponent/LayoutComponentStyle and Fill/SolidColor objects in `.riv`, plus optional source map. An Artboard-owned white Fill encodes the fixed host background independently of canvas clear. Rust, CLI and WASM/JS interfaces are available. |

Admission is strict: the complete stylesheet is validated, including unmatched or overridden declarations. Unsupported declarations do not silently disappear merely because they would lose the cascade. Rules that match host `html` or `body` are rejected; style authored box elements. A universal rule can therefore be rejected because it also matches a host element. This differs intentionally from a permissive browser's error recovery.

## Intentional exclusions and unresolved work

- Text rendering, font assets, images, SVG/media and all asset input are not admitted.
- Borders, border-radius, padding, margin, gaps, flex rows/wrapping/grow/shrink, alignment, min/max sizing, aspect ratio and positioning are not in the public declaration whitelist yet. Ordinary-file experiments are not public support.
- Gradients, background-image/repeat, group opacity, transforms, clipping/overflow, decoration and blending declarations are not admitted.
- Custom properties/var(), calc(), relative units, at-rules/media queries and stylesheet nesting are not in the current public computed-style path. Parser-internal groundwork does not establish admission.
- CSS Grid, scripts, interactions, bindings, animation and editor integration remain excluded. No implicit raster, fixed-layout or recompile-on-resize fallback is enabled.
- No CSS runtime requirements, policy installation, custom glyph rendering mode, new stream command or shader extension can be required by emitted files. The 36 former custom capabilities are unavailable; another ordinary-file encoding may still prove a feature possible.

Resource bounds currently enforced: viewport dimensions in `(0,16384]`; combined HTML/CSS UTF-8 length at most 1 MiB; at most 8192 authored elements; nesting guard 128; sizes between 0 and 1000000; selector list/rule expansion limits 1024 and selector nesting 32. The WASM bridge additionally caps serialized requests at 192 MiB. Inputs exceeding the compiler's smaller authoring limits still fail compilation.

## Evidence status

The 15-case public baseline corpus and two multi-object color palettes have CLI/WASM byte/source-map parity at three viewports (51 corpus comparisons); rejected-style diagnostics and owned-buffer/ABI checks also pass. Strict TypeScript API checks pass. Native checks use the unchanged importer, same compiled original and clone, and pinned Chrome 153.0.8010.12; the current durable evidence is `validation/public-color-receipt.json`: 120 baseline frames plus 16 palette frames passed geometry/pixel gates. All six unique palette pairs were reviewed; 45 unchanged baseline pairs retained their reviews through exact-image transfer. These results establish the bounded corpus, not arbitrary combinations or a qualified release. A further 24 inheritance regression frames pass; 17 Rust tests and 7 Node tests (57 corpus parity pairs) pass. Final compiler binaries reproduce the exact bytes/maps of all 20 rendered scenes; see validation/public-inheritance-receipt.json.

Earlier ordinary-layout experiments preserve 56 geometry/50 pixel passes and 6 direct-corner failures; a bounded corner composition passed 8/8. Those experiments are not public corner admission. See `validation/ordinary-layout-review.md` and the immutable compiler/runtime/renderer audits for source mechanisms and unresolved alternatives. Reverted-runtime native outputs are historical evidence only.

The CSS-wide sizing/color corpus adds 17 scenes and 136 passing geometry/pixel frames, with 51 unique pairs visually reviewed and 85 exact repeated/clone transfers. See `validation/public-css-wide-receipt.json`. CSS initial display is inline and initial flex-direction is row; those computed contexts remain explicitly rejected rather than mapped to the authoring reset.

The current combined corpus is `validation/public-background-receipt.json`: 51 scenes pass 408 geometry/pixel comparisons and 816 cyan/transparent canvas-clear checks. All native files were rerendered after adding host paint; prior reviews transfer only through exact image/source/reset identity. The white page is part of the explicit authoring profile; transparent page backgrounds and host style overrides remain unadmitted. The earlier white-only runs did not prove canvas independence; their missing-host-paint control is preserved in `validation/host-background-control.json`.

Text remains unadmitted. The initial ordinary embedded-font experiment proves decoding and cloned glyph runs, but all24 comparisons from three placement/paint candidates fail local pixel fidelity. The one-glyph fixture is not a general font qualification; see `validation/ordinary-text-review.md`.

Expanded ordinary-font experiments pass 48/88 additional frames, including selected 32/48/64px samples, but fail smaller sizes. These are file-composition candidates only; public text remains rejected. See `validation/ordinary-text-review.md` for the matrix and residual limitations.

Font-relative box lengths are now admitted: width/height accept `em` and `rem`; `font-size` accepts nonnegative px/em/rem/percent and inherit/initial/unset as a compiler-side length context. The explicit root reset is16px. Font-size em/percent uses the parent; dimension em uses the element's final font-size; rem uses the fixed root. Inherited dimensions copy computed lengths. Specified coefficients and computed lengths are bounded by1000000. Other font keywords, text, font shorthand and font-relative values on unadmitted properties remain rejected. Eight scenes pass64 geometry/pixel frames and128 clear controls, with24 pairs inspected; see `validation/public-font-relative-receipt.json`.
