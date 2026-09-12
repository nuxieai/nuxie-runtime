# Authored numeric tokens survive compilation

The compiler now preserves the authored numeric prefix when normalizing Number,
Percentage and Dimension tokens. Both ordinary declarations (including color
function arguments) and custom-property substitution use the same serializer.
Only decoded unit suffixes are normalized. The immutable runtime, renderer,
schema, dependencies and build configuration are unchanged.

The original minimized public case now emits width **999999.875**, rather than
**1000000**, directly in the ordinary Rive field. This repairs the compiler-only
normalization defect identified in [the prior diagnosis](content-box-rounding-review.md).
It does not establish that content-box padding, percentage resolution or world
rectangle projection is browser-equivalent at large coordinates.

## Token and resource contract

Signs, fractions and exponent spelling survive normalization. Integer consumers
still distinguish `1` from `1.0` and `1e2`, retain large signed integer values,
and reject values outside the admitted integer range. Nonfinite checks still
run before serialization. Variable expansion retains token separators.

Decoded unit names beginning with exponent-like `e` or `E` require an escape:
`1\65 2px` is a dimension with unit `e2px`; it must not become `1e2px` with
coefficient 100 and unit px. Case and decoded unit identity are preserved.
The shared scanner also supplies original numeric prefixes to provenance;
actual binary32 values and original decimal ideals remain separate facts.

Resource limits are unchanged. Long numeric expansions previously fit only
because normalization shortened their coefficients. Their full preserved text
now counts against the existing 64 KiB expansion and 1 MiB environment budgets,
and an overflow diagnoses without selecting a fallback or publishing output.
Optional provenance truncation is tested separately with escaped identifiers,
whose normalized native text remains compact without losing numeric precision.

## Compiler verification

The public regression suite was run before the repair: six of seven tests failed
on the intended semantic differences, while exact integer ordering passed.
The frozen repaired build passes **247 Rust tests, 41 Node tests and strict
TypeScript**, with native and WASM builds bound to contemporaneous source copies.
The independent public suite checks 31 cases at two viewports using exact float
bits and ARGB fields; 37 additional cases retain grammar/finite diagnostics.
New transport coverage compares 93 complete CLI/WASM outputs and 38 matching
no-output diagnostics, including a long numeric expansion resource case.

The shared helper checks 270 numeric-token combinations for token category,
value bits (including signed zero), decoded unit, integer spelling and idempotence.
The private flex-proof harness compiles with the added root module; this build
check does not requalify private layout proofs or rendering.

Intermediate failures remain recorded: old tests expected the removed rounding,
two provenance fixtures relied on numeric shortening, and explicit module lists
needed the new helper. The corrected tests preserve their intended contracts.
The complete green run uses a frozen native/WASM pair, avoiding mutable build
artifacts during parity and native comparisons.

## Historical output changes

Of 694 bound prior request/Rive/source-map triples, **691 remain byte-identical**.
The three changed outputs are the original direct, variable and fallback
numeric-provenance controls. All still compile and keep identical maps. Each
changes one width field from binary32 100.71399688720703 to
100.71428680419922, matching the preserved authored coefficient. Historical
outputs and the nonzero exact-regression result are retained. The [changed-output audit](public-numeric-regression-review.md) adds 24 passing
original/clone geometry and pixel comparisons with complete visual coverage.
The sources are transparent, so emitted fields and read-only geometry establish
the width correction. Exact historical-profile files produce the same geometry
and drawing streams as the rendered files after resizing; only their initial
Artboard dimensions differ. These three remain classified as corrected outputs,
not exact regressions.

## Remaining work

The 26 new visible scenes pass 208/208 geometry and 204/208 pixel checks; four
fractional-height failures at 240×160 remain. Ten preserved large-coordinate
scenes pass 48/80 geometry and 80/80 pixel checks. Those pixel passes do not
establish geometry correctness for large offscreen descendants. The [native evidence](public-numeric-native-review.md) records visual coverage
and supplemental local-width and color observations. Two supplemental fractional
controls add 16 geometry passes and four pixel passes, preserving 12 pixel
failures. All 304 native/control pairs have visual coverage: 65 directly
inspected pairs and 239 verified complete-RGBA transfers. The separate historical
output audit covers its additional 24 pairs.

Restoring native binary32 values from probe JSON confirms fixed child widths
999999.875 now match Chrome, with both zero and two million pixels of padding.
Percentage children with zero or right-side padding also match Chrome's local
and projected widths. Left/split padding retains a 0.0625px projected-rectangle
difference despite equal local widths. The two-million-padding outer size still
rounds to three million: subtracting padding loses content precision. The native
auto child is 1000000 versus Chrome 999999.875; the percentage child is 1000000
versus Chrome local 999999.8125 and rectangle 999999.75. These remain separate
composition/admission investigations, not a demonstrated impossibility.

The fractional-height source authors 25.499998092651367px. An exact 25.5px
control produces the same native images, while Chrome differs because the two
heights fall on opposite sides of its pixel boundary. Both browser sources and
their distinct failures are retained. Small color differences within the gates
are also recorded as actual interior RGBA histograms. Chrome's serialized color
strings are not assumed to equal its painted pixels, and neither renderer's
interiors are assumed perfectly uniform.

Fractional painting, nonempty wrong-type recovery and the NBSP normalization
counterexample remain open. This checkpoint does not claim general CSS numeric or color
arithmetic equivalence or promote a backlog feature on parser evidence alone.

The [checkpoint receipt](public-numeric-token-receipt.json) binds builds, tests,
source snapshots, output changes, native evidence and preserved failed attempts.
