# Background position/size after substitution

CSS Backgrounds position and size grammar:
https://drafts.csswg.org/css-backgrounds/#propdef-background-position .
Flat background shorthands now validate contiguous physical position components:
one/two-value keywords or lengths, and three/four-value edge-offset groups.
Keyword pairs may reverse axes; numeric pairs fix horizontal/vertical order.
Center cannot receive an edge offset. A layer permits one position component.
An immediately following slash introduces one/two nonnegative length/percentage
or auto size components, or one cover/contain keyword. Each comma resets the
position slot. Negative position offsets remain valid; negative sizes do not.

This is invalid-value handling, not background positioning/sizing rendering.
Valid excluded forms retain diagnostics. Image/functions, logical positioning,
clip-only keywords, vendor syntax and unknown dimension units retain existing
profile handling; no complete background grammar qualification is claimed.

Chrome51/51 references pass against background-position-cases.json, including
valid reversed keywords, edge offsets, negative offsets, size/layer combinations
and invalid axis/size/ordering cases. Public compiler red test reproduced an
unsupported-color diagnostic for left right; invalid cases should instead equal
unset. Two permanent visual fixtures added. Full module194/194, native/WASM
builds, TypeScript and JS9/9 corpus bytes/maps/requirements parity pass. New
native6/6 resize comparisons pass and both sheets inspected: transparency,
root geometry and nested responsive markers match Chrome without visible issues.

The complete surrounding custom-property lane passed252/252. All252 source and
browser/native image pairs match previously reviewed substitution-block-custom
or the two new inspected fixtures. No inspection queue; session76636 is terminal.
This localized increment is qualified: module194, Chrome51, JS9/parity,
TypeScript, new native6 and surrounding252. No renderer or tolerance changes.
Last full renderer baseline remains1477/1477; this change ran the complete
affected custom-property lane. S09/S10 retain remaining image/function/logical
syntax and unknown-unit grammar work; no broader qualification claim.

Evidence: output/playwright/html-to-riv/background-position-reference/results.json
and /tmp/background-position-{red,module}.log.

Durable logs/sheets/visual-inspection.json: output/playwright/html-to-riv/background-position/.
Surrounding pixels.log and baseline-comparison.json: background-position-custom/.
