# Substituted background keyword groups

Scope: the flat background shorthand grammar after custom-property substitution.
CSS Backgrounds specifies one repeat-style, one attachment, and up to two visual
boxes per layer, with color only in the final layer:
https://drafts.csswg.org/css-backgrounds/#propdef-background .

The repeat component accepts one repeat-x/y keyword or one/two contiguous
repeat/no-repeat/space/round keywords. Attachment permits one scroll/fixed/local.
Two border-box/padding-box/content-box keywords may be separated by other
components. Each comma starts a fresh layer; a layer need not contain an image.
These grammar checks do not add rendering support for valid excluded backgrounds.
Position, size, URL/image functions, clip-only keywords and vendor syntax remain
with existing profile diagnostics; this is not a complete background parser.

Chrome reference script background-keywords-reference.mjs passes18/18 direct
acceptance and substituted computed-style references. The public compiler red
regression produced unsupported-color for repeat repeat repeat. The fix makes
ten invalid combinations equal explicit unset and preserves errors for eight
valid excluded combinations. Existing RGB/HSL shorthand flattening also reaches
these checks without changing emitted colors.

Full module191/191 passes; native build passes. Two permanent fixtures cover
repeat, attachment, box multiplicity, separated repeat components, layer reset,
and a later background-color override. Native/WASM corpus parity passes (JS9/9), TypeScript passes, and the six new
Chrome/native geometry/pixel comparisons pass at240/390/768. Both comparison
sheets were inspected: reset transparency, nested marker layout, responsive
widths, and the later color override match. No tolerance changes.
The complete surrounding custom-property regression passed234/234. Every
source/browser/native image pair is identical to the previously reviewed
non-length-dimensions-custom baseline or the two newly inspected fixtures:
234/234 review transfers, no inspection queue. Session89455 is terminal.

This localized increment is qualified: module191, JS9 with native/WASM corpus
bytes/maps/requirements parity, TypeScript, Chrome18 references, new native6 and
surrounding234 pass. No renderer or tolerance changes. S09/S10 remain in progress
for remaining function/position/size/font grammar and other documented limits.
The last full renderer baseline remains1477/1477; this compiler change ran the
complete affected custom-property lane.


Evidence: /tmp/background-keywords-{red,module,build,wasm,types}.log and
output/playwright/html-to-riv/background-keywords-reference/results.json.

Durable logs and visual-inspection.json: output/playwright/html-to-riv/background-keywords/.
Surrounding pixels.log and baseline-comparison.json: background-keywords-custom/.
