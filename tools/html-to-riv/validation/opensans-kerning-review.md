# Open Sans legacy kerning with an empty GPOS table

Status: correction validated for the measured font and native glyph profile. This is shared text shaping, not ellipsis.

The existing OpenSans-Regular.ttf fixture has GPOS with an empty feature list,
plus a legacy kern table. The runtime previously disabled legacy kerning only
when the GPOS table was absent. HarfRust therefore applied fallback pair
kerning that the pinned Chrome reference does not apply.

Measured at24px using identical embedded font bytes and CSS shaping precision:
all individual character advances agree. Pairs `re` and `rd` each differ by
-0.48046875px. The failing full line has two of each; their -1.921875px sum
exactly explains the line-width discrepancy. `ffi` and `ffi long title` agree.
Evidence: `output/playwright/html-to-riv/opensans-pair-advances/pairs.json` and
`decomposition.json`. These are Chrome canvas width measurements compared with
the native outline probe; they establish advance behavior, not pixel parity.

A non-ignored runtime regression using the existing licensed Open Sans fixture
fails before the correction (`/tmp/html-opensans-empty-gpos-red.log`). The
suppression check now requires a GPOS kern feature, rather than merely a GPOS
table, before preserving that positioning path. Existing GPOS-kerning and
legacy-only tests remain required. AAT kerx behavior is unchanged.

Next checks: targeted runtime tests, native advance remeasurement, native/WASM
builds and publish parity, and the previously failing spacing/decorated pixels.
Do not infer broad rendering qualification until those results are recorded.

The focused runtime suite passes3/3 after the correction: legacy-only suppression,
empty-GPOS regression, and real GPOS kerning preservation. Log:
`/tmp/html-opensans-empty-gpos-green.log`.

## Validation after correction

- All measured character, pair and full-line advances match Chrome exactly;
  `pairs-fixed.json` preserves the results next to the failing observations.
- Open Sans ellipsis letter-spacing27/27 and decoration27/27 pass, including
  all seven previously failing comparisons. All14 changed native PNGs were
  visually inspected via six long-line sheets; other94/108 PNGs are identical
  to previously inspected runs. Identity receipts live in the output folders.
- All16 permanent Open Sans corpus fixtures pass at240/390/768 (48/48); all16
  comparison sheets were inspected. Wrapping, tabs, optional ligatures, underline,
  strikethrough and responsive card text agree. This includes the previously
  failing Open Sans strikethrough and undecorated240px controls.
- Native and compiler WASM builds, runtime host-free WASM check, and full accepted
  native/WASM publish-artifact parity pass.

Output folders under `output/playwright/html-to-riv/`:
`opensans-kern-fixed-spacing`, `opensans-kern-fixed-decorations`, and
`opensans-kern-fixed-corpus` (gallery.html). Original failing artifacts remain.
Logs: `/tmp/html-opensans-kern-fixed-spacing.log`,
`/tmp/html-opensans-kern-fixed-decorations.log`,
`/tmp/html-opensans-kern-fixed-corpus.log`, `/tmp/html-opensans-kern-parity.log`,
`/tmp/html-opensans-kern-runtime-wasm.log`.

Reproduce the pair probe using `underline-advance-components.mjs --check`,
`NUXIE_ADVANCE_FONT` pointing to OpenSans-Regular.ttf and `NUXIE_ADVANCE_TEXTS`
containing the texts from pairs.json. Reproduce the existing public ellipsis
controls with the Open Sans font environment plus `--letter-spacing-control`
or `--decoration-control`. No thresholds or font bytes changed. This does not
qualify all vector/DPR/host-state combinations or arbitrary script-specific
GPOS fallback behavior; those wider checks remain separate backlog work.

Full public compiler suite passes112/112 after the runtime correction;
log `/tmp/html-opensans-kern-module.log`.
