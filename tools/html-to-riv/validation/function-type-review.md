# Function value types after substitution

CSS Color functions produce colors; CSS Images functions produce images; numeric
math cannot produce a color or keyword. References:
https://drafts.csswg.org/css-color-4/ , https://drafts.csswg.org/css-color-5/ ,
https://drafts.csswg.org/css-images-4/ , https://drafts.csswg.org/css-values-4/ .

The pre-normalization structural scan now also checks known top-level types.
Color functions rgb/rgba/hsl/hsla/hwb/lab/lch/oklab/oklch/color/color-mix/light-dark/
contrast-color/device-cmyk are allowed in color and decoration color longhands,
background and text-decoration shorthands. Image functions (url, image, image-set,
cross-fade, element, paint and linear/radial/conic gradients including repeating
forms) are only compatible with background among currently supported properties.
Quoted and unquoted URL syntax are classified consistently. calc/min/max/clamp
are compatible with existing numeric length/factor/font/decoration slots and
background position/size, but cannot replace colors or enumerated keywords.

This is type invalidation, not new function rendering or argument validation.
Unknown functions remain profile diagnostics; nested calls are classified by the
outer function type. Compatible calls keep existing support/diagnostic behavior.
The mapping must be extended when new properties gain function-valued syntax.

Chrome96/96 references cover representative cross-type mistakes. A public
compiler test checks reset bytes and requirements, unrepresentable display and
box-sizing initial values, and compatible/unknown controls. Red output reproduced
unsupported-value for a color function used as width. Two permanent fixtures
exercise layout/background resets and inherited text/color/decoration. Remaining
qualification is now: module195/195, native/WASM builds, TypeScript, JS9/9
corpus bytes/maps/requirements parity and new native6/6 pass. Both sheets were
inspected: layout resets, inherited text/color and removed decoration agree with
Chrome, with minor existing glyph edge differences within unchanged limits.

The complete surrounding custom-property lane passed258/258. All258 source and
browser/native image pairs match previously reviewed background-position-custom
or the two new inspected fixtures. No inspection queue; session54924 is terminal.
This localized increment is qualified: module195, Chrome96, JS9/parity,
TypeScript, new native6 and surrounding258. No renderer or tolerance changes.
Last full renderer baseline remains1477/1477; this change ran the complete
affected custom-property lane. Compatible function argument grammar, unknown
functions/units, logical position syntax and documented boundaries remain open.

Evidence: output/playwright/html-to-riv/function-type-reference/results.json,
/tmp/function-type-{red,module}.log.

Durable logs/sheets/visual-inspection.json: output/playwright/html-to-riv/function-type/.
Surrounding pixels.log and baseline-comparison.json: function-type-custom/.
