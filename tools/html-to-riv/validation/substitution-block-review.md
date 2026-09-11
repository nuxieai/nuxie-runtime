# Bare blocks after substitution

CSS substituted ordinary values must match the target property's grammar:
https://drafts.csswg.org/css-variables/#invalid-variables . None of the currently
accepted ordinary properties permit bare top-level parentheses, square or curly
blocks. These now reset to unset after substitution, before ordinary-value
normalization. Normalization previously rejected them as unsupported-css-value,
bypassing invalid-value processing entirely.

The scan uses CSS tokens, skips function contents, and does not interpret bracket
characters in strings as blocks. A bare block before or after a function still
invalidates the value. Function-only expressions retain existing profile errors;
this does not implement calc or validate nested function grammar. Custom-property
storage itself remains permissive. Display/box-sizing resets still report
unsupported-initial-value because inline/content-box are outside the profile.

Chrome65/65 references cover thirteen properties and five forms. A public compiler
regression checks reset Rive bytes and text requirements, unsupported initial
values, and function/string acceptance against direct declarations. Parser error
codes can differ by direct/substituted phase; the first green control assertion
incorrectly required identical codes and was corrected. Both logs are retained.
Full module193/193 now passes. Red output is
retained. Two permanent fixtures exercise responsive reset layout/background and
inherited text wrapping/decoration/color. Native/WASM builds, TypeScript,
JS9/9 corpus bytes/maps/requirements parity and new native6/6 comparisons pass.
Both sheets inspected: reset geometry/transparency and inherited text wrapping
match Chrome with minor existing glyph edge differences within unchanged limits.

The complete surrounding custom-property lane passed246/246. All246 source and
browser/native image pairs match the previously reviewed font-prefix-custom
baseline or the two newly inspected fixtures; no inspection queue. Session24052
is terminal. This localized increment is qualified: module193, Chrome65,
JS9/parity, TypeScript, new native6 and surrounding246. No renderer or tolerance
changes. Last full renderer baseline remains1477/1477; this change ran the
complete affected custom-property lane. S09/S10 still require broader function,
background position/size, and other documented grammar qualification.

Evidence: output/playwright/html-to-riv/substitution-block-reference/results.json,
/tmp/substitution-block-{red,module}.log.

Durable logs/sheets/visual-inspection.json: output/playwright/html-to-riv/substitution-block/.
Surrounding pixels.log and baseline-comparison.json: substitution-block-custom/.
