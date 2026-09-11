# Substituted font prefix groups

CSS Fonts shorthand grammar: https://drafts.csswg.org/css-fonts/#font-prop .
Flat substituted font values now check one style, small-caps variant, weight and
width keyword component, with normal filling unused slots (at most four total).
Keyword sizes continue into mandatory family and optional line-height validation.
Oblique may consume a single deg/grad/rad/turn angle in [-90deg,90deg]. Valid
styled faces, variants, width keywords and keyword sizes retain profile errors;
this does not implement their rendering. Nested functions/vendor syntax remain
under the broader substitution grammar audit.

Chrome24 references pass for duplicate/missing prefix components and inherited
font longhands. Public compiler red test preserved unsupported-font-shorthand
where Chrome inherits. An additional Chrome discrepancy is retained in
font-prefix-reference/grad-boundary-reproducer.json: 100grad mathematically equals
90deg, but Chrome153 rejects it in this shorthand. The compiler still diagnoses
this valid excluded styled font; no claim of complete angle boundary parity.

Two permanent fixtures exercise style/variant/width duplication, oblique angle
range and duplicated style, overflowing normal slots, inherited wrapping and a
later font-size override. Module192/192, native/WASM builds, TypeScript and
JS9/9 corpus bytes/maps/requirements parity pass. New native6/6 resize comparisons
pass, with both sheets inspected. Inherited text wraps and aligns with Chrome;
minor glyph edge differences remain within unchanged limits.

The complete surrounding custom-property lane passed240/240. All240 source and
browser/native image pairs are identical to the previously reviewed
background-keywords-custom baseline or the two newly inspected fixtures; no
inspection queue. Session13902 is terminal. This increment is qualified within
the stated scope and angle-boundary limitation. S09/S10 remain in progress for
function/position/size grammar and other documented custom-property limitations.
The last full renderer baseline remains1477/1477; this localized compiler change
ran the complete affected custom-property lane.

Evidence: /tmp/font-prefix-{red,module}.log and
output/playwright/html-to-riv/font-prefix-reference/results.json.

Durable logs/sheets/visual-inspection.json: output/playwright/html-to-riv/font-prefix/.
Surrounding pixels.log and baseline-comparison.json: font-prefix-custom/.
