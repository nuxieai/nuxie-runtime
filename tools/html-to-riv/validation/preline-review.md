# Pre-line whitespace

A18 adds whitespace normalization and an explicit per-Text runtime policy.
The public compiler test first failed with unsupported-value. It now verifies
canonical output, scalar br offsets after collapse, inheritance, normal/initial
resets, CRLF normalization and empty/whitespace-only block acceptance.

## Semantics and runtime boundary

The compiler collapses ASCII spaces/tabs inside each hard line and trims them
at its edges, while retaining LF and explicit br identities. Empty normalized
blocks emit layout without a Text object. Anonymous whitespace-only flex text
remains discarded. Preserved invalid ASCII controls still produce diagnostics.

The runtime must distinguish pre-line from pre-wrap: terminal Unicode spaces
hang unconditionally in pre-line. The implementation shares the existing
cluster-aware line breaker with a distinct occurrence policy. Pre-wrap keeps
conditional forced-end hanging. Source characters and shaped identities remain
intact. See the [CSS Text whitespace processing rules](https://www.w3.org/TR/css-text-3/#white-space-phase-2).

Initial code also removed trailing Ogham glyphs per the draft. Visual inspection
caught missing marks despite the aggregate pixel checks passing. The draft marks
Ogham trimming at risk, and pinned Chromium retains the marks. This implementation
therefore retains Ogham ink and includes its advance in alignment to match the
browser. The focused regional check additionally exposed that alignment
difference after ink was restored; this is fixed as well. The six Ogham cases now opt into
a presence check over browser-derived text ranges with a known solid background:
reference ink must exist and native ink count must be at least half of it, using
mean RGB contrast greater than 32 to identify ink. Each of the four observed
ink bounds must also match the reference within one pixel, so a shifted thin
mark cannot pass merely by retaining half its coverage. All existing color, geometry
and regional thresholds still apply. The new check failed all six initial cases;
its synthetic regression proves a sparse missing mark can pass averages but
cannot pass the presence control. Evidence: preline-ink-red/gallery.html and
/tmp/html-preline-ink-red.log. The first full run was interrupted at 186 passes
to fix this issue; its incomplete results are not a final gate.

The version-2 requirements envelope carries text-css-pre-line-v1 with
css-pre-line-v1. Version-1 claims, missing mappings and missing capabilities
are rejected. The checked host installs the policy before layout. Native/WASM
output includes matching metadata. Pre-line tabs collapse, so they need no
font tab or wrapped-tab capability.

## Validation scope

Twenty-four fixtures cover collapse, alignment, soft wraps, hard/blank/leading/
trailing lines, empty content, br, long words, NBSP/em spaces, signed spacing,
inheritance, CRLF, normal line height, visible Ogham separators, mixed modes,
intrinsic row sizing, hidden text and Open Sans ligatures. Each is compared
against pinned Chromium at 240/390/768px using the imported scene's runtime
layout. No geometry or pixel threshold is relaxed.

A runtime regression distinguishes pre-line forced-end hanging from pre-wrap,
checks switching policies, resizing and default occurrence isolation. The
existing measurement/font replacement regression now runs both wrap policies.
The checked-host regression exercises capability absence and malformed envelopes.
The public contract, full native/WASM corpus parity and TypeScript cover the
new public API and requirements shape.

## Commands and evidence

- Red compiler regression: /tmp/html-preline-red.log.
- Public contract: /tmp/html-preline-contract.log.
- Six CSS runtime regressions: /tmp/html-preline-runtime.log.
- Initial 20-fixture native-glyph probe: /tmp/html-preline-initial.log,
  output/playwright/html-to-riv/preline-initial/gallery.html; 60/60 pass.
- Full gate: CARGO_INCREMENTAL=0, review directory preline-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`;
  /tmp/html-preline-gate.log.
- Clippy and boundary: /tmp/html-preline-{clippy,boundary}.log.

Visually inspected all 72 pairs: fifteen initial contact sheets for the first
20 fixtures, three probe contact sheets for the added four compositions, and
three corrected Ogham sheets. The native/browser originals confirmed the sparse
missing-mark failure independently of the contact sheets. Corrected Ogham and
the synthetic presence regression pass all seven focused checks.

The final full run uses review directory preline-final and log
/tmp/html-preline-final-gate.log. Final Clippy is
/tmp/html-preline-final-clippy.log. The final full glyph gate is **755/756**, retaining only the prior A09
em-layout-cascade240 raster failure. All **72/72 pre-line comparisons pass**;
maximum geometry error is 0.00390625 CSS px (limit 0.1). All **675 prior native
PNGs remain byte-identical**. All 72 new browser/native pairs match the inspected
probe/corrected-Ogham artifacts; the first 54 non-Ogham pairs also remain
identical to the initial inspection.

Six CSS runtime regressions, 69 module Rust tests, five JS/WASM parity tests,
checked-host rejection, TypeScript, two gallery tests, Clippy and boundary pass.
The new sparse-ink synthetic test passes in the full browser gate. All 27
renderer state controls pass separately (/tmp/html-preline-state.log).

Durable evidence: preline-final/gallery.html, review.json (675 unchanged and
geometry/error summary), inspection-comparison.json (72 unchanged inspected
pairs), and preline-{240,390,768}-{0..5}.png contact sheets.
Vector result: **70/72**, with all geometry within 0.00390625 CSS px.
The mixed-mode scene fails at 240px (mean RGBA error 1.763011 > 1;
mismatch ratio 0.012070 > 0.01) and 390px (mean RGBA error 1.200447 > 1).
Both were visually inspected with their difference images: words are present,
line placement agrees, and differences follow glyph edges. No tolerances were
widened. A18 stays partial across renderer profiles; broader language coverage
and these vector rasterization differences remain open.

Vector command: NUXIE_NATIVE_GLYPHS=0, review directory preline-vector,
`npm --prefix tools/html-to-riv test -- --grep preline-
--output=test-results-preline-vector`; /tmp/html-preline-vector.log.
Artifacts: preline-vector/gallery.html, review.json and failures-{240,390}.png.
All six Ogham ink-presence/alignment checks pass in both profiles.

Next independent authoring item is A19 text transforms.
