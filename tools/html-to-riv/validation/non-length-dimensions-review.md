# Non-length dimensions after custom-property substitution

CSS Values separates lengths from angles, time, frequency, resolution and flex
fractions: https://drafts.csswg.org/css-values-4/ . Chrome153.0.8010.12 rejects
these dimensions in length slots; the 182-case reference in
`non-length-dimension-reference.mjs` verifies substituted computed values equal
`unset` across 14 properties and 13 units.

Public compiler red test reproduced unsupported-value for width:var(--x) with
--x:2deg. The classifier now recognizes deg/grad/rad/turn, s/ms, Hz/kHz,
dpi/dpcm/dppx/x and fr case-insensitively in length slots, including flex basis,
radius, font metrics and decoration metrics. Valid excluded lengths such as vw,
cqw, ch, lh and cm retain diagnostics. Unknown units and nested functions remain
under the broader grammar audit. This adds invalid-value behavior, not new units.

Two public Rust regressions pass, including embedded-font text, font and
decoration shorthand, all 13 units in layout, preserved excluded-length diagnostics
and min-width's unsupported-initial-value boundary. Early test iterations exposed
existing min-width:auto and unequal default flex-factor profile exclusions;
reproducers/logs retained. A later flex-shrink:0 declaration is used to make the
reset scene representable, and explicit expected CSS uses the same override.

Artifacts: output/playwright/html-to-riv/non-length-dimensions-reference/.
Compiler red/green logs and Chrome reference JSON retained. Two permanent visual
fixtures added: custom-non-length-layout and custom-non-length-text. Full module,
native/WASM parity and real native resizing/pixel qualification are pending.
Do not report this increment fully qualified yet.

## Qualification update

Full module190/190 and JavaScript9/9 pass; the accepted corpus (including new
fixtures) has native/WASM Rive, source-map and requirements parity. Native/WASM
builds and TypeScript pass. Both new fixtures compile once at390 and pass all
six Chrome/native geometry/pixel comparisons at240/390/768. Both comparison
sheets inspected: layout reset widths/padding/margins align; inherited text size,
spacing, wrapping and underline placement agree, with minor existing glyph
raster differences within unchanged limits.

Logs, gallery and sheets: output/playwright/html-to-riv/non-length-dimensions/.
Broader custom-property regression is running in session88854, log
`/tmp/html-var-dimensions-custom.log`, output `non-length-dimensions-custom/`.
Resume before restarting. Compare completed review.json against
layout-contract-full and non-length-dimensions; new source-identical image changes
would need inspection. No renderer changes or new supported length units.

## Increment qualified

Surrounding custom-property regression completed228/228. Every source/browser/
native image pair is identical to the explicitly reviewed layout-contract-full
baseline or the two newly reviewed fixtures:228/228 transferred, no inspection
queue. Evidence: non-length-dimensions-custom/baseline-comparison.json and
pixels.log. Session88854 is terminal; no tasks are running.

This qualifies known angle/time/frequency/resolution/flex units invalidating
substituted length slots in the documented profile. Module190, JS9/parity,
TypeScript, Chrome182 references and native new6 plus surrounding228 all pass.
The previous full native1477-check baseline remains the last full renderer run;
this localized compiler increment used the complete custom-property lane.
Unknown units, function grammar, and remaining complex background/font prefixes
remain S09/S10 work; no full custom-property grammar or vector fidelity claim.
