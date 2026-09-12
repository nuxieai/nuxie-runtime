# Empty ordinary variable values compute as unset

The public compiler now treats a successfully expanded ordinary value containing
no CSS value tokens as invalid at computed-value time and computes it as `unset`.
This happens after complete substitution and at the declaration's existing cascade
priority. It preserves the source, name and importance. Earlier declarations do not
return. CSS initial values remain distinct from the authoring reset: flex direction
becomes row, box-sizing becomes content-box and automatic minima become auto.

Custom empty values stay valid custom-property data. They inherit through computed
aliases and suppress consuming fallbacks. Empty fallbacks, whitespace/comment-only
primaries and nested empty fallbacks reach the ordinary-property recovery path.
An empty segment beside a nonempty literal or inside a color function does not make
the whole value empty. Resource and syntax checking finish before recovery.

The classifier uses CSS token exhaustion, not a Unicode string trim. Declaration
capture and ordinary-value boundary trimming now remove only CSS whitespace, so a
standalone NBSP or vertical tab remains a nonempty token. Empty quoted strings,
functions, blocks and comma tokens also remain nonempty. Existing ordinary-property
admission still handles these values. No general wrong-type classifier was added.

This is compiler-owned computation before ordinary Rive emission. Runtime, renderer,
schema, shared dependencies and build configuration remain fixed. The output needs
no variable sidecar, host evaluator, post-import setter or resize recompilation.
The semantic rule follows the [CSS Values invalid-substitution definition](https://drafts.csswg.org/css-values-5/#invalid-at-computed-value-time).
The [receipt](public-empty-variable-receipt.json) binds the implementation and evidence.

## Coverage and intentional boundaries

`public-empty-variable-forms.json` covers all **33 currently admitted property
names**, including box-sizing, across **seven token forms**: empty fallback, empty
primary, whitespace primary, comment primary, mixed whitespace/comments, nested
empty fallback, and inherited empty alias with a local override of its original
source variable. The 231 forms have independent explicit-unset controls.

Thirty names (210 forms) compile to exactly the corresponding control output.
The remaining three names—display, flex and flex-shrink—still reject all seven
forms because their CSS initial semantics are outside the current public profile.
The compiler does not replace unsupported initial semantics with its authoring
defaults. Twenty-seven additional strict controls retain wrong-type, unsupported
CSS, contextual, resource and syntax boundaries, including plain and escaped
non-CSS whitespace tokens in both primary and fallback positions.

The representative native matrix contains **26 visible scenes**, including the
exact two formerly rejected empty-value sources from the previous variable-recovery
corpus. That prior corpus and its receipts remain unchanged. This matrix covers
responsive auto width, intrinsic height, inherited color and font context, cascade
rollback prevention, importance, main/item alignment, shorthand/longhand ordering,
gap/padding/margin resets, order before file emission, DOM selectors after order
changes, initial min/max values, initial content-box, empty aliases suppressing
fallback, unused empty fallbacks and empty segments concatenated with literal or
function tokens. It does not claim native qualification of every combination of
all 33 properties.

## Verification and visual review

The initial focused Rust run is preserved: one rejection test passed and three
tests failed on the expected missing admission. The final frozen source passes
**237 Rust tests and 40 Node tests**, native/WASM builds and strict TypeScript.
Rust compares all 231 forms and 26 visible cases with independent controls at
three viewports. New transport coverage has **708 accepted CLI/WASM pairs** and
**49 identical diagnostic/no-output pairs** (21 unsupported unset forms, 27 strict
controls, one expansion resource control). The resource test uses a valid 40 KiB
custom value after an empty first component; two later copies still exceed the
64 KiB expansion bound and produce input-limit. All **668 prior outputs** reproduce
exact Rive bytes and source maps.

New Chrome 153.0.8010.12 characterization passes **231/231 exact CSSOM-style and
DOM-rectangle comparisons** against the explicit-unset controls at 240×160. This is
new computed-value evidence, distinct from historical grammar audits and native
qualification. Additional custom-property observations show that NBSP and vertical
tab retain their code points and suppress fallbacks as nonempty tokens.

The frozen public compiler produces each native scene once. The unchanged importer
loads only its Rive bytes; read-only source metadata joins observations afterward.
The original and clone both resize through 240×160 → 390×200 → 768×120 → 240×160.
All **208 geometry and pixel comparisons** and **416 alternate-clear controls** pass
against pinned Chrome using the existing RustMetal RasterOrdering path and unchanged
tolerances. The 624 expected background-color assertions cover every authored box
in every frame; all measured boxes fit the captured viewports.

All **28 distinct complete pairs** were inspected at original resolution in nine
sheets: every scene's first pair plus both larger responsive widths. The observed
layouts and paint match the intended reset/inheritance/concatenation behavior above,
including transparent backgrounds revealing teal, navy currentColor, auto widths,
48px font-relative width, reordered gold/navy boxes, side-by-side row children,
remaining longhand padding and the expanded content-box rectangle. No visible
geometry or painting difference was found in these pairs.

The remaining **180 pairs** transfer through exact complete decoded RGBA equality
after explicit white-canvas extension to a common size. This includes every source
pixel and verifies that regions omitted by a smaller canvas are white. Responsive
widths only match references from the same viewport. Every image is hash-checked
against the render receipt, and every sheet placement retains all source pixels
without cropping or scaling. This is not a same-dimension hash claim.

Gallery: `output/public-empty-variable-r1/render/gallery.html`.

## Preserved remaining work

S09/S10 remain partial. Successfully substituted nonempty wrong-type values still
need browser-correct invalid-at-computed-value recovery. Valid unsupported CSS,
input limits and unqualified runtime contexts must remain distinguishable from
such grammar failures. Existing fractional painting and content-box coordinate
limitations are unchanged by this bounded empty-value qualification.

A separate nonempty normalization counterexample is preserved in
`output/public-empty-variable-r1/nonempty-boundary/receipt.json`:
`--e:\u00a0red\u00a0;color:var(--e)` (actual NBSP code points around red) emits the
exact literal-red file in both the pre-change and current frozen compiler, while
Chrome computes inherited navy. Downstream property parsers still trim some
Unicode whitespace. This is a pre-existing compiler bug, outside the empty-token
classifier and not an immutable-runtime limitation. The receipt records browser
observations and exact compiler control bytes; it makes no new native-render claim.
