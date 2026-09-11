# Validation strategy

The oracle is Chromium rendering the same authored HTML/CSS, exact font/image
bytes and the versioned reset, with authored rules scoped to the fragment. The compiler never calls the browser to obtain
layout. Tests import generated `.riv` bytes into the actual Nuxie runtime,
resize that same scene and draw it through the real renderer.

Chrome/Chromium is the acceptance target. Firefox probes are optional research
only: they do not gate features, determine supported syntax, or block this
backlog. CoreText measurements likewise provide diagnostic evidence only.

## Clean checkout and retained evidence

The normal entry points are `bash tools/html-to-riv/validation/run.sh geometry`
(Linux or macOS) and `bash tools/html-to-riv/validation/run.sh native-glyphs`
(macOS Metal). They build the compiler, probe and WASM, install the pinned
Playwright dependency/browser, run contract tests and generate a fresh gallery.
Font/image fixtures and browser geometry oracles used by these tests are checked
in. The package test additionally compiles a gradient using an isolated tarball.

Historical receipts reference local `output/playwright/html-to-riv/` or ignored
`test-results*` evidence bundles. Those binaries, streams and screenshots are
not shipped in the source checkout. Audit, exact-transfer and renderer-migration
scripts require the original bundles and their hash manifests; they cannot
establish historical visual coverage from a fresh checkout alone. Missing or
changed evidence must fail an audit. Generate and inspect a fresh gallery when
the original bundle is unavailable; a successful geometry run alone provides no
native visual qualification. The dated records below preserve both passing and
failed experiments and do not override the current support contract.

For the broader renderer unit suite, set `RIVE_RUNTIME_DIR` to the fixture
checkout pinned by `.github/workflows/_trusted-macos.yml` and pass
`-- --test-threads=1`: ownership-trace tests share process-level tracing state.
An unconfigured parallel run is retained in the P06 integration evidence; its
five failures pass in the configured full run (485 passed, six ignored).

## Gates

1. **Compiler contract and binary integration:** `cargo test -p
   nuxie-html-to-riv --locked`. Tests cover deterministic output, authored IDs,
   selector/cascade behavior, unsupported syntax (including unmatched Grid
   declarations), repaired HTML, resource bounds and malformed assets. The
   accepted corpus must parse with no unknown/dropped records. Runtime tests
   inspect imported layout, verify text/image draw commands and update a mapped
   text run plus viewport without recompiling.
2. **Browser geometry:** compile every fixture at 390px, then import and resize
   at 240, 390 and 768px. Compare every source element's x, y, width and height
   against `getBoundingClientRect`, within **0.1 CSS px**. This catches a compiler
   that freezes one viewport. This CPU lane is suitable for normal Linux CI.
3. **Browser pixels:** replay the native draw stream using the real Nuxie Metal
   renderer, compare its PNG with a Chromium screenshot. Never substitute a
   stub, FFI reference renderer or browser screenshot for native output. Missing
   binaries/backend failures fail the suite. The explicit CPU geometry project
   is never reported as visual validation.
4. **Oracle sensitivity:** deliberately remove an 8×8 object from a mostly empty
   page, displace it, change its color and change image dimensions. The actual
   pixel comparison function must reject those changes, including its text mode.
   Dedicated hidden-content fixtures additionally require exact RGBA equality
   against a white reference so sparse leaked glyphs cannot hide in aggregate
   tolerances.
5. **Architecture:** the pure-runtime boundary gate rejects compiler dependencies
   from protected runtime crates. A WASM library check validates portability of
   the compiler dependency closure.
6. **JavaScript/WASM integration:** build the real release WASM, then run
   `npm run test:js`. Five tests cover native/WASM byte and source-map equality
   across the entire accepted corpus, browser/Node asset byte representations,
   diagnostics and recovery, repeated calls, independent instances and output
   ownership. `npm run typecheck` verifies the exported TypeScript API, including
   failure-result narrowing and language-version restrictions. The browser card
   cases compile in Chromium through the shipping JavaScript client before
   native import/render, so the visual gate covers browser-produced artifacts.

## Pixel tolerances

All gates apply; no single global average can hide a missing small object.

| Metric | Bound |
| --- | --- |
| Pixelmatch color threshold | 0.1 |
| Mismatched pixel fraction | ≤0.5% for shapes/images; ≤1% for text fixtures |
| Whole-frame mean absolute RGBA error | ≤1 on the 0–255 channel scale |
| Each source box mean absolute RGB error | ≤10 on the 0–255 channel scale |
| Each source box interior mean absolute RGB error | ≤6, excluding a one-pixel edge band |

Text fixtures exclude Pixelmatch-detected antialiasing pixels from the mismatch
count; channel means still include every pixel. Shapes include antialiasing in
the mismatch count. Interior checks keep solid colors strict while allowing
independent half-pixel edge coverage from Chromium and Metal. Small boxes with
no interior use their whole-box error. Rounded corners still count in the local
metrics. These are bounded rasterizer differences, not exact pixel equality.

## Reproduce

From the repository root, on a Mac with Metal:

```sh
bash tools/html-to-riv/validation/run.sh native-glyphs
```

On a CPU CI host:

```sh
bash tools/html-to-riv/validation/run.sh geometry
```

The script installs the lockfile-pinned test dependencies and Playwright's
Chromium, builds native and WASM binaries, runs Rust and JavaScript tests plus
TypeScript checks, then runs the selected
browser project. Linux CI installs Playwright system dependencies separately.
The acceptance lane uses the experimental native-glyph text profile. The
`native` lane selects vector text and retains documented pixel differences; a
pass in one profile does not qualify the other. CI runs `native-glyphs`.

The native script currently selects Metal; other native backends can be
qualified by building `renderer-replay` with the relevant feature and running
`npm test` with `NUXIE_HTML_BACKEND` and optionally `NUXIE_HTML_RENDERER`.
Do not claim a backend is qualified merely because it can be selected.

`test-results/` contains the input, `.riv`, source map, native geometry, browser
geometry, draw stream, screenshots, diff and metrics. CI uploads evidence even
on failure. Metrics record the browser version and renderer backend. Tests use
one worker, DPR 1, a fixed locale, reduced motion, and await fonts/image decode.
Assets are local and no network resource affects the visual oracle.

## Visual review gallery

Every Playwright run writes **`test-results/gallery.html`** and `review.json`,
including failed runs. Open the HTML file directly in a browser or download the
CI artifact and open it locally. Screenshots and report data are embedded; there
is no CDN, server or dependency on paths on the machine that ran the tests.

The gallery includes fixture search, width/status filters, Chromium/Nuxie/diff
panels, 100% pixel display, an opacity overlay, per-element and global metrics,
HTML/CSS, geometry, errors and the selected run command. Failed cases sort first.
Missing images stay visibly unavailable. Geometry-only runs are explicitly
labelled **not visually validated**, even when their tests pass. Nonvisual
sensitivity checks and setup failures appear under Run checks and errors.
Geometry assertions are soft so mismatches still produce screenshots and pixel
diffs; their failures still fail the test and process.

`npm run test:report` exercises two actual subprocess runs: a missing-renderer
failure and a geometry-only success. It verifies that both write reports, keep
their real exit/result status and never invent native images or pixel metrics.
The complete validation command runs these checks before the full browser suite.

A filtered run is useful while developing a feature:

```sh
cd tools/html-to-riv
npm test -- --grep 'wrapped-row'
```

It does not replace the full gate. The report shows its command and selected
check count to make that distinction reviewable. Reports describe automated
results; they never claim that a human/visual review has been completed.

## Required workflow for each compiler feature

1. Define the supported semantics and interactions, and the cases that must
   still be rejected. Add focused fixtures before changing the lowering.
2. Implement and run the focused browser/native cases. Exercise narrow, default
   and wide sizes, nested layouts, limiting sizes and text/image interactions
   where relevant; do not accept a single ideal example as sufficient evidence.
3. Run `bash tools/html-to-riv/validation/run.sh native-glyphs` without filtering.
4. Open the gallery and inspect every new or changed fixture at all three
   widths. Check side-by-side appearance, then 100% scale and overlay alignment.
   Inspect typography, edge shapes, clipping, spacing and small painted objects.
5. Fix unexplained differences. Changes to thresholds require a documented
   rasterization explanation and passing error-sensitivity tests; do not simply
   relax thresholds until a new feature passes.
6. Update SUPPORT.md only after both automated and visual acceptance. In the
   change/PR notes record the command, reviewed fixture names and widths, observed
   differences and any intentional limitations. Retain the CI review artifact.

This workflow applies to the standalone module. It does not require an editor
application or product publishing integration.

## Initial evidence and limits

The first flex extension adds nine accepted fixtures at three widths: weighted
growth/shrinkage, column basis with limits, auto basis from explicit dimensions,
percentage basis, inflexible basis, wrapping with limits, text reflow, shorthand
cascade and image participation. That increment brought the full suite to **110 tests**
(102 scene comparisons and eight pixel-gate sensitivity checks). Three failed
exploratory cases are preserved in `deferred-cases.json` and must be rejected by
the compiler until their semantics are implemented and visually qualified.

The initial native suite passed **83 tests**: 13 hand-authored scenes plus 12
deterministically generated combinations at three widths (75 comparisons), and
eight pixel-gate sensitivity tests. It covers nested layout, percentages,
padding/margins/gaps, min/max dimensions, wrapping, alignment, cascade, hiding,
rounded/alpha paints, embedded PNG, Unicode Latin text and text wrapping. The
initial measured browser is Chromium 153.0.8010.12 with the Nuxie Rust Metal
backend, DPR 1, and the checked-in Inter regular font.

This is a regression corpus, not exhaustive CSS conformance. Unqualified areas
include other browsers, DPRs, native GPU backends, arbitrary fonts, complex
scripts/bidirectional text, very deep/wide layout interactions and performance
under maximum limits. New support needs both positive cases and explicit
rejection cases, at multiple viewport sizes, before changing SUPPORT.md.

The normal CI workflow runs Rust, WASM/JavaScript integration, TypeScript and
browser geometry.
The native Metal pixel job runs automatically for matching PRs and main pushes,
as well as dispatches (unless the dispatch explicitly disables it). Path filters
cover the module, schema, runtime, rendering API, renderer and replay tool.
Both jobs upload the gallery and evidence even on failure. Branch-protection
settings are outside this change: running the job does not by itself make it a
required merge check. **A geometry-only result is insufficient for compiler
release acceptance.** Missing or cancelled native runs are not passing evidence.

## RGB function extension

Four RGB fixtures cover legacy and modern syntax, mixed modern channel units,
layered alpha, clamping and fractional rounding, text inheritance and important
color declarations. Each runs at 240, 390 and 768px through the existing native
pixel gate. That increment brought the suite to **122 tests** (114 scene comparisons
and eight pixel sensitivity checks). Public compile-contract tests compare
known equivalent hex/RGB inputs and reject malformed and deferred syntax in
both inline styles and unmatched rules. The native/WASM parity test includes
all four new fixtures. No pixel threshold changed for color support.

## Current color extension

Three fixtures add nine browser/native comparisons at the standard widths for
`currentColor`: cascade and inline/important overrides, nested text and fill
inheritance, default black and transparency. That increment brought the suite to
**131 tests** (123 scene comparisons and eight sensitivity checks). The public
compile contract also compares current-color designs with equivalent explicit
colors. Native/WASM parity covers all three fixtures.

## Text alignment extension

Four fixtures add 12 native comparisons for centered/right-aligned wrapped
paragraphs, left overrides, inherited and important alignment, and flexible
text boxes resized after import. That increment brought the suite to **143 tests**
(135 scene comparisons and eight sensitivity checks). Public compile tests
verify inherited/explicit alignment equivalence and reject deferred alignment
syntax even in unmatched rules. All fixtures also enter native/WASM parity.

## Named and HSL color extension

Five qualified fixtures add 15 comparisons: all 148 named colors, named
text/currentColor, HSL hue/unit/clamping combinations, HSL text/alpha/cascade,
and intrinsic row text sizing. That increment brought the accepted suite to **158
tests** (150 scene comparisons and eight sensitivity checks). Public compiler
tests use known sRGB equivalents and aliases, reject system colors and
malformed/deferred HSL forms, and native/WASM parity includes all new fixtures.

## Executable known visual gaps

`validation/known-gaps.json` preserves accepted-input compositions that fail the
unchanged pixel gate. They are open work, not qualified fixtures or expected-pass
tests. Run the same browser/native comparison workflow against them:

```sh
cd tools/html-to-riv
NUXIE_HTML_KNOWN_GAPS=1 \
NUXIE_HTML_REVIEW_DIR=../../output/playwright/html-to-riv/known-gaps \
npm test -- --output=../../output/playwright/html-to-riv/known-gaps/artifacts browser.spec.mjs
```

This command currently exits nonzero and creates a separate failure gallery.
It is deliberately separate from the accepted-profile regression gate, so a
passing default gate does not claim that these compositions pass. Q09 remains
open until its realistic corpus is qualified. Unlike deferred-cases.json,
these are rendering gaps in accepted input, not expected compiler diagnostics.
See `validation/known-gaps.md` for findings and the next investigation.

## Explicit inheritance extension

Six fixtures add 18 comparisons for spacing/shorthand cascade, inherited
percentage dimensions, typography, currentColor backgrounds, flexible sizing,
and root inheritance from the fixed host. That increment brought the accepted suite to
**176 tests** (168 scene comparisons and eight sensitivity checks). Public
compile tests compare inherited values with their explicit equivalents across
supported layout/paint properties and reject malformed or unsupported uses.
Native/WASM parity includes all six new fixtures.

## Initial and unset extension

Five fixtures add 15 comparisons for CSS layout defaults versus the profile
reset, size-limit removal and transparent paint, initial typography, inherited
unset values, and flex longhand overrides. That increment brought the accepted suite to **191
tests** (183 scene comparisons and eight sensitivity checks). Compiler-contract
tests compare explicit equivalents and require diagnostics for initial values
outside the profile, including unmatched declarations. All five fixtures enter
native/WASM parity.

## Solid background shorthand extension

Three fixtures add nine comparisons covering shorthand/longhand precedence,
alpha, none/initial/unset resets and inherited currentColor backgrounds. That increment brought the
accepted suite to **200 tests** (192 scene comparisons and eight
sensitivity checks). Public compile tests verify equivalent emitted scenes and
strict rejection of unsupported background layers. Native/WASM parity includes
all three new fixtures.

## Unitless line-height extension

Three fixtures add nine comparisons for inherited multipliers at different font
sizes, fixed-pixel versus relative line-height, important declarations and unset.
That increment brought the accepted suite to **209 tests** (201 scene comparisons and eight
sensitivity checks). Public compile tests compare with independently specified
pixel equivalents and reject invalid multiplier values. All fixtures participate
in native/WASM parity and reuse imported scenes at the standard viewport widths.

## Font shorthand increment: qualification incomplete

Three fixtures add nine comparisons for weight/size resets, inherited unitless
line-height, important precedence and inherit/unset. The expanded suite contains
218 checks. Initial result: **216 passed, 2 failed**. The mixed-size shorthand
fixture fails pixel limits at 240 and 390px; geometry passes. These fixtures remain in the default suite. Their failures were subsequently
fixed by the baseline correction below; no threshold was widened. Native/WASM parity and 28 Rust tests pass.
See validation/font-shorthand-review.md for evidence and the next investigation.

## Text baseline correction

The baseline-correction suite passed **224 checks** (216 scene comparisons and
8 sensitivity checks). The former font-shorthand failures pass unchanged.
Single-glyph and fractional-size fixtures extend the baseline coverage; a public
contract regression preserves acceptance of tight 22px/27px leading. The full
run also passes 28 Rust tests, native/WASM parity, gallery and TypeScript checks.
Module Clippy passes with warnings denied. See validation/text-baseline-review.md.

The separate known-gap suite now contains nine checks: seven fail and two pass.
It adds a minimized Hg rasterization case while preserving both original
compositions. These are real failures with unchanged acceptance thresholds.
The diagnostic text-diagnosis.mjs script reports control experiments separately;
its antialiased browser control is never used as an acceptance reference.

## Normal line-height: implementation awaiting pixel qualification

The expanded main suite contains **233 checks: 230 pass and 3 fail**. All nine
new normal-line-height geometry comparisons pass. Pixel failures remain in
font-shorthand-normal-resets at 390px and normal-line-height-cascade at 240/390px.
They are kept in the default corpus; the main gate currently exits nonzero.
No fixture or tolerance was changed to qualify the feature. All 29 Rust tests,
JS/WASM parity, gallery, TypeScript and module Clippy checks pass. See
validation/normal-line-height-review.md for measurements and controls.

The diagnostic `validation/text-path-control.mjs` replays four bounded native
path streams in Chromium Canvas to isolate font rendering from path rendering.
It reports four passing Canvas/native controls and three failing DOM/Canvas
controls; these are diagnostic evidence, not additions to the accepted count.
See validation/text-rasterization-seam.md. The main gate remains 230/233 passing.

The diagnostic runtime probe also exports glyph IDs/positions/transforms for
`native-glyph-probe.swift`. Its CoreText control passes 12 smoothed comparisons
across three widths, without changing the main renderer. This is not production
qualification; see validation/native-glyph-review.md for commands and the open
transparent-mask/compositing work.

`validation/run-glyph-mask-controls.sh` is a separate experimental gate for
transparent native glyph masks through actual Metal image composition. It
passes 48 comparisons across three widths and four backgrounds, and exits
nonzero on a failed comparison. See validation/glyph-mask-review.md. It does
not replace the main compiler gate or qualify the unintegrated glyph backend.

`validation/run-glyph-mask-controls.sh rust` exercises the opt-in Rust/CoreText
rasterizer with bounded run masks instead of the Swift full-frame control. Its
48 browser/Metal comparisons and five native resource/position/color tests pass;
36 render API tests also pass. See validation/rust-glyph-mask-review.md for
that historical increment. Live invocation and caching now have their own lane:

`validation/run.sh native-glyphs` builds the opt-in adapter and runs the complete
compiler/native/WASM/browser gate. It passes 233/233 checks; the existing known-gap
specimens separately pass 9/9 with `NUXIE_NATIVE_GLYPHS=1`. Galleries label this
profile explicitly. `run.sh native` retains the original vector profile and
its recorded pixel gaps. See validation/live-glyph-review.md for commands, cache
and compatibility tests, visual inspection and remaining state/platform work.

The native-glyph lane additionally runs 27 live host-state controls for clips,
transforms, opacity and restoration. These supplement the shared pixel metrics
with white-background ink coverage, after visual review exposed double-applied
opacity that sparse-region averages missed. All 27 pass after baseline rounding
and opacity normalization fixes. See validation/glyph-state-review.md for
reproduction, before/after evidence and limits; this does not add CSS syntax.

## Em lengths (A09, partial)

The expanded experimental macOS glyph lane passes **247/248** main checks;
all 15 new geometry comparisons pass. Its 27 host-state controls pass in a
separate invocation because the main script stops on the pixel failure.
The default vector profile passes 12/15 new-only comparisons. There are
44 passing Rust tests, five JS/WASM tests, two gallery tests, TypeScript and
module Clippy. The failing fractional shape-edge fixture is preserved in the
main corpus, with a public compiler test proving byte identity to independently
authored px values. No reference reset or tolerance changed. Browser/native/diff
images were reviewed for all five new fixtures at all three widths.
See [em-review.md](validation/em-review.md) for commands and remaining failures.

## Root-relative lengths (A10)

The fixed 16px host-root contract passes all 12 new geometry and pixel checks
in both renderer profiles. Full experimental glyph lane: **259/260**, with only
the preserved A09 fractional-edge failure. All 45 Rust tests, five JS/WASM
tests, two gallery tests, TypeScript, Clippy and boundary validation pass;
host-state controls separately pass 27/27. New fixture images were visually
inspected at 240/390/768px; nine shape images are identical between profiles.
Default vector validation here is new-only. See
[rem-review.md](validation/rem-review.md) for scope, commands and evidence.

## Percentage min/max dimensions (A11)

All 24 new geometry/pixel checks pass in both renderer profiles. Full experimental
lane: **283/284**, retaining the A09 fractional-edge failure. Validation includes
48 Rust tests, native/WASM parity, 41 existing layout tests, one pinned exact
render comparison and 27 renderer-state controls. A flex-basis measurement bug
was reproduced with both px and percentage caps, then fixed in the vendored
layout engine by clamping the known cross size before measurement. Native
minimum/maximum text regressions and independent browser measurements cover the
change. All new visuals were inspected. See
[percentage-limits-review.md](validation/percentage-limits-review.md).

## Letter spacing (A12, partial)

Full experimental glyph gate: **307/311**. All 27 new source-box geometry checks
pass; 24/27 new pixel checks pass in both profiles. Multi-mark text fails at all
three widths because native spacing is applied per shaped glyph. The existing
A09 shape-edge failure remains. All 49 Rust tests, native/WASM corpus parity,
gallery checks, TypeScript, Clippy, boundary validation and 27 renderer-state
controls pass. New glyph-profile visuals were inspected in full. See
[letter-spacing-review.md](validation/letter-spacing-review.md) for measured
failures and the required compatibility investigation.

## Cluster-spacing host experiment

`NUXIE_CLUSTER_SPACING=1` selects an explicitly labeled experimental font mode in
the probe. Its full glyph gate passes **310/311** and its new-only spacing gate
passes 27/27 in both renderer profiles. Only the three multiple-mark PNGs differ
from the prior default full gallery; 300 other scene PNGs are identical. Those
corrected images were visually inspected. The default Rive interpretation remains
unchanged, so this does not yet qualify A12's shipping compiler-output contract.
See [cluster-spacing-review.md](validation/cluster-spacing-review.md).

## Manifest-driven runtime policies

Compiler output now includes versioned runtime requirements, with native/WASM
parity and a CLI sidecar. The checked host validates these before import and
retains cluster mode through font replacement. Full glyph gate **310/311**;
52 Rust, five JS/WASM, one checked-host rejection test, two gallery tests,
TypeScript, Clippy, boundary checks and 27 renderer-state controls pass.
New-only vector spacing passes 27/27. All 303 native PNGs equal the prior
experimental-mode gallery. See
[runtime-requirements-review.md](validation/runtime-requirements-review.md).

## Optional ligatures and CSS spacing capability

A new versioned CSS spacing policy preserves the older cluster-only contract.
The full glyph gate is **322/323**, retaining only A09's fractional-edge failure.
The twelve real-ligature comparisons pass; the combined vector spacing corpus
passes 39/39. All 303 previous native scene PNGs are byte-identical. All twelve
new pairs were visually inspected at 240/390/768px. All 53 Rust tests, five
JS/WASM tests, the checked-host test, two gallery tests, TypeScript, Clippy,
boundary validation and 27 renderer-state checks pass. See
[optional-ligatures-review.md](validation/optional-ligatures-review.md).

## Word spacing

Signed px/em/rem word spacing now maps to native runs and preserves responsive
layout. All 33 new geometry checks pass; 31/33 pixel checks pass in both renderer
profiles. The two failures expose fixed-width Unicode-space wrapping at 240px,
including a zero-word-spacing control. Full glyph gate: **353/356**, including
the retained A09 edge failure. All 315 prior native PNGs are unchanged; all 33 new
pairs were visually inspected. All 56 Rust tests, five JS/WASM tests, checked-host,
gallery, TypeScript, Clippy, boundary and 27 renderer-state checks pass. See
[word-spacing-review.md](validation/word-spacing-review.md) for the line-breaking
reproducer, plural source-map contract and next compatibility investigation.

## Preserved Unicode-space break capability

The checked runtime capability now preserves fixed-width spaces in word fitting
and breaks after them, fixing A13's two wrap failures. All 48 focused comparisons
pass in both renderer profiles. A newly exposed visible Ogham glyph admission
bug now produces an explicit rejection with Inter; the positive test embeds
licensed Noto Sans Ogham. Full glyph gate: **397/398**, retaining A09. Only two of
348 prior native scene PNGs change; all new and corrected pairs were visually
inspected. All 59 Rust, five JS/WASM, checked-host, gallery, TypeScript, Clippy,
boundary and 27 renderer-state checks pass. See
[preserved-space-breaks-review.md](validation/preserved-space-breaks-review.md)
for failed hypotheses, final policy, font provenance and deferred controls.

## Explicit br and hidden-content regression

A14 maps explicit breaks in text-only block containers to native newlines with
break identities and Unicode-scalar source offsets. Visual review exposed hidden
text drawing despite passing aggregate metrics; the compiler now hides emitted
drawables throughout display:none subtrees, and twelve exact blank-image controls
prevent recurrence. All 48 new comparisons pass in both renderer profiles.
Full glyph gate **445/446**, retaining A09. All 390 previous PNGs are unchanged;
all new pairs were visually reviewed. All 62 Rust tests, five JS/WASM tests,
checked-host, gallery, TypeScript, Clippy, boundary and 27 renderer-state controls
pass. See [explicit-breaks-review.md](validation/explicit-breaks-review.md).

## White-space nowrap (partial)

A15 accepts inherited normal/nowrap and retains explicit breaks. The new corpus
passes **26/30** in both renderer profiles; narrow centered/right overflowing
text differs from Chromium in block and flex containers. Full glyph gate:
**471/476**, including the existing A09 failure. All 438 prior native scene PNGs
are unchanged. All 30 new browser/native pairs were visually inspected. All
63 Rust tests, five JS/WASM tests, checked-host, two gallery checks, TypeScript,
module Clippy, boundary and 27 renderer-state controls pass. The four retained
failures require an explicit, compatible runtime overflow-alignment policy.
See [nowrap-review.md](validation/nowrap-review.md).


## Qualified nowrap overflow alignment

The opt-in `text-css-nowrap-alignment-v1` runtime policy fixes all four A15
failures while preserving short-line alignment after br and legacy Rive behavior.
All **36/36** nowrap comparisons pass in both renderer profiles. Full native-glyph
visual gate: **481/482**, with only the unchanged A09 failure. Exactly four prior
native PNGs change (the corrected failures); the remaining 464 are byte-identical.
All 36 browser/native pairs and difference images were visually reviewed.

The standard run script now includes the runtime regression for opt-in behavior,
repeated resize, wrapped text, policy disable and fresh default occurrences. It
passes alongside all 63 module Rust tests, five JS/WASM tests, checked-host tests,
TypeScript, two gallery tests, module Clippy, boundary and 27 renderer-state
controls. See [nowrap-review.md](validation/nowrap-review.md) for commands,
intermediate failures and final receipts.


## Preserved whitespace pre (partial)

The non-tab pre subset passes **48/48** comparisons in both renderer profiles.
Tests cover leading/trailing/repeated spaces, source newlines and blank lines,
space-only/newline-only block text, CRLF, br identities, alignment, spacing,
anonymous flex whitespace and inherited indentation. The flex controls first
failed 9/12 and then passed after whitespace-only anonymous flex items were
removed from layout. All 48 browser/native pairs and difference images were
visually inspected.

Full native-glyph gate: **529/530**, retaining A09 only. All 474 previous native
scene PNGs are byte-identical. All 65 module Rust tests, the runtime policy
regression, five JS/WASM tests, checked-host, TypeScript, two gallery tests,
module Clippy, boundary and 27 state controls pass. The retained tab rejection
was also tested independently after adding it to deferred-cases.json.
A16 remains partial until position-dependent tabs are implemented. See
[pre-review.md](validation/pre-review.md).


## Default preserved tabs

A16 now includes default eight-space tab stops through the checked
text-css-tabs-v1 capability. All **33/33** tab comparisons pass in both renderer
profiles. Coverage includes source/ink preservation, newlines/br, leading and
trailing tabs, consecutive stops, near-stop behavior, alignment/overflow, word
and letter spacing, font option/replacement retention, and Inter/Open Sans.
All 33 browser/native pairs and difference images were visually reviewed.

Full native-glyph gate: **562/563**, retaining A09 only. All **522** prior native
scene PNGs are byte-identical. All 66 module Rust tests, the runtime regression,
five JS/WASM tests, checked-host, TypeScript, two gallery tests, module Clippy,
boundary and 27 state controls pass. Custom tab-size remains unsupported. The
minimum near-stop advance intentionally follows pinned Chromium rather than the
newer CSS draft's half-ch rule. See [tabs-review.md](validation/tabs-review.md).


## Tight text-region checks

Whole-scene and element averages can miss a short word disappearing inside a
large text box. A visual review of prewrap-forced-hang exposed this blind spot.
Fixtures can now declare textRegions with an element id and UTF-16 start/end
offsets in its first text node. Browser DOM Range supplies the pixel rectangle;
it uses the existing regional RGB and interior limits without changing geometry
or whole-image tolerances. Invalid/empty ranges fail explicitly, and metric
artifacts retain the rectangles and errors. The missing-word control now fails
at 240px and passes at 390/768px in the native-glyph profile. The tighter check
also exposes two vector-only rasterization failures at the wider sizes. See
validation/prewrap-review.md.


## Pre-wrap direct mapping (partial)

Final native-glyph visual run: **597/605**. A09 remains; seven new pre-wrap
failures cover extra blank lines from trailing/all-space text, emergency splits
inside long words, and a missing word before a forced break. Focused results:
**35/42 glyph**, **33/42 vector**. The additional two vector failures are local
rasterization errors revealed by the tight word-region check; text remains
visible there. None of these failures is exempted or hidden by wider limits.

All **555** prior native images are unchanged. All 42 browser/native pairs and
difference images were inspected, including original images for the missing
word and multiline cases. The final text-region rerun produced exactly the same
597 browser/native image pairs as the initial run; only verification became
stricter. All 67 module Rust tests, runtime regression, five JS/WASM tests,
checked-host, TypeScript, two gallery tests, Clippy, boundary and 27 renderer
state controls pass. Wrapped tabs remain explicitly rejected. A17 is partial;
see [prewrap-review.md](validation/prewrap-review.md).

## Checked occurrence policies

Runtime requirements version 2 now identifies individual Text objects. The
existing nowrap/pre policy exercises this path; version-1 manifests retain their
published behavior. New contract and checked-host tests reject invalid mappings,
and a mixed normal/nowrap/pre/pre-wrap scene records identical draw commands
under the old and new envelopes. All 68 module tests and native/WASM parity pass.
The full visual result remains 597/605 with exactly the existing eight failures;
all 597 image pairs and their bounds are unchanged. See
[text-policy-review.md](validation/text-policy-review.md). A17's actual pre-wrap
line-breaking policy remains the next implementation step.


## Pre-wrap occurrence policy

The new checked text-css-pre-wrap-v1 policy fixes the seven original whitespace
semantic/paint failures. The expanded 24-fixture corpus passes all 72 native-glyph
comparisons, and the full gate improves to 634/635 with only the existing A09
fractional-edge failure. Exactly seven old PNGs changed; 590 stayed identical,
including all 555 non-pre-wrap images. There are 30 new comparisons. All 72
focused images are identical between the probe and final full run.

Vector geometry passes all 72 comparisons, but pixels pass 64/72. Long words at
240/390px, forced-hang at all widths, word-after-short at 768px and mixed-policy
composition at 240/390px exceed unchanged rasterization budgets. All eight were
visually inspected with their differences: text remains present and aligned;
the remaining differences follow glyph edges. These stay open alongside A07/Q09.
Three runtime regressions, 68 module tests, complete corpus JS/WASM parity,
checked-host rejection, TypeScript, two gallery tests, Clippy, boundary and
27 state controls pass. Wrapped tabs remain explicitly rejected. See
[prewrap-policy-review.md](validation/prewrap-policy-review.md).


## Line-relative wrapped tabs

The original deferred wrapped-tab case now compiles and retains its literal tab.
Sixteen fixtures add 48 comparisons with resizing, alignment, consecutive tabs,
forced breaks, spacing, two fonts and mixed whitespace modes. Native glyphs pass
48/48; vector passes 43/48. The full pre-wrap corpus is now 120/120 glyph and
107/120 vector, with exact geometry in all comparisons. The new vector failures
are the tight X region in the original reproducer at all widths and the dense
mixed-mode scene at 240/390px; they remain failures under unchanged limits.

The final full glyph gate is 682/683, retaining only A09. All 627 older native
PNGs and all 72 older vector PNGs are unchanged. All 675 native/browser pairs
are identical before and after the trailing-boundary cache optimization. The
16,384-tab regression and five CSS runtime tests pass, alongside 68 module
Rust tests, native/WASM corpus parity, checked host, TypeScript, gallery,
Clippy, boundary and 27 state controls. All 48 new pairs were visually inspected;
the unclear contact-sheet previews were checked against original images.
See [wrapped-tabs-review.md](validation/wrapped-tabs-review.md).


## Pre-line and sparse-ink validation

Pre-line adds 24 fixtures and 72 resized comparisons. Native glyphs pass 72/72;
vector passes 70/72. Maximum geometry error is 0.00390625 CSS px. The final full
glyph gate is 755/756, retaining only A09; all 675 older native PNGs are identical.
All 72 new pairs match visually inspected artifacts. Six runtime regressions,
69 module tests, complete native/WASM parity, host checks, TypeScript, gallery,
Clippy, boundary and 27 renderer-state controls pass.

Visual review caught sparse Ogham marks missing despite a passing aggregate
score. A new opt-in control checks ink coverage and bounds against the browser
inside direct-text regions with a known background. It caught all six initial
cases; both renderer profiles now pass them after preserving Ogham ink and
alignment advances to match Chromium. The CSS draft's at-risk Ogham trimming
is explicitly excluded. The synthetic missing/displaced-ink regression is part
of the full gate. No existing pixel or geometry threshold changed.

Two mixed-mode vector cases at 240/390px retain glyph-edge rasterization failures.
Their geometry and native-glyph output pass. See
[preline-review.md](validation/preline-review.md) for metrics, commands,
browser/spec distinctions, screenshots and remaining limits.


## Default uppercase/lowercase transforms

Twenty fixtures add 60 comparisons covering case expansion, contextual sigma,
combining marks, whitespace modes, source breaks, spacing, alignment, inheritance,
intrinsic sizing and composition. Native glyphs pass 60/60; vector passes 59/60.
All geometry is within 0.01171875 CSS px of Chromium. All 60 new pairs were
visually inspected. The dense mixed-style vector fixture at 240px retains a
mean-error rasterization failure; its glyphs and layout are present and aligned.

The full glyph gate is 815/816 with only prior A09. All 747 older native PNGs
are unchanged. Seventy module tests, six runtime regressions, complete native/
WASM parity, host checks, TypeScript, gallery tests, Clippy, boundary and 27
renderer-state controls pass. Case mappings use build-guarded Unicode 17.0 tables.
Source maps retain normalized source, rendered text and scalar boundaries;
expanding transformations keep explicit break identities mapped correctly.
Capitalization and locale-specific transforms remain A19 work. See
[text-transform-review.md](validation/text-transform-review.md).


## Capitalization

Capitalization adds 20 fixtures and 60 resized comparisons. Native glyphs pass
60/60, vector passes 59/60, and all geometry is exact. All 60 new pairs were
visually inspected. The dense mixed-style vector case at 240px retains a
mean-error rasterization failure; glyphs and layout agree. The full gate is
875/876 with only prior A09, and all 807 older native PNGs are unchanged.

The workflow now also rechecks 62 independent logical casing references directly
against Chromium. Those cases exposed word-boundary and supplementary-character
differences before qualification. The documented compatibility profile uses
simple titlecase with preserved tail case; it does not claim full Unicode or
locale-aware titlecasing. Pinned ICU4X data is confined to the compiler.

Seventy-two module tests, six runtime regressions, complete native/WASM parity,
checked-host tests, TypeScript, reference checks, gallery tests, Clippy, boundary
and 27 renderer-state controls pass. Current text transforms total 120/120 glyph
and 118/120 vector comparisons. See [capitalize-review.md](validation/capitalize-review.md).


## Language-specific casing

The 16-fixture locale corpus passes 48/48 native-glyph and 48/48 vector checks,
with exact geometry at all three widths and same-scene resizing. All new glyph
pairs were visually inspected; combining accents and Greek output were also
reviewed in vector rendering. All 867 older native PNGs remain byte-identical.
The full gate is 923/924, with only prior A09 failing.

Thirty-six language references augment the 62 capitalization references; both
are checked independently against Chromium. Seventy-five module tests, six
runtime regressions, full native/WASM parity, checked-host/TypeScript/gallery
checks, Clippy, boundary and 27 renderer-state controls pass. Combined transform
coverage is 168/168 glyph and 166/168 vector; earlier vector gaps remain open.
See [locale-review.md](validation/locale-review.md).

Width/kana conversion remains pending: pinned Chromium rejects those values.
Initial Firefox probes accept them but innerText omits their visual conversion.
See [width-kana-research.md](validation/width-kana-research.md) for the next
reference-validation work; no new feature is qualified by that probe alone.


### Width/kana reference harness (pre-implementation)

`npm run test:transform-reference` checks Firefox painted transformations against
explicit Unicode using one pinned, glyph-audited font. Ten positive references
match exactly and two negative spacing controls differ; all pairs were visually
inspected. `npx playwright install firefox` installs the additional reference
browser. This command is currently separate from the main Chromium gate while
compiler support is being built. It is not native renderer qualification.
See [width-kana-research.md](validation/width-kana-research.md).


Chrome remains the acceptance target. The optional Firefox width/kana reference
is supplemental research, not a replacement acceptance gate. Width/kana are
still rejected and deferred; work continues with A20 underline against Chrome.


### Underline reference (before implementation)

`npm run test:underline-reference` rechecks 48 independent Chrome measurements
of thickness, baseline position and exact scanline ink intervals for two fonts,
four sizes and both skip-ink modes. Transparent text and red decoration isolate
thin-line errors that whole-image averages could hide. All reference images
were inspected. This is browser behavior evidence; native underline support
and its acceptance gate remain pending. See validation/underline-research.md.


The underline geometry stage now has five runtime integration tests, included in
run.sh. Red/green tests cover closing edges and curve intersections; extended
checks cover three-crossing cubics, invalid/disjoint inputs and native Inter
outlines against recorded Chrome descender gaps. Final native underline drawing
is still pending, so these are geometry-stage results, not feature qualification.


### Underline runtime drawing stage

The compiler suite now checks resolved underline installation after import,
resizing one scene at 768/390/240, descender-gap segmentation, removal restoring
the original recording, and paint ordering before actual native rasterized
glyphs. A separate test exercises all 481 pinned-Chrome skip-ink boundary cases
through the runtime API, with None/All controls. The module suite passes 78 tests;
the expanded drawing test was rerun after adding glyph-path assertions.
Runtime integration coverage adds invalid resolved metrics to the five existing
geometry tests. Source attribution is retained with the pinned eligibility table.
These results establish runtime behavior only: no underline CSS, native/WASM
manifest qualification, or Chrome/native screenshot acceptance is claimed.
See validation/underline-research.md for remaining work and commands.


### Underline transport and runtime pixels

`npm run test:underline-runtime` compares one compiled scene at 240/390/768 in
both native renderer profiles against pinned Chrome. Explicit resolved records
are installed through version 3; this tests runtime transport, not CSS parsing.
Controls cover no decoration, solid, auto skip-ink, and changed thickness/offset.
All geometry is exact. 12/12 glyph and 8/12 vector comparisons pass; four vector
cases at 240 fail the unchanged mean error threshold, including the undecorated
baseline. All 24 browser/native/diff pairs were inspected in six contact sheets.
The command exits nonzero for those preserved failures. It also checks red-line
presence independently of black glyphs. See validation/underline-runtime-review.md.

The Rust suite now passes 81 tests, including three transport contracts. Native
host rejection/installation tests, TypeScript, five JS/native/WASM corpus parity
tests, module Clippy, and boundary checks pass. The compiler does not yet emit
underline records from CSS, so those parity results cover existing accepted
syntax, not end-to-end underline compilation.


### Underline CSS stage

`--grep underline-` now covers 17 actual CSS fixtures through compilation,
version 3 transport, native import and same-scene resize. Native-glyph: 51/51;
vector: 49/51. Geometry is exact. Vector from-font cases for Inter and Open Sans
at 390 exceed the unchanged interior RGB limit. All 102 image pairs were
inspected in `underline-css-{glyph-final,vector}/contact-*.png` and the galleries.
Six public compiler contracts cover parsing, shorthand reset, inheritance,
origin retention, metrics, capability/version composition and invalid values.
Twenty independent Chrome origin references pass in
`node validation/underline-origin-reference.mjs`. Native/WASM corpus parity now
includes the 17 underline fixtures and passes all five JS tests.
See validation/underline-css-review.md for results and remaining qualification.


Full underline-stage gate: 974/975 tests pass; the visual gallery has 965/966
passing cases, with the existing `em-layout-cascade` 240 failure. All 915 old
native PNGs shared with `locale-initial` are byte-identical. The 51 new underline
cases pass in the native-glyph profile. The 48 underline reference cases, 20
origin references and 27 renderer-state controls also pass; the two reference
scripts are now included in run.sh. Module Clippy and boundary checks pass.
The two focused vector from-font failures remain, so A20 is not complete.


### Underline sparse-pixel edge gate

Fixtures with `redDecoration:true` use full-frame saturated-red masks in addition
to existing checks. Bidirectional one-pixel support and 85–115% coverage catch
missing lines, overlong tails and isolated ink. Negative controls pass (all ten
pixel-gate tests). New edge cases: 37/39 glyph and 34/39 vector, exact geometry;
all 78 pairs inspected. Retained CJK failures identify hard-clip and glyph-overlap
raster differences. Whitespace, breaks, hidden and transparent controls pass.
88 Rust tests, full-corpus JS/native/WASM parity, Clippy and boundary pass.
Earlier 51 native underline PNGs are unchanged. Details and initial failed
reproducers: validation/underline-edge-review.md and underline-clip-research.md.

### A20 host-transform regression controls

Added the `underline` profile to `validation/glyph-state-control.mjs` and the
native-glyph gate. All 27 comparisons pass geometry and ordinary image/ink limits
but fail the stricter red-decoration check; failures are retained. All pairs
visually reviewed. Existing glyph controls remain 27/27 passing. This extends
validation coverage, not supported CSS syntax or underline qualification.
See `validation/underline-state-review.md`; A20 remains active.

A20 follow-up: isolated host-transform controls (`underline --no-restore`) pass
6/27; the other 21 retain sparse red-ink differences. Runtime geometry now keeps
full stripes and unsnapped clip rectangles for the pending hard-clip backend;
current drawing still uses the geometric fallback. See the updated
`validation/underline-state-review.md`. No new CSS syntax is qualified.

A20 hard-clip transport is implemented and tested: optional renderer operation,
precise recording and text-stream replay, malformed-input rejection, and explicit
unsupported-backend errors. Native coverage and runtime integration remain pending;
no visual qualification or new CSS support is claimed. Evidence:
`validation/underline-clip-research.md`.

A20 native hard-clip step: NativeMetalFrame implements device-space difference
masks. Four mask tests and five exact native pixel controls pass, including
transforms, overlapping exclusions, host clipping and restore. Underline text
still uses its previous fallback; Chrome qualification and runtime integration
remain pending. See `validation/underline-clip-research.md`.

### A20 hard-clip integration

Runtime underlines now use hard difference clipping where supported, restoring
state before geometric fallback if declined. Native glyph corpus: 90/90 passing
and visually reviewed; vector: 85/90 passing with the five known failures.
Host-state checks improve to 17/27 in each profile, retaining ten sparse red-ink
failures. A20 remains active. See `validation/underline-hard-clip-review.md` for
scope, receipts and outstanding qualification.

A20 fallback follow-up: actual runtime drawing is tested against immediate and
partial hard-clip refusal, including restoration before fallback. Metal offscreen
canvas forwarding now uses the shared clip implementation; native build and five
exact replay controls pass. Canvas-specific pixels remain unqualified. See
`validation/underline-hard-clip-review.md`.

A20 transform diagnosis: transparent-glyph controls isolate decoration pixels and
pass 23/27, compared with 17/27 with visible glyphs. A seven-case translation sweep
retains a narrow clip-edge transition difference between native and Chrome. These
are supplemental diagnostics, not substitutes for the failing original gates.
Evidence and reproduction commands: `validation/underline-transform-diagnosis.md`.

A20 precision investigation rejected SVG clipping as a hard-clip reference and
reverted a canonical-size-only outline experiment after it regressed the phase
sweep. Existing support is unchanged. Pinned Skia source is now available through
its official GitHub mirror; compare scaler/outline coordinates next. Evidence:
`validation/underline-transform-diagnosis.md`.

The diagnostic higher-precision shaping experiment passes all 7 underline
clip-phase cases (previously 3) and 18/27 visible-glyph host states (previously
17). The seven phase pairs were visually inspected; full host visual review and
broad text regression qualification remain pending. The temporary runtime switch
was removed; see `validation/underline-transform-diagnosis.md` and its retained
experiment patch. These are experimental results, not the production baseline.

`NUXIE_CSS_SHAPING_PRECISION=1` opts the native validation probe into the explicit
experimental runtime font policy. The default remains Rive precision. Its first
Chrome advance and font policy retention regression passes; see the underline
transform diagnosis receipt for adoption requirements and remaining validation.

`validation/opacity-overlap-reference.mjs` now preserves the distinction between
native per-draw alpha and Chrome group opacity using overlapping rectangles.
The native image matches Chrome per-element alpha within one channel value,
but differs from group opacity by up to 63. This diagnostic explains why
non-overlapping glyph controls cannot qualify P05 group compositing. Existing
underline opacity failures remain; no acceptance tolerance changed.

The full experimental CSS-precision native-glyph run passes 1014/1015 checks
(1004/1005 visual cases), retaining only the known em-layout-cascade 240px
failure. There are zero regressions against the previous 966-case baseline.
Changed/added image review and vector qualification remain pending. Detailed
counts, configuration and comparison artifacts are in the underline diagnosis.
The 96 changed/added native-glyph cases have now been visually reviewed.
Focused vector underline testing remains 85/90 with the same five failures and
no regressions; nine changed vector cases await inspection. This does not claim
fresh review of all 1005 native cases or broad vector text qualification.
Actual shaping size diagnostics now cover 36 samples across four fixture fonts,
from 0.01px to 1000000px, without missing glyphs or negative advances. This
supports the numeric scale guard but does not establish universal browser
fidelity at extreme sizes. See `validation/precision-size-reference.json`.

The wide-advance synthetic-font regression caught a real integer overflow in
the experimental size-dependent scale. The bound now accounts for font units
and hmtx advance range; actual two-em advances pass through 1000000px. It runs
from run.sh. The earlier full vector run finished at 987/1015 checks; all 28
failures reproduce without the precision option. See the diagnosis receipt for
post-fix checks still pending and the limits of that comparison.
Post-bound-fix verification passes 89 feature-enabled module tests, the scale
unit test, eight actual wide-advance samples, 36 four-font size samples and all
seven phase cases. Phase PNGs are byte-identical to the reviewed pre-bound-fix
output. The recorded corpus uses sizes below the new bound; see the diagnosis
receipt for exact evidence and the remaining capability work.

Precision capability emission and host installation are now implemented.
Automatic selection passes 7/7 phase cases with byte-identical reviewed images;
89 module tests, native/WASM corpus parity, host contract, types and Clippy pass.
See `validation/precision-policy-review.md` for the integration evidence and
pending full underline run. Historical environment-only notes above describe
pre-capability experiments, not the current default for compiled text.

Automatic precision integration completed its focused underline run: 90/90
pass and all 90 native PNGs are byte-identical to the reviewed explicit-policy
results. The runtime boundary also passes. Receipt: precision-policy-review.md.


### A20 nonpositive font metrics and fractional WASM metadata

The focused font-metric control passes 30/30 Chrome/native cases: five signed
font metrics, auto/from-font, and resize of each compiled scene to 240/390/768.
All five browser/native/diff contact sheets were inspected. Native/WASM Rive
bytes, source maps and runtime requirements agree for all ten source variants.
The public module suite passes 90 tests; the JavaScript suite passes six tests,
including accepted-corpus parity and fractional metadata regression coverage.
No visual thresholds changed. Absent metrics and remaining A20 rendering gaps
are not qualified by this control. See validation/underline-font-metrics-review.md.


The absent-post font probe is retained as a failing browser-acceptance diagnostic,
not a visual pass: Chrome reports `post: missing required table`. The extended
public underline test verifies absent versus zero/negative metrics and fallback
behavior; all eight underline tests pass. Exact inputs and OTS diagnostics are
preserved in output/playwright/html-to-riv/underline-font-metrics-absent/.
See the font-metric receipt for reproduction and scope.


### A20 device scale follow-up

Native-glyph/Metal DPR controls pass 25/27 comparisons across skip-ink auto/all/
none and widths 240/390/768. All DPR 2/3 cases pass; DPR 1 auto/all at 768 each
retain two extra red pixels beyond the existing support gate. Geometry passes
throughout; all nine overview contact sheets were inspected. Three source scenes
have native/WASM artifact parity and are reused across all sizes/DPR values.
See validation/underline-dpr-review.md. This does not qualify fractional DPR or
all renderer backends; A20 remains active.

A20 DPR failure reduced to transparent Latin text near a half-pixel clip edge.
Native advance accumulation in f64 alone does not account for the Chrome
difference; no rendering workaround applied. Reproducer and measurements:
`validation/underline-dpr-review.md` (reduced half-pixel failure).

A20 shaping diagnosis now isolates scalar advance quantization: p/q/j/space
each differ from Chrome by 1/65536px, accounting exactly for the reduced-line
width discrepancy. A failing component control and callback implementation seam
are recorded in `validation/underline-dpr-review.md`; the fix remains pending.

CSS shaping now uses a horizontal advance callback matching the observed Chrome
fixed-point conversion. DPR controls improve to 27/27; the reduced failure is
fixed. Module91 and overflow8 checks pass. Broader regression and remaining
DPR visual inspection are pending; see validation/underline-dpr-review.md.

The post-callback native underline suite completed: 90/90 pass. Its changed
images still require fresh review; gallery: output/playwright/html-to-riv/underline-callback-glyph/.

Post-callback DPR visual review is complete (all nine sheets). Existing underline
90/90 native PNGs are byte-identical to the reviewed baseline. Six JS parity
tests pass; broader corpus and host-state checks are in progress. Receipt:
validation/underline-dpr-review.md.

Post-callback host-state validation remains 18/27 with no new failing cases;
26/27 native images are identical and the one changed passing image was visually
reviewed. Runtime boundary passes. Full native-glyph corpus is still running.

Expanded callback widths pass 165/171: Japanese g at 36px reveals a remaining
scaling-order discrepancy; all Inter/Open Sans checks pass. Focused vector
underline remains 85/90 with the same five failures; six changed cases visually
reviewed. Full native-glyph corpus remains active. See underline-dpr-review.md.

First callback full corpus completed: 1004/1005 visual cases pass, same existing
em-layout-cascade failure; all 1005 native images identical to baseline. Later
pixel-width arithmetic improves font components to 170/171, with a long-string
36px discrepancy still open. Subnormal reciprocal fix is building and awaits
revalidation. Details: validation/underline-dpr-review.md.

Safe pixel conversion is validated by module91, overflow8 and DPR27/27. Font
widths remain170/171: the remaining long-string reference matches intermediate
Chrome accumulation boundaries and is still open. A20 remains partially
qualified; independent authoring work continues with A21. See the DPR receipt.


### Strikethrough Chrome controls (A21, reference only)

Run `npm run test:strikethrough-reference` in this module. The metric replay
checks 72 pinned Chrome records; the paint test checks 18 combinations with
126 captures using opaque glyphs and nested decoration controls. Both pass
on Chrome 153.0.8010.12. Metric sheets and representative paint sheets were
visually inspected. Chrome is authoritative; Firefox is not a gate.
Native/WASM compilation, native pixels and responsive resize qualification
remain pending. See `validation/strikethrough-review.md`.

A21 runtime progress: resolved strikethrough drawing now exists behind an
explicit occurrence API. Import/resize testing verifies one stripe per wrapped
line and after-glyph paint order in vector and native glyph rendering. Removing
both underline and strikethrough restores the original recording. Compiler
syntax and transport remain pending; this does not qualify Chrome/native
pixels. See `validation/strikethrough-review.md`.

A21 transport progress: version 4 and `text-solid-strikethroughs-v1` are
implemented with host validation and TypeScript declarations. CSS emission
remains pending. Initial manually supplied runtime controls pass 15/24 pixel
cases and 24/24 geometry comparisons; 2px stroke rasterization differs from
Chrome. Reproducers and review scope: `validation/strikethrough-review.md`.

A21 snapping diagnosis: a single transparent glyph reproduces the stripe
coverage mismatch. Rounding only the resolved offset fails at fractional
container positions. A stricter stripe-interior control catches the difference;
production behavior remains unchanged pending paint-coordinate snapping.
See `validation/strikethrough-review.md` for evidence and reproduction.

A21 phase reference: 384 Chrome stripe placements across two fonts, fractional
container positions and two-line layout now replay successfully. The matching
model rounds line paint origin and resolves ascent before snapping the stripe.
This is reference evidence; runtime snapping and CSS emission remain pending.
Details: `validation/strikethrough-review.md`.

A21 runtime snapping is implemented using a required version 4 `line_baseline`
metric and the original thickness. The runtime recovers each CSS line origin
after layout and snaps at paint time. Native phase pixels pass 32/32 with all
32 inspected; original resized scenes pass glyph 12/12 and vector 8/12, retaining
four vector 240px failures also seen without decoration. Module98, runtime
geometry7 and TypeScript checks pass. Public CSS emission remains pending.
See `validation/strikethrough-review.md` for scope, artifacts and remaining gates.

A21 CSS emission now passes 103 Rust tests and seven JavaScript tests, including
six new native/WASM parity cases. Run `strikethrough-runtime-control.mjs
--compiler-css` with a dedicated `NUXIE_STRIKETHROUGH_RUNTIME_DIR` to compare
compiler-produced decorations at three runtime widths. Initial result: glyph
12/12, vector 8/12, all geometry checks pass; four existing vector 240px failures
remain. All glyph cases are visually reviewed directly or through byte identity
with reviewed images. Broader qualification remains in
`validation/strikethrough-review.md`.

A21 composition evidence now covers combined lines, propagated thickness with
mixed font sizes, multiple origins, fractional explicit breaks and a price card.
Native-glyph pixels pass 26/27 decorated cases; the sole OpenSans 240px residual
also occurs without decoration (control 2/3). All 30 comparisons were visually
inspected. Expanded-corpus native/WASM parity and import checks pass. This does
not qualify the new fixtures on the vector profile or resolve host/DPR limits.
See `validation/strikethrough-review.md`.


A21 follow-up: vector compositions pass 18/30 (all 30 visually inspected).
Native-glyph DPR 1/2/3 controls pass 21/27 with visible text, and 27/27 with
transparent glyphs. All geometry and red-stripe checks pass; six visible-text
mean-channel failures remain at DPR 3. DPR 3 sheets are visually inspected;
DPR 1/2 review remains. Chrome is the sole browser reference. Reproduce with
`validation/strikethrough-dpr-control.mjs`, optionally `--transparent-glyphs`
or `--without-decoration`, using distinct `NUXIE_HTML_REVIEW_DIR` directories.
See `validation/strikethrough-review.md` for evidence and qualification limits.

The A21 undecorated DPR control reproduces the same six mean-channel failures
(21/27, or 7/9 unique viewport/scale combinations). This evidences a text
rendering residual independent of decoration; it remains a qualification gap.

A21 metric checks: 20 size/thickness extremes produce host-valid requirements
and identical native/WASM artifacts. Native-glyph 8px and 1px DPR matrices
each pass 27/27 with partial visual review recorded in the receipt. Giant
font rendering and other fonts/line heights remain unqualified. Reproduce
small-text controls with `NUXIE_STRIKETHROUGH_SIZE` and the DPR script.

A21 host-state controls now include strikethrough and native/WASM parity.
The isolated matrix passes 24/27 after visual review exposed darkened stripe/
glyph overlaps and prompted a targeted opacity-interior check. Half-opacity
fails at all three widths; this retains the P05 group-alpha limitation.
See the strikethrough receipt for commands, partial visual-review scope,
and initial aggregate passes that must not be treated as qualification.

A22 initial overflow-clip coverage: four fixtures × three widths, native-glyph
12/12, all visually inspected. Public import/resize checks and expanded corpus
native/WASM parity are included. Run the native project with `--grep text-clip`
and a separate review directory. See `validation/clipping-review.md` for scope.

A22 regression: 106 Rust tests pass; initial vector pixels pass 6/12, with all
12 visually reviewed and six failures preserved. See the clipping receipt.

A22 composition coverage now includes nested rounded clips, glyph-body cuts,
decorations, fractional edges and a cropped image card, with sibling restore
checks. Added glyph 18/18 and vector 10/18; all 36 visually reviewed. Cumulative
clipping fixtures: glyph 30/30, vector 16/30, all inspected. Expanded native/WASM
parity passes. Fourteen vector failures remain; see the clipping receipt.

A22 DPR matrix: 23/27, all visually reviewed. Four fractional-text pixel
failures remain; geometry and bottom-leak checks pass throughout. A deliberate
missing-clip control fails the leakage gate at all nine width/scale combinations.
Run `validation/clipping-dpr-control.mjs`, optionally `--negative-control` with
a separate review directory. Native/WASM parity passes. See the clipping receipt.

A22 fractional DPR isolation reproduces text failures without clipping (4/9).
Hiding clipped text leaves two sibling-only failures (7/9); hiding all text
passes 9/9. This evidences an independent text-rendering residual and preserves
the original 23/27 visible-text gate. Diagnostic controls and review scope are
recorded in `validation/clipping-review.md`.

Static hidden validation: Rust107 and expanded native/WASM parity pass.
Actual hidden fixtures pass glyph30/30 and vector16/30, retaining the same
14 vector failures. Every browser/native image is byte-identical to its
already visually reviewed clip counterpart; identity receipts are retained.
`overflow:hidden` is supported for static initial rendering only; programmatic
scrolling remains outside the module. See `validation/clipping-review.md`.

A22 host-state controls now compile mixed clip/hidden text and compare nine
transforms across three widths. Geometry and native/WASM parity pass. Pixels:
15/27 with restored copy, 19/27 isolated; all isolated cases visually reviewed.
Transformed text residuals remain. Existing glyph regression stays 27/27.
Reproduce with `validation/glyph-state-control.mjs clipping`, optionally
`--no-restore`. See the clipping receipt for limits and artifacts.

A23 ellipsis investigation is active; text-overflow CSS is still rejected.
Runtime-only controls pass 6/12 with all comparisons visually inspected. An
8px-wide box exposes a semantic mismatch (Chrome clips the first character;
Rive clips an ellipsis). Fractional cases also fail pixels. See
`validation/ellipsis-review.md`; the probe flag is diagnostic, not a compiler API.

A23 now has 24 Chrome-only narrow-box boundary controls (all visually
inspected), including ffi and a combining mark. They establish first-grapheme
preservation and expose a ligature reshaping requirement; retaining the first
original glyph alone is insufficient. No additional native/compiler support
is claimed. See `validation/ellipsis-review.md` and reproduce with
`node tools/html-to-riv/validation/ellipsis-narrow-reference.mjs`.

A23's expanded Chrome controls isolate the ligature marker mismatch: painting
an isolated prefix while preserving its original text-range width produces
exact reference pixels. The 24-case run passes its diagnostic assertions and
all cases were visually inspected. This is browser-only evidence, not native
qualification; compiler text-overflow acceptance is unchanged. Receipt:
`validation/ellipsis-review.md`, “Ligature marker placement isolated”.

A23 now includes an experimental runtime truncation planner (5/5 focused Rust
unit tests). It preserves original prefix advances and supplied grapheme
boundaries. Rendering integration, real boundary extraction, native/WASM
transport and CSS acceptance remain pending; see the ellipsis receipt.

A23's runtime boundary extractor now handles complete LTR shaped runs with
Unicode grapheme segmentation. Eight focused tests pass including real Inter
shaping against recorded Chrome first-range metrics; WASM compilation passes.
This does not yet enable renderer integration or CSS text-overflow. Details
and reproducible commands are in `validation/ellipsis-review.md`.

A23's CSS ellipsis experiment is now connected to probe rendering for one
unmodified LTR run/line. Native comparisons pass 18/24, all 24 visually
inspected; 8px A is fixed, while 8px ffi and fractional positions retain
local RGB failures. Flag-off glyph regression passes 27/27. Baseline scene
parity passes, but CSS text-overflow transport/acceptance is still pending.
See `validation/ellipsis-review.md` for flags, commands and remaining scope.

A23 correction: earlier ellipsis runs requested native glyphs but used vector
fallback because the adapter excluded ellipsis overflow. The experimental
CSS path now opts into the adapter; the runner verifies glyph-cache use.
All 24 native-glyph comparisons pass and were visually inspected, including
narrow ffi and fractional positions. Earlier vector failures remain retained.
Flag-off glyph regression stays 27/27. CSS acceptance and broader ellipsis
qualification remain pending; see the ellipsis receipt's adapter correction.

A23 alignment controls pass 27/27 with full visual review. Short-height
controls pass 9/12: height:8px grows to ~8.776px natively. The same failure
occurs with ellipsis disabled; all pixels pass, so this is a retained shared
text-layout geometry limitation. Height 20/39/80 controls pass. Both vertical
runs (12 each) were visually inspected; see the ellipsis/clipping receipts.

Short-height text layout is fixed: line-height spacing now belongs to an
internal text container, preserving authored element padding/heights and
automatic line-box sizing. All12 focused vertical comparisons pass and were
visually reviewed. The full native run passes1102/1105 checks; all1095
browser/native image pairs are byte-identical to retained baselines, with
the same three known240px failures. Full107 module tests pass plus the new
public height/resize regression (108 total); native/WASM corpus parity passes.
See `validation/clipping-review.md` for the precise limits and receipts.

A23 DPR controls pass69/72: only fractional-position text at DPR2 fails local
RGB at all three widths. A clip-only control reproduces those failures (6/9),
so they are not ellipsis-specific. All new DPR2/3 and clip-only comparisons
were visually inspected; DPR1 images are identical to the reviewed baseline.
Geometry, native-glyph usage and baseline scene parity pass. Tolerances remain
unchanged; see `validation/ellipsis-review.md` for reproduction and limits.

A23's experimental LTR preparation now handles multiple shaped runs. The
focused suite passes 9/9, including independent marker size/style and partial
second-run source offsets. Word-spacing controls pass 18/18 with full visual
review; original controls remain 24/24 with all 48 browser/native PNGs
byte-identical to the previously reviewed run. Native and WASM builds pass.
This does not qualify mixed-font/style pixels or block-style transport: CSS
text-overflow remains rejected. See `validation/ellipsis-review.md`,
“Multiple shaped runs and word spacing”, for commands and retained limits.

A23 decoration compositions now pass 27/27 with full Chrome/native visual
review. The initial 12/27 run exposed decoration extending through the
ellipsis marker. Runtime decoration coverage now stops at retained source
glyphs while marker painting remains intact. Failing artifacts are retained;
no thresholds changed. This is DPR1 experimental runtime qualification, not
CSS text-overflow acceptance. See the ellipsis receipt, “Decorations stop
before the marker”.

A23's ellipsis decoration matrix also passes 81/81 across DPR1/2/3. All 54
DPR2/3 comparisons were visually inspected; DPR1 images are byte-identical
to the previously reviewed run. Geometry, glyph-cache use and baseline
native/WASM parity pass without threshold changes. Fractional positioning,
other fonts and host transforms remain separate qualification work. See
`validation/ellipsis-review.md`, “Decoration DPR qualification”.

A23 letter-spacing controls pass 27/27 after removing source letter spacing
from the synthetic marker. Initial +2px narrow cases truncated too early;
visual review also caught a -1px case truncating too late despite its broad
pixel gate passing. Changed images were inspected and unchanged images
verified identical. The initial reproducers remain preserved. See the
ellipsis receipt, “Letter spacing excludes the synthetic marker”.

A23 host-state controls pass 27/27 isolated and 27/27 with a subsequent draw
after state restoration, all visually inspected. Tested DPR1 transforms,
clipping and opacity preserve ellipsis; baseline native/WASM parity passes.
An initial flex-box Chrome fixture was invalid and is explicitly retained as
a harness setup error. See `validation/ellipsis-review.md`, “Host transforms
and state restoration”, for reproduction and limits.

A23 now reserves a checked single-line ellipsis occurrence policy in the
portable requirements API. It requires CSS shaping precision and explicit
host support, and validates unique Text targets. The compiler does not emit
it and the native host does not advertise it yet; text-overflow remains
rejected. See `validation/ellipsis-review.md`, “Reserved occurrence contract”.

A23's native host now installs the checked ellipsis occurrence capability.
It validates nowrap, static LTR content and consistent font metrics before
installation. Native host tests pass 2/2; installed default/word-spacing
controls pass 24/24 and 18/18, with images identical to reviewed diagnostic
controls. The compiler still does not parse or emit text-overflow. See the
ellipsis receipt, “Checked runtime installation”, for exact chronology,
preconditions and remaining validation limits.

## Public single-line ellipsis (current A23 scope)

`text-overflow: clip | ellipsis` is accepted. It is non-inherited by default;
explicit `inherit`, `initial`, `unset`, cascade order and `!important` are tested.
For nonempty text, `ellipsis` requires a text-only `display:block` element,
`white-space:nowrap`, and `overflow:hidden` or `overflow:clip`, without explicit
line breaks or preserved control separators. Strings, two-value forms, `fade`,
wrapping and flex text contexts receive explicit diagnostics. Container-only
values have no effect on descendants unless explicitly inherited.

The compiler retains full source text and responsive layout. It emits the
checked `css-single-line-ellipsis-v1` occurrence policy and
`text-css-single-line-ellipsis-v1` capability, together with CSS shaping precision.
Hosts must install that policy; bare Rive bytes do not supply these semantics.
The native installer validates targets before enabling ellipsis. No diagnostic
runtime flags or browser-baked truncation are needed for this public path.

Qualification is partial: Inter LTR native glyph rendering is covered; fractional
DPR2 clipping residuals, vector rendering, broader fonts and mixed-font/bidi or
multiline ellipsis are not qualified. Existing failing reproducers and thresholds
are preserved. Earlier A23 entries below/above are historical stages; statements
that CSS parsing is pending are superseded by this section. See
`validation/ellipsis-review.md`, “Public CSS compiler path”.

Public CSS validation: compiler112/112, complete accepted native/WASM corpus parity,
public-path24/24 (48/48 images identical to reviewed controls), permanent
corpus15/15 at240/390/768 with all five comparison sheets visually inspected.

Open Sans expansion found a specific optional-ligature ellipsis mismatch (18/24;
all inspected). Narrow clip-only controls pass; the clip-only full matrix is23/24
with an independent long-line raster residual. Runtime correction now retains
original shaped glyphs, with11 focused tests passing; visual requalification is
pending. See the ellipsis receipt's Open Sans investigation.

Original-glyph correction validated: Open Sans24/24; all six changed native
images inspected. Inter24/24 and48/48 PNGs unchanged. Open Sans spacing23/27,
all inspected; four long-line residuals remain, including three untruncated
lines. Runtime WASM check passes. This supersedes the pending correction above;
see the ellipsis receipt for exact test scope and preserved failures.

Open Sans permanent corpus now covers two narrow ligature cases and a responsive
media card:9/9 Chrome/native comparisons inspected at240/390/768, compiler112/112
and complete accepted native/WASM corpus parity pass. The four remaining
Open Sans spacing failures all recur without ellipsis (clip-only22/27); this is
a shared rendering residual whose exact cause remains open. See the ellipsis
receipt and retained overlap evidence.

Retained-glyph composition regression: public Inter decoration81/81 across
DPR1/2/3 and word-spacing18/18, with all198 PNGs identical to reviewed baselines.
Open Sans decorations24/27, all inspected; three untruncated768px lines fail RGB
and decoration gates. See the ellipsis receipt; broader font/rendering
qualification remains partial.

The shared Open Sans drift is now fixed: an empty GPOS table previously enabled
legacy kerning fallback for re/rd. Runtime tests3/3 preserve valid GPOS kerning;
Open Sans spacing27/27, decorations27/27 and permanent corpus48/48 pass with
visual review, plus native/WASM parity and runtime WASM check. This supersedes
the Open Sans residual reports above for these tested cases. See
`validation/opensans-kerning-review.md` for scope and preserved failures.

S01 attribute selectors: full compiler115/115 and native/WASM corpus parity pass.
Five permanent fixtures pass15/15 Chrome/native comparisons at240/390/768; all
five sheets were visually inspected. Browser reference scoping now splits lists
only outside strings/brackets/parentheses, fixing quoted-comma corruption.
Initial12/15 harness failures are retained. Explicit s remains deferred because
pinned Chrome rejects it; i and ordinary case-sensitive matching are validated.
See `validation/attribute-selectors-review.md` and the case-flag receipt.

S02 adjacent siblings qualified: compiler118/118, full accepted native/WASM
artifact parity, and12/12 Chrome/native comparisons at240/390/768. All four
comparison sheets inspected. Coverage includes comments/whitespace, hidden
siblings, attribute chains, specificity and responsive card margins/widths.
See `validation/adjacent-selectors-review.md`.

S03 general siblings qualified: compiler120/120, full accepted native/WASM
artifact parity, and12/12 Chrome/native comparisons at240/390/768. All four
comparison sheets inspected, including hidden source operands, nonadjacent
matches, combined selectors and responsive stacks. See
`validation/general-siblings-review.md`.

S04 structural child selectors qualified: compiler123/123, full accepted
native/WASM artifact parity, and15/15 Chrome/native comparisons at240/390/768.
All five sheets visually inspected. Hidden siblings, tag-independent counting,
only-child, specificity and responsive compositions are covered. See
`validation/structural-selectors-review.md`.

S05 nth selectors qualified within the documented resource bounds: compiler
127/127, native/WASM builds and full accepted-corpus publish parity pass. Six
permanent fixtures pass 18/18 Chrome geometry and actual native pixel checks at
240/390/768 by resizing scenes compiled at 390. All six sheets inspected;
selection, colors, wrapping and responsive widths agree, with minor edge
rasterization differences within unchanged tolerances. See
`validation/nth-selectors-review.md`.

S06 negation qualified: full compiler 130/130, native/WASM builds and full
accepted-corpus artifact parity pass. Six permanent cases pass 18/18 Chrome
geometry/native pixel checks at 240/390/768 using scenes compiled at 390.
All sheets inspected, including responsive cards and nested nth/negation;
unchanged tolerances. See `validation/negation-selectors-review.md`.

S07 is / S08 where qualified within the documented selector profile: compiler
136/136, native/WASM builds and full accepted-corpus artifact parity pass. Eight
fixtures pass 24/24 Chrome geometry/native pixels at 240/390/768, resizing
scenes compiled at 390. All sheets inspected, including forgiving syntax,
specificity and responsive cards. No threshold changes. See
`validation/matches-any-selectors-review.md`.

S09/S10 started: four isolated custom-property resolver tests pass. Public
compiler support, parity and pixel qualification are still pending; no support
claim is made from these unit tests. See `validation/custom-properties-progress.md`.

S09/S10 public integration progresses: compiler146/146, native/WASM builds and
full accepted-corpus artifact parity pass. Initial pixels15/18 exposed obsolete
unused-fallback cycle semantics; direct Chrome and the current editor’s draft
confirmed short-circuiting. Corrected pixels18/18 at240/390/768 and all six sheets
inspected. Initial failures retained. Present-but-invalid substitution remains
unimplemented, so these items stay in progress. See custom-properties-progress.md.

S09/S10 empty substitution: compiler147/147, native/WASM builds/parity and
combined Chrome/native30/30 pass. Four new sheets inspected; six earlier sheets
byte-identical to reviewed prior versions. Empty final values apply unset while
empty components in otherwise valid values are preserved semantically. Logs,
gallery and identity receipt: output/playwright/html-to-riv/custom-properties-empty/.
Nonempty invalid substitutions remain pending.

S09/S10 invalid scalar/list tokens: compiler150/150, native/WASM builds/parity,
Chrome/native45/45 pass. Five new sheets inspected and ten previous sheets
verified byte-identical to reviewed versions. Gallery/logs/identity receipt:
output/playwright/html-to-riv/custom-properties-invalid/. Broader invalid-value
classification and resource/dynamic-name audits remain; items stay in progress.

S09/S10 resource validation: compiler153/153, native/WASM builds and full corpus
parity pass. Fixed stack overflow from512 properties ×32 nested inert groups
using an explicit continuation stack. Combined Chrome/native54/54 pass, three
new sheets inspected and15 earlier sheets verified identical. Evidence:
output/playwright/html-to-riv/custom-properties-resources/. Dynamic var names
remain rejected/deferred with direct Chrome153 evidence under custom-properties-names.

S09/S10 color/length keywords: compiler154/154, native/WASM builds/parity and
Chrome/native66/66 pass. Four new sheets inspected,18 previous sheets identical
to reviewed versions. Gallery/logs: output/playwright/html-to-riv/custom-properties-keywords/.
Function syntax and other property families remain pending.

S09/S10 scalar RGB/HSL invalidation: compiler156/156, native/WASM builds/parity
and Chrome/native75/75 pass. Three new sheets inspected;22 earlier sheets
verified identical to reviewed versions. Evidence:
output/playwright/html-to-riv/custom-properties-functions/.

S09/S10 flex/overflow enum invalidation: compiler158/158, native/WASM builds/parity
and Chrome/native87/87 pass. Four new sheets inspected (including text clipping);
25 prior sheets identical. Evidence: output/playwright/html-to-riv/custom-properties-enums/.

S09/S10 flex-factor invalidation: compiler159/159, native/WASM builds and full
accepted-corpus parity pass. Chrome/native93/93 pass at240/390/768 using scenes
compiled at390. Two new sheets inspected: growth/shrink distribution agrees,
with narrow edge-raster differences within unchanged tolerances. All29 prior
sheets are byte-identical. Evidence: output/playwright/html-to-riv/custom-properties-factors/.
The initial test failure documenting the equal-factor profile limit is retained.
See validation/custom-property-coverage.md for the remaining grammar audit.

S09/S10 flex shorthand: compiler160/160, native/WASM builds and accepted-corpus
parity pass. Chrome/native99/99 pass at240/390/768, compiled at390. Two new sheets
inspected;31 previous sheets are byte-identical. Evidence:
output/playwright/html-to-riv/custom-properties-flex-preserved/.

The first run passed96/99 and exposed a harness defect: CSSOM cssText serialization
loses pending var() shorthand components after a longhand override. The harness
now mutates selectorText on an adopted stylesheet, preserving declaration state.
validation/custom-property-shorthand-control.mjs compares original, serialized,
and preserved computed styles. Initial failures and direct Chrome evidence are
retained in output/playwright/html-to-riv/custom-properties-flex/. This run
requalifies the33 custom-property fixtures; the entire older corpus has not yet
been rerun with the revised harness. Tolerances remain unchanged.

Full harness regression completed:1,341/1,342 checks pass. All1,332 scene
comparisons are source- and image-identical to prior reviewed versions. The
existing em-layout-cascade240 fractional-edge failure remains (exact geometry);
A09 is still partial. See validation/stylesheet-preservation-review.md.

S09/S10 alignment invalidation: compiler162/162, native/WASM builds and accepted
corpus parity pass. Chrome/native108/108 pass at240/390/768 with scenes compiled
at390. Three new sheets inspected (stretch, start, inherited right-aligned Inter
wrapping);99 earlier scene/width comparisons are source- and image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-alignment/. Existing
full-corpus fractional-edge failure remains documented separately.

S09/S10 radius invalidation: compiler163/163, native/WASM builds and full accepted
corpus parity pass. Chrome/native114/114 pass at240/390/768, compiled at390.
Two new sheets inspected (square reset corners versus retained rounded controls);
all108 previous scene/width comparisons are source- and image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-radius/. Tolerances unchanged.

S09/S10 font longhands: compiler165/165, native/WASM builds and full accepted
corpus parity pass. Chrome/native123/123 pass at240/390/768, compiled at390.
Three new Inter sheets inspected: font sizes, regular weight inheritance,
line wrapping and inherited unitless line-height match. All114 prior comparisons
retain identical source and browser/native images. Evidence:
output/playwright/html-to-riv/custom-properties-font/. Tolerances unchanged.

S09/S10 family lists: compiler166/166, native/WASM builds and full accepted
corpus parity pass. Chrome/native129/129 pass at240/390/768, compiled at390.
Two new Inter sheets inspected; glyphs, wrapping and inherited family agree.
All123 prior comparisons retain identical source and browser/native images.
Evidence: output/playwright/html-to-riv/custom-properties-family/.

S09/S10 family keyword correction: compiler167/167, native/WASM builds and full
accepted-corpus parity pass. Chrome/native132/132 pass at240/390/768. New text
sheet inspected;129 prior comparisons source- and image-identical. Direct
Chrome control records accepted syntax separately from computed substitution
behavior. Evidence: output/playwright/html-to-riv/custom-properties-family-keywords/.
Leading CSS-wide words in multi-word substitutions remain unqualified.

S09/S10 leading CSS-wide family prefixes resolved: compiler168/168, native/WASM
builds and full accepted-corpus parity pass. Chrome/native135/135 pass at all
three widths. New sheet inspected;132 prior comparisons source/image-identical.
Direct, variable and fallback Chrome control retained with the public red test:
output/playwright/html-to-riv/custom-properties-wide-prefix/. The prior leading-
word qualification gap is closed for this font-family substitution behavior.

S09/S10 font shorthand cascade: compiler169/169, native/WASM builds and full
accepted-corpus parity pass. Chrome/native141/141 pass at240/390/768, compiled
at390. Both new sheets inspected;135 prior comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-font-cascade/.
No production-code or tolerance change was required for these cascade cases.

S09/S10 font shorthand grammar stage: compiler170/170, native/WASM builds and
accepted-corpus parity pass. Chrome/native147/147 pass at240/390/768, compiled
at390. Both new sheets inspected;141 prior comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-font-grammar/.
Excluded-prefix grammar combinations and other property families remain pending.

S09/S10 font prefix invalidation: compiler171/171, native/WASM builds and full
accepted-corpus parity pass. Chrome/native150/150 pass at240/390/768, compiled
at390. New text sheet inspected;147 prior comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-font-prefix/ includes
public red test, direct Chrome syntax control, logs and image hashes.

S09/S10 text-transform invalidation: compiler172/172, native/WASM builds and full
accepted-corpus parity pass. Chrome/native156/156 pass at240/390/768, compiled
at390. Two new sheets inspected: inherited uppercase/capitalization and valid
lowercase override match Chrome glyphs and wrapping.150 prior comparisons
source/image-identical. Evidence: output/playwright/html-to-riv/custom-properties-transform/.

S09/S10 white-space invalidation: compiler173/173, native/WASM builds and full
accepted-corpus parity pass. Chrome/native162/162 pass at240/390/768, compiled
at390. Two new sheets inspected: preserved repeated spaces/newlines, collapsed
spaces and wrapping match Chrome.156 prior comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-whitespace/.

S09/S10 decoration enums: compiler174/174, native/WASM builds and accepted-corpus
parity pass. Chrome/native168/168 pass at240/390/768, compiled at390. Both new
underline sheets inspected, including inherited none versus auto skip-ink;
162 prior comparisons source/image-identical. Public tests compare runtime
requirements as well as Rive bytes. Evidence:
output/playwright/html-to-riv/custom-properties-decoration-enums/.

S09/S10 decoration lines: compiler175/175, native/WASM builds and accepted-corpus
parity pass. Chrome/native174/174 pass at240/390/768, compiled at390. Both new
sheets inspected: local line removal and ancestor underline preservation agree.
168 prior comparisons source/image-identical. Public runtime-record comparisons
included. Evidence: output/playwright/html-to-riv/custom-properties-decoration-lines/.

S09/S10 decoration metrics: compiler176/176, native/WASM builds and accepted-corpus
parity pass. Chrome/native180/180 pass at240/390/768 from scenes compiled at390.
Both new sheets inspected: auto thickness reset versus explicit thickness,
inherited offset versus negative offset, and wrapping match Chrome. All174 earlier
comparisons are source/image-identical. Tolerances unchanged. Syntax/computed-value
control and public-before failure retained with logs, gallery and sheets at
output/playwright/html-to-riv/custom-properties-decoration-metrics/.
This incremental qualification does not resolve the recorded full-suite A09 edge
failure or qualify all remaining custom-property grammar.

S09/S10 underline position: compiler177/177, native/WASM builds and accepted-corpus
parity pass. Chrome/native183/183 pass at240/390/768 from scenes compiled at390.
The new sheet was visually inspected at all widths: underlines, skip-ink and
wrapping match Chrome. All180 earlier comparisons are source/image-identical.
Control, original public failure, logs and gallery:
output/playwright/html-to-riv/custom-properties-decoration-position/.
Only auto position is representable; non-auto inheritance remains outside this
qualification. Tolerances unchanged; earlier full-suite A09 failure remains.

S09/S10 decoration shorthand: compiler178/178, native/WASM builds and accepted-
corpus parity pass. Chrome/native189/189 pass at240/390/768 from390 compilation.
Both new sheets inspected: valid shorthand substitution, complete reset, later
longhands, important reset and offset preservation match Chrome. All183 earlier
comparisons source/image-identical. Evidence (including preflight failure):
output/playwright/html-to-riv/custom-properties-decoration-shorthand/.
Function-containing malformed shorthand grammar remains under audit; tolerances
unchanged and the earlier full-suite A09 edge failure remains unresolved.

S09/S10 decoration color functions: compiler179/179, native/WASM builds and
accepted-corpus parity pass. Chrome/native195/195 pass at240/390/768 from390
compilation. Both new sheets inspected at all widths: invalid RGB/HSL reset,
duplicate colors, later underline and valid HSL control match Chrome. All189
prior comparisons source/image-identical. Evidence and Chrome syntax control:
output/playwright/html-to-riv/custom-properties-decoration-functions/.
A mistaken negative test for modern numeric HSL was corrected using Chrome;
its failure is retained. Tolerances unchanged; broader grammar and the earlier
full-suite A09 pixel failure remain open.

S09/S10 decoration reset matrix: compiler181/181, native/WASM builds and accepted-
corpus parity pass. Chrome/native204/204 pass at240/390/768 from390 compilation.
Three new sheets inspected at all widths: missing/empty preserves ancestor lines,
CSS-wide fallbacks retain proper origins, and custom-wide inheritance differs
from invalidation/fallback recovery. Runtime-record equality also checks origins
that can overlap visually. All195 earlier comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-decoration-wide/.
No production changes or tolerance changes. Remaining grammar audits and the
previous full-suite A09 pixel failure remain open.

S09/S10 simple background invalidation: compiler182/182, native/WASM builds and
accepted-corpus parity pass. Chrome/native210/210 pass at240/390/768 from390
compilation. Both new sheets inspected at all widths: transparent reset removes
prior paint and later background-color restores the rounded blue control.
All204 previous comparisons source/image-identical. Original failure and Chrome
control retained at output/playwright/html-to-riv/custom-properties-background/.
Complex background grammar remains under audit. Tolerances unchanged; the earlier
full-suite A09 edge failure remains open.

S09/S10 background color functions: compiler183/183, native/WASM builds and
accepted-corpus parity pass. Chrome/native216/216 pass at240/390/768 from390
compilation. Both new sheets inspected: duplicate/function/layer resets, valid
HSL and later-color controls match Chrome. All210 previous comparisons are
source/image-identical, including decoration cases sharing the helper. Evidence:
output/playwright/html-to-riv/custom-properties-background-functions/.
Tolerances unchanged. Full background grammar and the earlier A09 failure remain.

S09/S10 realistic compositions: compiler184/184 and native/WASM accepted-corpus
parity pass. Chrome/native220/222 pass; all216 prior comparisons source/image-
identical. Both new light/dark sheets inspected at every width. Two240px failures
remain in the underlined note (interior RGB error6.2202 light/7.2728 dark against6).
Geometry is within0.03125px. Pixel-row analysis of the light note: Chrome underline
is a solid row199; native splits coverage across199/200 because of fractional
placement. No scene simplification or tolerance change applied.390/768 pass.
Evidence: output/playwright/html-to-riv/custom-properties-compositions/.
Next: diagnose runtime underline snapping under fractional global translation.
Earlier full-suite A09 failure also remains unresolved.

Fractional underline fix qualification: compiler184/184, runtime decoration8/8,
native/WASM builds and accepted-corpus parity pass. Original/minimal/control12/12
native pass and were visually inspected. A precise solid-blue-row control fails
on pre-fix images and passes all6 reduced/control renders after the fix, also in
the full run. Native full suite1,470/1,471 passes (1,460/1,461 scene comparisons plus
10 pixel-check tests). Only existing em-layout-cascade240 fails; its images are
unchanged. All1,461 scene pairs source/image-identical to reviewed old full,
custom-composition and fixed-control baselines. Current custom-property222/222
now pass. DPR27/27 passes; all54 images/request sources match reviewed callback
baseline. Vector focused3/12 remains unqualified; all four sheets inspected.
Evidence: validation/underline-fractional-translation-review.md and
output/playwright/html-to-riv/underline-fractional-{fixed,full,vector,dpr}/.
No tolerance widening or authored scene simplification; broader backlog remains.

A09 paint-bound diagnosis: exact logical geometry, original failure reproduced;
one-rectangle fractional reduction3/3 fails while integer3/3 passes. Experimental
runtime opt-in fixes original plus controls9/9 and nearby117/117. All original/
reduction/control sheets and nine changed nearby families visually inspected;
94/117 nearby pairs match explicit reviewed baselines. Compiler184/184 passes.
This is not published-feature qualification: serialization, host capability/target
validation, default-Rive controls, DPR and full regression remain to implement.
Receipt: validation/em-edge-paint-review.md. Diagnostic flag and reproducers
preserved; no tolerances changed.

Layout paint contract v5 integrated: Rust188/188, TypeScript, native/WASM builds
and JavaScript9/9 pass (full accepted-corpus parity included). Tests verify version,
capability, duplicate/out-of-range/wrong-type targets, old contracts and mixed text
records. Probe rejection tests cover unavailable capability, style-object target,
missing target and duplicates. Same Rive bytes without the policy retain identical
logical bounds but different paint commands. Native117/117 pass with no experimental
switch; all images match the previously inspected em-edge-nearby experiment.
DPR27/27 pass; comparison receipt stored in layout-contract-dpr. Full native run
started at output/playwright/html-to-riv/layout-contract-full/; completion pending.
Evidence: output/playwright/html-to-riv/layout-contract/. Tolerances unchanged.

Dedicated layout paint DPR checks: 27/27 pass (fractional rectangle, rounded
background, rounded overflow clip × widths 240/390/768 × DPR 1/2/3), with
native/WASM bytes/maps/requirements parity and same-scene resizing. All nine
comparison sheets inspected; small corner antialiasing differences remain within
unchanged limits. This supplements the separate underline DPR tests. See
validation/em-edge-paint-review.md and artifacts in
output/playwright/html-to-riv/layout-pixel-bounds-dpr/. Full regression and clone
qualification remain pending; no affine or fractional-DPR claim.

Clone-policy validation now passes: `cargo test -p nuxie-runtime --lib
css_pixel_bounds_survive_clone_and_can_be_disabled` ran 1/1 test successfully.
The test checks raw world paint bounds, opt-in snapped bounds, preserved behavior
on a prepared clone, reversible disabling on the clone, unchanged source paint,
and unchanged logical layout. Log: output/playwright/html-to-riv/layout-contract/clone.log.
Full visual session 65444 is still active; full regression and changed-image review
remain pending.

Full layout-contract native regression completed: 1477/1477 checks pass
(1467 scene comparisons plus 10 pixel controls), including the original A09 edge
failure. Review comparison transfers 1226 pairs from explicit reviewed baselines;
241 passing comparisons across 96 families still need visual inspection.
Artifacts: output/playwright/html-to-riv/layout-contract-full/, including pixels.log
and baseline-comparison.json. Receipt: validation/em-edge-paint-review.md.
Full process 65444 is terminal; no restart required.

Full layout-contract inspection progress: 64/241 changed scene comparisons
visually reviewed; 177 comparisons in 73 families remain. Flex, percentage sizing
and all 12 deterministic layout matrix sheets now reviewed. Full numeric gate
remains 1477/1477; thresholds unchanged. Queue and per-sheet observations:
output/playwright/html-to-riv/layout-contract-full/visual-inspection.json.

Third full-run inspection batch: all 38 remaining selector-family sheets,
76 changed comparisons, inspected. Includes adjacent/general siblings,
attributes, structural/nth selectors, negation, :is/:where and plan-card compositions.
Colors, visible-item ordering, dimensions and gaps agree with Chrome. Rounded
cards, buttons and stacks retain small corner antialiasing differences within
unchanged limits; no shifted straight edges observed. Queue now records
140/241 changed comparisons reviewed; 101 comparisons in 35 custom-property
families remain. Per-sheet observations and exact widths are retained in
`output/playwright/html-to-riv/layout-contract-full/visual-inspection.json`.

## Full native visual qualification complete

Final batch: all 35 remaining custom-property sheets / 101 changed comparisons
inspected. Colors, reset/empty regions, inheritance/fallback results, flex placement,
and themed-card geometry agree with Chrome; small rounded-corner antialiasing
differences remain inside unchanged limits. The inspection queue is empty:
241/241 changed comparisons inspected, plus 1226 image/source-identical pairs
whose review transferred from explicit reviewed baselines. All 1467 scene pairs
now have review evidence. Full numeric result remains 1477/1477 checks.

This closes native qualification of the version-5 layout paint contract and
resolves the original A09 fractional shape-edge failure. Rust188, JS9 (corpus
native/WASM parity and host rejection), TypeScript, native/WASM builds, clone1,
layout DPR27 and separate underline DPR27 already pass. Ordinary Rive policy
remains opt-in; no logical dimension rounding or tolerance widening.

A09 remains partial for vector text: earlier em-nested-typography and
em-font-shorthand-final-size 240px failures require fresh accounting. Focused
vector run started in session60933, `/tmp/html-layout-em-vector.log`, output
`layout-contract-em-vector/`. Resume that handle before restarting. Affine and
fractional-DPR qualification is not established. S09/S10 broader substituted-value
grammar audit remains independent work; the overall backlog is not complete.

## Focused vector result

The vector rerun terminated: 31/33 comparisons pass. It covers five em families (em-flex-sizing,
em-font-shorthand-final-size, em-global-values, em-layout-cascade,
em-nested-typography), four rem families, and two background controls.
All 11 browser/vector/diff sheets were visually inspected at all three widths.
Shape edges, spacing and responsive geometry align; text shows the existing
vector coverage differences. Original em-layout-cascade240 now passes.

The two failures are em-nested-typography240 (mean channel error1.2762630208)
and em-font-shorthand-final-size240 (1.0319140625), both above the unchanged1
limit and matching the historical failure metrics. No claim of image identity
with an old vector run is made. All 12 rem and six background-control comparisons
pass. Artifacts: output/playwright/html-to-riv/layout-contract-em-vector/,
including pixels.log, review.json and inspection/*.png. Sessions60933 and63329
are terminal; no work is still running.

A09 native qualification is complete; vector text fidelity remains partial.
Next independent compiler work resumes S09/S10 substituted-value grammar audit.

S09/S10 non-length dimension increment implemented: angle/time/frequency/
resolution/flex units invalidate substituted length slots; valid excluded lengths
retain diagnostics. Chrome182 references and two public Rust regressions pass.
Native/WASM and visual qualification pending. Receipt:
validation/non-length-dimensions-review.md.

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

S09/S10 background keyword grammar increment implemented: substituted duplicate
repeat/attachment/box groups reset to unset; valid excluded combinations retain
diagnostics. Qualified: Chrome18 references, full module191, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding234 pass. All234
source/image pairs match reviewed baselines. Remaining complex background grammar
is still in progress. Receipt: validation/background-keywords-review.md.

S09/S10 font prefix grammar increment implemented: duplicate style/variant/weight/
width prefixes, excess normal slots and malformed suffixes reset substituted font
values to inherited values. Oblique scalar angles checked; valid excluded styled
fonts retain diagnostics. Qualified: module192, Chrome24 references, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding240 pass. All240
source/image pairs match reviewed baselines. Chrome100grad boundary discrepancy
preserved; broader substitution grammar remains in progress.
Receipt: validation/font-prefix-review.md.

S09/S10 bare-block substitution increment implemented: top-level (), [] and {}
blocks reset ordinary values before normalization; function contents and quoted
strings retain existing behavior. Qualified: module193, Chrome65 references,
JS9/native-WASM corpus parity, TypeScript, new native6 inspected and surrounding246
pass. All246 source/image pairs match reviewed baselines. Broader grammar remains
in progress. Receipt: validation/substitution-block-review.md.

S09/S10 background physical position/size grammar increment implemented: invalid
axis/offset/size/slash groups reset substituted backgrounds; valid excluded forms
retain diagnostics. Qualified: module194, Chrome51 references, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding252 pass. All252
source/image pairs match reviewed baselines. Broader grammar remains in progress.
Receipt: validation/background-position-review.md.

S09/S10 function-type increment implemented: known color/image/numeric functions
in incompatible ordinary properties reset after substitution. Compatible and
unknown functions retain existing profile behavior. Qualified: module195, Chrome96,
JS9/native-WASM corpus parity, TypeScript, new native6 inspected and surrounding258
pass. All258 source/image pairs match reviewed baselines. Broader grammar remains
in progress. Receipt: validation/function-type-review.md.

L01 reverse flex directions implemented using existing format/runtime enums.
row-reverse/column-reverse preserve authored source identity and support explicit
inheritance, initial/unset and var(). Module197, JS9/native-WASM parity and
TypeScript pass. Both renderer profiles pass58/60; all sheets inspected or matched
to reviewed images, including corrected visible alignment fixtures. Two narrow
text failures reduce to a pre-existing normal-word-wrap bug (48px word block has
native height48 vs Chrome24); pre-line control passes. L01 remains partial while
that actionable runtime/compiler policy gap is fixed. S09/S10 broader grammar
remains open. Receipt: validation/reverse-flex-review.md.

CSS normal wrapping follow-up: the compiler now requires an explicit
text-css-normal-wrap-v1 capability and css-normal-wrap-v1 Text occurrence policy
(requirements versions2–5). Hosts install Text::set_css_normal_wrap(true) after
validation. Raw Rive behavior is unchanged. Overlong words overflow whole rather
than split by glyph. Module199, JS9/parity, host3 and TypeScript pass. Focused
native69/69 and vector60/69: all reverse-flex60 comparisons now pass in both
profiles; nine new normal-word vector comparisons retain local pixel failures.
All focused images are visually accounted for. Full native regression is running;
L01 remains partial until that result is reviewed. No new word-break/hyphenation
support and no tolerance changes. Receipt: validation/normal-wrap-review.md.

Full normal-wrap regression update: existing preserved Unicode-space fixtures
now expose fitting/alignment regressions (at least fifteen comparisons while the
run is active). The new policy incorrectly shares pre-line space-hanging rules.
Correction and regression rerun are required; focused success does not qualify
the feature. See validation/normal-wrap-review.md for evidence and next action.

Original normal-wrap full run completed1,550/1,582:15 Unicode-space failures
and17 decoration failures. The source space-width correction is staged; its
validation and the decoration glyph-range correction remain pending. Full review
comparison transfers1,508 unchanged pairs, with64 requiring inspection.
Receipt: validation/normal-wrap-review.md.

Corrected normal-wrap native coverage passes471/471: preserved Unicode spaces
retain fitting/alignment width, and collapsed trailing ASCII spaces no longer
extend decoration glyph ranges. All471 source/browser/native image pairs are
identical to reviewed baselines (layout-contract-full, function-type-custom and
normal-wrap); durable visual transfer is complete. Runtime3/3, module199/199,
host3/3 and JS9/9 native/WASM parity pass. Native rebuild succeeded after clearing
only reproducible incremental build cache. Vector471 coverage is running; full
native rerun remains pending. Earlier32 failures are preserved, not overwritten.
Evidence: validation/normal-wrap-review.md and normal-wrap-corrected artifacts.

Corrected vector lane:432/471 pixel checks, all471 geometry checks pass (maximum
0.034px). 303 pairs transfer prior visual review;18 more inspected,150 pending.
All39 pixel failures remain unqualified. Full corrected native regression is
running. Evidence: validation/normal-wrap-review.md.

Corrected vector visual review complete: all471 pairs accounted for by303
source/browser/render-identical transfers and168 direct inspections (56 sheets
at three widths). Whitespace, tab stops, overlong-word overflow, ligature-word
wrapping, decoration inheritance/reset/color/width and placement agree in the
reviewed scenes. Visible glyph raster differences remain;39 pixel failures are
not waived. Geometry471/471 passes (max0.034px), pixels432/471. Durable evidence:
normal-wrap-corrected-vector/visual-inspection.json and baseline-comparison.json.
Full corrected native regression remains running.

## Final corrected native qualification

Full native regression passes1,582/1,582 (1,572 scene comparisons plus10 controls),
/tmp/normal-wrap-corrected-full.log. All1,572 source/browser/native image pairs
are identical to reviewed baselines: layout-contract-full, function-type-custom
and normal-wrap-corrected. Transfer evidence and completed visual-inspection.json
are in normal-wrap-corrected-full. Both original normal-wrap regressions are
resolved without tolerance changes. All processes for this increment are terminal.

L01 reverse flex directions is qualified for its documented profile:60/60 reverse
comparisons pass in both glyph renderer modes, source identity tests pass,
native/WASM parity passes, and the full native regression is green and reviewed.
The normal-wrap policy has native qualification; its nine focused vector pixel
failures and the broader corrected vector lane's39 pixel failures remain open
renderer limitations. Vector geometry471/471 passes and all471 pairs are reviewed.
Do not interpret L01 qualification as a complete vector-text qualification or as
completion of the compiler backlog. Next independent item L02 remains unsupported;
its source/runtime investigation is in order-investigation.md.

L02 order is implemented but not qualified. Signed CSS integers, finite integer
clamping to signed32 endpoints, CSS-wide resets and var() use stable visual
sibling ordering while source IDs/paths and DOM selector matching remain authored.
No runtime or format extension is used. Browser paint/geometry/parity validation
is pending; see validation/order-review.md.

L02 current evidence: module201 and native/WASM builds pass. Native12/18;
six overlap pixel failures despite exact geometry. No-order/painted-parent
controls fail9/9, exposing a broader paint-traversal issue to investigate.
order remains unqualified. Evidence: validation/order-review.md.

L02 paint investigation: DrawRules attempts were rejected and removed; restored
public201/201 and native/WASM9/9 pass. Focused native remains12/18, with all18
images reviewed and six overlap failures. `order` remains unqualified. See
validation/order-review.md for experiments and the whole-subtree paint dependency.

L02 experimental update: default-off whole-subtree paint policy passes39/39
focused native cases (including corrected nested clips), all visually reviewed;
public suite202/202 passes. This still requires a published capability contract
and broad qualification. Ordinary compiler output does not yet enable it. See
validation/order-review.md for exact runs, failed attempts and remaining gates.

L02 shipping update: `layout-css-paint-order-v1` now accompanies scenes with
sibling layouts and the host installs it from the manifest; no experimental flag
is needed. Public203, host4, parity9 and focused native30 pass, all30 images match
reviewed baselines. Broad1612 native validation is in progress, so qualification
remains pending. See validation/order-review.md and README host instructions.

L02 direction correction: original full native1609/1612 failed reverse-overflow
at three widths. Direction-aware paint groups now pass45/45 focused native,
with33 identity transfers and12 direct reviews. Public205 and parity9 pass.
Vector42/45 has three explicit text-card raster failures; geometry45/45 passes
(max0.022491455px), all images reviewed. Corrected full native1624 run remains
active in /tmp/order-direction-full.log (session12827). Qualification is pending.

L02 final native qualification: corrected full1624/1624 passed; all1614 scene
pairs match reviewed source/browser/native image baselines and visual-inspection
has no remaining entries. Public205, host4, parity9 and types pass. Per-instance
policy, capability rejection, resize, direction, source identity and overlap tests
are covered. Focused vector42/45 pixels and45/45 geometry remain explicitly limited
by three text-card raster failures; no tolerance changes. Evidence:
validation/order-review.md and output/playwright/html-to-riv/order-direction-full.
All L02 processes are terminal; the overall backlog remains active.

L03 `align-self` is qualified for the native glyph profile. Accepted values:
auto, flex-start, center, flex-end, stretch and baseline, plus existing CSS-wide
and var() behavior. The property is not implicitly inherited; auto resolves the
parent alignment. Non-auto values require version6 typed layout targets and
`layout-css-align-self-v1` host support. Modern modifiers and other keywords
remain intentionally unsupported.

Public210 tests plus the added lifecycle regression pass; the focused runtime
suite is4/4 including432 Chromium box comparisons. Host5/5, native/WASM
parity9/9 and TypeScript checks pass. Full native1723/1723 and new compositions9/9
pass, with all image pairs visually accounted for. The27 text-baseline cases
include wrapping, leading, padding, images and nested intrinsic-width text.

Vector geometry108/108 passes (maximum error0.013671875px), but pixels pass78/108.
Thirty text raster failures remain explicit limitations of that profile. All108
pairs are reviewed; native glyph pixels pass for the same source scenes. No
tolerances changed. Exact logs, receipts and preserved reproducers are in
validation/align-self-investigation.md.

L04 `align-content` is qualified for the native glyph profile.
Accepted: flex-start, center, flex-end, stretch, space-between and space-around.
Omission uses the authoring reset's flex-start; initial/unset use stretch.
Explicit inherit and var() rules apply; the property is not implicitly inherited.
Other keywords and modifiers remain intentionally unsupported. Version7 and
`layout-css-align-content-v1` carry strict per-layout targets. Every wrapped
container receives an independent line policy, separating line placement from
item alignment and justification.

Public216/216 tests, host6/6, types and final
corpus native/WASM parity9/9 pass. Corrected mixed-size native297/297 comparisons
pass and all images are reviewed, including overflow and realistic compositions.
The oracle asserts intended mixed sizes to prevent a discovered selector
specificity mistake from returning; obsolete equal-size evidence is retained.
Vector geometry297/297 passes (maximum error0.009376526px), while pixels pass
290/297. Seven composition text raster failures remain explicitly open; all
vector pairs are visually accounted for. Full native regression2029/2029 passes;
all2019 image pairs have exact source/browser/native hash matches to reviewed
baselines. See validation/align-content-investigation.md for logs and receipts.

L05 distributed spacing is qualified for the native glyph profile. New values:
justify-content space-around/space-evenly and align-content space-evenly.
Version8 and layout-css-distributed-spacing-v1 gate host policies; original
CSS now passes288 Chromium geometry comparisons plus lifecycle tests. Typed
API checks, public220/220, host7/7 and native/WASM parity9/9 pass. Native297/297
pixel comparisons pass and all images are reviewed. Vector geometry297/297
passes, but8 text composition pixel failures remain open and reviewed. Full
native regression2326/2326 passes; all2316 image pairs match reviewed baselines. Existing exclusions
remain in place. See validation/distributed-spacing-investigation.md.

L06 wrap-reverse is implemented, with pixel qualification pending. It uses
Rive's existing wrap value2 and the existing independent line-alignment policy.
Public parser tests and336 Chromium reference layouts on original and cloned
scenes pass. Reversed stretch/space-between overflow fallback is corrected;
all98 isolated layout-engine tests pass. Public223/223 and host8/8 pass; native
and WASM builds pass. Expanded parity9/9 passes.
Native348+3 pixel comparisons pass, including an asserted multiline baseline.
All15 compositions and336/336 box cases reviewed; full
regression remains pending.
Vector geometry351/351 passes;14 text pixel failures are preserved and visually reviewed. See validation/wrap-reverse-investigation.md.

L06 vector review completed: all15 composition comparisons inspected; all336 box pairs have exact source and browser/native PNG identity with reviewed native cases. Geometry351/351 passes (maximum0.015625px); vector pixels337/351 pass. The14 text pixel failures remain explicit renderer limitations with unchanged thresholds. Full native qualification remains pending.

L07 auto margins: source implementation and tests are prepared but unqualified. Accepted syntax targets one-to-four `auto` or existing nonnegative lengths in margin shorthand, plus physical longhands and existing cascade rules. Padding auto, negative and percentage margins remain excluded. The runtime test corpus includes132 scenes/396 viewport references including root auto margins;136 visual fixtures/408 widths include four realistic text compositions. Builds and test execution await completion of the L06 regression to preserve its executable inputs.

L06 final native qualification: full2677/2677 passed (session47222 exit0, `/tmp/wrap-reverse-full.log`). All2667 image pairs match reviewed baselines in source HTML/CSS and both PNG hashes; the full visual receipt has no remaining inspections. Execution binaries, WASM and case corpus verified unchanged through completion before any L07 rebuild. Native wrap-reverse is qualified; vector geometry passes but14 reviewed text pixel failures remain explicit limitations.

L07 geometry milestone: both auto-margin layout-engine fixes pass396 Chromium reference viewports on original and cloned scenes; all100 standalone Taffy tests pass. The136 visual fixtures are merged. Full public/build/parity/pixel qualification remains pending; auto margins are not yet native-qualified.

L07 public validation:226/226 compiler tests pass, native publisher/probe and WASM builds pass. Focused408 native pixel comparisons, parity and host checks are running; visual qualification remains pending.

L07 focused native408/408 passes, parity9/9 and host9/9 pass. All12 native composition pairs visually reviewed. The396 box cases still need review; vector and full-native evidence remain pending. Tolerances unchanged.

L07 native visual review complete: all408 comparisons reviewed. Vector geometry408/408 passes (maximum0.0107421875px); six composition pixel failures await detailed visual review. Full native regression remains active; qualification pending.

L07 vector visual review complete: all408 pairs accounted for;402 pixel passes and6 preserved text raster failures. Auto-margin geometry and wrapping agree with Chromium. Full native regression remains pending.

L08 independent factors: source compile checks and JS types pass; runtime execution remains pending preservation of the active L07 regression inputs. Chromium captures cover96 scenes/288 viewports and three compositions/nine viewports. All nine browser composition images were directly reviewed with hashed receipt; these are fixture references, not native visual qualification. Prospective99 fixtures are separate from the active case corpus. Lifecycle tests include resizing, cloning, changing factors without resize, clearing policies and invalid values. Public/native/WASM/host/pixel/full regression gates remain outstanding.

L07 final native qualification: full regression15587 exited1 with3084 passes and one ENOSPC screenshot write failure (`/tmp/auto-margin-full.log`). The zero-byte Chromium image and failed run remain intact. Isolated same-input retry47545 passes1/1 (`/tmp/auto-margin-capture-retry.log`). All3074 remaining image pairs have exact source and both PNG identity with reviewed baselines; the one retry pair also matches its reviewed baseline exactly. All five input hashes verified unchanged through retry. Combined coverage3085/3085 is qualified for native; full receipt remaining[]. This is a full run plus one evidenced infrastructure retry, not a clean full-run exit. Six reviewed vector text pixel failures remain.

L08 executed milestone: public232/232 passes (59786, `/tmp/independent-flex-module3.log`), including288 Chromium geometry references on original/clone and same-viewport mutation/clear lifecycle. Native and WASM builds pass, parity9/9 (9452), host10/10 and typecheck pass. Focused native297/297 passes (51095, `/tmp/independent-flex-native.log`). All nine composition comparisons and72 unique box pairs directly inspected across12 sheets;216 exact box duplicates account for all288 box cases. Native visual receipt remaining[]. Tolerances unchanged. Vector297 run72737 active (`/tmp/independent-flex-vector.log`); full regression pending, so L08 remains unqualified overall.

L08 vector run72737 exited1:296/297 pixels pass; narrow panels body interior RGB error8.6835 exceeds unchanged6 threshold. Geometry independently audited297/297, max0.028076171875px; all source IDs and hidden states agree. All nine composition pairs directly reviewed (glyph raster differences), and288 box pairs match fully reviewed native sources and both PNG hashes exactly. Vector visual receipt remaining[]; pixel failure retained. Full native3382 regression20764 is running (`/tmp/independent-flex-full.log`). All five input hashes verified and recorded; do not rebuild or modify the case corpus until terminal.

L09 partial factors remain rejected by the compiler. Isolated layout-engine fix removes duplicate gap subtraction;432 Chromium reference viewports and all101 engine tests pass. This is not compiler or native-renderer qualification. L08 full regression continues with recorded inputs unchanged.

L09 evidence expanded to240 scenes/720 Chromium references; all101 isolated engine tests pass, including min/max constraints and mixed sibling factors/bases. Nine browser-only composition references directly reviewed, including deliberate partial-shrink overflow.244 prospective pixel fixtures and public original/clone test are prepared separately. Compiler admission and native/WASM/rendering execution remain pending L08 full regression.

L09 source candidate accepts finite factors in[0,10000]; sub-unit factors require version10 and layout-css-partial-flex-factors-v1 plus explicit factor payloads. Missing capability, wrong versions and inconsistent payloads reject before drawing. Equal and unequal partial factors are covered. Content-derived auto basis and indefinite percentage basis remain excluded. Source compile and type checks pass; executed public/native/WASM/pixel validation awaits completion of the unchanged L08 regression. This supersedes earlier source-admission notes, without claiming qualification.

Validation correction: L08 full20764 was invalidated and stopped after cargo rustc --test also rebuilt the publisher; post-build hash verification caught the change. The stopped run cannot qualify L08. Frozen executable copying and browser-harness hash verification are now implemented for the restart. L09 cascade/contract tests and720 public original/clone Chromium references pass in directly executed test artifacts. Full public suite9542 and coherent native/WASM builds are running; neither feature is fully qualified yet.

For long pixel runs, create a fresh snapshot with `node tools/html-to-riv/validation/snapshot-toolchain.mjs NEW_DIRECTORY` after coherent native/WASM builds. Pass its absolute path as NUXIE_HTML_TOOLCHAIN to Playwright. Snapshot copying refuses existing directories, copies rather than links all four artifacts, and records SHA256 hashes; browser.spec checks every hash and uses those copies, including the browser-loaded WASM. No renderer override is allowed with a snapshot. Keep source fixtures fixed through each full run. This prevents a later Cargo test/build from mutating the running validation executables.

L09 public suite9542 passes236/236 (`/tmp/partial-flex-module.log`). After adding244 pixel fixtures to the main corpus, the two tests that embed the corpus were rerun (55847 exit0, 96 tests; `/tmp/partial-flex-expanded-contract.log`). Native/WASM parity44630 passes9/9 across the expanded corpus. Frozen snapshot smoke84609 passes21/21; all21 source/browser/native image pairs match reviewed baselines exactly and receipt remaining[]. Negative checks confirm hash mismatch aborts before test execution and snapshot overwrite is rejected. Focused native732 run52606 active (`/tmp/partial-flex-native.log`), explicitly using partial-flex-toolchain. Full and vector validation remain pending.

L09 interim native visual review: first90 passed box comparisons captured from completed per-case artifacts while52606 continues. All61 distinct pairs inspected across11 contact sheets;29 exact duplicates covered by PNG hashes. Batch receipt remaining[] applies only to these90 cases, not the complete732 run. See partial-flex-native-review-batch1/visual-inspection.json and partial-flex-box-review-batch1/manifest.json. Remaining images, text compositions, vector and full regression still pending.

L09 interim visual coverage now360/732. Batch2 covers cases91..180:16 new unique pairs inspected,10 within-batch duplicates and64 exact pair matches to reviewed batch1. Batch3 covers all180 reverse-row cases:77 unique pairs inspected across13 sheets,103 exact duplicates. Receipts for each batch have no remaining inspections within their stated scope. Column cases, compositions and final run status remain pending;52606 still active.

L09 focused native52606 passes732/732 (7.5m, `/tmp/partial-flex-native.log`). Column batch4 completed:90 unique pairs inspected across15 sheets,90 exact duplicates, covering180 source cases. All nine text composition comparisons directly reviewed: unused free space, panel sizing and intentional narrow viewport overflow agree with Chromium. Native overall receipt tracks549 reviewed and183 remaining (reverse-column boxes plus original two-child reproducer). Batch5 sheets generated (81 unique pairs/14 sheets), none inspected yet. Vector732 run20863 active (`/tmp/partial-flex-vector.log`), using the same frozen toolchain. Full regression restart remains pending.

L09 focused visual review complete: native732/732 passes and all732 images accounted for. Final batch5 covers183 cases through81 directly inspected pairs and102 exact duplicates; audited all five batches cover723 boxes exactly once, plus9 reviewed compositions. Vector20863 exited1:731/732 pixel passes; all732 geometries pass (maximum0.01251220703125px),723 source/browser/native PNG pairs exactly match reviewed native images, and all9 composition pairs directly inspected. The240px panels text interior RGB error7.90357023690357 exceeds unchanged6 limit; preserved as a vector text raster limitation. Full native4114 regression restarted on frozen partial-flex-toolchain (session94020, /tmp/partial-flex-full.log); qualification remains pending.

L10 investigation started:33 current compiler rejections preserved and99 Chromium content-auto reference viewports captured. Content-derived auto basis remains unsupported; no qualification claimed. See validation/content-auto-investigation.md.

L10 working source is now experimental: intrinsic dimension/factor transport passes48 nested-box original/clone reference viewports. Initial full99 geometry comparisons expose21 row-text failures, preserved in content-auto-experiment-initial. Text measurement experiment and broader qualification remain pending; this is not supported-profile qualification.

L10 second experiment: removing the grow-zero restriction from intrinsic row text makes all99 geometry comparisons pass (maximum0.015625px; session80366 exit0, /tmp/content-auto-compare-intrinsic.log). Original390-compiled bytes were imported at240/390/768. Preserve both initial21-failure and corrected outputs. Geometry alone does not prove text wrapping or pixels; expanded coverage, public contract updates, WASM parity and visual qualification remain pending.

L10 expanded stress matrix adds48 scenes/144 viewports: direct and nested text, four directions, non-stretched cross sizing, unequal sibling factors, min/max, wrapping, nowrap and indefinite height. Initial comparison54725 exits1 with104 failing viewports (maximum490.390625px), preserved in content-auto-expanded-initial/geometry.json. These invalidate any broad interpretation of the earlier99-pass subset. Generalizing intrinsic text measurement beyond direct row children is the next experiment; expanded screenshots are captured but not yet visually reviewed.

L10 all-auto-width intrinsic measurement experiment builds successfully (74612 exit0). Expanded comparison17749 exits1: failures drop104→62 of144, maximum286.390625px. Evidence: content-auto-expanded-all-intrinsic/geometry.json and /tmp/content-auto-expanded-all-intrinsic.log. Nested row content now measures like direct text; remaining categories include mixed shrink/min-max distribution, column fit-content width (213.7421875 native versus220 Chromium at240), and nowrap intrinsic overflow (220 versus506.390625). These require runtime/measurement investigation; do not restrict the target feature to the passing examples. All working changes remain experimental. Two expanded browser-only references directly inspected and hashed;142 reference images remain unreviewed. Native/WASM/pixel qualification is still outstanding.

L10 runtime fit-content experiment: for CSS AutoWidth text, intrinsic box width is measured independently of wrapped-line advances; exact width stays authoritative and nowrap may overflow available width. Build97320 passes. Expanded comparison33587 exits1 with14/144 remaining failures (maximum48px), down from62: all previously failing column fit-content/nowrap cases now pass. Mixed-factor and min/max cases still fail, preserved in content-auto-expanded-fit-content/geometry.json. Added a focused measurement test for available box width, resize, policy removal, nowrap overflow and exact sizing. Test22503 is compiling; initial test used wrong wrap enum0, corrected on disk to1 before any pass claim. Main full regression94020 remains isolated on frozen binaries.

L10 measurement unit22503 passes1/1; compiled source includes corrected nowrap enum1. New font-free isolated matrix12 scenes/36 viewports fails12 public runtime comparisons, proving text shaping is not the sole cause. Direct Taffy test reproduces53 coordinate mismatches (97271 exit101; first32580 had an alignment-constant compile error, corrected). Evidence: content-auto-isolated-oracle, content-auto-isolated-initial/geometry.json, checked-in content-auto-isolated-boxes.json and /tmp/content-auto-taffy-initial2.log. Identified flex violation loop flooring the border box at0 instead of flooring its content box at0: shrinking children must retain padding/borders during redistribution. A targeted padding floor fix is under test78727, /tmp/content-auto-taffy-padding.log. Min/max intrinsic-basis clamping remains a separate suspected issue; not yet fixed or qualified.

L10 intrinsic min/max fix76859 passes all102 standalone Taffy tests, including the36 isolated Chromium references and existing720 partial-factor references. ContentSize measurement now ignores constraints on its requested axis while retaining cross-axis constraints; the parent applies min/max after computing the content basis. This complements the padding/border floor during flex freezing. Probe build95704 is active (/tmp/content-auto-minmax-build.log); public text/box geometry must be rerun with that new probe before claiming those failures resolved.

L10 rebuilt probe95704 passes build. Expanded144-viewports comparison now passes144/144 (maximum0.0104217529296875px) after padding-floor and intrinsic-min/max corrections; evidence content-auto-expanded-minmax/geometry.json. Batch91894 continues isolated36, original99 and new48 unbreakable-word references (/tmp/content-auto-minmax-comparisons.log). These passes are geometry-only; do not claim pixel qualification. Added content_auto.rs public cascade/factor-preservation tests, not executed yet. New16 unbreakable scenes/48 browser references captured separately (35924 exit0), images not yet inspected.

Completed geometry batch91894: expanded144/144, isolated36/36 and original99/99 pass. Unbreakable-word stress cases fail8/48 (maximum281px), preserved in content-auto-unbreakable-minmax/geometry.json. The remaining failures require min-content overflow investigation; earlier passes do not qualify arbitrary intrinsic text. Public focused tests are now running in /tmp/content-auto-public-focused.log.

L10 public focused51256 passes all4 tests: cascade/factor identity and content-auto48, independent288, partial720 original/clone geometry references. Min-content measurement experiment now measures the longest unbreakable segment as the lower bound of fit-content, preserving exact-width overrides. Probe92399 is building (/tmp/content-auto-min-content-build.log); extended runtime unit test and48-word-stress comparisons remain pending. Comparison helper now asserts exact source identity sets, visible nodes and unchanged compiled-byte hashes across viewports. Qualification still needs an explicit host capability for corrected intrinsic measurement/distribution, manifest validation and native/WASM/types/host updates; older hosts must not silently accept scenes depending on these fixes.

L10 min-content correction: probe92399 and measurement unit12134 both pass. Geometry batch66049 passes all327 viewports: original99, expanded144, isolated36, unbreakable48, with original390-compiled-byte hashes retained at each resize and exact visible-source identity sets checked. Maximum error across sets0.015625px. These are geometry results, not pixel qualification. Version11 layout-css-intrinsic-sizing-v1 capability and unique LayoutComponent target list now emitted for content auto basis or intrinsic auto-width text; validation rejects downgraded/missing/duplicate declarations and invalid object targets. Checked probe advertises the capability and supports a disable-control. JS types extended. New public contract tests81401 and typecheck58171 running; native/WASM/host/pixel validation remains pending.

L10 version11 tests81401 pass3/3 and native10842/WASM56160 builds pass. Full public16549 completes228 passes/11 failures, all in four targets with outdated intrinsic emission/version expectations. Updated exact manifest expectations, preserved text-policy negative checks separately from the new intrinsic requirement, and changed the nested width toggle regression to verify version11 intrinsic width is independent of nowrap alignment. Corrected targets19716 pass63/63; combined unchanged-source public coverage239/239, original failure logs retained. Parity77887 initially8/9 (old version4 decoration assertion); its whole existing-corpus byte/map/requirements comparison passed. Host78518 initially11/12, including new intrinsic host test passing; follow-up57519 exposed diagnostic-order assumptions in text-only negative cases. Those now isolate text requirements by removing intrinsic requirements explicitly. Host90517 and expanded parity32320 running; parity corpus now includes all109 L10 scenes separately without modifying the active full-regression main fixtures.

L10 host first-test updates complete: preserved-text negative tests isolate their older text contract from intrinsic requirements; synthetic underlines retain version11 instead of downgrading to3. Focused host35378 passes1/1; with the other11 passing tests from90517, all12 host tests are accounted for (including intrinsic width resize and fail-before-stream). Earlier host failures remain in /tmp/content-auto-host{,2,3}.log. Expanded parity32320 remains active (/tmp/content-auto-parity2.log). Created immutable content-auto-toolchain snapshot after successful coherent native/WASM builds; /tmp/content-auto-snapshot.log and snapshot manifest record all four artifacts. L08/L09 full94020 continues beyond3371 checks; do not start a concurrent pixel reporter run or merge L10 into its fixture corpus before it finishes.

L10 parity32320 passes9/9, including all109 geometry fixture sources and the existing main corpus. Added3 realistic compositions and a separate112-scene visual corpus (336 comparisons planned), with visible fills on non-text box cases. Browser composition capture initially exposed accent selector specificity errors; corrected #actions>#publish and #root>#pro, retaining original references. Corrected9 browser images all directly inspected and hashed in content-auto-composition-oracle-corrected/visual-inspection.json. Fixed-height narrow project/plan overflow is retained as an explicit stress condition. Corrected composition geometry41818 passes9/9 against frozen content-auto-toolchain (maximum0.0128936767578125px). Main corpus remains unchanged during full94020. Visual-corpus parity63030 is running (/tmp/content-auto-visual-parity.log), covering the corrected paint sources and new compositions; no native L10 images have been captured/reviewed yet.

Joint L08/L09 full native94020 passes4114/4114 (38.7m). All4104 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed auto-margin, independent-flex and partial-flex baselines; no images remain unreviewed. Frozen toolchain hashes revalidated. Receipt: partial-flex-full/visual-inspection.json and baseline-comparison.json. L08 and L09 are now native-qualified; their documented vector text raster failures remain. The invalidated earlier L08 run is retained and not used as qualification.

L10 focused native98870 passes336/336 (3.2m), frozen content-auto-toolchain. Initial51 text pairs reviewed:39 direct comparisons across13 sheets and12 exact duplicates, all mapped by hashed pairs. Receipt content-auto-initial-text-review/visual-inspection.json complete for that subset;285 overall native pairs remain unreviewed. Merged-corpus public tests45881 pass96/96. Vector336 run started using the same frozen snapshot; /tmp/content-auto-vector.log. Full4450 regression still pending.

L10 focused native review now covers144/336 pairs: initial51 text,9 realistic compositions, and84 box comparisons (60 directly viewed unique pairs plus24 exact duplicates). All ten box contact sheets inspected; hashed receipts in content-auto-native-box-review and content-auto-native-composition-review. Remaining192 expanded/unbreakable text comparisons reduce to54 three-width sheets by exact browser/native image hashes; generated sheets still require direct inspection. Vector12547 is terminal:239/336 pass,97 preserved pixel failures; all336 source-bound geometry comparisons pass, maximum0.015625px, recorded in content-auto-vector/run-summary.json. Vector visual review remains pending; no tolerance changes. Full native4450 regression started in session75206, /tmp/content-auto-full.log, using immutable content-auto-toolchain. L10 remains experimental pending complete review and regression qualification.

L10 native visual review advances to216/336 comparisons. All24 expanded column/column-reverse sheets (72 pairs) directly inspected, covering natural/mixed factors, constraints, wrap, nowrap and indefinite height. Reviewed CSS overflow and second-column placement match Chromium at240/390/768. Hashed receipt: content-auto-native-expanded-text-review/visual-inspection.json. Remaining120 pairs are expanded rows and unbreakable-word cases. Full regression75206 remains live; no qualification claim yet.

L10 full regression initial failure: inherit-typography at390/768 fails pixel gates. Directly reviewed all three viewport pairs in content-auto-full-initial-failures/inherit-typography.png;240 wrapped text aligns correctly, but390/768 unwrapped native text stays left while Chromium right-aligns it. This is a real positioning regression, not raster noise. Hashed visual receipt preserved beside sheet. Suspected new intrinsic AutoWidth line wrapper loses available width for alignment. Full75206 continues on frozen snapshot; preserve fixture and failing output, fix current source separately before requalification. Native focused review now216/336, with120 remaining.

L10 alignment fix: retain TextSizing::AutoWidth for intrinsic measurement, but use Fill for internal line-box and text participant widths so resolved content width remains available to text-align. Renamed intrinsic_row_text to intrinsic_text. Publisher84065 builds; original99, expanded144 and unbreakable48 geometry comparisons all pass (291 total). Targeted public56340 passes existing7 tests; new resize/clone alignment regression93914 passes1 test covering left/center/right, normal/nowrap and240/390/768/240 on original and clone. Initial test compile errors99439 are preserved, corrected before pass. Six independent replays of inherit-typography and letter-spacing-center at240/390/768 pass unchanged geometry/pixel gates and are all directly reviewed; receipt content-auto-alignment-fix-replay/visual-inspection.json, source/browser references preserved. This replay uses updated native compiler and frozen runtime; it is not WASM or full-corpus qualification. WASM build89627 running (/tmp/content-auto-alignment-wasm.log). Original full75206 continues on immutable pre-fix snapshot to collect regressions; its failures are not fixed by changing working source. Need new coherent snapshot, parity and focused/full native/vector requalification.

L10 corrected toolchain content-auto-alignment-toolchain frozen after successful WASM89627 build; probe and renderer hashes verified identical to original L10 snapshot. Native/WASM parity50715 still running (/tmp/content-auto-alignment-parity.log). Host69080 passes12/12. Added18 separate content-auto-alignment-cases.json fixtures across left/center/right, normal/nowrap, column/row/nested layouts. Chromium89961 captured54 references; geometry59499 passes54/54 at maximum0.010101318359375px against corrected frozen toolchain. New references still need pixel validation/direct review; main corpus remains unchanged during full75206. Pre-fix focused native review now264/336: all48 unbreakable pairs accounted for by18 direct inspections and30 exact duplicates;72 expanded row pairs remain. Corrected-build review transfer must verify exact pixels before counting these historical reviews.

L10 pre-fix focused native review complete: all336 image pairs accounted for by direct inspection and exact hashed equivalence. Final24 expanded row/reverse-row sheets (72 pairs) reviewed; wrapping, unequal factors, constraints and overflow agree. Overall receipt content-auto-native/visual-inspection.json has remaining[]. This is focused historical evidence, not qualification of the corrected compiler or full corpus. Corrected native/WASM50715 passes9/9 (160.85s). Accepted-corpus parity now includes main cases plus18 alignment stress scenes, replacing the redundant112-scene list already merged into main; expanded parity running /tmp/content-auto-alignment-expanded-parity.log. Full public suite running /tmp/content-auto-alignment-full-public.log. Original full75206 remains live on pre-fix frozen snapshot.

L10 pre-fix vector box audit59103 passes:84/84 source and browser/native PNG pairs exactly match fully inspected native boxes. Recorded transfer in content-auto-vector-box-review/visual-inspection.json. Vector text252 pairs, including97 failed gates, still require review; no vector qualification claimed.

L10 corrected full public48156 exits0:43 test targets,228 passed,0 failed as counted from /tmp/content-auto-alignment-full-public.log. This is the observed cargo --tests result; do not substitute earlier combined-suite counts. Vector composition review completed9 pairs:7 preserved failures,2 passes; shapes and wrapping match while text edges/weight visibly differ. Hashed receipt content-auto-vector-composition-review/visual-inspection.json. Overall historical vector review93/336,243 text pairs remain. Added replay-oracle.mjs for isolated frozen-toolchain rendering of captured font-only Chromium oracles: checks artifact hashes, compiles390 once, reuses bytes at three widths, compares geometry and existing pixel gates, retains source/PNG hashes and failures incrementally, uses no shared reporter. Corrected alignment stress54 native run33250 is active, /tmp/content-auto-alignment-stress-native.log. Expanded parity98949 and original full75206 remain live.

L10 corrected alignment stress native33250 exits0:54/54 geometry and real Metal pixel comparisons pass unchanged gates against saved Chromium references. Same390-compiled bytes at240/390/768; frozen content-auto-alignment-toolchain hashes retained in replay.json. Contact sheets67365 generating; no direct inspection credited yet. Expanded parity98949 exits0:9/9 tests including18 new alignment sources plus complete main corpus,197.40s. Original pre-fix full75206 remains live.

Corrected L10 alignment stress review complete: all54 native pairs directly inspected on18 sheets, hash receipt content-auto-alignment-stress-native/visual-inspection.json. Normal/nowrap, left/center/right, unequal-factor rows and nested side panels agree across all widths, including intentional overflow. This qualifies this focused native stress subset only; full corrected corpus still pending.

L10 alignment stress vector41134 exits0:54/54 geometry and pixel checks pass unchanged thresholds; visual review remains pending,18 sheets generating8611. Preserved original112 visual sources and336 Chromium screenshots in content-auto-preserved-visual-oracle with exact source comparisons and image SHA provenance. Corrected native336 replay72049 started using content-auto-alignment-toolchain and this preserved oracle, /tmp/content-auto-corrected-native.log. Isolated oracle replay avoids shared reporter state while original full75206 continues. After corrected native finishes, compare all original source/PNG pairs to completed historical review; directly inspect changed pairs, then run corrected vector corpus. Main cases.json remains unchanged during original full run.

Corrected L10 alignment stress vector visual review complete: all54 pairs on18 sheets directly inspected and hashed. Positioning/wrapping/overflow agree with Chromium; visible vector text-edge differences remain within existing gates. Native and vector focused54 stress subsets now both pass and are fully reviewed. Receipt content-auto-alignment-stress-vector/visual-inspection.json. Corrected original336 native72049 still running; full75206 remains pre-fix and unqualified.

Corrected L10 native replay progress:196 completed pairs exactly match both PNG hashes and HTML/CSS of fully reviewed pre-fix native baseline; no changed images in this completed subset. Interim comparison and visual receipt are explicitly marked in progress in content-auto-corrected-native. Remaining comparisons still running72049; do not infer336 completion.

L11 baseline captured while L10 rendering continues:25 frozen corrected-L10 compiler rejections and75 Chromium references at three widths; sources indefinite-basis-cases.json, receipt indefinite-basis-initial/receipt.json, oracle indefinite-basis-oracle/oracle.json. No compiler changes and no screenshot review yet. Research/expanded coverage and implementation remain pending; L10 qualification retains priority.

Corrected L10 native72049 exits0:336/336 geometry and native pixels pass unchanged gates. Exact HTML/CSS and both PNG comparison accounts for336 reviewed pairs, with0 changed pairs remaining. Receipt content-auto-corrected-native/visual-inspection.json. Corrected vector336 replay24164 started on the same frozen toolchain, /tmp/content-auto-corrected-vector.log. Original pre-fix full75206 still active; corrected full qualification remains pending.

L11 admission-only experiment: removed the percentage-basis rejection in a temporary source build, preserving all runtime code. Initial51058 failed compilation because parent_main_definite is also needed for propagation; corrected99503 builds. Experimental binary and exact source are copied into indefinite-basis-experiment. Restored the original guard immediately afterward; working source still rejects L11. Publisher restoration build83038 active. Compare56094 tests75 Chromium box viewports with the isolated admission-only compiler and frozen corrected-L10 probe, /tmp/indefinite-basis-admission-geometry.log. This experiment is solely to distinguish admission from runtime defects; it is not support or contract qualification. L10 pixel processes use immutable snapshots and are unaffected.

Admission-only geometry56094 exits1:57/75 comparisons pass,18 fail, maximum56px. Failing scene names: indefinite-basis-column-0-fixed, indefinite-basis-column-100-fixed, indefinite-basis-column-30-fixed, indefinite-basis-column-reverse-0-fixed, indefinite-basis-column-reverse-100-fixed, indefinite-basis-column-reverse-30-fixed. Original deferred reproducer still matches geometry at all three widths; its pixel behavior remains unchecked. Source guard and native publisher restoration83038 completed successfully. Preserve geometry.json with experimental compiler/probe hashes before further runtime changes.

Corrected L10 vector24164 exits1:239/336 pixel passes,97 failures, all336 geometry checks pass. All336 source/browser/native pairs exactly match preserved pre-fix vector outputs;93 comparisons already visually reviewed transfer,243 remain. Receipt content-auto-corrected-vector/baseline-comparison.json and visual-inspection.json. No tolerance changes.

L11 engine regression87650 exits101 with exactly18 intrinsic root-height mismatches (188 vs132) across fixed-height column/reverse-column cases; auto cases pass. Changed Taffy basis resolution so only authored Auto falls back to the main-size property; unresolved non-auto basis proceeds to content measurement. Standalone full engine64072 passes103/103 including the36-viewpoint root-size regression. This remains experimental runtime behavior: public probe rebuild is running in /tmp/indefinite-basis-probe-build.log; rerun75 public geometry cases with isolated admission-only compiler before claiming those failures fixed. Compiler guard remains restored. L10 frozen toolchains are unaffected.


L11 public runtime experiment61957 completed with75 comparisons:57 pass and18 fail, maximum16px. The content-fallback change fixes all18 prior fixed-column failures but introduces18 fixed-row/reverse-row failures (root width272 instead of Chromium288). Exact before/after case mapping is in output/playwright/html-to-riv/indefinite-basis-content-fallback-geometry/before-after.json; geometry.json pins compiler and probe hashes. This disproves qualification from the103 passing engine tests alone. Expanded the engine regression to the full24-case/72-viewport matrix to cover intrinsic row widths as well as column heights; public75-case oracle retains the original overflow reproducer. Compiler admission remains guarded. Investigate intrinsic contribution sizing separately from flex-basis resolution; do not special-case percentages by direction merely to match these fixtures.

Corrected L10 vector visual review now covers105/336 pairs,231 remaining: three column text contact sheets directly inspected across240/390/768, plus exact image equivalence to one additional scene. Boxes/text positioning agree visually; glyph edge differences and97 numeric pixel failures remain recorded. Receipt: output/playwright/html-to-riv/content-auto-corrected-vector/column-text-review.json. No tolerances changed.

Corrected L10 vector review now covers117/336 pairs,219 remaining. Reverse-column text sheets were directly inspected at all three widths, with one additional scene transferred through exact mapped image equivalence. Receipt: output/playwright/html-to-riv/content-auto-corrected-vector/reverse-column-text-review.json. Numeric results remain239 pass/97 fail.

Cache-mode experiment97653 completes:102 engine tests pass, expanded matrix fails18 fixed-column heights188 vs132; all row widths now agree. This establishes that the earlier apparent column success depended on reusing a ContentSize cache result for InherentSize. Keep sizing modes separated; investigate the column intrinsic main-size algorithm rather than restoring cache aliasing. /tmp/indefinite-basis-cache-mode-engine.log preserves the new failure set. No public probe was rebuilt with this cache change yet.

Corrected L10 vector review now covers129/336 pairs,207 remaining, after direct inspection of three row text sheets at240/390/768 and mapped exact-image transfer to one additional scene. Receipt: output/playwright/html-to-riv/content-auto-corrected-vector/row-text-review.json. Numeric239/97 results unchanged.

Column-sizing engine41014 passes104/104, including the72-viewpoint row/column matrix and a new cache regression that requires ContentSize/InherentSize measurements with identical constraints to remain distinct. Experimental intrinsic column sizing now takes its extent from flex bases and minimum/padding floors using the existing line-sum path; rows retain intrinsic contributions. Probe rebuild pending in /tmp/indefinite-basis-column-probe-build.log. This does not qualify min/max/wrap/nested behavior or compiler support; the next gate is the75-viewpoint public geometry oracle and previous L10 geometry corpora. Preserve earlier failures.

Corrected L10 vector review now covers141/336 pairs,195 remaining; reverse-row sheets directly inspected at three widths, plus exact image-equivalence transfer. Receipt: output/playwright/html-to-riv/content-auto-corrected-vector/reverse-row-text-review.json. Numeric239/97 results unchanged.

Corrected L10 vector visual review covers150/336 pairs,186 remaining. Natural row/nested text and original intrinsic-label sheets directly inspected at three widths; receipt content-auto-corrected-vector/natural-row-text-review.json retains hashes and observations. Existing numeric failures unchanged.

Public column-sizing comparison85175 completed57/75 pass,18 row failures, maximum16px. The passing engine matrix therefore does not establish runtime parity: inspect mapped runtime styles and measurement context. Expanded comparison initially aborted at unsupported border shorthand; preserved its folder/log. Updated content-auto-compare.py to save compiler rejections and continue. Complete run36497 records168 compared viewports,62 geometry failures (max236px),8 rejected scenes in indefinite-basis-expanded-geometry-complete/geometry.json. Rejections remain explicit, not passing cases. All192 Chromium references are preserved. Next replace unsupported shorthand in new versioned fixture captures or keep those as negative cases, and investigate accepted-case failures.

Parent-aware row-contribution experiment46886 passes104/104 engine tests, including the72-viewpoint host topology. Auto-width single-line rows now use intrinsic contributions even under definite available width; wrapped rows remain on the existing line-sizing path and require further investigation. Before-source preserved in indefinite-basis-experiment/flexbox.rs.before-row-contributions. Probe build pending in /tmp/indefinite-basis-row-probe-build.log; rerun original75, expanded168 accepted viewports and L10 geometry with frozen new probe. No public runtime success claimed yet.

Corrected L10 vector visual review now159/336 pairs,177 remaining. Three mixed/constrained row sheets directly inspected across240/390/768; receipt content-auto-corrected-vector/mixed-constrained-row-review.json. Numeric239 pass/97 fail unchanged.

Corrected L10 vector review now177/336 pairs,159 remaining after six additional sheets directly inspected (wrapped/constrained and nowrap/indefinite rows). Receipts: content-auto-corrected-vector/wrapped-row-review.json and nowrap-indefinite-row-review.json. Numeric239/97 results unchanged.

Row-contribution public results:48764 original75/75 pass (max0.005356px);80404 prior L10 original99/99 pass (max0.015625px). Expanded78344 completes168 comparisons with38 failures (max52px),8 syntax rejections, improving from62 failures/max236px. Remaining names preserved in indefinite-basis-row-expanded-geometry/failing-scenes.json. L10 expanded144 and unbreakable48 comparisons started separately; no pixels for this experimental runtime yet.

L10 regression geometry completed: original99, expanded144 (68131, max0.010422px), unbreakable48 (42244, max0.011719px), all291 pass using probe-row-contributions. Added eight item-max scenes/24 viewports to the engine oracle (now96 viewports). Red57930 reproduces12 fixed/auto row root widths141 vs173. The intrinsic shrink fraction divided by max(1, shrink*inner_basis) but reconstruction multiplied by max(1,shrink)*inner_basis, doubling reduction for partial shrink. Aligning the denominator with reconstruction passes full engine48116:104/104. Source-before retained; rebuild pending /tmp/indefinite-basis-shrink-probe-build.log. Still check zero/subpixel inner bases and negative margins, then public original/expanded geometry and pixels; no qualification from this unit result alone.

Corrected L10 vector visual review now186/336 pairs,150 remaining. Indefinite nested and reverse-natural row sheets directly inspected at three widths; receipt content-auto-corrected-vector/reverse-natural-review.json. Existing239/97 numeric results unchanged.

Intrinsic-shrink public results completed: original72447 passes75/75 (max0.005356px); expanded57483 has26 failures/168 comparisons (max52px) and8 border rejections, fixing all12 item-max failures. Edge49880 has48/48 compared viewports failing (max34px), plus16 rejected scenes; geometry remains finite. Preserve this edge regression before any zero-denominator adjustment or support qualification.

Child-padding floor engine38863 passes105/105, including all48 small-content root-width comparisons and the96-viewpoint main regression matrix. Runtime probe rebuild pending in /tmp/indefinite-basis-padding-probe-build.log. Next freeze the rebuilt probe and run original, expanded, edge and prior L10 geometry; do not qualify from the engine pass alone.

Corrected L10 vector visual review now204/336 pairs,132 remaining. Six reverse mixed/constrained/wrapped sheets directly inspected at all three widths. Receipts: content-auto-corrected-vector/reverse-mixed-constrained-review.json and reverse-wrapped-review.json. Numeric239/97 unchanged.

Child-padding runtime original12931 passes75/75 (max0.005356px), and edge77781 fixes all48 previously failing accepted viewports (max0.000001px). Edge run exits1 because16 negative-margin fixtures remain explicitly rejected; do not describe this corpus as wholly passing. Expanded39693 result is retained in /tmp/indefinite-padding-expanded-geometry.log. No tolerance changes.

Child-padding expanded39693 completed148/168 geometry passes,20 failures,8 border syntax rejections. All remaining failing scenes are wrapped row/reverse-row variants (with/without root max-width). Added16 wrapping/root-max cases/48 viewports to the parent-aware engine oracle, now144 viewports; regression8899 is running in /tmp/indefinite-basis-wrap-engine-red.log. Candidate issue: the definite-space wrapped-row path compares an outer size with inner available width; investigate alongside intrinsic line measurement, not by forcing a browser-baked size.

Corrected L10 vector visual review213/336 pairs,123 remaining. Reverse nowrap and indefinite row sheets directly inspected across three widths. Receipt: content-auto-corrected-vector/reverse-nowrap-review.json. Numeric239/97 unchanged.

Wrapping engine8899 completed with36 root-axis mismatches across the20 publicly failing viewports (width plus height mismatches count separately). Log /tmp/indefinite-basis-wrap-engine-red.log. Experimental inset correction compares wrapped line outer size with available inner width plus the container inset; source-before preserved. Full engine96001 pending in /tmp/indefinite-basis-wrap-inset-engine.log. Height and single-line wrapped intrinsic-width defects may remain; do not infer overall fix from this arithmetic change.

Corrected L10 vector visual review222/336 pairs,114 remaining. Natural column/nested and reverse indefinite nested sheets directly inspected. Receipt content-auto-corrected-vector/natural-column-review.json. Visible text differences and numeric239/97 results retained.

Inset experiment96001 completes104 pass/1 failing engine test. Root-axis mismatches drop36→20:16 heights remain stale at132 vs80, and4 wide fixed-row widths remain272 vs288. Added an experimental auto-row remeasurement after inline size is established: regenerate item dimensions/bases and recollect flex lines before flexible lengths/cross sizing. Single-line wrapped rows now use intrinsic contribution sizing like nowrap rows. Full engine12163 pending in /tmp/indefinite-basis-row-reresolve-engine.log. Source-before preserved; no runtime verification claimed.

Corrected L10 vector visual review231/336 pairs,105 remaining. Mixed/constrained column sheets directly inspected at three widths; receipt content-auto-corrected-vector/mixed-column-review.json. Existing239/97 numeric results unchanged.

Auto-row remeasurement12163 passes105/105 engine tests, including144 parent-aware basis/wrap viewports and48 small-content viewports. This resolves all root-axis wrapping regressions in the current engine matrix. Runtime rebuild pending in /tmp/indefinite-basis-reresolve-probe-build.log; verify original75, expanded168, edge48 and earlier L10 text/box geometry with a frozen probe. Rebuilding alone is not public or pixel qualification.

Corrected L10 vector visual review249/336 pairs,87 remaining. Six column constrained/wrapped/nowrap/indefinite sheets directly inspected at three widths. Receipts: content-auto-corrected-vector/wrapped-column-review.json and nowrap-column-review.json. Numeric239/97 unchanged.

Corrected L10 vector review258/336 pairs,78 remaining after three further column sheets directly inspected. Receipt: content-auto-corrected-vector/reverse-natural-column-review.json. Numeric239/97 unchanged.

Corrected L10 vector review267/336 pairs,69 remaining. Reverse mixed/constrained column sheets directly inspected; receipt content-auto-corrected-vector/reverse-mixed-column-review.json. Numeric239/97 unchanged.

Corrected L10 vector review285/336 pairs,51 remaining after six reverse column sheets inspected. Receipts: content-auto-corrected-vector/reverse-wrapped-column-review.json and reverse-nowrap-column-review.json. Numeric239/97 unchanged.

Corrected L10 vector review complete336/336 pairs,remaining[]. Final21 direct comparisons plus30 mapped image-equivalent pairs recorded in content-auto-corrected-vector/final-unbreakable-review.json. Numeric239 pass/97 fail retained without tolerance changes; visual inspection does not qualify the failing vector profile. Corrected native336 previously passed and was fully reviewed by exact source/image transfer.

Row-reresolve public geometry:27721 original75/75 pass (max0.005356px);99871 expanded168/168 compared viewports pass (max0.01125px),8 border shorthand rejections;6757 edge48/48 compared viewports pass (max0.000001px),16 negative-margin rejections. All291 accepted geometry comparisons pass, without waiving24 rejected scenes. Earlier L10 geometry runs75585 original99,11266 expanded144,55776 unbreakable48 started using frozen probe-row-reresolve. Pixel/contract/native-WASM parity qualification remains outstanding.

Earlier L10 geometry75585/11266/55776 all pass (99+144+48=291). L11 native pixel experiment96473 passes75/75 numeric geometry/pixel gates; direct review pending. Explicit native-pixel-experiment manifest contains only executed native artifacts and makes no WASM parity claim.
Original L10 full run completed4320 pass/130 fail out of4450; preserved playwright-results.json, cases-at-run.json and terminal-summary.json in content-auto-full. Now merged18 separately qualified alignment fixtures into main corpus (1486 scenes; expected4504 full tests). Initial corrected launch52708 failed before tests because root npx fetched a different Playwright version; log retained. Restarted via module-local @playwright/test/cli.js using immutable corrected L10 toolchain and fresh content-auto-alignment-full-v2 output. Keep cases.json unchanged during this run.

Original L11 native pixel sheets generated88828; directly inspected9/75 comparisons including the historic overflow-painting reproducer at240/390/768. Orange nested overflow, thin blue strip and subsequent green sibling match Chromium visually; hashes and remaining66 pairs in indefinite-reresolve-original-pixels/visual-inspection.json. Expanded pixel24814 runs216 accepted viewport comparisons; provenance preserves both original oracles and all24 excluded syntax rejections in indefinite-accepted-expanded-oracle/provenance.json. Filtering negative fixtures is explicit and makes no support claim for their syntax.

Expanded L11 pixel24814 completed204 pass/12 fail out of216; all failures are retained in indefinite-reresolve-expanded-pixels/replay.json. Initial failure inspection identifies local RGB error for fractional-width child/inner regions (e.g.0.25px). Geometry passes; direct visual review of these failures remains required. No thresholds changed.

L11 thin-box paint correction: two runtime regression tests pass. Expanded216 and dedicated threshold108 viewport comparisons pass in both native/vector profiles (648 total), using frozen experimental toolchains and unchanged geometry/pixel gates. Each scene is compiled at390 then resized240/390/768. Twelve repaired regions were inspected at6x zoom and transferred between profiles only after exact source and browser/native image hashes matched. Full visual review, public admission and native/WASM qualification remain pending. See validation/indefinite-basis-investigation.md and output/playwright/html-to-riv/indefinite-thin-expanded-native-v2/thin-fix-visual-inspection.json. Corrected L10 full4504 remains running independently against its immutable pre-L11 toolchain.

L11 text expansion:96 scenes/288 Chromium references in indefinite-text-oracle-v2 cover0%/30% basis, four directions, direct/nested text, unequal factors, constraints, wrapping and nowrap. Initial generation missed some semicolon-delimited auto bases; preserved initial capture/cases-initial.json, corrected all flex basis declarations, then independently recaptured v2. Frozen thin-fixed runtime plus original admission compiler fails176/288 geometry and pixel comparisons (no syntax rejections), preserved indefinite-text-initial-geometry and indefinite-text-initial-native. Directly viewed first row0%/240 browser/native images: native root/card backgrounds collapse while text overflows those narrow backgrounds and overlaps siblings. Experimental compiler now enables intrinsic AutoWidth measurement/occurrence policy for auto-width percentage-basis text; source admission guard restored and guarded binary rebuilt. Isolated html-to-riv-percentage-text improves geometry to208/288 passes (80 failures, max286.382813px), all remaining failures in row/reverse-row cases. This is incomplete: first row case now over-expands its intrinsic container; investigate available-space/min-content contribution sizing next. Source-before retained in indefinite-basis-experiment/lib.rs.before-percentage-text. No support claim or parity qualification. Original75 thin-fixed replay passes with exact source/PNG identity; visual review now36/75 pairs, transferred by hashes.

L11 row fit-content experiment: definite available width previously measured each item against the whole available width, then summed contributions. Added nonwrapped auto-row intrinsic min/max passes and fit-content clamp, preserving overflow when min-content exceeds available space (CSS Sizing3 https://www.w3.org/TR/css-sizing-3/#fit-content-size). Existing105 engine tests pass; new synthetic measured-leaf regression checks available80/240/768 against intrinsic limits120/540 and passes. Frozen probe-row-fit-content built with native glyph support; text288 and accepted expanded box216 public geometry runs pending. Wrapping is intentionally unchanged in this experiment, not removed from the intended feature. Source-before and logs retained.

Corrected L10 full64749 terminal:4504/4504 pass in21.7min, zero skipped/unexpected/flaky. Immutable content-auto-alignment-toolchain used throughout. Preserved playwright-results.json, cases-at-run.json and terminal-summary.json in content-auto-alignment-full-v2. Full image equivalence/review audit remains required; numeric success alone is not visual qualification.

Row fit-content first probe retained80/288 text failures and increased maximum error to488.765625px, although216/216 boxes pass. New measured-leaf regression correctly fails on pre-fix source (available80 incorrectly yields240 instead of120), then passes after fix. Runtime bridge inspection found MinContent and MaxContent both mapped to Undefined. Mapping MinContent to zero AtMost reduces text failures to16/288, max286.382813px;216/216 boxes remain passing. All16 are wrapped rows/reverse rows at240/390. Experimental wrapped fit-content now uses the widest per-item min-content contribution and reconstructs max-content across lines;106 engine tests pass. Public probe build pending /tmp/indefinite-wrap-fit-probe.log. Original native pixel direct review reaches45/75, with nine additional reverse-row pairs directly inspected and hashed.

Wrapped fit-content public runtime24637 passes288/288 text geometry comparisons, max0.0286255px;41991 passes216/216 expanded box geometry comparisons, max0.0112495px. Same390-compiled bytes resized240/390/768 throughout. Frozen indefinite-wrapped-fit-toolchain contains experimental percentage-text compiler and wrapped-fit/min-content runtime, with native-only manifest and no WASM claim. Native text288 pixel replay started in indefinite-text-wrapped-fit-native; earlier L10 original/expanded geometry regressions also running. Full visual, capability/admission and parity qualification still outstanding.

L10 native qualification completed against immutable content-auto-alignment-toolchain:4504/4504 full tests pass; all4494 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed baselines (4104 from partial-flex-full,390 from corrected native336 plus alignment54). Full visual-inspection.json records all transfers and zero remaining pairs. Prior public228, host12 and native/WASM parity9 passes support the version11 intrinsic-sizing contract. Native content-derived auto basis is qualified;97 focused vector-text pixel failures remain preserved and reviewed. This receipt qualifies the frozen L10 build; subsequent experimental L11 runtime/compiler changes require their own regression validation.

L11 wrapped-fit text native12885 passes288/288 pixel and geometry comparisons. Earlier L10 original31141 passes99/99 and expanded52843 passes144/144 geometry checks on the same frozen candidate runtime/compiler. Text full visual review, vector replay, public acceptance/contract and native/WASM parity remain outstanding; L11 remains guarded.

L11 text vector58039 terminal:200/288 pixel passes,88 pixel failures, all288 geometry passes. First mixed-nested row0% sheet directly inspected at allthree widths: matching layout/wrapping/clipping with glyph edge/weight differences; local RGB failure remains unwaived and raster cause unproven. Native direct review9/288 pairs and vector3/288 recorded with hashes. Proposed version12 capability contract written in validation/indefinite-basis-contract.md; no admission or host implementation claimed. Added120 equal/zero-factor scenes (360 Chromium viewports) from the four-direction original basis matrix; public geometry run11172 pending in indefinite-factor-initial-geometry.

Equal/zero-factor matrix11172 completed306/360 geometry passes,54 failures (max18px), all fixed-width row/reverse-row cases with equal0/0,.5/.5,1/1 factors. Legacy equal-factor encoding erased authored main dimension when writing percentage basis. Compiler candidate now uses independent-factor transport for every explicit percentage basis, preserving authored dimension separately. New public percentage_basis regression12790 passes; corrected360/360 geometry2694 passes (max0.0187378px). Frozen indefinite-factor-toolchain created. Native360 pixel run60216 and earlier288 text geometry83621 pending; source guard remains restored, guarded publisher rebuild18675 pending. Before-source and failures preserved. Native text visual review now18/288 pairs; vector3/288,88 vector numeric failures remain.

Factor native60216 passes360/360 geometry/pixel comparisons; nine pairs directly inspected with hashes in indefinite-factor-native/visual-inspection.json. Text recheck83621 passes288/288 geometry on factor compiler. Contract foundation implemented: RuntimeCapability::LayoutCssIndefiniteBasisV1, version12 validation, optional intrinsic targets at12 with unchanged target/capability consistency, JS types, native host support/disable switch, and max-version retention. Percentage/content-auto focused public tests2092 pass; checked host39231 passes13/13 including version12 rejection before stream; TypeScript passes after adding12/new capability to explicit type expectations. Compiler admission remains guarded, and no end-to-end version12 compile or WASM parity claim is made yet.

L11 source admission candidate now emits version12 and layout-css-indefinite-basis-v1 for previously guarded indefinite percentage sizing. Runtime version never downgrades for intrinsic text descendants. Historical overflow fixture left active rejections but is preserved verbatim in indefinite-basis-cases.json and a positive public regression. Focused admission49/49 tests pass; actual compiled-scene checked host13/13 passes; native/WASM parity9/9 passes across previous plus349 L11/threshold scenes. Original/clone lifecycle70951 passes120 scenes through240/390/768/240 (960 instance/viewport comparisons). Frozen four-artifact indefinite-v12-toolchain created. Main corpus now1835 scenes; full native5551 regression48669 running in indefinite-v12-full, and final public suite48691 running. Keep cases.json unchanged during this full run. Admission is implemented but L11 is not qualified: full regression, complete visual accounting and residual vector failures still need resolution/reporting.

Final public suite48691 stopped at an outdated partial_flex_exclusions assertion rejecting indefinite percentage basis. Updated that assertion to require version12/matching capability while retaining malformed-factor and unsupported min-content keyword rejection checks. Full suite restarted42051 with --no-fail-fast; full native5551 continues unchanged. Native text visual accounting now172/288 (36 direct plus136 exact-image transfers); each transfer refers to a directly reviewed pair and requires its own geometry/pixel pass. Added12 separate auto-basis/percentage-authored-main fixtures (36 viewports) to cover the previously guarded implicit percentage path; their oracle capture is pending. Main full-run cases stay unchanged.

L11 admitted public suite42051 passes233/233 across45 targets. Auto-basis percentage-main replay72798 passes36/36 geometry/native pixels, visual review pending. Native text visual review completed288/288: 129 directly inspected pairs plus159 exact complete-image transfers, each independently passing geometry/pixels. Receipt indefinite-text-wrapped-fit-native/visual-inspection.json has zero remaining. One reduced-sheet ambiguity was checked using the original768px browser/native images. Vector88 failures remain unwaived and vector review remains incomplete. Full5551 runtime regression48669 still running.

L11 visual review update: indefinite-factor-native now accounts for184/360 full image pairs:54 directly inspected (18 scenes at240/390/768) and130 exact browser/native PNG-pair transfers to directly inspected representatives. The176 remaining pairs are explicit in visual-inspection.json. Column/reverse-column percentage overflow and row viewport clipping match Chromium in these inspected scenes; all360 independent geometry/pixel gates already pass. Cross-corpus receipts are separate and their counts must not be added to this receipt. Final version12 vector text replay is terminal:200/288 pixel passes and88 preserved failures, with288/288 geometry passes. Its288 source/browser/native image pairs exactly match the earlier vector replay, but only three have an inspected baseline;285 remain visually unreviewed. Evidence: indefinite-v12-text-vector/baseline-comparison.json and visual-inspection.json. Full5551 native regression remains live; no qualification or tolerance change is claimed.

L11 factor visual review is complete for the focused native replay:360/360 image pairs accounted for, comprising153 directly inspected pairs (51 three-viewport sheets),198 identical-image transfers within the factor corpus and9 transfers to directly reviewed original-basis representatives. Every transfer matches both complete PNG hashes; all720 candidate PNGs,51 contact sheets and18 external representative PNGs were rehashed against receipts. Each of360 candidates independently passes geometry and pixel gates. The receipt is indefinite-factor-native/visual-inspection.json with zero remaining. These are visual-output transfers, not claims of source equivalence. Main frozen version12 full regression is still running; expanded/thin/auto-main and vector visual review remain outstanding. L11 remains unqualified, and no tolerance changed.

L11 implicit percentage-main path: all36 comparisons in indefinite-auto-main-native now directly visually inspected (12 sheets,240/390/768), with all72 source PNGs and12 sheets rehashed. Empty, nested-box and text fixtures cover all four flex directions. Responsive line wrapping, padding, intrinsic extent and sibling placement match Chromium. All36 geometry/native pixel gates pass on the final version12 toolchain. Added the12 fixtures to JavaScript native/WASM artifact parity corpus, alongside the unchanged1835 main scenes; that parity rerun is pending. Main cases.json stays frozen during full5551 regression. This focused evidence does not complete L11 qualification.

L11 follow-up: JavaScript parity rerun94023 passes9/9 tests, including all1847 positive scene inputs (1835 main plus12 implicit percentage-main) with identical native/WASM RIV bytes, source maps and runtime requirements. Log: /tmp/indefinite-auto-main-parity.log. Original-basis native visual coverage is now complete75/75 in indefinite-thin-original-native/cross-corpus-visual-inspection.json:12 directly reviewed reversed-column pairs plus63 complete-image transfers to directly inspected baselines, with hashes checked from files. Expanded native full-frame review remains15/216; threshold native0/108. These counters do not replace prior regional thin-box inspections. Remaining review and full regression still prevent L11 qualification.

L11 thin-pixel threshold visual review complete: native108/108, with15 directly inspected representative pairs and93 exact whole-image transfers. Five sheets include the full frame plus nearest-neighbor8x details of the top-left16x16 region at240/390/768. Invisible zero/subthreshold boxes and one-pixel horizontal/vertical bars at5/64px, including origin0.5 rounding, match Chrome. All216 native PNG files verified against hashes; all108 independently pass geometry/pixels. Vector108/108 has exact HTML/CSS and complete browser/native image identity with this fully reviewed native corpus, verified from files; its visual receipt has zero remaining. Evidence: thin-pixel-native-v2/visual-inspection.json and thin-pixel-vector/visual-inspection.json. Expanded boxes and vector text review plus the ongoing full regression remain outstanding; no tolerance or support scope changed.

L11 full native regression48669 is terminal and passes5551/5551 in25.8minutes: expected5551, skipped0, unexpected0, flaky0, errors0. Saved authoritative Playwright JSON, terminal log, unchanged1835-scene source corpus and SHA256 terminal-summary.json in indefinite-v12-full. This replaces earlier running status. Full visual qualification remains pending. Expanded native review has27/216 pairs accounted for in visual-inspection.json (18 direct plus9 exact-image transfers); the earlier15-pair cross-corpus receipt overlaps and must not be summed. First six row min/max/root-max/nested-percent/zero-pixel scenes directly inspected across240/390/768 and match Chrome. No gates changed; L11 remains unqualified pending remaining visual accounting and explicit vector limitations.

L11 expanded native visual review now accounts for111/216 full-frame pairs: 63 direct and48 exact-image transfers; 105 remain. Reviewed forward/reverse nested percentages, zero-pixel/zero-percent controls, min/max constraints and column wrapping agree with Chrome across240/390/768. Rehashed all222 reviewed candidate PNGs and21 contact sheets. Receipt: indefinite-thin-expanded-native-v2/visual-inspection.json. Cross-corpus and regional receipts overlap and are not additive. Full5551 native tests remain passed; L11 visual qualification remains incomplete.

L11 expanded-box visual review complete216/216:117 directly inspected pairs (39 sheets),89 complete-image transfers within the expanded corpus and10 to directly reviewed original/factor baselines. All432 candidate PNGs,39 contact sheets and20 external representative PNGs rehashed; every candidate independently passes geometry/pixels. Reverse constraints, wrapping, nested percentages, zero-basis distinctions and shrink edges match Chromium in the inspected frames. Vector216/216 has exact HTML/CSS and browser/native PNG identity with the fully reviewed native corpus; its files were also rehashed and its gates pass. Receipts: indefinite-thin-expanded-native-v2/visual-inspection.json and indefinite-thin-expanded-vector/visual-inspection.json, both with zero remaining. Full5551 numeric regression remains passed. Remaining work includes final full-run image accounting and incomplete vector-text review;88 vector text pixel failures remain preserved and unwaived. L11 is not yet qualified.

L11 final native full-run visual accounting complete:5541/5541 scene pairs exactly match reviewed baselines in HTML/CSS and both complete PNG files. Rehashed candidate and baseline images; source receipts are explicitly pinned by SHA256. All5551 tests pass, including10 controls; zero skipped/flaky/unexpected/errors. Evidence: indefinite-v12-full/baseline-comparison.json, visual-inspection.json, terminal-summary.json and compare-baselines.py. There are no remaining native full-run image changes requiring inspection. Focused implicit-percentage-main36 is separately reviewed and included in1847-scene parity. Vector-text review remains incomplete and its88 pixel failures remain unwaived; overall L11 status remains qualification pending until its evidence/limitations are fully recorded.

L11 version12 vector-text visual review now covers104/288 pairs with27 direct inspections and exact complete-image transfers;184 remain. Inspected natural/mixed/nowrap forward and natural reverse rows at240/390/768. Wrapping, badges, card placement and clipping match Chrome, while glyph edge/weight differences are visibly present. The receipt retains per-case pixel failures in both direct and equivalent-image entries; inspection is not a pixel waiver. All288 geometry gates still pass and88 pixel failures remain unchanged. Receipt: indefinite-v12-text-vector/visual-inspection.json. Native full5551 passes and all5541 native scene images are accounted for; L11 remains qualification pending.

L11 vector-text review advances to171/288 pairs (54 direct,114 within-corpus image transfers,3 earlier external transfers);117 remain. Reverse mixed/nowrap rows and natural/mixed columns inspected at240/390/768. Browser/native text wrapping, clipped labels, overflow obscured by later siblings and nested badge placement match visually; glyph edge differences persist. Per-case failures remain in the receipt, and all88 pixel failures remain unwaived. Evidence: indefinite-v12-text-vector/visual-inspection.json. Native regression and visual accounting remain complete; L11 qualification remains pending.

L11 vector-text review now238/288 (81 direct,154 internal exact-image transfers,3 external);50 remaining. Constrained/natural/mixed columns and reverse columns inspected across240/390/768. Text overlap, sibling occlusion, background extents and badges match Chrome, with visible glyph raster differences retained. The88 pixel failures are unchanged and unwaived. Receipt: indefinite-v12-text-vector/visual-inspection.json. Native full regression/visual evidence is complete; vector review remains pending.

L11 final vector-text visual review complete288/288: 123 directly inspected pairs across41 sheets,162 within-corpus whole-image transfers and3 earlier reviewed external transfers. All576 candidate PNGs and contact sheets rehashed; every recorded direct/transfer pixel failure agrees with the unchanged replay. All288 geometry comparisons pass; pixels remain200 passes/88 failures. Wrapping, overlap, clipping and badge placement agree visually, including original-resolution inspection of an ambiguous reduced preview. Glyph edge/weight differences remain visible and unwaived. Evidence: indefinite-v12-text-vector/visual-inspection.json with zero remaining. Native full5551 tests and5541 image pairs remain passed and reviewed. L11 qualification audit is next; this does not qualify the vector text renderer.

L11 audit verified all four frozen toolchain hashes, public233 tests across45 targets, and saved qualification-audit.json in indefinite-v12-full. Closed missing final-toolchain vector lanes: original75/75, equal-factor360/360 and implicit-main36/36 all pass geometry/pixels. Original75 and factor360 have exact source and complete-image identity with reviewed native baselines (files rehashed), and their visual receipts are complete. Implicit-main24/36 match reviewed native images;12 text pairs differ and require direct review despite passing pixel gates. No live replay remains for these runs. Native qualification is still pending consolidation and those12 vector inspections; existing88 vector stress-text failures remain unwaived.

L11 is now native-qualified against frozen indefinite-v12-toolchain. Final audit and exact validation scope are in validation/indefinite-basis-qualification.md. The remaining12 implicit-main vector text images were directly inspected and match wrapping/placement; all36 pass unchanged pixel gates and are reviewed. Vector stress-text88 pixel failures remain explicitly unqualified and unwaived. Next priority is L12 content-box sizing; no new syntax is admitted by this status update.

### L12 investigation (not yet supported)

Content-box sizing now has a [contract and validation plan](validation/content-box-contract.md), paired border-box controls and preserved pre-feature compiler diagnostics. Public support remains border-box only until runtime transport and Chrome/native qualification are complete. Border combinations are preserved for P01.

L12 compiler integration update: content-box is implemented experimentally with version-13 occurrence requirements. Public Chrome geometry, native/WASM parity and host admission tests pass; real renderer pixel and expanded composition qualification remain pending. This supersedes the earlier “not yet supported” implementation status, not the qualification gate.

L12 expanded validation: 56 additional nested/wrapping/text/clip/font-relative scenes pass all 168 Chrome/native geometry and pixel comparisons. Initial visual inspection covers 36/240 pairs; the rest remain pending. Positioning reproducers are preserved for the separately unsupported positioning feature.

L12 composition progress: expanded parity passes across 1983 inputs; eight realistic cards pass 24/24 native comparisons. Visual review remains partial: initial36/240, expanded30/168, cards9/24. Qualification is still pending.

L12 cards: native24/24 pass and all24 visually reviewed; native/WASM parity1991 inputs passes. Vector cards retain24 geometry passes but24 unwaived pixel failures. Remaining initial/expanded native and vector visual reviews prevent qualification.

L12 visual update: vector cards24/24 reviewed with all24 pixel failures retained. Initial native60/240 and expanded48/168 reviewed; native cards24/24 complete. Qualification remains pending.

L12 expanded native visual review: 102/168 pairs accounted for with verified screenshot hashes; 66 remain. No new mismatches found. Initial native review and full regression qualification remain outstanding.

L12 full native regression is running against the frozen v13 toolchain with5983 expected tests, including all144 content-box scenes. Expanded native visual review132/168; remaining reviews and terminal regression results are pending.

L12 expanded native visual review is complete168/168, with hashes verified and no mismatches found. Initial native review and full regression results remain outstanding; qualification is pending.

L12 initial native review now114/240; remaining126 pairs and the active full native regression still prevent qualification. No new discrepancies found.

L12 initial native visual review now covers 174/240 pairs (126 direct, 48 exact image transfers). Reverse-row border-box and column zero/percentage/minimum/basis controls match Chrome. Remaining 66 pairs and full regression completion still prevent qualification. Expanded and card visual receipts remain complete.

L12 initial native visual review is complete240/240 (174 direct, 66 exact full-image transfers); all PNG and sheet hashes verified. Together with expanded168 and cards24, all432 focused native pairs are reviewed and pass. Vector cards24 failures remain unwaived and reviewed. Full native regression and coverage audit remain pending.

L12 coverage audit: padded images (fixed/percentage/max-width/rounded clip, both box modes) pass 24/24 Chrome/native geometry and pixel comparisons, with all 24 directly visually reviewed in `output/playwright/html-to-riv/content-box-image-native/visual-inspection.json`. Edge coverage passes 54/54 comparisons in `content-box-edge-native-v2`; its visual review is pending. The initial edge replay stopped on an ellipsis fixture missing the documented `display:block` precondition; both rejected inputs and the failure log are preserved. Corrected fixtures explicitly set block display. Added both corpora to native/WASM parity; its terminal result and the active full regression remain pending. No tolerance changes.

L12 audit completion: edge native54/54 now directly visually reviewed; image native24/24 already reviewed. Expanded native/WASM parity passes all9 tests across2017 inputs; receipt `output/playwright/html-to-riv/content-box-edge-native-v2/parity-receipt.json` records corpus and matching frozen compiler hashes. Total focused native coverage is510 passing, visually reviewed comparisons. Full regression and additional vector text coverage remain pending.

L12 expanded vector coverage passes168/168 geometry and pixel comparisons. Visual accounting currently105/168:96 exact full-image transfers from reviewed native results plus9 direct text/clip/max-width reviews;63 remain. Receipt: `output/playwright/html-to-riv/content-box-expanded-vector/combined-visual-inspection.json`. Existing vector card failures remain unwaived. Full native regression is still running.

L12 expanded vector visual review is complete168/168 (72 direct,96 exact full-image transfers), with screenshot and sheet hashes verified. All168 geometry and pixel gates pass. Vector cards retain24 reviewed, unwaived pixel failures. Full native regression and final evidence audit still prevent qualification.

L13 preparation: `validation/aspect-ratio-cases.json` defines64 prospective fixtures covering forward/reverse flex axes, both box modes, one/both/auto dimensions, percentage widths, min/max and flex basis. Chrome153 references captured at192 viewports in `output/playwright/html-to-riv/aspect-ratio-initial-oracle/`. All64 are rejected by frozen v13 compiler; rejection receipt is `aspect-ratio-initial-admission/receipt.json`. This is pre-implementation evidence, not support qualification. A separate auto+ratio probe is in progress.

L13 existing-runtime probe: `tests/aspect_ratio_runtime_investigation.rs` compiles a ratio-free input, injects existing property524, installs required content-box/flex policies, clones the artboard and resizes both instances240→390→768→240. Explicit exploratory run fails:40/64 scenes match Chrome throughout;24 fail, with320 coordinate mismatches in auto, max-width and flex-basis variants across all directions and both box modes. Evidence: `output/playwright/html-to-riv/aspect-ratio-runtime-investigation/receipt.json` and `run.log`. The test is explicitly ignored by default while the feature is under investigation; it must be invoked with `--ignored`. This is neither a public compiler test nor a supported feature. Runtime corrections are required before qualification.

L13 used-main correction: flex hypothetical cross sizing now applies an available preferred ratio to the final flexed main size, accounting for content-box padding. Existing Taffy106/106 tests pass. A non-ignored12-scene Chrome original/clone regression passes96 instance/viewports. Full exploratory corpus improves to52/64 scenes matching, with12 failing and256 coordinate mismatches: transferred max constraints and column auto intrinsic sizing remain; column auto additionally gains a width mismatch pending the intrinsic-main correction. Evidence: `output/playwright/html-to-riv/aspect-ratio-used-main/receipt.json`. No public aspect-ratio support or qualification claimed. The ongoing content-box regression uses its immutable pre-L13 v13 toolchain and does not validate this new engine patch.

L13 constraint-transfer correction now retains explicit preferred dimensions when transferring opposite-axis min/max limits through a ratio. Full runtime probe matches60/64 scenes; all8 max-width cases are fixed, and4 column-auto scenes remain (128 coordinate mismatches). Taffy106 tests pass. Receipt: `output/playwright/html-to-riv/aspect-ratio-constraints/receipt.json`. An intrinsic-inline-to-block correction is being tested; neither public CSS support nor qualification is claimed.

L13 runtime foundation now passes all64 scenes across512 original/clone instance-viewports. The column flex basis derives its block size from fit-content inline size and preferred ratio, respecting box sizing. Taffy106 tests pass. The entire probe is now a regular non-ignored regression and passes (no filtered cases); its prior failures remain preserved. Receipt: `output/playwright/html-to-riv/aspect-ratio-intrinsic-column/receipt.json`, regular run `regular-test.log`. This verifies existing-property runtime geometry only: public compiler syntax/transport, auto+ratio distinction, expanded stress cases, native/WASM parity and actual renderer pixels remain outstanding.

L13 parser foundation: `src/aspect_ratio.rs` retains the auto flag and normalized optional ratio;3 internal tests pass for both auto orders, comments, numbers, degenerates and invalid/unrepresentable values. Nonzero numeric underflow is rejected instead of silently becoming an auto/zero ratio. Math functions, units, percentages and negative ratios are excluded from this parser stage. Receipt: `output/playwright/html-to-riv/aspect-ratio-parser/receipt.json`. The parser is not connected to public CSS admission yet; combined auto/ratio runtime policy, transport and renderer gates remain outstanding.

L12 native qualification complete for frozen v13: full5983/5983 tests pass, and all5973 scene pairs exactly match reviewed source and full-image baselines. Focused native510/510 passes/reviewed; parity2017 inputs and host14 pass. Vector expanded168/168 passes/reviewed, while card24 pixel failures remain reviewed and unwaived. See `output/playwright/html-to-riv/content-box-v13-full/completion-receipt.json` and `validation/content-box-contract.md`. Subsequent L13 engine edits are not covered by this frozen snapshot and need separate regression evidence.

L13 ratio reference-box runtime tests pass: bare64 scenes/512 instance-viewports, combined/control12 scenes/96 instance-viewports, and64 additional clear-original/retained-clone size comparisons. Wrong target rejection and Taffy106 pass. The independent occurrence policy preserves authored box-sizing and clone state. Receipt: `output/playwright/html-to-riv/aspect-ratio-reference-box/receipt.json`. Compiler cascade/transport, host admission and pixel qualification remain outstanding.

L13 public compiler integration: numeric ratios and optional auto flag now pass through cascade (including initial/unset/inherit/variables/important), wire aspectRatio and version14 `layout-css-aspect-ratio-v1` requirements with unique layout targets and explicit ratio reference-box policy. Public76-scene original/clone oracle passes608 instance-viewports;2 contract tests,15 host tests, native/WASM builds and type checking pass. Host rejects old/unknown versions, missing capability, empty/duplicate/invalid targets and malformed policy flags before drawing. Full public suite,2093-input parity, native pixel runs and visual review are pending. Frozen toolchain: `output/playwright/html-to-riv/aspect-ratio-v14-toolchain/`. Qualification is not claimed; image auto sizing, math-function values and additional interactions remain pending.

L13 expanded validation: full public suite245 tests across52 targets passes with0 ignored; native/WASM parity passes9 tests across2093 inputs. Auto/ratio discriminator native36/36 geometry and pixel comparisons pass; visual review remains pending. Receipt: `output/playwright/html-to-riv/aspect-ratio-public-integration/receipt.json`.

L13 visual progress: auto discriminator36/36 reviewed (18 direct and18 exact-image transfers). Initial corpus25/192 accounted for (24 direct plus1 exact-image transfer); remaining167 pending. Expanded96-scene corpus captures288 Chrome viewports for min-width/max-height/conflicting constraints/stretch/wrap/nested ratios across all four directions, both box modes and bare/auto forms. Native replay is running against frozen v14; no expanded pass is claimed yet.

L13 expanded native run completed240/288 passing;48 geometry/pixel failures are all conflicting min/max cases and remain preserved in `aspect-ratio-expanded-native/`. Current corrections normalize a maximum against its minimum before transfer, and derive an automatic dimension from the constrained preferred opposite dimension while retaining raw authored dimensions for flex basis. Expanded public oracle now includes172 scenes; its new align-self policy installation fixes a test-harness omission that falsely reported stretch failures. Corrected runtime/public tests are running, with evidence under `output/playwright/html-to-riv/aspect-ratio-conflict-correction/`. No new renderer pass or qualification is claimed yet.

L13 conflict correction now passes the full172-scene public Chrome oracle across1376 original/clone instance-viewports, plus the auto reference-box lifecycle test and Taffy106 tests. Receipt: `output/playwright/html-to-riv/aspect-ratio-conflict-correction/receipt.json`. The corrected native probe is rebuilding; the48 frozen-v14 renderer failures remain unwaived until a fresh corrected replay and visual review complete.

L13 corrected native replay completes with516/516 geometry and pixel comparisons passing: initial192, auto36, expanded288. Corrected auto36 are visually accounted for by exact source/full-image identity with their reviewed baseline. Initial review remains48/192 on the first snapshot, and expanded corrected review is pending. Original48 conflict failures remain preserved as historical reproducers. Expanded parity2189 inputs is running. Toolchain: `aspect-ratio-v14-corrected-toolchain`; replay directories end in `native-v2`.


### L13 corrected conflict visual review

The corrected expanded corpus has48/288 pairs visually accounted for (36 direct,
12 exact full-image transfers), covering every conflicting min/max case across
four flex directions, both box modes, bare/auto ratios and three widths. Chrome
and native agree on panel size, inset child, sibling placement, reverse anchoring
and narrow viewport overflow. All288 expanded numerical comparisons pass; the
original48 failures remain preserved. Native/WASM parity is terminal and passes
2189 inputs across9 tests. L13 remains unqualified: other visual review,
text/composition/image coverage and the corrected-runtime full regression remain.
See validation/aspect-ratio-investigation.md for the current evidence table and
output/playwright/html-to-riv/aspect-ratio-expanded-native-v2/visual-inspection.json
for hashed review records. Chrome is the reference; tolerances are unchanged.


### L13 text and composition probe

Added28 scenes in validation/aspect-ratio-text-cases.json: four flex directions,
both box modes, percentage width with wrapping, flex basis with auto ratio,
fixed height with ellipsis, and four responsive media cards. Chrome153 captures
84 viewports; public native replay compiles each scene once at390 then resizes
at240/390/768 with unchanged RIV hashes. Native geometry/pixels84/84 pass;
24 pairs are visually accounted for and60 remain. Vector geometry84/84 passes,
but12 media-card pixel comparisons fail; all12 failure pairs are reviewed and
unwaived (9 direct,3 exact-image transfers). Vector passing72 review remains.
Native/WASM parity passes2217 inputs across9 tests. The first fixture omitted
the required display:block for ellipsis; its rejected run is preserved and the
corrected corpus was recaptured. No runtime edit or tolerance change was needed.
Receipt: output/playwright/html-to-riv/aspect-ratio-text-native-v2/receipt.json.
L13 remains unqualified: finish reviews, text original/clone regression,
image sizing and the full corrected-runtime regression.


### L13 clone regression discovered

The new public text original/clone regression fails reproducibly with96 coordinate
mismatches in the four media cards, only on the clone after its first resize.
Original instances and the clone at its first240px layout pass. A reduced
one-row/two-label footer with no aspect ratio reproduces18 coordinate mismatches
in0.08s at subsequent390/768/240 sizes: View report shrinks from85.84px to12.86px
and wraps to192px high instead of24px. This is evidence of a broader intrinsic
text clone/resize issue; its root cause is not yet established. The regular,
non-ignored tests in tests/aspect_ratio_oracle.rs preserve the failure. Chrome
oracles, repeated logs and test snapshot are recorded in
output/playwright/html-to-riv/aspect-ratio-text-clone/receipt.json. Previous
fresh-import native84/84 pixel results do not establish clone lifecycle fidelity.
Next: finish minimizing the footer, distinguish measurement/policy/cache state,
fix the cause, then rerun the full text oracle and renderer regression.


### Text occurrence policy clone correction

Text clones now retain host-installed CSS wrapping, nowrap alignment, ellipsis,
underline and strikethrough policies. Derived shaping, measurement and drawing
state starts fresh; serialized RIV bytes and fresh-import admission are unchanged.
The prior single-label and footer failures pass without reinstalling policies on
the clone. Public aspect-ratio oracle now passes4 tests covering202 scenes and
1616 original/clone instance-viewports, including all28 text/composition scenes.
The root cause was the generated Text clone path copying only serialized base
properties; it now delegates to Text::clone_core. Historical failures and the
wrap-reinstallation control remain in aspect-ratio-text-clone/receipt.json.
Policy independence unit test passes, covering copied policies, fresh derived state, clone/source independence and default behavior. New frozen-probe pixel/full
regression is still required; old renderer receipts predate this correction.


### Post-clone regression evidence

The full module suite passes248 tests across53 targets with0 ignored. The new
frozen aspect-ratio-v14-clone-toolchain includes the Text clone policy correction.
Text native replay passes84/84 geometry and pixel checks; all84 pairs have exact
HTML/CSS and browser/native PNG identity with the now fully reviewed v2 baseline
(72 direct inspections,12 exact-image transfers). Current receipt:
aspect-ratio-text-native-v3/visual-inspection.json. Vector replay remains72/84
pixels and84/84 geometry; all84 sources/images are identical to its prior run,
with12 failure reviews transferred and72 passing reviews pending. No failure is
waived. Logs and hashes are in aspect-ratio-text-clone/receipt.json. Remaining
L13 work includes shape/vector visual reviews, authored-ratio image sizing and
full corrected-runtime regression; qualification is still withheld.


### Authored-ratio image admission

L13 now admits exactly one auto image dimension with a bare positive numeric
aspect-ratio, retaining the auto dimension and other length/percentage in the
RIV layout. Both-auto natural sizes and combined auto ratios with an automatic
dimension remain rejected pending L14. Existing explicit-both sizing is retained.
24 Chrome image fixtures/72 viewports cover four flex directions, both box modes,
asymmetric padding, percent width, fixed height and min/max constraints. Public
original/clone geometry passes192 instance-viewports; native geometry/pixels72/72
pass. Visual review20/72 is complete so far;52 remain. Native/WASM parity2241
inputs/9 tests passes. Original24 admission failures remain preserved.
Receipt: output/playwright/html-to-riv/aspect-ratio-image-native/receipt.json.
Full module suite is running; full native regression and remaining image review
are pending. L13 is still unqualified; no tolerances or runtime code changed for
this admission increment.


### Image visual review complete; full native regression running

All72 authored-ratio image comparisons are now directly visually reviewed across
four directions, both box modes, three sizing variants and three widths. Image
quadrants, padding, constrained sizes, reversed anchoring and viewport overflow
agree with Chrome; no thresholds changed. Full module suite passes250 tests
across53 targets with0 ignored. Receipt: aspect-ratio-image-native/receipt.json.
The complete existing native regression is running5983 tests against immutable
aspect-ratio-image-toolchain, including the ratio engine and Text clone fixes.
Output: output/playwright/html-to-riv/aspect-ratio-v14-full/. Follow session8434
and run-state.json; no full pass or qualification is claimed before completion
and review. Focused aspect-ratio corpora remain separate from the1979-scene
main corpus and are not implicitly counted in this5983-test run.


### Recording and auditing direct visual review

For a replay-oracle output with review-sheets, open each named sheet and inspect
all three Chrome/native/difference rows before recording it. The checked-in
helper records that human/agent inspection; it does not perform inspection:

```sh
python3 validation/record-visual-review.py /absolute/replay-output case-name --notes 'Specific observations from all three widths'
python3 validation/record-visual-review.py /absolute/replay-output --audit
python3 validation/test_record_visual_review.py
```

The recorder rehashes every replay image and every previously reviewed sheet,
rejects unknown case names and any failing numerical comparisons, and writes
atomically. Within the same replay it can account for other cases only by exact
browser/native image-pair identity with a directly inspected case. Cross-run
review transfer still requires matching HTML/CSS as well as full-image hashes
and a verified source receipt. Failing comparisons retain separate explicit
failure-review receipts; this recorder cannot waive them. Five integrity tests
cover transfer, tampered images/sheets, invalid names, failures and forged counts.
The expanded ratio102/288 and image72/72 receipts pass the new audit.


Expanded ratio review now accounts for180/288 comparisons (108 direct,
72 exact full-image transfers), including reversed-row constraints,
stretch/wrapping, nested ratios and forward-column min/max/stretch. All existing
image/sheet hashes pass the checked-in audit. Remaining108 comparisons are
explicitly listed in aspect-ratio-expanded-native-v2/visual-inspection.json.
The existing full native regression remains running under session8434. These
focused reviews do not establish that the full regression is complete.


Expanded L13 shape review is complete for the corrected frozen snapshot:
288/288 Chrome/native pairs (174 direct inspections,114 verified identical
image transfers), with a passing image/sheet integrity audit. See
`validation/aspect-ratio-investigation.md` and
`output/playwright/html-to-riv/aspect-ratio-expanded-native-v2/visual-inspection.json`.
This does not qualify L13 overall or substitute for the running full regression.

Current L13 shape replay and review:516/516 comparisons pass on the frozen
`aspect-ratio-image-toolchain` (initial192, auto36, expanded288). Every pair
matches audited prior reviewed HTML/CSS and full browser/native PNG bytes.
The `aspect-ratio-*-native-v3/visual-inspection.json` receipts retain transfer
provenance. Image stress, remaining text/vector review and full regression are
still separate qualification requirements; see `validation/aspect-ratio-investigation.md`.

L13 image flex stress adds56 Chrome scenes (168 native comparisons) and448
public original/clone instance-viewports, all passing. Review remains168/168;
see `aspect-ratio-image-stress-native/receipt.json` and
`validation/aspect-ratio-investigation.md`. The JS parity corpus includes these
56 scenes; its2297-input run has not yet completed.

L13a substituted-math rejection fix passes four public aspect-ratio tests and51
custom-property tests. Reproducers and red/green logs are preserved under
`aspect-ratio-math-admission/`; rebuilt WASM parity and math evaluation remain
pending. See `validation/aspect-ratio-math-research.md`.

Frozen L13 runtime regression complete:5983/5983 tests,5973/5973 exact-source/image
review transfers, zero skipped/flaky tests. See
`aspect-ratio-v14-full/completion-receipt.json`. This frozen compiler predates the
substituted-math diagnostic fix and internal numeric evaluator; their qualification
remains separate. Focused L13 corpora are not included in this full-run count.

Experimental L13a public integration passes14 math scenes/112 original-clone
instance-viewports, plus14 unit/5 aspect-ratio/51 custom-property tests. Two numeric
limit cases remain unqualified. Rebuilt parity/native pixels/review are pending;
see `validation/aspect-ratio-math-research.md`.

L13a frozen math toolchain:42/42 native comparisons pass and reviewed (12 direct,
30 exact-image transfers), integrity audit passes. Ten JS tests/2311 inputs and
six numeric-limit diagnostic cases pass parity. Full module session81921 remains
running; see `aspect-ratio-math-native/receipt.json`.

L13a precision probes found and fixed premature f32 literal rounding;15 unit and
five public tests pass. Frozen math parity/pixels predate this correction. Chrome
small-positive rounding remains an open correctness gap; see
`validation/aspect-ratio-math-research.md`.

L13a numeric boundary update: Chrome literal and math component clamping is now
measured at exact f32 neighbors; fixed-point layout conversion remains unresolved.
See `validation/aspect-ratio-math-research.md` and the retained
`aspect-ratio-math-precision-v3` probe/source receipt. This is investigation evidence,
not additional qualified syntax. The preprecision full module run passed 258 tests;
postprecision binary parity and visual qualification remain pending.


L13a component-clamp correction: literals and math now preserve numeric source
precision and apply Chrome's per-component f32 clamp before division. A new
public regression failed before the fix and exposed both generic serialization
and cached-token rounding errors. All 15 unit and six public aspect-ratio tests
pass. Sixteen responsive row/column fixtures pass Chrome geometry over 128
original/clone instance-viewports (240→390→768→240). Chrome screenshots cover
48 viewports. Receipt: `output/playwright/html-to-riv/aspect-ratio-clamp-oracle/receipt.json`.
Native/WASM rebuild, pixel comparison and visual review are still pending for
this revision; no qualification claim. Chromium's `LayoutRatioFromSizeF` uses a
16-iteration continued-fraction approximation with a 1e-6 error bound and a
truncated fallback. Its small-ratio degeneration and layout saturation require
further work independently of the now-correct component clamp.

Current component-clamp toolchain: native and WASM builds passed; immutable
`aspect-ratio-clamp-toolchain` hashes retained. All48 focused Chrome/native
geometry and Rust Metal pixel comparisons pass without tolerance changes. Visual
review complete (12 direct +36 exact-image transfers), receipt audit passed.
Evidence: `output/playwright/html-to-riv/aspect-ratio-clamp-native/receipt.json`.
Native/WASM parity session53724 is still running; do not count it as passed.
L13a remains active for the separately documented numeric/cascade gaps.


### L13a current approximation stage

The preceding component-clamp snapshot is fully validated for its focused
corpus: 10 JavaScript tests pass, including2327 native/WASM corpus inputs and
six diagnostic cases; all48 native pixels pass with complete visual review.
`aspect-ratio-clamp-native/receipt.json` supersedes earlier running status.

Current source additionally preserves Chrome's lossless 26.6 component-pair
conversion and bounded continued-fraction approximation before scalar emission.
The regression for a nonzero component just above the SizeF clamp failed before
this change and passes afterward. Chrome84-row precision-v4 measurements also
confirm that `1/1048576` retains a ratio while `calc(1/1048576)` degenerates.
15 unit tests, seven public compiler tests and all eight existing aspect-ratio
oracle tests pass after the change. Sixteen additional responsive approximation
fixtures pass128 original/clone instance-viewports. They cover row/column,
pi/e/square-root/golden-ratio literals, the degenerate boundary and lossless tiny
ratios under an explicit220px maximum height. Their48 Chrome screenshots are
retained. Receipt: `output/playwright/html-to-riv/aspect-ratio-approximation-oracle/receipt.json`.

This revision still needs immutable binary parity and native pixel review.
The bounded fixtures do not qualify unbounded layout saturation; Chrome's
33554432px cap remains an open runtime mismatch. Infinity and broader cascade
semantics remain pending. No tolerance changes or full L13a qualification.

Approximation snapshot validation: native/WASM builds passed and immutable
`aspect-ratio-approximation-toolchain` retained. All48 focused Chrome geometry
and native Rust Metal pixel comparisons pass with unchanged tolerances. Visual
review complete:36 direct +12 exact-image transfers; integrity audit passed.
Receipt: `output/playwright/html-to-riv/aspect-ratio-approximation-native/receipt.json`.
Parity76652 and full compiler suite19327 remain confirmed running. Numeric
saturation/infinity and broader cascade qualification remain open.

Approximation terminal results supersede running status: full compiler suite
passed 263 tests across 53 result groups; native/WASM passed all10 JS
tests including2343 corpus inputs and six diagnostic cases. Logs retained in
`aspect-ratio-approximation-native`. All48 focused pixels and visual reviews
remain passed. L13a is still active for unbounded saturation, infinity and
broader cascade semantics; this focused evidence does not qualify those gaps.


L13a nonfinite stage: Chrome25-case probe separates finite literal overflow from
IEEE infinity. Numeric literal1e400 saturates before arithmetic, so
calc(1e400/1e400) yields1; calc(infinity/infinity) yields NaN then zero.
Compiler now preserves that distinction and converts nonnegative oversized
components through the existing finite layout-ratio representation instead of
rejecting them early. New public regression failed before the correction and
passes afterward. All15 unit, eight public compiler and nine existing ratio
oracle tests pass. Twenty-two new bounded row/column fixtures pass176
original/clone instance-viewports; Chrome66 screenshots retained. Evidence:
`output/playwright/html-to-riv/aspect-ratio-nonfinite-oracle/receipt.json`.
Current source needs rebuilt parity and native pixels/visual review; earlier
approximation receipts predate this change. Unbounded layout saturation and
broader cascade semantics remain open. No tolerance changes or full qualification.


Nonfinite snapshot validation complete for its focused bounded corpus: native
and WASM builds passed; full compiler suite 265 tests across 53 groups
passed;10 JS tests include2365 native/WASM corpus inputs and six invalid-math
diagnostic cases. All66 Rust Metal/Chrome geometry and pixel comparisons pass;
visual review30 direct +36 exact-image transfers, integrity audit passed.
Receipt: `output/playwright/html-to-riv/aspect-ratio-nonfinite-native/receipt.json`.

Unbounded saturation remains a reproduced runtime bug. Eight Chrome fixtures
are retained in `aspect-ratio-saturation-oracle`: four explicit oversized-length
controls receive compiler diagnostics; four admitted ratio fixtures fail at all
three widths. For width120 and ratio1e-6, Chrome height33554432 versus native
120000000; with an infinite denominator native reaches4026531840. The mirrored
large-ratio width also exceeds Chrome's cap. `native-geometry.json` preserves
actual frozen-toolchain bounds and diagnostics. These are geometry reproducers,
not pixel-qualified cases. Do not treat bounded green tests as covering them.


L13a saturation correction in progress: the four admitted unbounded fixtures now
match Chrome through32 original/clone instance-viewports. The new public
regression failed before the runtime change; all11 aspect-ratio oracle tests
pass afterward. Taffy now has an optional ratio-derived dimension limit, applied
at initial size transfer, column flex basis and post-flex cross sizing. CSS ratio
occurrences enable Chrome's33554432 limit; ordinary styles default to None.
The occurrence flag survives cloning. This is arithmetic saturation, not an
authored max-size constraint. All107 Taffy unit tests pass including opt-in and
authored-size preservation controls. The added optional field increases Style
by8 bytes; documented size assertions updated after the expected failure.
Evidence: `output/playwright/html-to-riv/aspect-ratio-saturation-oracle/implementation-receipt.json`.
Native runtime toolchain rebuild is underway; fresh parity/pixels, expanded
saturation compositions and broader cascade qualification remain pending.
Previous frozen toolchains retain the failing runtime and are not evidence for
this correction. No tolerance change or full qualification claim.

Saturation stress expansion:24 scenes/72 Chrome captures cover row/column,
both box-sizing modes, padding, auto+ratio, max constraints and flex shrink.
Public lifecycle test68400 is running. First native replay stopped before any
rendering because the rebuilt probe lacked native-glyph-controls; failure retained
in `aspect-ratio-saturation-native`. Correct probe build85538 is running. These
are pending validation gates, not pixel failures or passing evidence.


Saturation stress public test completed:24 scenes/192 original-clone
instance-viewports passed. Initial rebuilt probes/renderers lacked required
native-glyph-controls/native-metal flags; both failed before rendering and are
preserved as build-profile failures. Correct immutable toolchain is now
`aspect-ratio-saturation-toolchain-v3`, with explicit build commands retained.
Native replay of four initial scenes reports12/12 passing; expanded replay94783
is running with53/72 comparisons observed passing. All84 still need visual
review; full regression and parity remain pending. Use v3, not the earlier
incomplete build profiles, for subsequent runtime validation.


Saturation validation update: full compiler suite267 tests across53 groups
passed. Initial12 and stress72 Chrome/native geometry + Rust Metal pixel
comparisons pass; all84 visually reviewed (36 direct,48 exact-image transfers)
and both integrity audits pass. Receipts in `aspect-ratio-saturation-native-v3`
and `aspect-ratio-saturation-stress-native-v3`. This closes the reproduced
ratio-derived size cap gap for the tested28 scenes/224 original-clone viewports.
The prior2365-input parity job45256 is running; new28 saturation inputs were
added to the parity corpus after that job started and require a subsequent run.
Full L13 replay/cascade qualification remains pending. No tolerance change.


L13a scalar cascade correction: Chrome12-case probe confirms malformed scalar
ratios supplied via var() reset to auto instead of reviving an earlier ratio.
A new public regression failed before the correction and now passes. Invalidation
only classifies the bounded scalar grammar; valid unsupported sin(1) and typed
calc(1px/1px) retain diagnostics. Escaped CSS-wide fallback remains supported.
15 unit,10 public compiler and51 custom-property tests pass. The initially
incorrect escaped-keyword test used --ratio:inherit, which inherits the custom
property; corrected to var(--missing, escaped-inherit) to test substitution.
Logs: `output/playwright/html-to-riv/aspect-ratio-cascade-probe/receipt.json`.
Eighteen responsive fixtures are being captured. Their public geometry/parity/
pixels and review remain pending. Typed invalid math such as calc(1px) still
needs typed classification; it is an explicit open cascade gap.
Prior saturation2393-input expanded parity passed before this source change.


Scalar cascade gates completed:18 scenes/144 original-clone instance-viewports,
54 Chrome/native pixel comparisons, full visual review (3 direct +51 exact-image
transfers), audit passed. Native/WASM10 tests include2411 corpus inputs and six
unsupported diagnostic cases; full compiler suite270 tests/53 groups passed.
Frozen `aspect-ratio-cascade-toolchain` and `aspect-ratio-cascade-native/receipt.json`
retain sources, binaries and logs. This does not yet qualify typed arithmetic.

New18-row Chrome probe in `aspect-ratio-typed-math-probe` distinguishes invalid
unit-bearing expressions from valid cancellation. calc(1px), calc(1px+1px) with
proper sum whitespace, min(1px,2px), mixed-type sums and px/s all reset to auto
when substituted. px/px, percent/percent, s/s and px*s/px/s produce numbers;
em/px additionally depends on computed font size. Typed invalidation must track
dimensional exponents rather than reject every expression containing units.
These are retained browser measurements, not implemented/qualified syntax.


Typed ratio arithmetic implementation stage: quantities now carry six dimensional
exponents through products/division; sums and comparisons require matching types,
and final components must be dimensionless. Absolute length, angle, time,
frequency, resolution units and percentages use canonical conversion. Numeric
prefix parsing retains double precision including escaped units. Typed zero is
not dimensionless. Known typed-invalid var() substitutions now reset to unset;
valid cancellation is compiled. Relative/context-dependent units still diagnose.
This only extends aspect-ratio math, not other property grammars. Existing term
and nesting limits remain. Source: https://www.w3.org/TR/css-values-4/#calc-type-checking
and the adjacent absolute-unit definitions; Chrome18-row probe independently
checks invalidation and cancellation.
New unit regression failed before the implementation. All16 unit,11 public
compiler and51 custom-property tests pass. Receipt:
`output/playwright/html-to-riv/aspect-ratio-typed-math-probe/implementation-receipt.json`.
Thirty-four responsive fixtures are being captured. Public oracle/parity/native
pixels/visual review, conversion-edge tests and relative units remain pending;
extreme converted dimension overflow requires investigation. Earlier frozen
cascade receipts predate this evaluator change. No qualification claim.


Typed-math focused validation:34 scenes/272 original-clone instance-viewports,
102/102 Rust Metal pixels + Chrome geometry pass. Visual review complete:9 direct
+93 exact-image transfers; integrity audit passed. Full compiler suite273
tests/53 groups passed. Immutable `aspect-ratio-typed-toolchain`; evidence
`aspect-ratio-typed-native/receipt.json`. Parity20278 remains running over2445 inputs.

Conversion-edge probe20 rows: ordinary absolute-unit conversions and escaped
units match geometry, but four extreme cases do not. Frozen native comparison
in `aspect-ratio-unit-conversion-probe/native-comparison.json` preserves compiler
inputs, RIVs and bounds: huge in/in gives native0 versus Chrome120px; huge in/px
and px/in lose finite ratios96 and1/96; tiny ms/ms gives native120 versus Chrome0.
The first probe-script invocation had a JS escape error before browser execution;
corrected script and successful capture are retained. Investigate Chrome's unit
conversion/evaluation order; do not guess an overflow cutoff or drop these cases.
Relative units and full unit-conversion visual qualification also remain open.


Unit-conversion order correction: exact Chromium153.0.8010.12 source reveals
CSSParserToken clamps numeric tokens to +/-f32::MAX while retaining double
precision inside that range. Earlier docs claiming f64::MAX were incorrect;
equal huge literals did not distinguish the limits. New Chrome probes for
1e40/1e39 and 1e40-1e39 do distinguish them. Numeric token clamping corrected.
Typed division uses multiplication by a reciprocal, unlike eagerly simplified
scalar division. Preserving that order fixes tiny ms/ms underflow. Four recorded
extreme conversion mismatches now pass new unit/public equivalence regressions.
17 unit,11 existing public ratio,14 existing ratio-oracle and51 custom-property
tests pass; the additional public extreme-conversion test passes separately.
Tag-matched source URLs/hashes, red/green logs and24 Chrome probe rows retained in
`aspect-ratio-unit-conversion-probe-v2/implementation-receipt.json`.
Fresh binary parity and geometry/pixel/visual gates remain pending; prior typed
snapshot parity completed10 tests/2445 inputs before this correction. Relative
units and full L13 qualification remain open. No tolerance changes.


### L13a fractional source-size regression (current)

The frozen unit-order snapshot passes all 10 JavaScript parity tests over 2469
inputs, but responsive validation is **not qualified**: 71/72 native comparisons
pass; case 19 at 768px derives 43084.797px height versus Chrome 43084.5px.
The public original/clone oracle and full module run both fail on that geometry.
Logs and the failed comparison are preserved in
`output/playwright/html-to-riv/aspect-ratio-unit-order-native/receipt.json`.

Nine minimal Chrome fixtures distinguish numeric conversion from source-size
quantization: literal `1 / 96` reproduces the same failure as typed math, while
an exact 1/64px source dimension removes it. Chrome truncates both 448.8px and
448.808px to 448.796875px before ratio transfer. The public regression exercises
original and clone instances through 240/390/768/240 resizing. A CSS-only runtime
source-quantization correction is under test; ordinary Rive ratio behavior is
unchanged. See `validation/aspect-ratio-rounding-cases.json` and
`tests/assets/aspect-ratio-rounding-oracle.json`. Native pixels, visual review,
broader fractional constraints, and final current-source qualification remain
pending. Tolerances are unchanged.

The first correction build stopped on disk exhaustion, before test execution.
`cargo clean -p nuxie-html-to-riv` removed 10.8GiB of regenerable build artifacts;
all frozen toolchains and validation outputs were preserved. The corrected
rebuild is tracked separately from this infrastructure failure.

The candidate source-quantization correction now passes all16 public aspect-ratio
oracle tests, including the previously failing unit-order fixture and9 new minimal
rounding scenes with original/clone resizing. Evidence: `aspect-ratio-rounding-oracle/public-green.log`.
Fresh runtime probe build and pixel qualification are pending.

Fractional-source focused replay now passes33 scenes/99 Chrome geometry and Rust Metal
pixel comparisons, with264 original/clone instance-viewports passing public checks.
Visual review:18 directly inspected comparisons +81 exact-image transfers; both
receipt audits pass. Frozen `aspect-ratio-rounding-toolchain` includes the rebuilt
runtime probe; compiler/WASM artifacts are unchanged from the unit-order snapshot.
Receipts: `aspect-ratio-unit-order-native-v2/receipt.json` and
`aspect-ratio-rounding-native/receipt.json`. Full module validation passed277 tests/53 groups; expanded2478-input
parity passed all10 tests. The first expanded parity attempt failed with
ENOENT during concurrent publisher relinking; its log is preserved and the rerun
started after the publisher hash matched the frozen artifact and passed. No tolerance changes.


L13a fractional stress:40 Chrome scenes/120 viewports now cover row/column,
border/content boxes, bare/auto ratios, fractional padding, fixed/grow/shrink,
and min/max constraints. The previous frozen rounding toolchain fails60/120
comparisons (geometry); the public original/clone stress test is also red.
Evidence: `aspect-ratio-rounding-stress-native/receipt.json` and preserved
`public-red.log`. Earlier33-scene receipts remain valid for their frozen scope.

Diagnosis separates individually truncated padding edges from container gap
conversion before flex allocation. For a768px row with100px sibling and8.808px
gap, Chrome uses gap8.796875px and derives659.203125px main size; transferring
1/96 gives63283.5px. Truncating the float-computed659.192px only after allocation
instead gives63282px. This requires correcting upstream layout inputs, not
widening tolerance or adjusting the ratio. A per-edge padding correction is
under test; container gap precision and wider fixed-point semantics remain open.
Full-module277/53 and parity10/2478 passes predate this padding candidate.

Per-edge padding candidate result: all16 previous oracle tests still pass;
the stress oracle now fails16 scenes/48 distinct scene-viewports, all grow/shrink.
Fixed-padding and min/max cases now pass. Current remaining error is1.5px after
ratio amplification of container gap precision. Candidate source remains unqualified
until gap policy and fresh native/WASM/pixel/visual checks complete. Evidence:
`aspect-ratio-rounding-stress-native/padding-candidate.json`. No live jobs remain.

Gap correction: resolved flex gaps now truncate to1/64px before allocation,
including percentage-gap re-resolution. The runtime enables this explicit Taffy
policy throughout a solve tree containing opted-in CSS aspect ratios; ordinary
Rive trees retain float gaps. Original/clone behavior derives from current solve
styles. All17 public aspect-ratio oracle tests pass, including all40 stress scenes;
all108 isolated Taffy library tests pass, including an opt-in gap allocation test.
Logs: `aspect-ratio-rounding-stress-native/gap-public-green.log` and
`gap-taffy-pass.log`. Fresh native pixels/visual review and expanded parity pending.

Gap focused validation complete:40 scenes/120 Chrome geometry + Rust Metal pixels
pass;320 original/clone instance-viewports pass. Visual review33 direct +87 exact
image transfers, audit passed. Full module278 tests/53 groups,
Taffy108 tests, and parity10 tests/2518 inputs all pass. Frozen
`aspect-ratio-gap-toolchain`; receipt `aspect-ratio-gap-native/receipt.json`.
Nested pixel-gap layouts and full L13/relative-unit qualification remain pending.
Percentage gaps remain intentionally unsupported by the compiler; the engine
percentage re-resolution path is not a compiler support claim.
No live jobs remain; tolerances are unchanged.


Nested pixel-gap qualification:16 new scenes/48 Chrome viewports. Frozen gap
snapshot fails21/48 comparisons. Ancestor per-edge pixel-padding correction
now leaves only4 depth-two column wrapper-width failures (12 viewports); child
ratio dimensions match. All17 earlier oracle tests still pass. Percentage gaps
remain intentionally rejected, including custom properties/fallbacks (9 public
admission controls pass). Current ancestor-padding source is not pixel-qualified.

An isolated experiment omitted post-flex known height during intrinsic column
cross measurement. It fixed extreme wrapper widths but ordinary wrappers became
20px versus Chrome23.59375px (16px marker +7.59375px padding). The experiment was
reverted; the complete fix must account for intrinsic content minimum and ratio
sizing together. Logs/reproducers: `aspect-ratio-nested-gap-native/receipt.json`,
`ancestor-padding-candidate.json`, and `intrinsic-height-experiment.log`.
No live jobs remain. Full L13/relative-unit qualification remains open.


Nested-column candidate now passes all18 public ratio oracle tests. CSS intrinsic
inline measurement excludes post-flex allocated height, while ratio-derived
intrinsic contributions retain their min-content width floor. A separate
`css_intrinsic_sizing` Taffy policy is enabled by the runtime for CSS ratio solve
trees, independent of `quantize_gap`; defaults preserve ordinary Rive behavior.
Ancestor pixel padding is truncated per edge before available-size allocation.
All108 engine tests pass. Explicit policy storage adds8 bytes to Style on this
build (String560, Arc528); size assertions updated, no visual tolerance changes.
Evidence: `aspect-ratio-nested-gap-native/intrinsic-policy-pass.log` and
`taffy-pass.log`. Fresh native pixels, visual review, full module, and expanded
2534-input parity remain pending. CSS sizing context:
https://www.w3.org/TR/css-sizing-4/ (Chrome captures remain acceptance evidence).

Nested intrinsic sizing focused validation complete:16 scenes/48 native comparisons
and128 original-clone instance-viewports pass; all48 comparisons directly visually
inspected, audit passed. Full module280 tests/53 groups, engine108,
parity11 tests/2534 inputs (plus9 percentage-gap diagnostic controls) all pass.
Frozen `aspect-ratio-nested-toolchain`; receipt `aspect-ratio-nested-native/receipt.json`.
Full L13 current-toolchain replay, relative-unit math and broader intrinsic cases
remain pending; no tolerance changes or live jobs.


Current nested-toolchain text/image regression:108 scenes/324 Chrome geometry and
native Rust Metal pixel comparisons pass. Review coverage complete:318 exact
compiler-input/full-image transfers used +6 direct comparisons (including both
changed768px reverse-column image pairs). All source and target image hashes,
full compiler inputs including assets, prior direct-review receipts and sheets
were verified. Thin image-edge diffs pass the unchanged image tolerance.
Receipts: `aspect-ratio-current-text-native/receipt.json`,
`aspect-ratio-current-image-native/receipt.json`, and
`aspect-ratio-current-image-stress-native/receipt.json`.

Repeatable cross-run review tool:
`python3 validation/transfer-visual-review.py PREVIOUS_REPLAY CURRENT_REPLAY`
then the same command with `--audit`. The previous run must have complete audited
direct/within-run review. Changed source or PNG pairs remain unreviewed. Fresh
inspection may overlap transferred pairs; coverage counts deduplicate identities.
Five integrity guard checks passed, including changed CSS despite identical pixels,
stale PNG hashes and a tampered receipt; original evidence was not modified.
The existing within-run tool continues to record actual direct inspections.
Full current-snapshot shape/math and vector-fallback qualification remain open.


Current native L13 cohort replay complete:14 shape/math cohorts,393 scenes/1179
comparisons pass; all1179 have audited exact full compiler-input and PNG-pair
review transfers. Combined with current text/images108 scenes/324 comparisons
and nested16 scenes/48 directly reviewed comparisons, the frozen nested toolchain
now passes517 scenes/1551 native comparisons with complete visual coverage.
Summary: `aspect-ratio-current-native-summary.json`; per-cohort receipts in
`aspect-ratio-current-*-native`. Source/asset/image identity is checked before
any review transfer. No live jobs remain and no tolerances changed.

This closes the pending current native shape/math cohort replay. Relative-unit
ratio math, vector fallback qualification and broader intrinsic-sizing semantics
remain open. It is not a rerun of every legacy compiler-gallery scene; the two
clone-only minimal reproducers are covered separately by public runtime tests.


L13a font-relative ratio math implemented, validation in progress: em/rem
expressions inside calc/min/max/clamp now evaluate using the final computed
font-size, or the profile's fixed16px host root respectively. Numeric literals,
typed dimensional cancellation, and existing clamping rules remain unchanged.
The font pass runs before ratio math regardless of declaration order. Inherited
ratios retain their computed value; custom-property expressions use the consumer's
font. Recognized invalid dimensional math resets through substitution as before.
Viewport/container, font-metric and line-height units remain unsupported.

All19 public ratio oracle tests pass, including24 new Chrome scenes/192 original
and clone instance-viewports. Fresh publisher/WASM builds are in progress;
full module, expanded parity, native pixels and visual review remain pending.
Fractional/extreme font contexts need further qualification. Evidence:
`aspect-ratio-font-math-oracle/implementation-receipt.json`. Earlier frozen
snapshots predate this compiler feature; no tolerance changes.

Font-relative ratio focused validation passes24 scenes/72 native comparisons
and192 original-clone instance-viewports. Visual review30 direct +42 exact-image
transfers, audit passed. Full compiler suite283 tests/53 groups and
parity11 tests/2558 inputs pass with fresh native/WASM artifacts. Frozen
`aspect-ratio-font-math-toolchain`; receipt `aspect-ratio-font-math-native/receipt.json`.
Fractional/extreme font contexts remain the next qualification step. No live jobs
or tolerance changes; broader L13 completion remains open.


### Font-boundary ratio precision investigation

The 32-scene font-boundary Chrome corpus remains unqualified. The computed font
size ceiling correction (10000px, applied before inheritance and em resolution)
passes its public regression. After aligning the public oracle with the native
probe's `layout_bounds()` API, the remaining failures are one scene at390/768px
on both original and clone: height errors0.25/0.5px. The 16 public ratio tests and
19 of20 ratio-oracle tests pass; the boundary oracle correctly remains red.

An f64-transfer experiment did not fix these failures and was removed. Evidence
is preserved in `output/playwright/html-to-riv/aspect-ratio-font-boundary-precision-investigation/`:
candidate source, failing test logs, restored-arithmetic log, and numerical
comparison. The computed pair1/15999 becomes a rounded f32 scalar in the current
compiler. Exact pair arithmetic matches all three Chrome heights; using f64
arithmetic on the already-rounded scalar does not. This proves the next
implementation direction, not runtime qualification. Preserve the pair through
an explicit runtime contract and cloned occurrence policy, then repeat public,
native/WASM, native-pixel and visual gates. Existing tolerances stay unchanged.

Full module test run with `--no-fail-fast` completed: 284 passed, 1 failed across 53 result groups. Only `aspect_ratio_font_boundaries_match_chrome_after_clone_and_resize` failed. Full log and counts are preserved in the precision-investigation receipt.


### Exact computed ratio pair implemented (version15, qualification pending)

The compiler retains the computed integer ratio pair, normalized by GCD, in a
version15 requirement with `layout-css-aspect-ratio-pair-v1`. Every ratio target
must have two positive signed-32-bit components; incomplete/invalid pairs and
mismatched versions/capabilities are rejected. Version14 remains readable with
its original scalar semantics. Older hosts cannot accept the new capability.
Occurrence-local runtime style and clones retain the pair. Opted-in flex ratio
transfers use26.6 integer multiply/divide with truncation and signed-32-bit
saturation, following the retained Chrome `LayoutUnit::MulDiv` source.

All20 public Chrome ratio-oracle groups now pass, including the font-boundary
original/clone resize regression. Full compiler286 tests/53 groups, Taffy109,
and JS type checking pass. The first compile borrow error and expected Taffy
style-size assertion failure were corrected; original logs remain preserved.
Evidence: `output/playwright/html-to-riv/aspect-ratio-pair-implementation/receipt.json`.
Fresh native/WASM parity, native pixels, visual review and broader pair-boundary
stress remain pending; no full L13 qualification or tolerance change.


### Version15 focused font-boundary qualification

Frozen `aspect-ratio-pair-toolchain` passes all32 font-boundary scenes:96/96
Chrome geometry and Rust Metal pixel comparisons, plus256 original/clone
instance-viewports in the public oracle. All96 views are reviewed and audited:
21 directly inspected on seven three-width sheets,75 verified exact-image
transfers. Viewport screenshots verify visible content; separate geometry
assertions cover the full heights of extremely tall boxes. No tolerance changes.
Native/WASM parity completes11/11 tests, including the additional32 boundary
inputs. Full module286 tests/53 groups, Taffy109 and type checks remain passed.
Receipt: `aspect-ratio-pair-font-boundary-native/receipt.json`.

Broader qualification is running from `aspect-ratio-pair-replay-plan.json`:
20 cohorts/543 scenes/1629 native comparisons, including the full28-scene text
cohort as a separate group. The older2-scene text repro remains an additional
cohort and does not substitute for full text coverage. Pixel/geometry failures
and changed review images must be resolved or explicitly retained; none of these
in-progress broader runs is claimed qualified. Pair-boundary stress, vector
fallback and full L13 backlog completion remain open.


Replay-plan correction: the historical `aspect-ratio-text-native` directory held
only six completed comparisons from a partially failed old oracle, not a complete
two-scene corpus. Replaying that oracle reaches an intentionally unsupported
ellipsis combination (missing display:block). Frozen pre-pair font-math and
current pair compilers reject the exact same input with identical diagnostics;
`aspect-ratio-pair-text-native/historical-admission-control.json` preserves this
control. The failed run is retained and is not counted as pixel qualification.
The valid broader replay scope is19 cohorts/541 scenes/1623 comparisons, including
the corrected full28-scene text oracle. Full text84/84 passes with audited exact
review transfer from the original v2 direct-review receipt. The initial transfer
attempt against the historical v3 receipt schema and its successful follow-up
are both retained. Remaining replay groups are still running.


Version15 broader native replay completed:19 valid cohorts/541 scenes/1623
comparisons all pass Chrome geometry and Rust Metal pixel thresholds. Audited
exact-source/image review transfers cover1559 comparisons;64 changed image
comparisons (8 image,56 image-stress) require fresh visual inspection. These are
passing numeric comparisons but are not yet visually qualified. All four replay
groups are terminal; no replay process remains running. Summary:
`output/playwright/html-to-riv/aspect-ratio-pair-replay-summary.json`.


Version15 broader visual qualification complete: all1623 native comparisons
(541 scenes across19 cohorts) pass geometry, pixel thresholds and audited review
coverage. Twenty newly inspected three-width image sheets close the64 changed
views; coverage combines direct/within-run review and prior exact-source/image
transfers without double counting. Thin image-edge differences were inspected
and retained within unchanged existing thresholds, not described as pixel exact.
Together with the separately reviewed32-scene/96-view font-boundary corpus,
current version15 evidence covers573 scenes/1719 comparisons. Full module286,
Taffy109, native/WASM11 tests and type checks pass. No processes remain from the
broader replay. Full L13 and vector fallback remain open.

Next:64 exact-pair stress fixtures cross eight numeric ratios with row/column,
border/content box and bare/auto ratio semantics, including fractional padding
and gaps. Chrome capture and public/native/visual qualification are pending;
these fixtures are not included in the completed counts above.


Exact-pair stress qualification completed:64 scenes/192 native geometry/pixel
comparisons and512 original/clone instance-viewports pass. Visual review60 direct
+132 verified identical pairs is complete and audited. Expanded native/WASM
parity11 tests passes. The separate host-loading suite still asserted version14;
its failure was preserved, then expectations and malformed-pair/missing-capability/
legacy-version14 controls were updated:15 host tests pass. Compiler/runtime code
and frozen binaries did not change during this qualification. Receipt:
`aspect-ratio-pair-stress-native/receipt.json`. Current native ratio evidence now
covers637 scenes/1911 comparisons, all reviewed; full L13/vector limitations
remain open. No live validation processes remain.


Full version15 legacy native regression launched with isolated report/config and
frozen binaries:1979 source scenes plus deterministic matrix and gate controls.
Live session17229, `aspect-ratio-pair-full/run.py`; inspect its terminal result
before qualification. Early cases pass; no full-run result is claimed.

L14 investigation started independently while that full run executes. Chrome32
natural-image cases/96 captures complete; frozen version15 rejects all32 with
preserved diagnostics. Runtime image measurement exposes decoded dimensions,
but independently picks axes; replaced-element ratio/constraints need probing.
See validation/intrinsic-image-investigation.md. No language admission change yet.


Existing-runtime intrinsic image probe:32 initial scenes now match Chrome for all
256 original/clone instance-viewports after restoring automatic units AND Hug
scale type on automatic axes, setting intrinsic measurement, and selecting the
natural/content-box or authored ratio with version15 exact-pair policy. Initial
units-only experiment left Fixed scale type and failed240/256; it is preserved
but does not establish a valid automatic-size limitation. Probe output and source
snapshots: `intrinsic-image-runtime-investigation/receipt.json`. Public compiler
admission remains unchanged. Next add non-square asset coverage, then implement
the compiler mapping and complete parity/native-pixel/visual gates. No engine
change was needed for this initial32-scene seam test.


L14 non-square/runtime and public mapping:64 wide/tall cases pass512 runtime
clone/resize comparisons. Compiler natural-image admission now passes all96
square/wide/tall scenes (768 instance-viewports), preserving auto dimensions and
using embedded PNG metadata. Intrinsic measurement uses existing Rive fields;
version15 exact-ratio capability carries natural or authored preferred ratio.
Public pre-implementation rejection and green logs are preserved. Full suite
exposed a stack overflow in the depth/resource-limit test because a PNG decoder
local enlarged the recursive compiler frame. Metadata reading moved to the asset
module; the failing resource-limit test now passes without stack-limit changes.
Full module rerun is live (`/tmp/intrinsic-image-full-public-v2.log`); fresh binary
parity/native-pixel/visual gates remain pending. Evidence:
`intrinsic-image-implementation/receipt.json`. The still-running frozen full
L13 regression predates L14 and cannot qualify these compiler changes.


L14 initial native qualification advances: rebuilt/frozen intrinsic-image
publisher and WASM pass all288 native geometry/pixel comparisons (96 square,
wide and tall scenes). All768 public clone/resize instance-viewports pass; full
module288 tests/53 groups and type check pass. Combined native/WASM11 + host15
suite passes26 tests. The native probe/renderer are unchanged from the exact-pair
snapshot. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-implementation/receipt.json. Visual review remains pending.
32 image/text card compositions were added and Chrome capture started; those
compositions are not included in the passing counts. The full pre-L14 frozen
regression remains independently live in session17229.


L14 card composition stage:32 image/text card scenes pass96 native geometry/pixel
comparisons and256 original/clone instance-viewports. Initial image suite plus
cards now totals128 scenes/384 native comparisons and1024 instance-viewports.
No new compiler/runtime change was needed for cards. Their32 inputs were added
to native/WASM parity, which is running in session44031 (log
`/tmp/intrinsic-image-card-parity.log`). Initial image visual inspection covers
108/288 views after inspecting18 square-asset three-width sheets and auditing
54 direct+54 exact-image transfers;180 initial image views and96 card views still
need review. Thin sampling-edge differences remain within unchanged thresholds.
The full pre-image frozen regression remains live in session17229. No full L14
qualification claim. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-card-native/receipt.json.


L14 validation follow-up: the expanded native/WASM corpus, including all32 card
inputs, passes all11 JavaScript tests (0 failures); preserved log:
`output/playwright/html-to-riv/intrinsic-image-card-native/native-wasm-parity.log`.
Four tall-image column/content-box sheets were directly inspected at all three
widths: natural ratio, bare authored ratio, constrained height and max-height.
Initial-suite review now covers126/288 views, with162 still pending; card review
remains pending for96 views. Visible overflow, padding and sibling placement
match Chrome; sampling-edge differences retain existing tolerances. This is
partial L14 qualification; broader flex stress and the current compiler full
regression remain required. Chrome is the sole browser acceptance reference;
existing Firefox research is historical supplemental evidence, not further work
or a release gate.


L14 tall-image visual review is complete:12 additional three-width sheets were
inspected (row border/content-box and column border-box); the four column
content-box sheets were recorded in the preceding batch. The audited initial
suite now covers198/288 comparisons through direct review and exact image-pair
transfers, leaving90 wide-image comparisons. Card review remains pending.
Natural-ratio overflow, authored-ratio sizing, constraints, padding and sibling
placement match Chrome visually; no tolerance changes were made.


L14 initial visual gate completed:96 square/wide/tall image scenes pass288
Chrome/native geometry and pixel comparisons, all now covered by audited visual
review (162 direct views and126 exact image-pair transfers). All20 selected
wide-image sheets were inspected at240/390/768, including fixed-height overflow,
authored and natural ratios, both box models, row/column layout and constraints.
Thin sampling edges remain within unchanged tolerances. Card sheets are generated
(28 sheets covering96 views) but not yet inspected. Expanded native/WASM parity
passes. Broader flex stress and full current-compiler regression remain required.
The separate pre-L14 aspect-ratio full run completed with5983 tests passing;
its5973 scene-pair baseline image audit is underway, so this does not yet claim
visual qualification of that full run or full L14 qualification.


L13 full frozen aspect-ratio-pair regression is now qualified:5983 tests pass,
including5973 scene pairs whose exact HTML/CSS and complete browser/native PNG
bytes match the reviewed v14 baseline. Rehashed source receipts, comparison data
and image files; shared harness/reset hashes match and current harness hashes
remain unchanged. There are0 remaining image comparisons. Receipt:
`output/playwright/html-to-riv/aspect-ratio-pair-full/receipt.json`.
This full run predates L14 compiler changes; intrinsic-image full qualification
remains separate and pending.


L14 card visual review advances: all14 selected wide-image card sheets inspected
at240/390/768; audited coverage is48/96 comparisons (42 direct and
6 exact image-pair transfers). Tall-image card review remains pending.
Text wrapping, card extents, note placement, image sizing and viewport overflow
match Chrome; glyph antialiasing and image sampling differences retain existing
tolerances. Full legacy regression using the frozen intrinsic-image compiler is
now running in session11402, with independent output and source snapshots under
`output/playwright/html-to-riv/intrinsic-image-full`; no result claimed yet.


L14 card visual gate completed:32 scenes/96 Chrome-native comparisons all audited
(84 directly viewed comparisons on28 sheets plus12 exact image-pair transfers).
Combined initial and card suites now pass384 native geometry/pixel comparisons
with complete visual coverage, and1024 public original/clone instance-viewports.
Native/WASM parity passes for these inputs. Tall natural/fluid viewport overflow,
fixed-height images, authored ratios, text wrapping and note placement match
Chrome. No tolerance changes. Full intrinsic-image legacy regression remains
running in session11402. Added128 flex stress fixtures spanning wide/tall assets,
both box models, four directions and natural/stretch/grow/shrink/wrap/
wrap-reverse/conflicting min-max/percentage-height combinations; Chrome capture
is running in session38355. These new cases are not yet qualified or counted as
passing.


L14 flex stress exposes an unwaived regression:384 comparisons complete,360
pass and24 fail. All failures are8 row/row-reverse align-items:stretch scenes
(two PNG ratios, two box models, three widths). The public compiler original/
clone resize test also fails and is checked in with the128-scene Chrome oracle.
For wide border-box row stretch, Chrome produces629px image width from the
stretched260px border-box height; native retains104px, moving following siblings
by525px. Taffy flex-basis passes a known cross dimension into ContentSize leaf
measurement, which suppresses aspect_ratio; the native intrinsic leaf returns
natural image width. This is the current diagnostic lead, not a qualified fix.
Preserved receipt and public/native red logs:
`output/playwright/html-to-riv/intrinsic-image-flex-native/receipt.json`.
No failures or tolerances waived. The expanded128 inputs are also included in the
native/WASM parity corpus; that expanded run remains pending. Full frozen legacy
regression continues separately in session11402.


L14 stretch diagnosis: first candidate marked intrinsic CSS leaves as replaced
and retained ratio transfer in leaf ContentSize sizing. The public128-scene
regression still fails;109 existing Taffy tests pass. This is an unqualified
experiment, not a fix. Before-source snapshots and red/engine logs are preserved
in `output/playwright/html-to-riv/intrinsic-image-stretch-investigation`.
Temporary INTRINSIC_TRACE instrumentation is compiling/running in session2445
to determine whether the actual measurement node reaches that path; remove the
instrumentation after diagnosis. Frozen legacy regression11402 remains separate.


L14 stretch fix candidate now passes the128-scene public compiler oracle through
1024 original/clone instance-viewports. The image ratio lives on the layout
wrapper, not a ratio-bearing measured leaf; the ineffective leaf experiment and
trace were removed. CSS intrinsic wrappers are marked replaced, and single-line
row stretch supplies ratio-derived flex basis and automatic minimum width.
All110 Taffy tests pass, including a new wrapper regression (Chrome629px expected
width with24px horizontal/18px vertical padding) and a non-replaced control.
Broader24-test ratio oracle suite is running in session18335. Pixel replay with
rebuilt runtime artifacts remains pending; prior24 native failures stay preserved
and unwaived. The full frozen pre-fix regression11402 cannot qualify this runtime
change. Evidence: intrinsic-image-stretch-investigation/receipt.json.


L14 stretch candidate passes all24 public aspect-ratio oracle tests, including
initial/nonsquare images, cards, flex stress, text, numeric boundaries and cloned
scene resizing. Fresh publisher/probe/Metal renderer/WASM build and freeze is
running in session97527 under intrinsic-image-stretch-toolchain. Added128 stretch
edge cases covering four directions, both assets/box models, explicit flex basis,
zero automatic minimum, min/max width, max-height, authored ratio and auto
cross/main margins; Chrome capture40400 is running. These edge cases have not
yet passed public/native validation. Prior red pixel evidence remains preserved.


L14 stretch edge public gate passes:128 additional scenes/1024 original-clone
instance-viewports match Chrome (flex bases, zero minimum, min/max limits,
authored ratio, automatic margins in all four directions). Native replay is
running for original flex stress46564 and edges10549; expanded native/WASM
parity10336 and full public suite57044 also run. Fresh immutable artifacts are
available in intrinsic-image-stretch-toolchain. Updated the investigation page
with current status, separating historical pending/rejection statements. No
pending native result counted as passing.


L14 stretch candidate full public suite passes291 tests/53 groups. Expanded
native/WASM corpus passes all11 tests including both128-scene flex/edge cohorts.
Corrected flex native replay passes384/384 geometry and pixel comparisons.
Four corrected wide/tall row/reverse-row stretch sheets directly inspected at
all three widths cover24/384 views after audited exact image-pair transfers,
including every previously failing stretch view. The remaining360 flex views
still need review; edge native replay10549 is running. No tolerances changed.
Receipts: intrinsic-image-stretch-investigation/receipt.json and
intrinsic-image-flex-native-v2/receipt.json.


L14 stretch edge native gate passes384/384 comparisons, matching its128-scene
public resize/clone gate. Both stress cohorts now pass768 native comparisons.
Original flex review covers53/384 views after six additional wide-row sheets;
331 flex views and384 edge views remain unreviewed. Edge review sheets are being
generated (74 selected sheets, session69303). Replaying128 previously reviewed
initial/card scenes against the fixed runtime in intrinsic-image-regression-native
(session recorded by the tool) to verify no visual regression; its merged oracle
retains exact original Chrome captures and source inputs. No tolerance changes.


L14 visual review continues: ten additional wide border-box reversed-row/column
sheets inspected at240/390/768. Audited corrected flex coverage is now109/384,
with275 remaining. Wrapping, growth/stretch, constraints, sibling offsets
and viewport overflow agree with Chrome; thresholds unchanged. All74 edge review
sheets are generated but remain unreviewed. Regression replay12429 continues.


L14 fixed-runtime initial/card regression is fully accounted for:384/384 native
geometry/pixel comparisons pass, and all384 full compiler inputs and browser/
native PNG pairs match their independently audited initial/card review receipts.
Transferred288 initial and96 card views, with0 remaining. Receipt:
`output/playwright/html-to-riv/intrinsic-image-regression-native/receipt.json`.
Corrected flex review advances to142/384 after six column/reversed-column sheets;
242 flex views and384 edge views remain. Growth/stretch overflow above the
viewport, reverse wrapping, min/max constraints and percentage heights visually
match Chrome. Existing thresholds unchanged. Full fixed-runtime legacy gate
remains required; pre-fix full regression11402 is independent.


L14 full fixed-runtime legacy regression started in session44428 with frozen
intrinsic-image-stretch-toolchain, isolated output/source hashes and fixture
snapshot under intrinsic-image-stretch-full. The pre-fix run11402 is still live
and preserved separately. Eight additional flex review sheets inspected; audited
coverage now166/384 with218 remaining. Content-box row and reversed
column constraints, wrapping, percentages, sibling positioning and overflow
match Chrome. Edge384-view inspection remains pending.


L14 corrected-flex review advances to202/384 audited comparisons after twelve
wide content-box reversed-row/column sheets. The wide-completion check caught
remaining row grow/shrink views; wide review is not complete. These and tall
views total182 remaining. The inspection receipt itself passed its hash/count
audit. Edge visual review remains pending; full fixed-runtime regression44428
was re-polled and remains live. No tolerance changes.


L14 wide-image flex review is now complete, verified by an empty wide subset of
remaining comparisons. Eight additional grow/shrink and tall-row sheets reviewed
at240/390/768 bring audited corrected-flex coverage to225/384, leaving
159 tall-image views. Tall wrapping overflow, natural sizes and conflicting
constraints match Chrome, with sampling differences under unchanged thresholds.
Edge review and full fixed-runtime regression remain pending.


L14 corrected-flex visual review: ten additional tall-image sheets inspected at
240/390/768 (percentage height, reverse row natural/wrap/reverse wrap/conflicting
limits, and column natural/stretch/grow/wrap). Audited coverage is now 275/384,
with 109 comparisons remaining. Chrome/native placement, sizing and overflow
agree; image sampling seams remain under unchanged thresholds. The authoritative
receipt is `output/playwright/html-to-riv/intrinsic-image-flex-native-v2/visual-inspection.json`.
Both full regression handles (11402 and 44428) were confirmed live this turn;
neither is counted as complete. Stretch-edge visual inspection remains pending.
Chrome is the sole browser reference; Firefox is not a qualification gate.


L14 review update: twelve further tall-image column and content-box wrapping
sheets inspected at all three widths. Corrected-flex audited visual coverage is
326/384; 58 comparisons remain. No new layout discrepancy or tolerance change.
The initial L14 pre-stretch full regression completed with exit 0: 5,983 tests,
5,973 scene comparisons. All 5,973 HTML/CSS and browser/native PNG pairs match
the qualified exact-ratio baseline; source receipt and image hashes were audited,
and all six shared harness/reset hashes match both the prior run and current
files. Evidence: `output/playwright/html-to-riv/intrinsic-image-full/receipt.json`,
`baseline-comparison.json`, and `visual-inspection.json`. This qualifies that
frozen pre-stretch toolchain only. Current stretch full run 44428 is still active;
focused stretch-edge visual review also remains pending.


L14 corrected-flex visual review is complete: 384/384 audited comparisons,
zero remaining. The final twenty tall content-box constraint/wrapping and both
box-model grow/shrink sheets were inspected at 240/390/768. The narrow shrink
case correctly reaches its padding floor in both Chrome and native output;
wide grow/shrink, reverse placement and overflow also agree. Image sampling
seams remain under unchanged thresholds. `intrinsic-image-flex-native-v2/receipt.json`
and `visual-inspection.json` record completion. This completes focused flex
review only; stretch-edge review and current fixed-runtime full regression
remain pending. Full regression session 44428 was confirmed live this turn.


L14 stretch-edge visual review started: twelve wide border-box row/reversed-row
sheets inspected at 240/390/768, covering flex basis, width/height constraints,
authored aspect ratio and cross-axis automatic margin. Audited direct inspection
and identical-image transfers cover 66/384 comparisons, leaving 318. Geometry,
overflow, sibling positions and image appearance agree with Chrome under the
unchanged thresholds. Evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Current full stretch regression session 44428 was confirmed
live; no restart or completion claim. Corrected-flex coverage remains complete.


L14 stretch-edge review advances to 143/384 audited comparisons (241 remain).
Twelve wide-image column/reversed-column and content-box row sheets were viewed
at 240/390/768. Basis, width limits, authored ratio, automatic margins and overflow
match Chrome. Some fractional image boundaries show thin horizontal sampling
seams within the unchanged pixel thresholds; no layout discrepancy was found.
Updated evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Full stretch regression session 44428 was confirmed live.


L14 stretch-edge wide-image visual review is complete, checked by an empty wide
subset in the audited remaining list. Twelve final wide content-box sheets and
four tall row constraint sheets inspected at 240/390/768 bring total coverage to
212/384, leaving 172 tall-image comparisons. Sizing, padding, sibling placement
and overflow agree with Chrome; thin image sampling seams remain within unchanged
thresholds. Evidence is in `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Full stretch regression session 44428 remains live as polled.


L14 stretch-edge tall-row review: eight additional authored-ratio, automatic-margin
and reversed-row basis/constraint sheets inspected at 240/390/768. Audited coverage
is 250/384, leaving 134 comparisons. Chrome/native sizing, padding, overflow and
sibling alignment agree; thin image sampling seams remain within unchanged gates.
Evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json` and
`receipt.json`. Full stretch regression handle 44428 was confirmed live this turn.


L14 tall-column stretch-edge visual review advances to 307/384 audited comparisons,
77 remaining. Eight additional sheets inspected at 240/390/768 cover basis,
width limits, authored ratio, automatic margin and reversed-column constraints.
Chrome/native sizing, overflow and sibling placement agree; a thin fractional
ratio boundary sampling seam stays within unchanged gates. Receipts updated in
`intrinsic-image-stretch-edge-native`. Full regression session 44428 is live,
with the log advancing through test 3148; no completion claim.


L14 stretch-edge review advances to 340/384 audited comparisons, with 44 remaining.
Eight tall reverse-column and content-box row sheets inspected at 240/390/768
cover authored ratio, auto margin, basis, minimum width and maximum height.
Layout and overflow match Chrome; the thin fractional column-ratio image boundary
seam remains within unchanged thresholds. Review and aggregate receipts updated
in `intrinsic-image-stretch-edge-native`. Full run 44428 was confirmed live.


L14 stretch-edge visual review complete: 384/384 audited comparisons, none remaining.
The final ten tall content-box and automatic-main-margin sheets were inspected
at 240/390/768. Layout, overflow, image sizing and free-space allocation agree
with Chrome; existing thin sampling seams remain under unchanged thresholds.
The focused initial/card, corrected-flex and edge cohorts now total 1,152 passing
Chrome/native comparisons with complete audited visual coverage (including exact
source/image transfers for initial/cards). SUPPORT.md and the edge receipt were
updated. This does not complete L14 qualification: full fixed-runtime regression
session 44428 is still live, and broader residual validation remains as documented.


L14 additional qualification launched against the frozen stretch toolchain: prior
aspect-ratio image and image-stress replays (80 scenes / 240 comparisons), session
67395; JavaScript host and TypeScript checks, session 56307. Both use separate
evidence directories. Current full regression 44428 was confirmed live. Updated
L14 backlog row and investigation current-status block to reflect complete focused
visual coverage, preserving historical stage evidence below. New runs remain pending.


L14 fixed-runtime prior-image regression completed: 72 image and 168 image-stress
comparisons pass geometry and native pixels. All 240 complete compiler inputs
and browser/native PNG pairs are identical to the reviewed exact-ratio baseline.
The baseline direct/within-run and cross-run review union was audited before
transfer. Evidence: `intrinsic-image-stretch-prior-images/audit.py`, both prior
image native output directories' `visual-transfer-union.json` and `receipt.json`.
No new inspection was claimed for identical outputs. Host/TypeScript session
56307 and full regression 44428 remain pending as last confirmed live.

L14 host follow-up: session 56307 completed with exit 0 for both the JavaScript
host suite and strict TypeScript check. Frozen WASM hash was verified before the
run. Logs and receipt: `intrinsic-image-stretch-host`. Full native regression
44428 remains the outstanding running validation.


L14 vector fallback validation launched independently under session 69222, using
NUXIE_NATIVE_GLYPHS=0 and the frozen stretch toolchain. It replays all 384 focused
initial/card, flex and edge scenes (1,152 comparisons) against saved Chrome
geometry and screenshots. Source probe confirms that zero disables native glyph
handling. Separate output and launch receipt: `intrinsic-image-stretch-vector`.
No vector pass or visual qualification is claimed yet; text failures will remain
unwaived. Full native session 44428 is still live, with log progress past 4,100.


L15 independent preparation: added 64 percentage-spacing Chrome fixtures covering
all flex directions, both box models, definite/intrinsic hosts and mixed/auto
spacing. Capture session 62216 targets 192 views. Frozen L14 rejection is preserved
in `percentage-spacing-investigation` (exit 1, unsupported-value). Scope and next
validation steps are in `validation/percentage-spacing-investigation.md`.
No percentage-spacing support is claimed; L14 native/vector runs continue.


L14 vector initial/card replay completed: all 384 geometry checks pass; 303 pixel
comparisons pass and 81 text-card pixel comparisons fail, unwaived. The first
failing 390px wide border-box natural card was visually inspected: image and
layout match, with text raster differences. All failures and that inspected
PNG hash triple are preserved in `intrinsic-image-stretch-vector-initial-cards/receipt.json`.
Remaining vector cohorts and full native regression are still running (69222,
44428). L15 Chrome capture completed: 64 scenes / 192 views, Chrome153.0.8010.12;
its receipt records the oracle hash. This is reference evidence, not L15 support.


L15 source investigation found that runtime root margins lose their unit when
there is no layout parent; padding units are preserved. Added eight top-level
spacing fixtures and launched 24 Chrome references (19118). The initial 64 scenes
remain unchanged. Documented compiler retained-value and substitution-validation
seams in `validation/percentage-spacing-investigation.md`. Runtime root-wrapper
behavior still needs testing; no implementation or percentage support claimed.
Native full and vector replay sessions 44428/69222 were confirmed live this turn.


L14 vector replay is terminal: 1,152/1,152 geometry checks pass, 1,071 pixel
comparisons pass, and 81 text-card pixel failures remain unwaived. All 768 flex
and edge comparisons have complete audited exact-input/PNG visual transfers
from the native-glyph cohorts. Initial/card vector visual review remains partial.
Evidence: `intrinsic-image-stretch-vector/receipt.json` and both cohort receipts.
Full native 44428 remains live, past test 5,200. L15 root capture completed with
8 scenes / 24 Chrome views; oracle hash recorded in its investigation receipt.

L14 current qualification update: the frozen stretch-runtime full native regression completed with 5,983/5,983 checks passing and 5,973/5,973 Rust Metal scene comparisons. Exact HTML/CSS and full Chrome/native PNG bytes match the reviewed pre-fix baseline for all 5,973 comparisons; all six shared harness/reset hashes were verified unchanged. Evidence: `output/playwright/html-to-riv/intrinsic-image-stretch-full/{receipt,visual-inspection,baseline-comparison}.json`. Vector fallback remains partial: 1,152 geometry passes, 1,071 pixel passes and 81 unwaived text-card pixel failures. Audited image-only visual coverage is now 1,056 comparisons (288 initial plus 768 flex/edges); two failing text-card views are inspected and 94 card views remain for inspection. Chrome is the sole browser reference; Firefox is not a qualification gate. No tolerance changes.

L15 experimental implementation now retains nonnegative physical padding/margin percentages (finite 0–10000%) through publication and runtime resizing, including mixed lengths and automatic margins. Two public syntax/cascade tests and two Chrome-oracle tests pass: 72 scenes, 576 original-and-clone viewport updates at unchanged 0.1px tolerance. Authored-root cases pass without runtime changes. Negative margins, logical properties and Grid remain excluded. Native/WASM parity, Rust Metal pixel/visual checks, expanded compositions and full regression remain pending; this is not a qualification claim. Evidence: `output/playwright/html-to-riv/percentage-spacing-investigation/implementation-receipt.json`.

L15 rendered validation found eight unwaived pixel failures among 216 initial/root Chrome comparisons; all 216 geometry checks pass. The directly inspected 390px mixed-spacing fixture places the inner rectangle one pixel lower in native output: Chrome y=28.484375 versus native y=28.5, implicating fractional percentage resolution. Failure PNGs and receipts are preserved in `percentage-spacing-validation` and the `percentage-spacing-*-native` directories. Initial native/WASM parity passed ten tests; a missing frozen probe symlink caused the remaining test to fail, and that isolated test passes after the symlink repair. Added 32 Chrome composition fixtures covering nesting, wrapping/reverse wrapping, limits, intrinsic images, text, cascade and intrinsic hosts; captured 96 reference views. Removed the obsolete margin:10% rejection assertion; full public regression is rerunning. No tolerance changes; L15 remains experimental.

L15 composition test is red: 176 coordinate mismatches across the 32-scene original/clone resize corpus. Examples: row wrap tail.y=109.171844 versus Chrome225.20313 at240; intrinsic row card.width=60 versus Chrome72.34375. Full public regression stopped with exit101 at this new test (27 sibling oracle tests passed). Preserve `percentage-spacing-validation/public-full-v2.log`; investigate wrapped-line free-space allocation and cyclic percentage padding/intrinsic sizing separately from the eight pixel-rounding failures. Qualification remains incomplete.

L15 diagnosis correction: wrapped-line mismatches came from the public oracle helper omitting installation of the emitted align-content policy. Added the same installation as the native probe; wrapping errors disappear. The frozen composition native replay confirms 96 comparisons with 12 geometry failures, all intrinsic-container views, and 11 pixel failures. Corrected public test retains 144 intrinsic coordinate mismatches. A separate column padding reference-axis candidate is under test; no runtime fix is qualified yet. Evidence: `percentage-spacing-validation/composition-align-content.log` and `percentage-spacing-composition-native/replay.json`.

The L15 column padding-axis experiment completed with the same 144 intrinsic coordinate mismatches and was reverted exactly to its saved source. No runtime change retained from this experiment. The next target remains intrinsic percentage sizing; the public helper align-content correction is retained.

L15 reduced reproducer: 16 scenes / 48 Chrome views / 128 original-and-clone viewport updates isolate control, padding, margin, combined spacing, fixed parent, disabled shrink, content-box and nested cases in both directions. Baseline reports 184 coordinate mismatches; row padding loss reproduces even with a fixed parent and disabled shrink. `percentage-spacing-validation/minimal-investigation.json` records the evidence. A parent-width-preservation measurement experiment is running against all four percentage-spacing oracle tests; not yet qualified.

L15 parent-width experiment is terminal: initial/root oracles still pass; minimal coordinate mismatches fall from184 to64 and composition mismatches from144 to48. The candidate remains unqualified with failing tests preserved; investigate the remaining column cases and require full regression before qualification. Log: `percentage-spacing-validation/parent-width-experiment.log`.

L15 geometry candidate passes all four Chrome-oracle tests: 120 scenes / 960 original-and-clone viewport updates at unchanged 0.1px tolerance. Preserving parent width for auto-width row measurement and remeasuring content-sized columns after inline width resolves removes the focused geometry failures. This is a broad candidate, not a qualified runtime change: full compiler regression is running (97293), and dedicated runtime checks, fresh parity, native pixel reruns and visual review remain pending. Evidence: `percentage-spacing-validation/geometry-candidate-receipt.json`. Existing frozen pixel failures remain unwaived.

L15 geometry candidate regression: full compiler297 tests across54 groups pass with0 ignored; Taffy111 unit tests pass, including a new direct row/column percentage-padding intrinsic measurement test. The initial workspace-level Taffy command could not run a non-workspace package; the manifest-path invocation completed successfully. Fresh candidate binaries/WASM are building in `percentage-spacing-geometry-toolchain` (38491). The runtime candidate remains unqualified for pixels; previous pixel failures are unwaived. Receipt: `percentage-spacing-validation/geometry-candidate-receipt.json`.

L15 fresh geometry-candidate validation is terminal: native/WASM11 tests pass; native360/360 geometry comparisons pass,347 pixel comparisons pass and13 pixel failures remain unwaived (initial6, root2, composition5, minimal0). Focused rounding references add27 scenes/81 Chrome views; all agree with truncating each percentage edge to1/64px before summation. A dedicated public rounding test uses stricter0.001px tolerance without changing standard gates. Evidence: `percentage-spacing-validation/geometry-native-receipt.json`.

L15 rounding experiment passes all five oracle groups:147 scenes /1176 original-and-clone viewport updates, including27 rounding scenes at stricter0.001px tolerance. Independent edge truncation removes the numerical rounding reproducer. This is still a generic diagnostic implementation: explicit CSS opt-in, requirements contract, measurement scope/performance review, new parity, native pixel/visual checks and full regression remain required. Evidence: `percentage-spacing-validation/rounding-candidate-receipt.json`. The13 frozen-candidate pixel failures are not yet claimed fixed.

L15 explicit opt-in implemented: requirements version16 adds `layout-css-percentage-spacing-v1` and unique `layout_percentage_spacing` occurrence IDs. Rust validation checks capability/version/target consistency; compiler emission, native probe installation, cloned runtime state and JavaScript API types are updated. Taffy defaults remain off; the policy enables measurement and percentage edge precision across the opted-in solve tree. All five focused oracle groups still pass (147 scenes/1176 instance-viewports), and Taffy111 tests pass. Contract tests are running; fresh parity/pixels/full regression and opt-out/performance coverage remain pending. Evidence: `percentage-spacing-validation/opt-in-receipt.json`.

L15 version16 syntax/cascade/contract tests completed:3/3 pass, including missing-capability, downgrade, duplicate/missing occurrence and mixed aspect-ratio contract checks. Fresh parity and pixel qualification remain pending.

L15 opt-out coverage passes: the same Taffy tree toggles false/true/false, restoring float geometry when CSS precision is disabled. All112 Taffy tests pass. Added version16 host tests for unsupported-capability rejection before stream output, invalid/duplicate/missing targets and compile-once viewport replay. Updated TypeScript version/capability assertions; strict typecheck passes. Initial frozen build failed on a missing probe import and is preserved; corrected build runs in `percentage-spacing-v16-r2-toolchain` (57092). Host tests, fresh parity and441 native comparison reruns await the corrected toolchain.

L15 version16 focused gates are green:441/441 Chrome/native geometry and real Rust Metal pixel comparisons pass;27/27 native/WASM and host-contract tests pass. All13 earlier geometry-candidate pixel failures are fixed in this fresh run, with thresholds unchanged and red artifacts retained. Visual inspection has begun:6 directly inspected views plus3 exact within-run image transfers account for9/441 comparisons,432 remaining. Added reusable `validation/make-replay-sheets.py` to prepare unscaled comparison sheets separately from review recording. Full public suite19571 remains running; full native regression, visual completion and nested measurement cost/coverage audits remain outstanding. Receipt: `percentage-spacing-validation/v16-receipt.json`.

L15 full public suite is green:299 tests across54 groups,0 ignored. Full frozen native regression started under48657 in `percentage-spacing-v16-full`. Nested-cost smoke check covers4/8/16/32 levels with3 native probes each: all complete; percentage median13/16/16/26ms versus pixel14/14/16/19ms. These include process/import/layout/stream costs and are not an isolated CPU benchmark or general performance guarantee. Visual coverage is now12/441 audited comparisons; root mixed-spacing repro reviewed at all widths.

L15 rounding visual review complete:81/81 comparisons audited,30 directly inspected views and51 exact full-PNG pair transfers within the run. All ten representative sheets were inspected for padding/margin/combined boundary placement and clipping. Total focused visual coverage93/441, 348 remaining. Full native regression48657 remains active as last confirmed live; no qualification claim yet.

L15 root visual review complete:24/24 directly inspected and audited across row/column flow, percentage padding/margins, mixed values and auto margins. Total visual coverage114/441, 327 remaining. Full native regression48657 was confirmed live during this review; last observed check419 passing. All qualification limitations remain in the version16 receipt.

L15 minimal intrinsic visual cohort complete:48/48 audited comparisons,36 direct views and12 exact-image transfers. Inspected row/column padding, combined margins, controls, fixed parent, margin-only and nested cases at all widths. Total focused coverage162/441, 279 remaining. Full native regression48657 confirmed live; no completion claim.

L15 text/image composition visual review adds24 direct views: row/column × both box modes at three widths. Text wraps and surrounding spacing align; minor glyph raster differences persist within existing limits. Image size/insets align overall; thin image/quadrant boundary differences remain within existing limits and are explicitly recorded, not described as byte-identical. Composition coverage30/96; total186/441, 255 remaining.

L15 wrapping visual review: all eight row/column × border/content-box × wrap/wrap-reverse sheets inspected at 240/390/768; placement, wrapping and percentage spacing align with Chrome. Composition review now54/96; audited focused total210/441,231 remaining. Full native regression48657 confirmed live. Chrome remains the sole browser reference; Firefox is not a qualification gate. L15 remains experimental pending complete visual and regression/coverage audits.

L15 nested/size-limit visual review: eight row/column × border/content-box × nested/limits compositions inspected at all three widths. Insets, constrained card sizes, sibling positions and boundary overflow/clipping align with Chrome. Composition78/96 audited; focused total234/441,207 remaining. Full native regression48657 confirmed live; L15 qualification remains incomplete.

L15 composition visual review complete:96/96 audited, including the final cascade and intrinsic row sheets. Four additional basic row sheets inspected at all widths for percentage padding, margin, auto-margin and intrinsic padding. Focused visual total267/441, 174 remaining, all in the initial matrix. Full native regression48657 confirmed live; no qualification claim yet.

L15 basic matrix review continues: six intrinsic border-box and definite content-box row sheets directly inspected at all widths. Parent sizing, child insets, box expansion and auto-margin distribution align with Chrome. Audited total288/441, 153 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 matrix review: intrinsic content-box padding/mixed/auto-margin plus reversed border-box definite padding/margin/mixed directly inspected at all widths. Parent sizing, overflow, right anchoring and physical insets align with Chrome. Audited total309/441, 132 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 reversed-row review adds six sheets at all widths, covering border-box intrinsic spacing, definite auto-margin and content-box definite padding. Left overflow/clipping, insets, parent widths and box expansion align with Chrome. Audited total330/441, 111 remaining. Regression48657 confirmed live; latest observed check3131 passing. Qualification remains incomplete.

L15 visual review adds remaining reversed content-box row sheets and definite column padding at all widths. Intrinsic sizing, auto spacing, clipping and vertical sibling positions align with Chrome. Audited total348/441, 93 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 column border-box visual review adds definite margin/mixed/auto and intrinsic padding/margin/mixed sheets at all widths. Parent widths, child insets and vertical sibling spacing align with Chrome. Audited total372/441, 69 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 column content-box review adds six sheets at all widths: percentage padding expansion, mixed/auto spacing and intrinsic parent sizing visually align with Chrome, including horizontal overflow. Audited total390/441, 51 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 reverse-column visual review adds six sheets at all widths. Bottom anchoring, physical percentage insets, margin separation and intrinsic sizing align with Chrome. Audited total411/441, 30 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 focused visual review complete:441/441 comparisons audited across initial192, root24, composition96, minimal48 and rounding81. All ten final reverse-column sheets viewed at every width; bottom anchoring, insets, intrinsic sizing and overflow align with Chrome. Receipts verify direct sheet/PNG hashes and exact within-run image transfers. Existing text/image raster differences remain documented within unchanged thresholds. Full native regression and broader interaction coverage audit remain pending; L15 is not yet qualified.

L15 reverse-composition coverage expansion:32 new scenes/96 Chrome-native geometry and pixel comparisons pass with unchanged thresholds. Public original/clone resize regression passes256 instance-viewport updates. Covers both reversed directions and box models across nesting, wrapping, reverse wrapping, limits, images, text, cascade and intrinsic hosts. Six views directly inspected;90 remain. Expanded native/WASM parity4427 and full regression48657 confirmed live. Evidence: `percentage-spacing-validation/reverse-composition-receipt.json`. This extends the previous441 fully reviewed comparisons; qualification remains incomplete.

L15 full native regression completed:5983/5983 checks pass; all5973 Chrome/Rust Metal scene comparisons have exact HTML/CSS and full browser/native PNG identity with the reviewed intrinsic-image-stretch baseline. Six shared harness/reset hashes match the baseline and current files. Audited receipt: `percentage-spacing-v16-full/receipt.json`. Expanded reverse-composition native/WASM parity11/11 passes;27/96 new views visually audited,69 remaining. L15 remains unqualified pending expanded visual completion and interaction coverage audit.

L15 reverse-composition visual audit advances to51/96: all intrinsic and nested cases in both reversed directions and box modes directly inspected at240/390/768. Intrinsic parent sizing, percentage insets, content-box expansion and offscreen overflow align with Chrome.45 views remain; full regression and parity already pass. Qualification remains incomplete pending visual and interaction audit.

L15 reverse-composition visual review now72/96. All image and text compositions inspected at every width; placement and spacing align with Chrome. Thin image perimeter/quadrant and glyph raster differences are visible within unchanged limits and explicitly recorded. Remaining24 views cover limits and cascade; no qualification claim yet.

L15 reverse-composition visual review complete:96/96 directly inspected and audited, including final limits/cascade sheets across both reversed directions and box modes. Total focused Chrome/native geometry, pixel and visual coverage is537/537. Full native5983 checks/5973 audited scene pairs and expanded parity11/11 pass. Remaining interaction coverage audit is still required before qualification.

L15 interaction audit found a real remaining defect:40 new scenes/120 views yield114 combined passes and6 geometry failures, all column/column-reverse content-box with sub-unit growth and percentage basis. Five of these also fail pixels. Public helper omission for justify-content was corrected; six previous spacing groups pass, new interaction group remains red. Native probe independently reproduces failures. Preserved logs and `percentage-spacing-validation/interaction-receipt.json`; no tolerance changes. Parity79504 remains pending.

L15 basis diagnosis: four reduced Chrome scenes preserve failure after removing all card children and after disabling growth; literal pixel padding passes. This isolates percentage-padding adjustment rather than content measurement or sub-unit distribution. Candidate resolves content-box basis padding against inline width under existing CSS spacing opt-in, leaving basis axis and default behavior unchanged. Public spacing suite70977 running; not yet qualified. Expanded interaction parity11/11 passes. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 basis-axis correction passes all eight public spacing oracle groups:223 scenes/1784 original-and-clone viewport updates. New Taffy regression checks both column directions with policy false→true→false; all113 layout-engine tests pass. This confirms inline-axis padding adjustment while preserving main-axis percentage basis and default behavior. Full public3140 and new frozen native/WASM build14590 are running. Fresh pixels, visual review and full native regression remain required; prior six rendered failures are not yet claimed fixed. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 basis-axis full public regression passes302 tests/54 groups,0 ignored. Frozen v16-r3 native/WASM toolchain built successfully. Started669 focused comparisons across eight cohorts (44178), expanded parity/host checks48956, and full native regression80940 in `percentage-spacing-v16-r3-full`. Shared browser harness and main corpus must remain frozen while80940 runs. Fresh rendered qualification remains pending.

L15 corrected v16-r3 focused replay is terminal:669/669 geometry and native pixel comparisons pass, parity/host27/27 pass. All537 prior comparison pairs transfer visual review with exact compiler-input and full PNG identity. Reduced12/12 directly inspected; both formerly failing column compositions6/6 directly inspected. Total555/669 visual coverage,114 interaction views remaining. Reduced zero-growth240 retains a thin tail-edge raster difference within unchanged limits. Full native80940 confirmed live; vector fallback audit remains separate. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 v16-r3 aspect-ratio interaction visual cohort complete:24 direct views across four directions and both box models. Ratio sizing, percentage insets and viewport overflow match Chrome visually. Interaction coverage30/120; aggregate579/669,90 views remain. Full native80940 confirmed live.

L15 row clipping interactions inspected at all widths in both box models and directions: clip boundaries and hidden footer match Chrome. Aggregate visual coverage591/669,78 interaction views remain. Started full669 vector-fallback comparisons with native glyphs disabled (68274); this uses the same frozen v16-r3 toolchain and Chrome references. Full native80940 confirmed live.

L15 column clipping visual inspection complete across both directions/box models:12 more views align with Chrome. Aggregate603/669 reviewed,66 remain. Vector replay68274 remains live; composition cohort reports failures requiring inspection, while interaction/reduced/initial/root/minimal completed successfully. Full native80940 confirmed live. No failures waived.

L15 vector fallback replay completed:669 geometry passes,651 pixel passes,18 unwaived text pixel failures in composition/reverse-composition cohorts. Authoritative cohort results override runner process exit0 because the runner continues after failures. Receipt: `percentage-spacing-validation/vector-v16-r3-receipt.json`. Row order visual review adds12 native views; aggregate615/669,54 remain. Full native regression still pending.

L15 v16-r3 focused native visual review complete:669/669 comparisons audited. Final54 direct views cover column order, remaining grow/basis and all distribution interactions at240/390/768. Insets, reverse placement, free-space allocation, alignment and viewport overflow match Chrome visually. Interaction receipt120/120 passes hash audit. Full native80940 confirmed live;18 vector text pixel failures remain unwaived and require investigation. L15 remains unqualified.

L15 vector failure review:all18 failing text views plus6 passing text controls directly inspected; differences are concentrated in glyph pixels while wrapping and surrounding shape placement align. Another477 vector views transfer from fully audited native sources with exact compiler-input/full-PNG identity;501/669 vector views reviewed,168 non-text composition views remain. Single-scene240px vector replay fails deterministically twice with identical PNGs; native-glyph toggle passes against the same Rive bytes and Chrome image. This isolates a rendering-path difference, not its root cause. Reproducer:percentage-spacing-validation/vector-text-diagnosis.json. No failure waived; scene minimization remains next.

L15 vector text minimization now reproduces on a single display:block text element:Quiet weekend,110px width,Inter16px/1.4. Fresh pinned Chrome153.0.8010.12 controls show the same local text RGB failure after independently removing siblings, zeroing all spacing and removing wrappers; all geometry comparisons pass. This demonstrates percentage spacing is not required for the rendering residual. Single-scene and reduced reproducers remain preserved in percentage-spacing-validation/vector-text-diagnosis.json. Further typography minimization and runtime diagnosis remain; no failure waived.

Vector text diagnosis adds15 pinned-Chrome typography controls. Individual words/glyphs pass but retain nonzero RGB error; the full nowrap phrase still fails. Integer line heights22/23/24 pass, while22.4 fails. Fractional sweep22.125/22.25/22.5/22.75/22.875 is preserved with metrics in percentage-spacing-validation/vector-text-diagnosis.json. Baseline placement, coverage and horizontal placement remain ranked hypotheses; no runtime change or root-cause claim yet.

Vector baseline experiment:temporary opt-in snapping passes both reduced and original single-scene reproducers. Expanded24 text composition views improve from6 passes to16, with8 residual pixel failures; geometry remains checked. Runtime source restored exactly after freezing diagnostic probe; no shipping change retained. Native glyph rasterizer already snaps final world baselines, whereas vector paths keep fractional baselines. This explains part of the error but is not a complete fix; final draw-time transform and remaining coverage need investigation. Evidence:percentage-spacing-vector-snap-experiment/receipt.json. Separately, all669 vector views now have visual audit evidence(645 exact transfers plus24 direct text inspections);18 original failures remain unwaived.

Vector snap audit separates two residuals:16/24 views have exact intended world baselines, while content-box views can miss by0.5625px. All border-box768 failures already have exact baseline snaps, so transform timing is insufficient as a full explanation. In the row-border-box768 text rectangle, summed ink is105740 Chrome,92139 snapped vector and108466 native glyph; this diagnostic shows a remaining coverage difference(about12.9% less vector ink), without changing gates. Audit files:percentage-spacing-vector-snap-experiment/baseline-audit.json and border-768-ink-profile.json. Next:font smoothing/outline coverage and draw-time transform seam; no shipping changes retained.

Native font-smoothing diagnostic now runs at the glyph-producing probe seam. Renderer-only attempt did not exercise rasterization and is preserved as an invalid diagnostic. Fresh probe toggling changes text RGB error from2.0653 to4.1162; both native controls pass. Source restored after freezing diagnostic probe. Ink measurements and unchanged gates are recorded in percentage-spacing-vector-smoothing-experiment/receipt.json. Font smoothing contributes to the coverage difference but is not yet a complete vector explanation or fix.

L15 corrected full native regression completed:5983/5983 checks pass; all5973 Chrome/native scene pairs match exact HTML/CSS and full PNGs from the reviewed prior baseline. Six shared harness hashes match baseline and current files. Receipt:percentage-spacing-v16-r3-full/receipt.json. Draw-time baseline diagnostic improves text24 from16 to20 passes; source restored, four pixel residuals remain. Consolidated investigation:validation/vector-text-baseline-investigation.md. Original vector18 failures remain unwaived; L15 retains vector qualification work.

L16 signed-margin parser implemented as an unqualified candidate. Public tests went red(2 failures) before the change and now pass2/2:shorthand/longhands, px/em/rem/percent, custom values, explicit magnitude bounds and negative-padding/dimension rejection. Obsolete margin:-1% rejection updated while padding:-1% remains rejected. Chrome/runtime overlap, extents, resize/clone, parity and regression gates remain pending; no visual support claim. Logs:output/playwright/html-to-riv/negative-margins-{red,green}.log.

L16 initial geometry gate passes48 Chrome scenes/144 reference views and384 original-and-clone viewport updates. Covers4 directions×2 box models×pixel/percent/auto/wrap/large-negative-extents/intrinsic cases. Focused public regression passes58 tests; obsolete auto-margin and custom-property rejection assertions removed, accepted behavior covered by new tests. Frozen native/WASM build45752 running in negative-margin-toolchain. Pixel, visual, parity, full regression and expanded composition qualification remain pending. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json.

L16 initial native pixels pass144/144 comparisons at unchanged geometry/pixel gates. Full public9353 remains active. Initial JS parity failed because mutable target/debug/html-to-riv disappeared during Cargo build; frozen-binary retry50563 is running and the failed log is preserved. Native visual review, vector replay and expanded coverage remain pending; no qualification claim.

L16 full public regression passes305 tests,0 ignored. Initial vector replay144/144 geometry/pixel comparisons passes. Frozen JS suite passes10 tests including corpus parity; sole failure was a missing probe symlink, corrected and the host-contract test rerun successfully(1/1). Both failed infrastructure logs retained. Native visual review48/144 complete:all large-negative-extents and percentage cases inspected across directions and box models; overlap, clipping and reverse anchoring align with Chrome. Remaining96 visual views, expanded compositions and full native regression stay open. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json.

L16 expanded composition geometry passes64 scenes/192 Chrome views/512 original-and-clone viewport updates. Includes growth/basis, order, clipping, nested percentage margins, size limits, distribution, text and images across all directions/box models. Native replay8214 and expanded parity2073 are running. Full frozen native regression46959 started in negative-margin-full; preserve shared harness and main corpus while it runs. Basic visual review72/144:all intrinsic cases now inspected, matching parent sizing, overlap and clipping. Qualification remains incomplete.

L16 basic visual review complete:144/144 native views directly inspected and audited; all144 vector comparisons transfer with exact compiler-input and full browser/native PNG identity. Final auto/pixel-margin sheets match Chrome placement, overlap, reverse anchoring and clipping. Expanded native/WASM parity completes11/11. Composition vector replay completes183/192 with9 unwaived pixel failures; native192/192 passes numerically, with visual review pending. Full native regression46959 confirmed live. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json. Qualification remains incomplete.

L16 composition visual review48/192: all clipping and nested-margin scenes inspected at240/390/768 across four directions and both box models. Clip boundaries, overflow, overlap and reverse placement align with Chrome. Nested content-box cases retain thin vertical teal/purple boundary raster differences within unchanged gates, explicitly recorded. Aggregate native visual192/336;144 composition views remain. Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 composition visual review96/192: all grow/basis and min/max-limit scenes inspected at240/390/768 across four directions and both box models. Responsive sizing, sibling shrinkage, overlap and reverse anchoring match Chrome. Aggregate native visual240/336;96 composition views remain (order, distribution, text and images). Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 composition visual review144/192: all order and distribution scenes inspected at240/390/768 across four directions and both box models. Subtree overlap paint order, space-evenly allocation, cross-axis alignment and narrow overflow match Chrome. Aggregate native visual288/336;48 text/image views remain. Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 focused native visual review complete336/336 (basic144 plus composition192). Final48 text/image views inspected at240/390/768 across four directions and both box models. Wrapping, auto text-card height, image quadrants, overflow and overlap align with Chrome. Small native glyph coverage differences near the end of weekend remain within unchanged gates and are recorded. Full native46959 confirmed live; composition vector audit and9 unwaived pixel failures plus interaction coverage audit remain.

L16 vector visual audit complete336/336: basic144 exact transfers; composition168 exact compiler-input/full-PNG transfers plus24 directly inspected text views. All9 failures are local RGB errors in row, row-reverse and column border-box text at every width. Wrapping and surrounding geometry agree with Chrome; visible glyph coverage differences retained, including passing controls. Explicit failure receipt:negative-margin-composition-vector/failure-visual-inspection.json. Full native46959 confirmed live; pixel diagnosis and interaction coverage audit remain.

L16 vector minimization: four fresh pinned-Chrome controls/12 views reproduce9 local RGB failures with all geometry passing. Removing every margin or removing siblings retains failure at all widths; changing line-height22.4px to22px passes3/3. Negative margins therefore are not necessary for the residual; fractional line-height sensitivity is consistent with the prior vector baseline investigation, without proving identical root cause. Receipt:negative-margin-text-minimize-vector/diagnosis-receipt.json; direct visual review of these reduced sheets remains pending. Full native46959 remains live (last observed check2242 passing).

L16 coverage audit adds32 scenes/96 pinned Chrome153 references for aspect ratio, reverse wrapping/distribution, stretch and percentage/auto-margin combinations across all directions and box models. Permanent public original/clone resize test and JS parity corpus added. Public20907 and frozen native replay48811 started; qualification pending results and visual review. Reduced vector diagnosis12/12 now directly inspected, including residual glyph differences in the passing22px control. Full native46959 confirmed live; no tolerance changes.

L16 interaction native96/96 and original/clone public test pass. Aspect-ratio subset24/96 views directly inspected and audited; ratio sizing, column shrinkage, overlap and reverse placement align with Chrome. Remaining72 views cover reverse wrapping, stretch and percentage/auto combinations. Expanded parity10449 and vector replay68859 started; full native46959 confirmed live.

L16 interaction visual48/96: reverse-wrap subset24 views inspected across all directions/box models. Narrow content-box row line stacking, distribution and overlap match Chrome. Vector replay96/96 passes at unchanged gates. Remaining48 views cover stretch and percentage/auto combinations. Expanded parity10449 and full native46959 confirmed live.

L16 interaction review complete96/96: stretch and percentage/auto-margin combinations inspected at all widths, matching Chrome sizing, overlap and overflow. All96 vector views transfer with exact compiler inputs and full PNG identity; transfer audit passes. Expanded native/WASM parity11/11 passes. Total focused native432/432 numeric and visually audited; vector423/432 with9 reviewed unwaived text failures. Full native46959 confirmed live; final qualification audit remains.

L16 final public suite passes307 tests across55 groups,0 ignored. Current support table now reflects implemented percentage/signed spacing and separates candidate acceptance from qualification. Consolidated scope/coverage audit added to validation/negative-margins-review.md;432 native views audited,423/432 vector pixels pass with9 unwaived text failures. Full native46959 remains live; no final qualification claim.

### Relative positioning reference capture (2026-09-10)

L17 remains investigating. The separate `relative-position-oracle` artifact records
192 scenes / 576 Chrome153.0.8010.12 captures and 3,456 passing browser reference
invariants. No native or visual qualification is claimed. Auto-height percentage
offsets require a runtime discriminator. See `validation/relative-position-investigation.md`.

### Negative margins native qualification (2026-09-10)

Full native5983/5983 passes, including5973 scene pairs with audited exact
HTML/CSS and complete browser/native image identity to the reviewed L15 baseline.
All six frozen harness hashes match. Focused native432/432, original/clone1152
updates, public307 and native/WASMJS11 pass. L16 is native-qualified; nine vector
text pixel failures remain unwaived. Evidence: `negative-margin-full/receipt.json`,
`negative-margin-full/visual-inspection.json` and `validation/negative-margins-review.md`.

### Relative positioning engine correction (2026-09-10)

The default-off engine policy passes15360 Chrome-reference coordinate checks
across192 scenes and768 same-tree resize updates. Disabled output exactly matches
the legacy baseline (128 coordinate discrepancies across16 scenes). All114 engine
tests and runtime compile check pass. This does not qualify public CSS, actual
Rive clones, parity or pixels. See `validation/relative-position-investigation.md`
and `relative-position-engine/matrix-receipt.json`.

### Relative positioning runtime transport (2026-09-10)

Injected-wire geometry passes192 scenes /1536 original+clone viewport updates,
with30720 coordinate checks, effective engine-policy assertions and independent
clear/retained-clone controls. Runtime geometry also matched before forwarding
the engine flag: do not infer an imported-scene defect from the isolated Taffy
reproducer. Public CSS and pixels remain unqualified. See
`validation/relative-position-investigation.md` and the pinned
`relative-position-engine/runtime-transport-receipt.json`.

### Relative positioning public compiler geometry (2026-09-10)

Public candidate syntax tests went from2 failures to3 passes. Existing-wire
compiler output passes192 Chrome scenes through1536 imported original/clone
viewport updates without installing the experimental relative-position policy.
The full module suite is running in `relative-position-engine/full-public.log`.
Pixel, paint-order/composition and native/WASM qualification remain open.

Full module run completed successfully:312 tests across57 groups, no failures
or ignored tests. See `relative-position-engine/public-receipt.json`.

Relative positioning frozen replay: initial576 geometry passes, pixels529/576;
composition192 geometry passes, pixels159/192. Initial native/WASM11/11 passes.
A directly inspected overlap pair confirms positioned paint order differs from
Chrome. All failures remain unwaived; visual audit is incomplete. Aggregate:
`output/playwright/html-to-riv/relative-position-receipt.json`.

Positioned paint runtime discriminator:24/24 geometry/pixels pass, all24 directly
reviewed and audited. Six historical overlap failures fixed with diagnostic
source-marker injection. Larger192/576 replays are running; public host contract
and full qualification remain open. See `validation/positioned-paint-order-investigation.md`.

Positioned paint expansion: all576 initial and192 composition diagnostic geometry/
pixel comparisons pass, correcting80 earlier failures. Larger visual audit pending.
Lifecycle2 and compiler5 tests/types pass. Version17 positioned-paint capability
and target payload now emitted/validated; full module session24071 running.
Rebuilt native/WASM and executed public host/parity gates remain open.

Version17 full public module run completed: 317 tests across 58 groups,
no failures or ignored tests; session24071 exit0.

Frozen version17 qualification: host17/17 and expanded native/WASM11/11 pass.
Public discriminator24/24 numeric with24 exact-input/full-PNG reviewed transfers;
public composition192/192 numeric passes. Diagnostic composition visual audit
12/192 complete, with observed thin image-edge residuals within unchanged gates.
Public initial576 and composition vector192 remain live (4559,92100). Evidence:
`positioned-v17-receipt.json` and `validation/positioned-paint-order-investigation.md`.

L17 public v17 initial native replay completes576/576 geometry and pixels. Composition vector completes192/192 geometry,183/192 pixels with9 unwaived text failures. Diagnostic composition direct visual review reaches24/192 after row-border-box both-positioned/order/text/wrap-ratio sheets at all three widths; stacking, wrapping, ratio sizing and overflow match Chrome, with small recorded glyph coverage differences. Full native regression26540 confirmed live by session poll. Larger visual audit and vector qualification remain. Receipt:output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 nested clipping composition visual review is complete across all four directions and both box models at240/390/768. Seven additional sheets/21 views directly inspected: translated child clipping, reversed outer right/bottom clipping and ordinary sibling placement match Chrome. Composition visual total45/192,147 remaining; receipt hash audit passes. Initial vector72993 and full native26540 confirmed live by handle polls. No qualification claim or tolerance change.

L17 initial vector replay72993 terminal0:576/576 geometry and pixel comparisons pass. Composition visual review reaches66/192 after all nested-positioned cases across directions/box models are directly inspected at all widths; descendant escape, sibling overlap, right viewport edge and lower host overflow match Chrome. Review receipt audit passes;126 composition views remain. Full native26540 confirmed live. Nine composition vector text failures remain unwaived; initial visual qualification is still pending.

L17 two-positioned-sibling visual coverage complete across all directions and box models:21 more views directly inspected at240/390/768. Positioned green sibling overlap, subtree coverage, reverse-row ordinary sibling overlap and column-reverse overflow match Chrome. Composition review87/192 with105 remaining, receipt audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain open.

L17 flex-order composition review complete across four directions and both box models:21 additional views directly inspected. Whole-subtree placement, purple sibling overlap, narrow viewport clipping and reverse-column order match Chrome. Composition visual audit108/192,84 remaining; hash audit passes. Full native26540 confirmed live by handle poll. Vector text residuals remain open.

L17 ordinary-sibling overlap composition review complete across directions/box models:21 additional views directly inspected at240/390/768. Positioned subtree coverage, unchanged sibling footprint, reverse-row clipping and column-reverse overflow match Chrome. Composition visual audit129/192,63 remaining for text/images/wrap-ratio; receipt hash audit passes. Full native26540 confirmed live. Nine vector text failures remain open.

L17 image composition visual coverage complete across all directions/box models:21 additional views directly inspected at240/390/768. Image quadrants, percentage offsets, translated parent and sibling placement match Chrome. Thin horizontal image raster differences in border-box and sparse right-edge differences in content-box are explicitly recorded within unchanged gates. Composition review150/192,42 remaining for text/wrap-ratio; receipt audit passes. Full native26540 confirmed live; vector text failures remain open.

L17 native text composition review complete across all directions/box models:21 more views directly inspected. Two-line wrapping, auto card height, nested text offset and sibling placement match Chrome. Small glyph coverage differences near the end of weekend remain documented within unchanged native gates. Composition visual audit171/192,21 wrap-ratio views remaining; hash audit passes. Full native26540 confirmed live. Nine separate vector text failures remain open.

L17 composition native visual audit complete192/192. Final21 wrap-ratio views directly inspected: ratio sizing, narrow wrapping, percentage relative translation, auto host size and viewport clipping agree with Chrome. All192 public v17 composition comparisons receive audited review transfer with exact canonical compiler inputs and full Chrome/native PNG identity. Public discriminator24 also reviewed. Initial576 visual audit, vector text failures/visual evidence, remaining interactions and full native26540 regression still pending;26540 confirmed live. Receipt:output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 composition vector failures directly reviewed:9 failing border-box text views plus3 passing column-reverse controls. Glyph coverage differs while wrapping and box geometry agree; passing controls also retain visible glyph differences, and no root cause is claimed. Another168 non-text views transfer from fully audited diagnostic native source with exact canonical input and full PNG identity. Vector visual total180/192;12 content-box text controls remain. Nine failures stay unwaived. Receipt:positioned-v17-composition-vector/failure-visual-inspection.json. Full native26540 confirmed live.

L17 composition vector visual audit complete192/192:24 directly inspected text views plus168 exact canonical-input/full-PNG transfers. Final12 content-box text controls retain visible glyph coverage differences despite passing unchanged numeric thresholds; wrapping, card heights and offsets match Chrome. Rechecked all192 unique receipt entries and full image/sheet/transfer hashes, with independent source review audit. Nine original pixel failures remain unwaived. Full native26540 confirmed live; initial corpus visual review and remaining interaction audit still pending.

L17 initial native visual audit begins with12 directly inspected row baseline views across definite/intrinsic host heights and both box models. Box dimensions, child insets, sibling gaps and viewport clipping match Chrome. Exact full-image controls bring audited coverage to36/576;540 remain. Full native26540 confirmed live. Nine vector text failures remain open.

L17 initial reverse-row baseline review adds12 direct views across both box models and host-height modes. Right anchoring, left-edge overflow, child inset and host bottom agree with Chrome. Exact full-PNG controls bring initial visual coverage to72/576;504 remain. Receipt audit passes; full native26540 confirmed live.

L17 column baseline review adds12 direct views covering definite/intrinsic host height and both box models. Vertical order/gaps, teal inset, box dimensions and host bottom match Chrome. Exact-image controls bring initial native visual audit to108/576;468 remain. Receipt audit passes; full native26540 confirmed live.

L17 initial baseline controls complete across four directions, both box models and definite/intrinsic heights. Final12 reversed-column views directly inspected: bottom anchoring, leading free space, gaps and intrinsic host size match Chrome. Exact-image baseline/auto/static-offset controls bring audited coverage to144/576;432 offset views remain. Receipt audit passes. Full native26540 confirmed live.

L17 percentage-offset visual review adds24 views across row/reverse-row, both box models and definite/intrinsic container heights. Responsive translation, unchanged sibling flow, overlap and viewport clipping match Chrome at240/390/768. Initial native visual coverage168/576,408 remaining; receipt hash audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain unwaived.

L17 positive percentage-offset visual coverage complete across four directions, both box models and definite/intrinsic heights. Another24 column/reverse-column views directly inspected: overlap, sibling flow, horizontal translation and visible bottom overflow match Chrome at240/390/768. Initial native review192/576,384 remaining; receipt audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain unwaived.

L17 frozen full native regression completes5983/5983 checks with exit0. All5973 Rust Metal scene pairs have exact HTML/CSS and full Chrome/native PNG identity with the reviewed negative-margin baseline; comparison audit leaves0 unreviewed. Harness and all four frozen toolchain hashes verified. Completion receipt: output/playwright/html-to-riv/positioned-v17-full/receipt.json. Initial right/bottom percentage row review adds12 direct views across box/height modes; left/top viewport clipping and unchanged sibling flow match Chrome. Initial visual coverage204/576,372 remaining. Nine composition vector text failures and remaining qualification audit stay open.

L17 right/bottom percentage reverse-row audit adds12 directly inspected views across both box models and host-height modes. Increasing overlap over green, top viewport clipping, unchanged sibling flow and host size match Chrome. Initial visual216/576,360 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures remain open.

L17 right/bottom percentage column review adds12 direct views across box models and host heights. Top/left viewport clipping, disappearing teal at768, sibling gaps and host sizing match Chrome. Initial visual228/576,348 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 right/bottom percentage-offset coverage complete across four directions, both box models and height modes. Final12 reversed-column views inspected: upward overlap over green, left clipping, sibling flow and host size match Chrome. Initial visual240/576,336 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset row review adds12 directly inspected views across both box models and height modes. Correct right/down translation without stretching, green overlap, sibling flow and host sizing match Chrome. Initial visual252/576,324 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset reverse-row review adds12 direct views across both box models and height modes. Right/down translation without stretching, right viewport gap, narrow purple left clipping and sibling flow match Chrome. Initial visual264/576,312 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset column review adds12 direct views across box models and height modes. Translation without stretching, thin green overlap, fixed sibling flow and container sizing match Chrome. Initial visual276/576,300 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 opposing-inset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: translated card retains bottom gap and dimensions; sibling flow and leading space match Chrome. Initial visual288/576,288 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem row visual review adds12 direct views across both box models and host-height modes. Constant right/up translation across widths, green overlap, top gap, sibling flow and host sizing match Chrome. Initial visual300/576,276 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem reverse-row review adds12 direct views across box models and host heights. Right-edge placement, top gap, teal inset, sibling flow and narrow purple clipping match Chrome. Initial visual312/576,264 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem column review adds12 direct views across box models and host heights. Constant right/up translation, top gap, increased separation above green, sibling flow and host sizing match Chrome. Initial visual324/576,252 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: upward green overlap, space below card, sibling flow and host size match Chrome. Initial visual336/576,240 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset row review adds12 direct views across box models and height modes. Translation, narrow green overlap, unchanged dimensions, sibling flow and host size match Chrome. Initial visual348/576,228 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset reverse-row and column review adds24 views across both box models and height modes. Reverse-row right edge stays flush; column translation leaves the expected small gap above green. Card dimensions, nested teal inset, sibling flow, clipping and host bottoms match Chrome. Initial visual372/576,204 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: translated orange/teal dimensions, gap below card, separation from green, leading space and host bottoms match Chrome. Initial visual384/576,192 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 left-offset row visual review adds12 views across both box models and height modes. Rightward orange/teal translation, green overlap, unchanged vertical placement and sibling flow, narrow purple clipping and host sizing match Chrome. Initial visual396/576,180 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset reverse-row visual review adds12 views across both box models and height modes. Narrow right margin, increased separation from green, unchanged vertical placement, reverse-row flow, purple left clipping and host bottoms match Chrome. Initial visual408/576,168 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset column visual review adds12 views across both box models and height modes. Horizontal translation, preserved top padding and vertical gaps, aligned sibling flow, nested teal inset and host bottoms match Chrome. Initial visual420/576,156 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset visual coverage complete across all four directions, both box models and height modes. Final12 reverse-column views inspected: bottom gap, green separation, leading space, sibling alignment and card/host dimensions match Chrome. Initial visual432/576,144 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset row visual review adds12 views across both box models and height modes. Leftward translation, reduced left margin, larger gap before green, unchanged sibling flow and narrow purple clipping match Chrome. Initial visual444/576,132 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset reverse-row visual review adds12 views across both box models and height modes. Leftward orange/teal translation, narrow green overlap, increased right margin, unchanged sibling flow and purple left clipping match Chrome. Initial visual456/576,120 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset column visual review adds12 views across both box models and height modes. Reduced left margin, preserved vertical gaps and top padding, sibling alignment, nested padding and card/host dimensions match Chrome. Initial visual468/576,108 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset visual coverage complete across all four directions, both box models and height modes. Final12 reverse-column views inspected: left margin, bottom padding, separation from green, leading space, nested padding and card/host dimensions match Chrome. Initial visual480/576,96 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset row visual review adds12 views across both box models and height modes. Upward orange/teal translation, reduced top gap, unchanged sibling positions and horizontal gaps, narrow purple clipping and host dimensions match Chrome. Initial visual492/576,84 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset reverse-row visual review adds12 views across both box models and height modes. Upward translation, preserved right anchoring and horizontal gaps, reduced top gap, narrow purple left clipping and host dimensions match Chrome. Initial visual504/576,72 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset column visual review adds12 views across both box models and height modes. Reduced top gap, increased separation above green, preserved left alignment, nested padding and sibling/card/host dimensions match Chrome. Initial visual516/576,60 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: narrow green overlap, increased space below card, left alignment, nested padding and sibling/card/host dimensions match Chrome. Initial visual528/576,48 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset row visual review adds12 views across both box models and height modes. Upward translation, reduced top space, preserved horizontal gaps, sibling placement, narrow purple clipping and card/host dimensions match Chrome. Initial visual540/576,36 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset reverse-row visual review adds12 views across both box models and height modes. Upward translation, preserved right anchoring and horizontal gaps, reduced top gap, purple left clipping and card/host dimensions match Chrome. Initial visual552/576,24 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset column visual review adds12 views across both box models and height modes. Upward translation, reduced top gap, increased separation above green, preserved left alignment and sibling/card/host dimensions match Chrome. Initial visual564/576,12 remaining; receipt audit passes. Nine vector text failures remain open.

L17 initial visual audit complete:576/576 native views reviewed,0 remaining. Final12 reverse-column bottom-offset views match Chrome in translation, narrow gap below green, bottom space, nested inset and host sizing. All576 initial vector views transfer with identical canonical compiler inputs and full Chrome/native PNG bytes; independent transfer audit passes. Composition native192/192 and full native5983/5983 already pass with complete visual evidence. Nine composition vector text pixel failures remain unwaived; remaining interaction qualification audit stays open. Evidence: output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 interaction corpus adds32 scenes across four directions/two box models and auto margins, stretch, negative margins and percentage padding. Pinned Chrome153.0.8010.12 captures96 views. Frozen v17 native replay passes96/96 geometry and pixels (exit0); public original/clone resize regression passes (exit0). Permanent oracle and expanded native/WASM parity corpus added; parity50657 remains running. Visual review96 views and vector replay remain pending. No tolerance changes or qualification claim.

L17 interaction vector replay passes96/96 geometry and pixels (exit0). First12 native views directly inspected: row/border-box auto margins, stretch, negative margins and percentage padding match Chrome at all widths, including bottom overflow and sibling overlap. Visual audit12/96,84 remaining. Expanded parity50657 completed with exit0; nine prior composition vector text failures stay open.

L17 row/content-box interaction review adds12 direct views at240/390/768: auto-margin free-space placement, stretched height, bottom overflow, negative-margin overlap and growing percentage padding match Chrome. Visual audit24/96,72 remaining. Native/vector96/96 and expanded parity11/11 pass; nine earlier composition vector text failures remain unwaived.

L17 reverse-row/border-box interaction review adds12 direct views: right viewport truncation, auto-margin placement, stretched height/bottom overflow, negative-margin sibling gaps and percentage-padding inset match Chrome at all widths. Visual audit36/96,60 remaining. Native/vector96/96 and parity11/11 pass; nine prior composition vector text failures remain open.

L17 reverse-row/content-box interaction review adds12 views at all widths: outer card sizing, teal inset, right truncation, auto-margin placement, stretched height/bottom overflow, negative-margin gaps and percentage padding match Chrome. Visual audit48/96,48 column/reverse-column views remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 column/border-box interaction review adds12 views: auto-margin right placement, viewport truncation, horizontal stretch, negative-margin overlap, percentage-padding inset and sibling vertical flow match Chrome at all widths. Visual audit60/96,36 remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 column/content-box interaction review adds12 views: outer dimensions, auto margins, horizontal stretch, overlap, negative margins and percentage padding match Chrome. The768 percentage case preserves purple overflow below the host. Visual audit72/96,24 reverse-column views remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 reverse-column/border-box interaction review adds12 views: auto margins, right truncation, horizontal stretch, bottom overflow, negative-margin gaps and percentage-padding bottom placement match Chrome. Visual audit84/96,12 remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 interaction visual audit complete96/96. Final12 reverse-column/content-box views match Chrome, including growing percentage padding, bottom overflow and purple top-viewport clipping at768. All96 vector views transfer with exact canonical compiler inputs and full Chrome/native PNG identity; transfer audit passes. Public original/clone resize test and expanded native/WASM parity11/11 pass. Nine composition vector text failures remain unwaived; final feature qualification audit remains.

L17 final focused evidence audit verifies19 aggregate file hashes,888 passing native comparisons with complete image audits,672 initial/interaction vector comparisons with complete audits, and192 composition vector geometry comparisons with183 pixel passes/nine unwaived failures. Existing full native5983-check receipt remains separate from focused coverage. L17 stays partial for vector text; no external blocker is claimed. Next independent implementation item is L18 absolute positioning.

L18 initial runtime diagnostic:28 scenes/84 pinned Chrome views captured. Existing absolute wire passes geometry for24 direct-child scenes (auto/start/end/percent/opposing/auto-margin across four directions), but four static-ancestor scenes produce128 coordinate mismatches across original/clone and240/390/768/240 resize updates. At390 the card x is32 instead of39; at768 it remains32 instead of76.796875. This isolates use of the immediate static wrapper as containing block rather than the positioned host. Public syntax remains rejected; the failing regression is retained in tests/absolute_position_runtime.rs. Next: investigate runtime containing-block representation without flattening paint ancestry or baking viewport coordinates. Evidence: output/playwright/html-to-riv/absolute-position-initial-receipt.json. No pixel or visual qualification is claimed.

L18 containing-block expansion adds16 scenes/48 Chrome views. Combined44-scene original/clone diagnostic fails256 coordinates across12 scenes. All-auto nested cases and positioned-wrapper controls pass; nested left-only cases fail only x, nested top-only only y, confirming each auto axis must retain its immediate-parent static position while explicit insets use the nearest positioned ancestor. Existing flexbox absolute solver iterates immediate children and uses the same AlgoConstants for both roles. Merely reparenting solve nodes would lose the now-proven correct auto-axis behavior. Next implementation needs a separate containing-block size/origin input plus retained static-position context, with coordinates converted back to the original parent. Paint/clipping ancestry must stay intact. Evidence: output/playwright/html-to-riv/absolute-position-containing-block-receipt.json. No syntax admission or pixel qualification yet.

L18 engine seam implemented: opt-in css_absolute_containing_block carries padding-box size/origin relative to the immediate parent. Explicit insets and size resolution use that context; auto-axis alignment retains original parent constants; explicit-axis coordinates convert back to the original parent. Default None preserves existing behavior. New percentage-x/auto-y regression fails before implementation and passes after, including context resize and clearing; all115 engine tests pass. Style memory-size assertions reflect the added field. Runtime population is not yet installed, so the44-scene public-import diagnostic remains unresolved and no CSS admission or pixel qualification is claimed. Logs: output/playwright/html-to-riv/absolute-containing-block-engine-red.log and absolute-containing-block-engine-green.log. Next: live containing-block ancestry/measurement installation and expanded end-inset, size and auto-axis regressions.

L18 engine end-inset/percentage-size/opposing-inset tests now cover all four flex directions. A new nonzero-inset auto-margin regression first fails x140 vs125 in parent-local coordinates. Opt-in containing-block free space now subtracts opposing insets before distributing auto margins; all116 engine tests pass. Added four Chrome nested nonzero-inset auto-margin scenes (containing corpus20 scenes/60 views, total48 scenes). At390 all four directions produce card{x:145,y:93,width:90,height:70}, confirming the solver expectation after subtracting its test parent origin. Permanent diagnostic oracle expanded. Runtime context installation and native pixel qualification remain pending. Logs: output/playwright/html-to-riv/absolute-containing-block-end-insets-{red,green}.log; browser evidence:absolute-position-containing-block-expanded-oracle/oracle.json.

L18 runtime containing-block integration passes all48 diagnostic scenes across384 original/clone resize updates (240/390/768/240, exit0). Opt-in authored static/positioned markers are cloned and locate the nearest containing ancestor; bounded layout passes update live padding-box sizes/origins without changing parentage. Initial integration retained72 resize-coordinate failures because unchanged static parents skipped updated descendants. Comparing new solved geometry and propagating descendant changes through those parents resolves the remaining failures. Original failing log preserved. Public absolute admission, capability/installer validation, lifecycle clearing tests, broader nested/auto-size coverage and native pixels remain pending. Evidence: output/playwright/html-to-riv/absolute-position-runtime-context-receipt.json.

L18 lifecycle checks pass: policy clearing/re-enable at three widths, retained-clone independence, invalid style target rejection, and changing the nearest positioned ancestor. Three relative-position original/clone regressions pass. Added12 nested absolute scenes with percentage size/insets, end insets and opposing-inset auto sizing across four directions. Combined60 scenes/180 Chrome references pass480 original/clone resize updates plus lifecycle checks. No tolerance changes. Public CSS admission, checked host contract and native pixel evidence remain pending. Evidence: output/playwright/html-to-riv/absolute-position-runtime-nested-receipt.json.

L18 draft version18 transport added with layout-css-absolute-position-v1 and layout_absolute targets. Validation requires nonempty unique absolute targets contained within valid positioned targets, the matching capability and version18; old hosts reject missing capability. Two roundtrip/malformed contract tests and all five relative-position compiler tests pass. TypeScript capability/version union and type checks pass. Computed positioning now uses an explicit Static/Relative mode instead of a boolean; absolute admission remains intentionally gated until the host installer is connected. The compiler does not emit version18 yet. Runtime wire/mode verification, JavaScript host integration, rebuilt WASM parity and pixel gates remain. Evidence: output/playwright/html-to-riv/absolute-position-contract.log and absolute-position-contract-types.log.

L18 checked runtime installer implemented as Artboard::set_css_absolute_position_policy_occurrence. It validates unique non-root layout targets, absolute subset and exact agreement with imported absolute position wires before changing containing-block or paint policies. Initial mutable-artboard version exposed a RefCell borrow conflict; occurrence-based validation releases the artboard borrow before callbacks and resolves it. All60-scene/480-update geometry checks and existing lifecycle checks pass with installer use; six malformed-list cases per scene reject without disturbing the valid installed policy. Probe and public oracle host now install the v18 policy before layout/draw; probe compile check pending. Compiler emission and pixel qualification remain pending. Evidence: output/playwright/html-to-riv/absolute-position-installer-occurrence.log (exit0); initial failure retained in absolute-position-installer.log.

L18 candidate public syntax now accepts position:absolute with existing physical insets (auto, signed px/em/rem/percent, zero, shorthand, CSS-wide values and variables). Version18 and layout_absolute are emitted with positioned-paint requirements after descendant compilation. Public output passes all60 Chrome scenes across480 original/clone resize updates. Import-time validation initially rejected cached solver styles; checking the imported position wire fixes it (initial failure preserved). Three absolute contract/cascade tests and five relative tests pass. Public host installation is connected. Fixed/sticky, logical insets and general inset math remain rejected. Rebuilt WASM/native tools and expanded parity corpus are in progress; pixel/visual qualification and broader compositions remain pending.

L18 public validation update: corrected native-glyph-controls build completes; the earlier misconfigured probe and failed replay logs remain preserved. Frozen absolute-v18-native-toolchain passes all180 focused native geometry/pixel comparisons (84 initial,96 nested) against pinned Chrome. Expanded native/WASM parity passes11/11. Checked-installer whole-policy clear/reinstall now passes repeated390/768/240 resizing with retained-clone independence. Direct review currently covers9 nested views; remaining visual review, vector qualification, broader compositions/interactions and full regression remain pending. No tolerances changed. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 vector replay also passes180/180 geometry/pixel comparisons using the same frozen toolchain with nativeGlyphs=0. This is numeric evidence; vector visual qualification is still pending. Receipt: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 composition expansion:64 scenes/192 pinned Chrome views now cover overlap, positioned siblings/descendants, clipping, text, images and wrapping/aspect-ratio combinations across four directions and two box-sizing modes. New public original/clone test retains64 failing coordinates in eight text scenes: auto-width absolute text measures90 vs87 (border-box) or106 vs103 (content-box), exactly the3px left inset. Frozen pre-fix native replay has24 geometry failures and0 pixel failures, illustrating why geometry is an independent gate. Existing full compiler regression passes323 tests across60 groups before this new failing test. Nested direct visual review advances to33/96. Solver candidate now subtracts horizontal insets/margins from available auto-width measurement under the CSS policy; verification and rebuilt pixel replay pending. Failure logs preserved in absolute-v18-composition-{geometry,native}.log; aggregate receipt absolute-v18-public-receipt.json.

L18 inset-width fix verified: both public absolute oracle tests pass (124 scenes,992 original/clone resize updates), and all116 engine tests pass. The64 composition coordinate failures are resolved. Frozen pixel evidence remains pre-fix; rebuild/parity/pixel reruns and full visual qualification remain required. Logs: absolute-v18-composition-inset-width.log and absolute-v18-inset-width-engine.log.

L18 rebuilt inset-width toolchain passes372/372 native geometry/pixel comparisons. Vector geometry passes372/372, pixels354/372:18 text comparisons fail local RGB error across six composition scenes; no waiver. Direct inspection of row/border-box vector text at all3 widths confirms glyph coverage differences with matching wrapping and box edges. Corresponding native text sheets pass and are reviewed. Expanded native/WASM parity including64 composition scenes passes11/11. Pre-fix nested native review45/96; post-fix composition native6/192. Full visual audits and cross-run transfer remain pending. Frozen manifests, initial failure evidence and new results are recorded in output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 post-fix full compiler regression passes324 tests across60 groups with0 failures/ignored tests. Log: output/playwright/html-to-riv/absolute-v18-inset-width-full-regression.log. Visual qualification and18 vector text pixel failures remain open.

L18 deterministic interaction expansion adds48 scenes/144 Chrome views: auto margins, stretch, negative margins, percentage padding, min/max opposing-inset sizing and flex factors, four directions and two box-sizing modes. Initial public geometry has256 coordinate failures in eight auto-margin scenes; pre-fix pixel replay preserves24 failing views. With either opposing inset auto, CSS auto margins must resolve to zero rather than consume free space. Opt-in runtime correction now passes all172 scenes/1376 original-clone resize updates; extended engine start/end/mixed-axis margin checks and all116 engine tests pass. Rebuilt pixels, expanded parity and post-fix regression pending. Native nested visual review57/96. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 auto-inset margin toolchain is frozen after successful featured build. Native/vector144-view interaction replays running. Initial expanded parity run failed because Cargo temporarily replaced the publisher during rebuild (ENOENT), not artifact disagreement. Tests now accept NUXIE_NATIVE_COMPILER to pin a frozen publisher; frozen retry running. Initial failure remains in absolute-v18-interaction-parity.log.

L18 corrected auto-inset margin interaction replays pass144/144 native and144/144 vector geometry/pixel comparisons. Direct visual review, frozen parity result and regression of prior corpora against the latest toolchain remain pending.

L18 latest runtime regression passes325 tests across60 groups with0 failures/ignored. All516 native focused comparisons (initial84,nested96,composition192,interaction144) pass with the auto-inset-margin toolchain. Native nested direct review reaches72/96, with24 views remaining. Frozen publisher parity exposed a second harness assumption: probe path was derived as examples/probe; NUXIE_NATIVE_PROBE now pins that executable explicitly. Both-tool frozen retry and latest vector replays running; prior failures preserved. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 nested visual group complete96/96 with independent audit. Exact canonical compiler inputs and full Chrome/native PNG pairs transfer all96 views to latest native and vector replays; both transfer audits pass. Latest full focused vector geometry passes516/516;18 composition text pixel failures remain unwaived. Frozen publisher+probe parity now passes11/11. Initial/composition/interaction visual review and final qualification remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 public host-negative test passes: two missing capabilities, four malformed absolute target lists, and two schema-valid lists inconsistent with imported absolute wires reject; restoring valid requirements succeeds. Probe adds diagnostic NUXIE_DISABLE_CSS_ABSOLUTE_POSITION. Initial native direct visual review9/84. Full baseline native Playwright suite launched with frozen auto-inset-margin toolchain and isolated absolute-v18-full-native outputs; no result claimed yet. Host log: output/playwright/html-to-riv/absolute-v18-host-negative.log.

L18 initial visual audit progresses to42/84 views (33 direct,9 exact image-pair transfers),42 remaining. Newly reviewed row percentage/opposing/auto-margin/static-ancestor and reverse-row auto/start/end/percentage cases match Chrome across all widths. Receipt audit passes. Full native baseline remains live under session9759; partial progress is not a completion claim. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 initial visual qualification completes84/84 (75 direct views plus9 exact image-pair transfers); independent audit passes. All84 views transfer to the latest vector replay with matching canonical compiler input and full Chrome/native PNG hashes; transfer audit passes. Together with the completed nested96 views, initial/nested visual coverage is180/180 in both profiles. Composition and interaction visual review,18 unwaived vector text pixel failures, and the live full native baseline remain pending. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition visual review reaches28/192 (24 direct plus4 exact image-pair transfers), audit passed. Eight column/border-box scenes cover positioned siblings, nested overflow, clipping, image placement, paint order, overlap, two-line text and responsive aspect-ratio growth at three widths. No geometry/paint-order mismatch observed; sparse image/text edge differences remain within existing gates. Remaining composition/interaction review and18 unwaived vector text failures stay open. Receipt: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition visual audit reaches56/192 (48 direct,8 exact image-pair transfers). All eight column/content-box scenes inspected across240/390/768: sizing, clipping, overflow, sibling paint order, text wrapping and responsive ratio agree with Chrome. Image case has visible internal-edge diff outlines despite passing existing gates (240px:256 mismatched pixels, max geometry error0.006251px); retained as an asset-rendering observation, not pixel identity. Full baseline session9759 confirmed live; no completion claim.18 vector text failures remain unwaived. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches68/192 (60 direct,8 exact image-pair transfers), audit passed. Reverse-column border-box overlap, both-positioned siblings, nested positioned descendants and clipping match Chrome at all three widths.124 composition views remain, alongside interaction review and18 unwaived vector text pixel failures. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches79/192 (72 direct,7 exact image-pair transfers), audit passed. Reverse-column border-box order/text/image/wrap-ratio sheets inspected at all widths. Text baselines and wrapping, image crop, growing ratio coverage and bottom overflow agree with Chrome; glyph/image edge differences are retained, not claimed pixel-identical. Remaining composition views:113; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches91/192 (84 direct,7 exact image-pair transfers), audit passed. Four reverse-column content-box overlap/positioned/nested/clip cases inspected at all widths: enlarged box dimensions, static sibling locations, ancestor-relative teal placement and clipping edges agree with Chrome. Remaining composition views:101; interaction and18 vector text failures remain open. Full native session9759 confirmed live with progress through test2965. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches102/192 (96 direct,6 exact image-pair transfers), audit passed. All forward/reverse column compositions now directly inspected across both box-sizing modes and three widths. Final reverse-column content-box text/image/order/ratio cases preserve matching layout, wrapping, crop and overflow. Image internal-edge and glyph-edge differences remain recorded; no pixel identity claimed.90 row/reverse-row views remain. Full native session9759 confirmed live; interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches114/192 (108 direct,6 exact image-pair transfers), audit passed. Row border-box overlap/nested/clip/order scenes inspected at all widths. Orange sibling overlap, teal overflow/clip boundaries and purple viewport cropping agree with Chrome. Remaining composition views:78; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches123/192 (117 direct,6 exact image-pair transfers), audit passed. Row border-box text/image/ratio sheets reviewed at all three widths, completing that group. Two-line wrapping and top clipping, image quadrant placement, responsive sibling coverage and bottom overflow agree with Chrome; sparse glyph/image differences remain recorded. Remaining composition views:69; interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches135/192 (129 direct,6 exact image-pair transfers), audit passed. Row content-box overlap/nested/clip/order inspected at all widths; larger orange bounds, exposed purple strips, teal overflow/clip and narrow viewport cropping agree with Chrome. Remaining composition views:57; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches144/192 (138 direct,6 exact image-pair transfers), audit passed. Forward-row content-box text/image/ratio reviewed at all widths, completing forward-row coverage. Wrapping, crop, image geometry, responsive sibling coverage and overflow agree with Chrome; visible image/glyph edge differences retained.48 reverse-row views remain; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches156/192 (150 direct,6 exact image-pair transfers), audit passed. Reverse-row border-box overlap/both-positioned/nested/clip inspected across all widths: right-tracking static siblings, host-relative teal placement, changing overlap and clip edges agree with Chrome. Remaining composition views:36; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition review reaches168/192 and audit passes, completing reverse-row border-box coverage.24 reverse-row content-box views remain. Saving review initially failed ENOSPC; full native session9759 then exited1 with ENOSPC creating Playwright worker artifacts. This is an incomplete infrastructure-failed run, not a qualification pass. Removed13 older rebuildable runtime rlib archives (6.82GB), preserving latest two and all rendered evidence; review save/audit now pass. Full baseline needs fresh replay. Interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition review reaches180/192 and audit passes. Reverse-row content-box overlap/both-positioned/nested/clip agree with Chrome at all widths;12 final views remain. Disk recheck shows53GiB available. Fresh full native baseline launched in absolute-v18-full-native-retry (session43262) with frozen toolchain; failed ENOSPC run preserved separately. Interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition visual review completes192/192 (186 direct,6 exact image-pair transfers); independent audit passes. Final reverse-row content-box order/text/image/ratio cases agree with Chrome for wrapping, clipping, changing overlap and responsive overflow at240/390/768. Image/glyph edge differences remain explicitly recorded, not pixel identity. Initial+nested+composition native visual coverage now372/516; interaction144 remains. Full baseline retry session43262 confirmed live. Composition vector review and18 unwaived text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native interaction visual review reaches36/144, all direct and independently audited. Forward-column border-box and content-box cases cover auto margins, flex factors, min/max constraints, negative margins, percentage padding and stretch at240/390/768. Visible bounds, overlap order, sibling exposure and responsive growth match Chrome; no colored discrepancy marks observed. Native focused visual coverage now408/516;108 interaction views remain. Vector interaction/composition review and18 unwaived vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-interaction-native/visual-inspection.json.

L18 native interaction review reaches72/144, all direct and independently audited. Reverse-column border-box/content-box auto-margin, flex-factor, min/max, negative-margin, percentage-padding and stretch views match Chrome across240/390/768, including changing purple overlap and reversed sibling placement. Native focused visual coverage444/516;72 row-direction interaction views remain. Complete JavaScript suite passes12/12, including expanded native/WASM artifact parity and invalid absolute host contracts (absolute-v18-javascript-full12.log). Vector review and18 unwaived text failures remain open; full native baseline retry remains in progress.

L18 native interaction review reaches108/144, all direct and independently audited. Forward-row border-box/content-box auto margins, flex factors, min/max, negative margins, percentage padding and stretch match Chrome at240/390/768. Checked responsive purple occlusion, thin exposed top/right/bottom strips, growing orange dimensions and teal inset. Native focused visual coverage480/516;36 reverse-row interaction views remain. Full baseline retry session43262 confirmed live; vector review and18 unwaived vector text failures remain open. Receipt: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-interaction-native/visual-inspection.json.

L18 focused native visual qualification completes516/516. Interaction144/144 directly inspected and independently audited; final reverse-row content-box cases match Chrome for overlap, constrained sizing, padding growth and separation at240/390/768. All144 interaction views transfer to vector using exact canonical compiler inputs and full Chrome/native PNG hashes; transfer audit passes. Vector initial/nested/interaction coverage totals324/324. Vector composition192 review remains, including18 unwaived text pixel failures. Full native baseline retry session43262 remains active. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 vector composition failure inspection completes18/18 across six text scenes and three widths. Chrome/native/diff sheets show matching two-line wrapping and box placement but visible glyph coverage differences on Quiet/weekend. Independent receipt audit verifies each original PNG and sheet hash plus exact coverage of every failing replay identity. All18 local RGB failures remain unwaived;174 passing composition views still need visual review or exact-input/full-image transfer. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-composition-vector/failure-visual-inspection.json.

L18 focused visual coverage is complete in both profiles:516/516 each. Vector composition192/192 combines24 direct text views with168 transfers requiring identical canonical compiler inputs and full Chrome/native PNG pairs from audited native review. Independent audit verifies unique full coverage, source receipt/replay hashes, target PNG/sheet hashes and failure identity. Numeric result stays174/192 composition passes and18 unwaived text failures (overall vector498/516). Six passing text views also show visible glyph differences; recorded without pixel-identity claims. Full native baseline retry remains active. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-composition-vector/composition-visual-coverage.json.

L19 z-index investigation started with48 Chrome reference scenes/144 views (153.0.8010.12). Reference only: compiler/runtime support remains pending; no support or qualification claim. Auto versus integer zero requires distinct context scope, including static flex items. See validation/stacking-context-investigation.md.

L19 pure stacking scheduler implemented;13 targeted runtime tests pass (six new context tests plus seven existing auto-position tests). Public z-index syntax and occurrence metadata are still pending, so this does not qualify support. See validation/stacking-context-investigation.md and output/playwright/html-to-riv/stacking-v19-planner-tests.log.

L19 public compiler implementation now accepts `z-index: auto` and signed integers, including explicit zero, CSS-wide resets/inheritance and custom-property substitution. Integer values create atomic stacking contexts for supported layout boxes, including static flex items; they do not establish positioning containing blocks. Fractional, dimensional, multi-value and math expressions are rejected. Requirements version19 adds `layout-css-stacking-v1` and unique non-root `{object_id, level}` targets in `layout_stacking`; positioned/absolute metadata is optional when only static flex contexts occur. Hosts must validate and install the stacking policy. Native probe installation and JavaScript declarations are connected. Qualification remains pending: five focused compiler/recorded-runtime tests pass, but native/WASM parity, explicit containing-block invariance, stacking clip integration, resize geometry and actual Chrome/native pixel comparisons remain open. This implementation is provisional, not a visually qualified support claim.

L19 first frozen public native replay completes144 views:144 geometry pass,132 pixel pass,12 unwaived paint failures in reverse directions (equal-level CSS order and column-reverse negative-descendant/equal-sibling cases). Row-reverse equal-order three-width sheet directly inspected: Chrome teal covers green overlap, native green covers teal. Other nine failing views remain uninspected. Existing css_ordered_drawables reverses child traversal for reverse flex directions; its interaction with context scheduling needs investigation before changing baseline behavior. Native/WASM toolchain frozen in stacking-v19-toolchain; expanded JavaScript parity running. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 tie-order fix passes6/6 focused tests, including the previously failing reverse-direction regression with original/clone resize. Pure scheduler tests running session26453; rebuild/freeze native probe and replay all144 Chrome views next. No post-fix pixel success claimed yet.

L19 corrected frozen native and vector replays each pass144/144 geometry and pixel comparisons. All12 previously failing views directly re-inspected; green/orange and teal/green overlaps now agree with Chrome. Native review audit covers21/144 views (12 direct plus9 exact image-pair transfers);123 native and vector review remain. Pure scheduler14/14 and public48-scene clone/resize/clear/reinstall geometry test1152/1152 updates pass. Initial failure evidence retained. Expanded text/image/clip compositions, host-negative imported-target checks, full regression and remaining visual qualification remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 host-negative integration passes: missing capability, malformed context lists, style/missing targets, fractional/out-of-range levels reject before rendering; restored valid requirements succeeds. Probe diagnostic capability disable added without changing rendering semantics. Native visual audit42/144 (18 direct,24 exact within-run image-pair transfers) includes row auto versus zero at all widths: teal escape and green overlap match Chrome. Full compiler regression running session3670;102 native views, vector transfer/review and composition expansion remain. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition expansion adds32 scenes/96 Chrome views: four directions, text/image auto versus zero contexts, nested clipping, negative parents and deep auto escape/zero containment. Initial text fixture24px line height rejected by existing Inter natural-metric constraint; initial references and failed replay logs retained. Corrected28px fixture recaptured with pinned Chrome153.0.8010.12. Native/vector replay running sessions26631/65233; expanded13-test JavaScript suite running38651. Initial native visual review48/144 (24 direct,24 exact pair transfers) after ancestor-clip/static-flex sheets inspected at all widths. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native/vector each96/96 geometry/pixel pass. Expanded JavaScript suite13/13 passes including32 composition fixtures native/WASM artifacts. Public resize test now80 scenes/1920 original-clone updates across stacking install/clear/reinstall, all Chrome geometry pass. First12 native composition views (row text/image auto/zero) directly inspected and audited: matching two-line text, quadrant placement and context-dependent green occlusion.84 composition native views and vector review/transfer remain, alongside96 initial native views. Full visual completion and broader regression gate remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review24/96 complete and audited after all forward-row cases inspected. Nested clips hide deep rose, negative parent descendants stay behind host, and deep auto versus zero yields expected rose/green overlap. Added80 stacking fixtures to the regular browser regression gate. Frozen full native suite launched6223 tests (session41911), isolated stacking-v19-full-native outputs; no terminal result claimed. Initial/native composition remaining visual views96/72; vector transfers/review pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review48/96 completed and audited, covering all forward/reverse row cases at three widths. Reverse text/image right viewport crop, nested clipped rose/teal and hidden negative-parent subtree match Chrome. Reverse auto/zero variants have no green overlap, so their review proves placement/crop rather than context distinction (covered by forward-row overlaps).48 column/reverse-column composition views remain. Full baseline41911 confirmed live this turn; no terminal claim. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native visual audit72/96 complete, adding all forward-column text/image/clip/negative/deep scenes. Chrome/native match text wrapping, image placement, green/purple overlap, nested clipping and negative-parent hiding. Column auto/zero pairs do not expose sibling overlap, so their visual evidence is limited to placement/visibility. Final24 reverse-column views remain. Full native baseline41911 confirmed live. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition visual review96/96 complete and audited. Final reverse-column views match Chrome; negative-parent rose overflow correctly remains visible as strip below host background. Exact canonical inputs and full browser/native image pairs transfer72/96 vector views, audit passes;24 vector text views differ and need direct inspection (numeric gates already pass). Initial48/144 native review and full baseline still pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition visual coverage is now complete in both profiles: native96 direct; vector24 direct text views plus72 audited canonical-input/full-image transfers. Independent disjoint-set audit covers all96 vector cases (stacking-v19-composition-line-height-vector/visual-coverage.json). Column text wraps and baselines match Chrome with small visible glyph-edge differences within unchanged thresholds. Auto/zero column pairs establish placement, not overlapping context discrimination. Initial native96 views and initial vector review remain; full6223 native gate confirmed live through session41911. No qualification claim yet.

L19 initial native visual audit now covers78/144 views (45 direct,33 exact pair transfers);66 remain. New forward-row reviews confirm negative descendant hiding, CSS order placement and absolute sibling levels. Reverse-row negative auto hides teal while zero exposes teal above parent; positive auto/zero have no sibling overlap. All inspected crops and overlaps match Chrome. Initial vector audit remains pending; composition96/profile coverage is complete. Full baseline41911 confirmed live this turn.

L19 initial native review now93/144 (60 direct,33 exact image transfers), audit passed. All forward/reverse row views covered; remaining51 column views. Reverse-row static flex context preserves root-relative absolute placement; ancestor clipping crops teal at parent boundaries; absolute sibling overlaps and negative sibling phases match Chrome at all three widths. Initial vector review, lifecycle expansion and full baseline completion remain. Session41911 confirmed live.

L19 initial native audit covers111/144 views (72 direct,39 exact transfers);33 remain. Forward-column negative-auto hides teal while negative-zero paints it above orange; positive auto/zero lack sibling overlap and prove placement only. All widths visually agree with Chrome. Initial vector review and lifecycle expansion remain; full6223 baseline session41911 confirmed live.

L19 initial native visual audit now126/144 (87 direct,39 exact transfers). All forward-column views covered;18 reverse-column views remain. CSS-order and negative sibling overlap, static context placement and ancestor clipping match Chrome at all widths. Initial vector transfer must await completed native review. Full baseline41911 confirmed live; lifecycle expansion remains.

L19 focused visual coverage complete: initial native144/144 (105 direct,39 exact within-run image transfers); vector144/144 canonical-input and full browser/runtime PNG transfers audited. Composition96/profile already complete, for240/profile total. Final reverse-column inspection matches Chrome including orange/green overlap, root-relative static-context child placement, ancestor clipping and teal overflow beyond host bottom. Full6223 baseline remains live session41911; dynamic clip lifecycle, expanded overlap interactions and full regression audit remain before qualification.

L19 dynamic clip lifecycle regression added in tests/positioned_paint_runtime.rs. Four existing tests pass; new regression fails clip-depth assertion after toggling clipping, even after correcting initial expected outer artboard clip. Reproducer: stacking-v19-live-clips-reproducer.log. Matrix includes original/clone, four directions and auto/zero/negative parent contexts but stops on first failure; do not claim full matrix execution. Diagnose stale runtime clip plan versus test assumptions next. Focused static visual coverage remains240/profile complete; full baseline still pending.

L19 dynamic clipping diagnosis: cached CSS plan omitted ordinary proxy for initially unpainted/unclipped host. Deferred groups reopened its live clip, but ordinary child background missed it. runtime_tree now supplies proxy for plain layout containers in CSS plans. Focused five-test run active8481 (stacking-v19-live-clips-proxy-fix.log); no passing claim yet. Frozen full baseline41911 remains pre-fix and live; its result cannot qualify this new runtime change. Fresh renderer qualification and regressions required after focused tests.

L19 clip-proxy fix focused regression completed:5/5 pass, including864 dynamic clip/resize draws across four directions, three parent context policies, originals/clones and repeated host/parent clipping transitions. Paint order and clip save/restore balance pass. Original failure retained. Fresh frozen renderer comparisons and broader regression still required; existing full41911 uses pre-fix snapshot.

L19 clip fix qualification: vector composition96/96 passes with clip-toolchain, visual audit pending. Native replay failed preflight because concurrently running cargo test rebuilt the probe without native-glyph-controls before snapshot copy. Keep failed snapshot/log; wait compiler suite21908 terminal, then rebuild featured probe and freeze a new snapshot sequentially. Do not treat native preflight failure as a pixel result. Earlier full baseline41911 remains independent/pre-fix.

L19 clip fix full compiler suite terminal:331 passed,0 failed across62 result groups (stacking-v19-clip-compiler-tests.log). Vector composition72/96 exact-input/full-image transfers audited;24 text views remain. Featured probe rebuild now runs sequentially after suite completion (session48219, stacking-v19-clip-probe-sequential-build.log); freeze new snapshot only after terminal success. Native replays and broader visual regression remain pending.

L19 clipping fix featured snapshot passes all240 focused geometry/pixel comparisons in each profile. Audited exact source/full-image transfers cover native240 and initial vector144. Composition vector72 transfer from native plus24 matching earlier directly reviewed vector text images independently cover96/96 (stacking-v19-clip-featured-composition-vector/visual-coverage.json). Full compiler331 and live lifecycle864 draws pass. Fresh full regression, expanded overlap interactions and dynamic clip pixel comparisons remain; pre-fix full41911 cannot qualify this fix.

L19 overlapping text/image corpus adds16 scenes/48 Chrome153.0.8010.12 views. All48 have visible inner/sibling intersection;24 auto/zero pairs have identical geometry and distinct Chrome pixels (stacking-v19-overlap-oracle/discriminator.json). Public geometry now96 scenes/2304 original-clone lifecycle updates passes. Native/vector replay52624/46455 and expanded parity running; visual review pending. Corrected frozen full6223 native gate launched96889 in stacking-v19-clip-full-native, independently of pre-fix41911.

L19 overlap native visual audit12/48 complete: forward-row text/image auto/zero pairs at all3 widths. Auto exposes full text/quadrants, zero green occludes most content; Chrome/native edges and text wrapping match. Remaining36 native and48 vector views; parity82349 confirmed live. Corrected full96889 and pre-fix41911 confirmed live. Numeric overlap48/profile and2304 public lifecycle updates already pass.

Correction to preceding live note: pre-fix baseline41911 returned terminal success in this turn:6223/6223 passed in28.5m. Full visual audit remains pending, and this tie snapshot does not include the clip lifecycle fix. Corrected full96889 remains running.

L19 overlap native visual audit24/48 complete: all forward/reverse row pairs reviewed. Reverse zero context hides visible text/image except top strip, auto reveals viewport-cropped content; Chrome/native agree.24 column native views and48 vector reviews remain. Expanded accepted native/WASM corpus passes; initial JS run10pass/3host-path ENOENT, targeted retry with explicit frozen NUXIE_NATIVE_PROBE passes3/3. Both logs preserved; all13 tests accounted for without rerunning passing corpus. Corrected full regression remains pending.

L19 overlap native visual review36/48 complete and audited. Forward-column auto shows full text/image above green, zero occludes same regions as Chrome, including thin exposed strips. Final12 reverse-column native views and48 vector reviews remain. Corrected full96889 confirmed live this turn; dynamic clip pixel validation remains open.

L19 overlap native visual review48/48 complete and audited. Reverse-column auto/zero text and image occlusion matches Chrome at all widths. Vector27/48 exact canonical-input/full-image transfers audited;21 text views still need review. Full96889 confirmed live. Dynamic clip pixels and full regression audit remain before qualification.

L19 overlap vector review now36/48 covered:27 audited exact-input/full-image transfers plus9 directly inspected row text views. Wrapping, baseline, viewport crop and green occlusion match Chrome; sparse colored glyph-edge differences remain visible within unchanged thresholds.12 column text views remain. Full corrected regression and dynamic clip pixel gates remain pending.

Stacking overlap audit completed: all 48 native views directly reviewed; vector coverage is 21 directly reviewed views plus 27 audited exact compiler-input/full-PNG transfers, with disjoint complete coverage recorded in `output/playwright/html-to-riv/stacking-v19-overlap-vector/visual-coverage.json`. This brings focused stacking comparisons to 288 per renderer profile, all passing with audited visual coverage. Sparse vector glyph-edge differences remain visible within unchanged gates. Dynamic clip pixel qualification and the corrected full regression remain pending; L19 stays active.

Dynamic stacking clip pixels: added `examples/stacking_clip_probe.rs` and `validation/stacking-clip-lifecycle.mjs`. Twelve public compiler scenes now produce 864 sequential original/clone draw frames, cycling both ancestor clips and widths, including initially unpainted/unclipped hosts. Recording v3 adds required explicit frame boundaries; earlier borrow/recording failures are retained. Chrome/real renderer run `stacking-v19-clip-lifecycle-pixels-v2` is active (session 67980). This is new validation infrastructure, not qualification: terminal comparisons, discriminators, new-request parity and visual inspection remain. The corrected full6223 regression remains independently active (session96889).

Dynamic stacking clipping now passes all864 real renderer/Chrome geometry and pixel comparisons over12 public compiler scenes, original/clone instances, both ancestor clip toggles and repeated widths. All60 distinct pairs were directly inspected in20 contact sheets;804 additional pairs have audited exact full-image transfers. All720 repeated/clone states are pixel-identical, and144 host/parent clip discriminators show changed pixels in both renderers. New-request native/WASM byte/map/requirements parity passes12/12 (initial CLI-only languageVersion argument error preserved). Evidence: `output/playwright/html-to-riv/stacking-v19-clip-lifecycle-pixels-v2/{replay,visual-inspection,lifecycle-audit}.json` and `stacking-v19-clip-lifecycle-parity-v2/receipt.json`. This qualifies the rectangular shape clip lifecycle interaction only; broader L20 rounded/text/image overflow remains separate. The corrected full6223 gate and its visual audit remain pending; L19 stays active.

The completed pre-clip stacking full baseline now has6213/6213 scene pairs audited against complete prior reviews with exact complete compiler-input and full browser/native PNG identity; all6223 numeric checks passed. Reusable audit: `validation/audit-gallery-transfer.py`; evidence: `output/playwright/html-to-riv/stacking-v19-full-native/baseline-comparison.json`. The corrected clip snapshot is still running and must pass its own audit. Separately prepared27 prospective L20 rounded overflow fixtures; no rounded-overflow qualification is claimed.

L20 initial rounded overflow:27 scenes/81 Chrome comparisons pass both renderer profiles;216 original/clone repeated-size geometry checks and27 native/WASM parity cases pass. Native visual audit covers36/81 views (18 direct plus18 identical hidden controls), including rounded shape/image boundaries and text clipped through glyphs. Remaining45 native views and vector audit remain; no full L20 qualification claimed. The first resize test used an absolute-policy installer for a relative-only contract; corrected to the shipping probe relative installer, preserving the failed log. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L19 qualification completed for the documented subset: corrected full6223/6223 checks and6213/6213 visual transfers pass; regular overlap48/48 checks and visual transfers pass. Focused288/profile and dynamic clip864 frames retain complete visual evidence. See `output/playwright/html-to-riv/stacking-v19-qualification.json`. Broader overflow continues under L20; unrelated vector text limitations remain tracked.

L20 initial visual audit complete in both profiles:81/81 comparisons each. Native54 direct+27 exact hidden-control transfers; vector18 direct+9 within-run transfers+54 audited exact-input/full-PNG cross-run transfers, with disjoint coverage checked. All108 profile-specific visible-versus-clip/hidden discriminators preserve geometry and change pixels. Sparse vector glyph/corner antialiasing differences remain within unchanged gates. Initial27-request parity and216 clone/resize geometry checks pass. L20 remains active for nested/positioned compositions, CSS-wide resets and axis/clip-margin investigation. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested expansion:24 scenes/72 comparisons pass each renderer profile;24 new public native/WASM parity cases and combined51-scene/408 original-clone resize checks pass. Native visual coverage is36/72 after inspecting absolute underlined text clip/initial and relative image unset across all widths. Remaining visual review and CSS-wide discriminators are open. Original br fixture rejected by the documented block-context rule; corrected display:block oracle and failed runs retained. Added reusable `validation/parity-fixtures.mjs`. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested audits complete:72 views/profile, native18 direct+54 exact image transfers; vector6 direct+18 within-run exact transfers+48 exact-input/full-PNG transfers. All36 profile-specific reset groups verify identical geometry, inherit=clip, initial=unset and changed pixels when the inner clip is released. Combined focused coverage is153/profile;51 public parity cases and408 clone/resize checks pass. Axis investigation captured18 prospective scenes/54 pinned Chrome views; all18 inputs are explicitly rejected by the current compiler. Runtime currently has only a two-axis rounded clip path; axis policy/transport and clip-margin support remain implementation work, not external blockers. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 axis foundation implemented: compiler overflow values now retain specified x/y states and compute their coupling, preserving hidden versus clip and distinguishing computed auto. Existing admission is unchanged until runtime support lands. All333 compiler tests across63 result groups pass; native build passes. New native outputs for51 reviewed overflow inputs are byte-identical to previous Rive/map/requirements artifacts. WASM build56207 is in progress. Renderer current device clip bounds provide a bounded exact half-plane intersection path without an arbitrary visible-axis extent; render API/replay/backend/policy/transport work remains.

Axis value-model verification completed: WASM build passes and all51 overflow inputs pass native/WASM byte/map/requirements parity on the new compiler snapshot (`overflow-axis-model-compiler`). No public axis syntax has been admitted yet.

L20 axis renderer implementation added: optional clip_axis API, precise recording and typed replay command, glyph-adapter forwarding, and Metal device-bound half-plane clipping. Geometry tests3/3 pass. Metal-feature test20200 and stream test40563 remain running; real pixel validation and runtime/compiler policy installation remain pending. No new public axis syntax is admitted.

Axis renderer verification: Metal-feature build and3 geometry tests pass; full render-stream suite7 tests passes, including precise axis round-trip, invalid-input rejection and unsupported-backend error. Actual pixel controls remain next.

Axis renderer frame wiring and pixel controls: the first stream run failed explicitly with UnsupportedOperation("clipAxis") because NativeMetalFrame lacked forwarding. Added forwarding in NativeMetalFrame and its canvas wrapper; the rebuilt frozen renderer completes144 Chrome153.0.8010.12 controls.108 pass;36 rotated/sheared x/y/both clips fail unchanged thresholds. Translation, fractional translation, scale, reflection and all unclipped controls pass. Rotated x/integer and unclipped rotation sheets directly inspected at all3 widths: native clip exposes excess teal while uncut transform and restored purple agree. This is renderer-only diagnostic evidence, not public axis syntax admission or responsive compiler qualification. Preserve both failed runs. Next: diagnose affine clip polygon/backend consumption before runtime/compiler policy integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-frame/failure-audit.json.

Axis renderer affine failure fixed: diagnostic output showed initial overallClipPixelBounds uses i32 sentinel extremes. Polygon intersections therefore reached billions of pixels, losing ordinary-edge precision on conversion to f32. Intersecting existing bounds with actual frame dimensions before constructing the strip fixes all36 failures. Fresh frozen renderer passes144/144 Chrome controls, with unchanged thresholds; Metal-feature geometry tests3/3 pass. Nine rotated integer x/y/both views directly inspected: clipped bounds and restored purple agree, sparse slanted-edge antialiasing remains.135 visual views remain unaudited. Original failure corpus/debug output retained. Public axis syntax remains rejected pending runtime policy, compiler/host transport, clone/resize and public parity qualification. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/replay.json and overflow-l20-receipt.json.

Axis renderer visual coverage now36/144: all rotated/sheared x/y/both integer/fractional views directly inspected and audited at three widths. Bounds, viewport crops and restored purple match Chrome; thin slanted-edge antialiasing differences remain within unchanged gates. New validation/audit-axis-renderer.py verifies432 distinct-axis pair comparisons across36 groups and288 restored-region controls against current PNG hashes. These controls prove axis differences and save/restore isolation; they do not replace remaining108 visual reviews or public compiler integration. Evidence: overflow-axis-renderer-finite-frame/{visual-inspection,control-audit}.json under output/playwright/html-to-riv.

Axis renderer visual audit now78/144 after directly inspecting translation and scale X/Y/both integer/fractional sheets at all widths. Placement, visible extents and restored purple agree with Chrome; fractional right/bottom edges retain thin coverage differences under unchanged thresholds. Coverage includes exact within-run full-image transfers recorded by the review tool, not additional direct inspection claims.66 views remain. Numeric144/144 and control audits432 axis distinctions/288 restored regions remain passing. Public compiler axis admission is still pending runtime integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/visual-inspection.json.

Axis renderer controls complete:144/144 Chrome comparisons pass, with audited visual coverage (120 directly inspected views and 24 exact full-image transfers). Final fractional-translation/reflection clips and all unclipped controls agree in extent/cropping/restore; thin fractional edge coverage differences remain within unchanged gates.432 axis-discrimination and288 restored-region checks pass. This qualifies the direct Metal renderer diagnostic matrix only. Runtime/public compiler axis syntax remains pending. Shared integration must cover both LayoutComponent.draw_proxy and begin_css_ancestor_clip, which currently read base.clip and clip a world rounded path. Live dimensions, transform restoration, clone policy and plain-container proxy inclusion must all be preserved. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/{replay,visual-inspection,control-audit}.json.

Axis runtime integration foundation: ordinary draw_proxy and deferred begin_css_ancestor_clip now share begin_layout_clip, preserving caller-owned save/restore and ClipSaved bookkeeping. Positioned-paint regression5/5 passes, including864 original/clone dynamic clip/resize draw frames. Initial cargo package-name typo failed before tests and is preserved; corrected nuxie-html-to-riv run passed. No new overflow syntax admitted. Next: occurrence policy and transform-aware strip dispatch, with proxy/path invalidation, clone handling and checked compiler/host transport. Existing direct-renderer144-view qualification remains separate from this runtime integration. Evidence: output/playwright/html-to-riv/overflow-shared-clip-runtime-tests-corrected.log.

Transform-aware axis renderer API added: clip_axis_transformed composes an additional local matrix for clipping while restoring the exact prior drawing matrix, avoiding inverse-transform roundoff. Recording/replay accepts optional finite clipAxis matrix; adapter and Metal frame/canvas forwarding included. Expanded render-stream7/7 passes. Fresh local-transform mode completes144/144 Chrome pixel controls; complete authored HTML/CSS and browser/native PNG byte identity transfers all144 reviews from the audited finite-frame run. Control audit432 distinctions/288 restored regions passes. No public compiler axis syntax admitted. Runtime occurrence policy and clone/resize/host integration remain next. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-transformed/{replay,control-audit,visual-identity-transfer}.json.

Live runtime axis policy implemented: LayoutComponent stores optional CssOverflowAxis (horizontal/vertical), clones it, keeps a drawable proxy through toggles, and computes each strip from current layout width/height and world transform. The shared ordinary/deferred clipping helper dispatches clip_axis_transformed; clearing restores the imported two-axis clip flag. An unsupported renderer fails explicitly rather than drawing without the requested clip. Positioned-paint6/6 passes, including32 new original/clone frames with axis switches, repeated240/390/768/240 sizes, clear/reinstall, deferred clips and sibling restore isolation; prior864 dynamic two-axis frames remain covered. This is runtime stream/lifecycle evidence, not public CSS or runtime pixel qualification. Atomic checked installer, versioned compiler/host transport, public admission/parity and actual runtime pixel corpus remain. Evidence: output/playwright/html-to-riv/overflow-axis-live-runtime-tests.log.

Axis overflow checked scene installer added: Artboard.set_css_overflow_axes_occurrence validates the root, all object IDs/types and duplicates before mutations, clears omitted overrides back to imported clipping, releases the artboard borrow during per-layout invalidation, and rebuilds draw order. Positioned-paint7/7 passes. New regression proves invalid lists retain prior policy, clearing restores the imported clip flag and clipPath drawing, clones retain their own axis, and clone replacement leaves original cleared. Existing32 axis lifecycle and864 ordinary clip frames remain covered. Versioned compiler/host transport, public syntax and runtime pixel qualification remain open. Evidence: output/playwright/html-to-riv/overflow-axis-installer-runtime-tests.log.

Axis overflow contract foundation added: version20, layout-css-axis-overflow-v1 and strict layout_axis_overflow entries {object_id,axis:x|y}. Validation enforces version/capability coupling, unique non-root layout targets and optional stacking coexistence; empty entries are omitted from older outputs. Public Rust exports and TypeScript version union updated. Native probe maps requirements through the checked runtime installer. Focused2/2, full compiler337 tests across64 groups and TypeScript checks pass. Featured probe build25280 is running; initial mistaken example target failed before building and is preserved. Public CSS axis syntax/emission, WASM/parity and runtime pixel qualification remain pending. Evidence: output/playwright/html-to-riv/overflow-axis-contract-{tests,full-tests,types-final}.log.

Public axis CSS parsing/emission implemented. overflow accepts1/2 values and overflow-x/y one; visible/clip/hidden, normal precedence, CSS-wide keywords and supported custom substitutions covered. Inherit copies computed values. Final visible/hidden coupling rejects computed auto; clip/visible emits version20 axis requirements, both-axis pairs use existing clipping. Focused6 tests pass including prior clipping regressions; old axis rejection cases replaced with unsupported scrolling pairs. Initial new test diagnostic-vector access compile error preserved. Full compiler84510 is running. New support section explicitly marks public parity/runtime scene geometry/pixels pending, separate from direct renderer qualification. Evidence: output/playwright/html-to-riv/overflow-axis-public-tests-corrected.log.

Public axis overflow initial corpus passes: corrected full compiler339 tests/65 groups, fresh native/WASM builds,14 public parity cases,42 Chrome geometry/pixel comparisons per renderer profile, and combined65-scene520 original/clone resize geometry checks. Four prospective visible/hidden scenes remain intentionally rejected for computed auto. Initial replay failed on missing copied oracle PNGs; exact42 reference files copied with hashes and fresh native-images output passes. Twelve one-axis native views directly inspected: responsive clip extents, visible-axis escape, viewport crop and square clip over rounded orange background match Chrome.30 native visual reviews and vector audit remain, followed by composition/host/full regression expansion and clip margins. Evidence: output/playwright/html-to-riv/overflow-axis-public-{native-images,vector,parity} and overflow-l20-receipt.json.

Public initial axis corpus visual review complete:42 native views (24 direct,18 exact within-run image transfers) and42 vector exact compiler-input/full-PNG transfers audited. New48-scene composition matrix covers shape/underlined multiline text/image, relative/absolute children, X/Y clips, painted/plain hosts and outer rounded clip/visible controls with positioned sibling overlap. All144 Chrome comparisons pass per renderer profile,48 new native/WASM parity cases pass, and combined113-scene904 original/clone repeated-size geometry checks pass. Composition visual review/discriminator audits remain pending; no full L20 qualification claimed. Evidence: output/playwright/html-to-riv/overflow-axis-composition-{native,vector,parity,oracle} and overflow-axis-composition-resize.log.

Axis composition review now42/144 native views (12 direct plus exact full-image transfers) after unpainted text/image X/Y sheets inspected. Text line/underline clipping, quadrant cropping, outer rounded bounds and green sibling overlap agree with Chrome. Outer-clip audit found original X cases mostly fit outer height, so they prove placement rather than nested clipping. Preserved them and added24 short-outer X variants. New72 comparisons/profile and24 public parity cases pass;72 paired scene groups verify144 browser/runtime image changes across the outer clip toggle. Combined137-scene1096 original/clone size checks pass. Short-outer visual review and remaining composition native/vector review still pending. Evidence: output/playwright/html-to-riv/overflow-axis-{composition,shortouter}-discriminators.json and overflow-axis-shortouter-{native,vector,parity}.


### Short outer axis clipping visual audit (2026-09-10)

The 72 Chrome/native comparisons in `overflow-axis-shortouter-native` now have complete audited visual coverage: 36 direct views and 36 exact full-image transfers within the run. Inspected shapes, underlined text and quadrant images at 240/390/768px. Outer clipping removes lower content while visible controls preserve it; rounded background paint remains independent of the straight x-axis clip. The vector replay has 48 audited exact compiler-input/full-PNG transfers; 24 changed text views still require direct inspection. Numeric gates pass in both profiles; this does not complete L20. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json` and the per-run visual receipts.


### Short outer vector review completed (2026-09-10)

All 72 short-outer vector comparisons now have audited visual coverage: 12 directly inspected views, 12 exact within-run transfers and 48 exact compiler-input/full-image cross-run transfers, with no overlapping counts. Glyph-edge differences are visible in the difference sheets and pass the unchanged numeric thresholds; clipping boundaries, line placement, underlines and sibling/background layering agree with Chrome. The new `validation/audit-combined-review.py` revalidates both underlying receipts and records their set union in `overflow-axis-shortouter-vector/visual-coverage.json`. Main composition review, host compatibility expansion, clip-margin and full regression work remain open; L20 is not yet qualified.


### Main composition shape review (2026-09-10)

Native composition review now covers 90/144 views (33 direct, 57 exact within-run transfers), with the receipt independently audited. All shape variants are covered. Y-axis clips preserve horizontal escape until the outer clip trims it; x-axis clips retain vertical extent and square edges independently of rounded orange backgrounds. Green sibling occlusion/exposure and responsive orange right-cap placement agree with Chrome. Tall x cases remain placement controls, with actual vertical outer-clipping discrimination supplied by the completed shortouter corpus. Text/image composition reviews and remaining L20 qualification work are still open.


### Main composition native review complete (2026-09-10)

All144 main composition native views now have audited visual coverage (63 direct, 81 exact within-run transfers). Text reviews confirm clipping of line beginnings under X and top/bottom lines under Y, preserved underlines and correct sibling/background layering. Image quadrant boundaries and clipped horizontal bands agree with Chrome at240/390/768. Vector replay has96 audited exact compiler-input/full-PNG transfers;48 changed text views require direct review. No thresholds changed. Evidence: per-run visual-inspection.json and visual-transfer.json, summarized in overflow-l20-receipt.json. L20 remains active pending vector review, host compatibility expansion, clip-margin and full regression integration.


### Main composition vector review complete (2026-09-10)

All144 vector composition views now have complete audited coverage: 21 directly inspected, 27 exact within-run transfers and96 exact compiler-input/full-image cross-run transfers, with no overlapping counts. Review confirms glyph clipping, underlines, background corners and positioned sibling layering against Chrome at240/390/768. Glyph-edge raster differences remain visible but satisfy unchanged thresholds. The combined audit revalidated both underlying receipts. Initial42, composition144 and shortouter72 axis views per profile now have complete visual coverage. This finishes the current focused visual corpus, not L20: host compatibility expansion, clip-margin and full regression integration remain.


Axis host regression discovered (2026-09-10): new public host test reproduces missing axis clip for an unpainted, unpositioned container after rebuilding the featured probe. Earlier focused pixel passes do not qualify this configuration. Preserved request, Rive, requirements and stream: `output/playwright/html-to-riv/overflow-axis-plain-host-reproducer/`. Runtime drawable lifecycle investigation is active; do not claim general axis clipping qualified. Logs: overflow-axis-host-initial.log and overflow-axis-host-focused.log.


### Plain axis proxy lifecycle fix (2026-09-10)

The checked installer now creates missing drawable proxies for plain layouts after import. It places nested proxies using complete owner ancestry, preserving subtree boundaries. The first insertion attempt opened clips after child paint; that failing trace is retained in overflow-axis-proxy-nested-debug.log. Corrected runtime tests pass8/8, including32 new original/clone frames with repeated install, clear, reinstall and resizing; existing positioned/stacking lifecycle tests also pass. Rebuilt native host tests pass18/18, including missing axis capability and malformed manifest rejection before stream output. Eight prospective plain/nested/painted x/y fixtures are in validation/overflow-axis-plain-cases.json. Pixel/geometry qualification and parity against a newly frozen toolchain remain pending; prior corpus receipts refer to their original binaries. Logs and source hashes are in overflow-l20-receipt.json.


### Plain-container fix pixel qualification (2026-09-10)

Frozen overflow-axis-proxy-toolchain passes24/24 Chrome geometry/pixel comparisons in each profile and8/8 public native/WASM parity cases. All24 native views were directly inspected; all24 vector views transfer with exact compiler-input/full-PNG identity and an independent audit. Single/nested x/y clips match expected red extents and preserve the blue sibling outside the clipped subtree. Painted x cases retain a thin fractional background-edge difference at390 within unchanged tolerances. Public original/clone resize regression now passes1160 updates across145 scenes. Initial resize run failed only its obsolete1096 count assertion, preserved in overflow-axis-plain-resize.log; corrected count passes in overflow-axis-plain-resize-final.log. Prior focused corpora and full regression still need rerunning with the fixed runtime. L20 remains active.


### Fixed-runtime regression started (2026-09-10)

Full Rust compiler suite passes340 tests across65 result groups (overflow-axis-proxy-full-tests.log). Composition reruns pass144/144 per profile; all144 native views transfer with exact compiler-input/full-image identity from the reviewed baseline. Vector identity audit remains pending. Full native gallery is running with frozen overflow-axis-proxy-toolchain in isolated overflow-axis-proxy-full-artifacts and overflow-axis-proxy-full-native directories (session18728). Initial/shortouter profile reruns are sequentially running in session14767. These processes are not yet completion evidence.


### Fixed-runtime focused regression complete (2026-09-10)

Initial42, composition144 and shortouter72 reruns pass in both profiles with the proxy fix:258 comparisons per profile. Every compiler input and full browser/runtime PNG pair is identical to its previously reviewed baseline. All six transfer receipts were independently audited. The transfer helper now supports --combined-source, revalidating direct and cross-run evidence before transferring a combined baseline; this does not create new visual inspection claims. Full6271-check native gallery remains live in session18728, with completion and gallery audit still pending.

Clip-margin investigation (2026-09-10):48 prospective scenes/144 pinned Chrome captures and48 compiler rejections recorded. Margin changes affect two-axis clip controls; hidden and single-axis images stay unchanged despite accepted computed styles. See validation/overflow-clip-margin-investigation.md. Syntax remains unsupported.


### Clip-margin value parser (2026-09-10)

Added internal ClipMargin parser with content/padding/border origins, optional nonnegative px/em/rem offsets in either order, unitless zero, case-insensitive identifiers and CSS comments. em uses the supplied computed font size; rem uses the documented16px root. Two focused tests pass, covering accepted forms and duplicate components, negative/percentage/unitless-nonzero/unknown-unit/nonfinite rejection. CSS-wide values stay in cascade handling; math functions are not yet supported. This parser is not connected to public CSS admission: runtime clip bounds, transport, cascade and pixel qualification remain to implement. Log: output/playwright/html-to-riv/overflow-clip-margin-parser.log.


### Live clip-margin bounds foundation (2026-09-10)

Runtime `CssOverflowClipMargin` now validates finite nonnegative offsets and resolves local clip bounds from the current LayoutComponent size and used asymmetric padding. Content origin insets each edge before expansion; padding and border origins coincide while compiler borders remain unsupported. The resolver rejects nonfinite resolved edges and does not mutate layout or background paint. It is not yet connected to drawing, transport or public CSS admission. Focused runtime tests cover repeated size/padding changes and invalid/overflowing numeric inputs; log: output/playwright/html-to-riv/overflow-clip-margin-runtime-bounds.log. Both focused runtime tests passed (2/2); this is geometry-unit evidence only, not renderer or browser qualification.


### Clip-margin rounded runtime experiment (2026-09-10)

Separate CSS clip geometry and occurrence policy are implemented internally. Content-edge outsets use asymmetric live padding; circular border corners may produce elliptical clip corners. The background paints before its own margin clip opens, while ordinary and deferred descendants share the same clipping function. Clones retain the policy, and clearing restores imported clipping. Five geometry unit tests and nine integration tests pass, including 64 margin lifecycle frames plus the existing positioned/stacking suite. Logs: overflow-clip-margin-runtime-path.log and overflow-clip-margin-runtime-background.log under output/playwright/html-to-riv.

Pinned Chrome153 source confirms `ShadowContourFollowsBorder` is stable and overflow-clip-margin uses its coverage-adjusted radius formula. This supersedes the earlier assumption that the published snapshot's simple cubic spread rule alone is sufficient. Immutable source receipt: output/playwright/html-to-riv/overflow-clip-margin-chrome-source/receipt.json.

Public CSS admission and checked transport remain unimplemented. `replay-oracle.mjs --clip-margin-experiment` explicitly strips the unsupported declaration from the compiler request and records the diagnostic runtime injection alongside the original browser CSS. These runs are runtime experiments, not public compiler or native/WASM qualification. Numeric/pixel and visual review receipts remain required.


Initial clip-margin runtime experiment completed:144/144 Chrome geometry/pixel comparisons per renderer profile pass, with complete native visual audit and144 exact-input/full-PNG transfers to the explicit vector-v2 run. Runtime injection and original browser CSS are checked before review transfer. Sources and images remain recorded as experiments, not public compiler qualification. The first vector-named run accidentally used nativeGlyphs=1; preserved as a native repeat and excluded from vector evidence. See overflow-l20-receipt.json.clipMarginRuntimeExperiment. Public admission, checked transport, parity and broader live composition remain.


### Public clip-margin candidate: version21 (2026-09-10)

`overflow-clip-margin` now accepts an optional content-box, padding-box or border-box and an optional nonnegative px/em/rem length in either order, including unitless zero and comments. At least one component is required. Default is padding-box0. It is non-inherited; initial/unset reset it, and inherit copies the computed parent value. em uses the final computed font size; rem uses the documented16px root. Cascade, important and custom-property/fallback resolution apply. Negative lengths, percentages, duplicate components and malformed values reject; known invalid values substituted through var() become unset. Math functions and other length units remain unsupported.

The pinned Chrome profile applies the margin only to computed two-axis clip. Hidden and one-axis overflow retain their no-effect behavior. A nondefault effective margin emits version21 with layout-css-overflow-clip-margin-v1 and layout_overflow_clip_margins. The checked host rejects unsupported capabilities, malformed/duplicate/root/non-layout targets, negative/nonfinite offsets and conflicts with axis policies before drawing. Border/padding reference edges currently coincide because compiler border painting remains unsupported.

Full compiler348 tests, host19 tests, typecheck,48 native/WASM parity cases and1544 combined original/clone resize updates pass. Initial48 scenes/144 Chrome geometry/native pixel comparisons pass in each renderer profile. All144 public pairs match the reviewed runtime experiment; promote-clip-margin-review.py audits original browser CSS, the diagnostic-to-manifest policy change, all other compiler inputs/runtime policies, and full PNG identity. No diagnostic injection is used by these public runs. Evidence: output/playwright/html-to-riv/overflow-l20-receipt.json.clipMarginPublic.

L20 remains active for expanded small-radius/cascade controls, realistic text/image/positioning compositions, live resize pixels and full version21 regression. The prior axis proxy snapshot completed6271 checks and6261 exact-input/full-image visual transfers; this does not substitute for a new-runtime regression.


### Chrome153 signed-margin correction (2026-09-10)

The expanded20-scene corpus produced57/60 pixel passes and three preserved failures for a negative custom-property value. Direct image inspection showed Chrome shrinks the clip by8px while the compiler reset it to zero. A computed-style probe confirms Chrome153 accepts both direct negative lengths and negative var() substitutions; negative content-box values also remain negative. Percentages and invalid identifiers through var() reset to0px. Evidence: output/playwright/html-to-riv/overflow-clip-margin-negative-variable-computed.json and overflow-clip-margin-expanded-native/replay.json.

This corrects the earlier published-spec-based nonnegative assumption. The in-progress version21 candidate now admits finite signed px/em/rem offsets; negative margins contract clipping. The parser, substitution validator, runtime constructor and contract validation have been updated; focused tests are running. The original v21 toolchain and failing PNGs remain frozen. An18-scene direct-negative matrix adds square/small/rounded corners, content/padding origins and offsets-8/-24/-80px including empty clips. Signed public native/WASM rebuild, parity and pixel verification remain pending. Earlier348-test/144-pixel candidate receipts do not qualify the signed change.

Signed-margin focused compiler/contract/runtime tests6/6 pass. Native featured build and18-scene Chrome capture are running; signed native/WASM parity and pixels remain pending.


Signed-margin correction validation: native/WASM build and38 expanded/signed parity cases pass; host19 passes. Both renderer profiles pass all54 direct-negative comparisons and60 expanded comparisons, including the three preserved negative-variable reproducers. The public overflow test now covers231 scenes/1848 original-clone resize updates; the margin lifecycle test covers192 positive/negative/empty-clip frames with background and deferred-descendant clip-state assertions. Native visual audit currently covers27/54 signed and21/60 expanded views; remaining review is open. Frozen toolchain: overflow-clip-margin-signed-toolchain. Evidence: overflow-l20-receipt.json.clipMarginSignedCorrection.


2026-09-10 continuation: Signed-margin visual review is complete for54/54 native views, with54 audited exact-input/full-PNG transfers to the vector profile. The current signed toolchain also passes144/144 initial-corpus regression comparisons. New72-scene shape/text/image compositions cover relative/absolute descendants, clipped/visible outer ancestors, content/padding origins and offsets-8/0/24; native/WASM parity72 and216/216 geometry/pixel comparisons per profile pass. Composition visual review remains pending. The public original/clone resize test now includes303 scenes and2424 updates and passes; the earlier stale expected-count assertion failure is preserved separately. Expanded visual review, live same-scene pixel qualification and fullv21 regression remain open; L20 is still active. Evidence: output/playwright/html-to-riv/overflow-l20-receipt.json.clipMarginSignedCorrection.


2026-09-10 lifecycle follow-up: The clip-margin lifecycle test now compiles the public CSS property and checks its emitted manifest before applying the checked runtime installer. Both lifecycle/atomicity tests pass. Optional NUXIE_CLIP_MARGIN_RECORDING emits192 sequential original/clone frames with repeated240/390/768/240 resizing and clear/reinstall controls; validation/clip-margin-lifecycle.mjs compares these streams with the equivalent live Chrome DOM. Pixel comparison is running (session82244), not yet qualified. The full frozenv21 native regression is running separately (session52141). First composition review covers3/216 views; the remaining213 are open. Runtime policy transitions are validation controls, not support for author scripting or interactions.

Lifecycle pixel run completed:192/192 sequential original/clone geometry and native pixel comparisons pass against pinned Chrome153. Visual review remains pending; fullv21 regression continues in session52141.

2026-09-10 expanded clip-margin audit: all60 native views now have complete direct/exact-image visual coverage; all60 vector views have audited exact-source/full-PNG transfers. Inspected cases include border-box default/expansion, declaration comments/order, final-font em computation, inherit, content-box origin and intermediate radii. Sparse corner antialias differences are preserved under existing thresholds. Signed54/profile and expanded60/profile visual audits are complete; composition and lifecycle reviews plus initial signed review transfer and fullv21 regression remain pending.

2026-09-10 visual continuation: initial signed-toolchain regression144/144 now has an audited promotion from the directly reviewed runtime corpus, checking unchanged source CSS, matching public policy and exact full PNG pairs. Composition review covers21/216 views, including underlined text and images crossing expanded/contracted and ancestor clips. Lifecycle review covers96/192 frames after direct inspection of12 unique square-host image pairs and exact-image transfers across repeated original/clone frames. A thin host right-edge rasterization difference at390px also appears in clear controls and remains under unchanged thresholds. Rounded lifecycle96 and composition195 views remain open; fullv21 regression remains running in session52141.

2026-09-10 lifecycle visual audit complete:192/192 original/clone frames now covered by28 directly inspected unique Chrome/native pairs and164 exact-image transfers. Rounded and square hosts preserve background/siblings through clear/reinstall and empty clips. Native PNG hashes are identical for every repeated logical state across original/clone and resize cycles. The lifecycle runner now asserts that invariant and writes repeat-stability.json; validation rerun session14866 is in progress. Fullv21 regression session52141 remains active; composition195 views remain unreviewed.

Repeat-stability runner validation passed:192/192 Chrome geometry/pixel comparisons;36 distinct logical states and156 repeated frames have identical native PNG hashes. Receipt: overflow-clip-margin-lifecycle-stability/repeat-stability.json. The earlier directly reviewed lifecycle run remains the visual source.

2026-09-10 composition audit: all12 absolute-image cases now visually covered at all three widths, including content/padding origins, contracted/default/expanded margins and clipped/visible ancestors. Overall composition coverage is51/216 with165 remaining. Matching image bounds, quadrant boundaries, sibling overlap and ancestor clipping were inspected; sparse corner rasterization differences remain visible within unchanged thresholds. Fullv21 regression session52141 continues.

2026-09-10 relative-image review complete: all24 image composition scenes (72 views) now visually covered, including relative/absolute placement, content/padding margins and clipped/visible outer ancestors. Overall composition review is84/216 with132 remaining, now text and solid-shape cases. Fullv21 regression session52141 remains active. No tolerance changes.

2026-09-10 absolute-text audit: all12 absolute underlined-text composition scenes now reviewed at all three widths. Glyph/underline clipping, expanded lower lines and ancestor clipping match Chrome within unchanged thresholds. Overall composition review is114/216, with102 remaining. Relative text and solid-shape cases remain; fullv21 regression session52141 is still running.

2026-09-10 relative-text audit complete: all24 text composition scenes (72 views) now visually covered. Relative/absolute glyph fragments and underlines match Chrome across content/padding margins and outer clipping. Overall composition review is147/216 with69 solid-shape views remaining. Fullv21 regression session52141 remains active. No tolerance changes.

2026-09-10 solid-shape audit complete: all216 native composition views now have audited visual coverage. Contracted/expanded contours, relative and absolute descendants, ancestor clipping, host paint and siblings match Chrome within unchanged thresholds. Vector composition216/216 geometry/pixel checks pass;144 shape/image views have audited exact-source/full-PNG review transfers. The72 vector text views need separate visual inspection. Fullv21 regression session52141 remains live (latest observed5348 checks).

2026-09-10 vector composition continuation:18 absolute-text/outer-clip views directly inspected. Glyph and underline cutoffs match Chrome; vector glyph-edge raster differences remain visible under unchanged passing gates. Audited combined coverage162/216 (18 direct,144 exact-source/image transfers,zero overlap);54 text views remain. Native composition audit is complete216/216.

2026-09-10 composition visual qualification complete:216/216 views per profile pass geometry/pixel checks and now have complete audited visual coverage. Vector review combines72 directly inspected text views and144 exact-source/full-PNG shape/image transfers with zero overlap. Relative/absolute text, underlines, signed content/padding margins, ancestor clipping and visible-overflow sibling composition were inspected at240/390/768px. Glyph-edge raster differences remain visible within unchanged thresholds. Fullv21 regression session52141 remains live; full regression audit is pending.

2026-09-10 signed version21 full tests complete: compiler348 tests across68 result groups pass (session76332 exit0), and full Chrome/native6271 checks pass in24.3min (session52141 exit0). Full regression exact-input/full-PNG visual audit is running in session17462 against overflow-axis-proxy-full-native; do not claim full visual qualification until that audit completes. Focused composition216/profile and lifecycle192 visual audits remain complete.

2026-09-10 full signed-version21 visual audit complete:6271 checks passed, and all6261 scene pairs have exact complete-input and full Chrome/native PNG identity with the audited axis-proxy baseline. Audit session17462 exited0. All303 accepted overflow scenes are now included in the normal browser regression gate (909 additional views); four prospective axis pairs computing to auto remain intentionally rejected and covered by compiler tests. The regular-suite integration run is active in session14833, with its own image audit pending.

2026-09-10 P01 border baseline:54 prospective uniform-solid-border scenes captured162 Chrome153.0.8010.12 references across width/radius/box-sizing/overflow combinations. The frozen version21 compiler rejects all54 with unsupported-property diagnostics; no border support is admitted. Evidence: output/playwright/html-to-riv/border-solid-initial-oracle and border-solid-initial-admission. Runtime investigation identified used-border retention, content measurement and inner clipping as required work.

2026-09-10 L20 qualified for the documented static-overflow subset: regular-suite930/930 checks pass and all930 full-input/full-PNG review transfers audit successfully (session61601 exit0). Together with6271 full regression checks/6261 reviewed pairs, focused renderer audits, public parity,2424 resize updates and192 live clip-margin frames, this closes L20. Scrolling remains excluded and nonzero borders must extend/requalify clipping in P01.

2026-09-10 border width resolution:42 computed-style observations captured with pinned Chrome153 at page zoom1 and DPR1/2. Both DPRs compute positive subpixel widths as1 CSS px, floor larger fractional widths, and map thin/medium/thick to1/3/5px. The internal resolver now matches all30 width observations using final-font em and fixed16px-root rem; three unit tests pass, including invalid/overflow lengths. Reset controls confirm border-style:solid alone leaves0px, border:solid resets to3px, and border:8px without style computes0px. This does not qualify border painting or admit public border CSS. Evidence: output/playwright/html-to-riv/border-width-resolution-receipt.json.

2026-09-10 border shorthand foundation: internal Border/BorderStyle/BorderColor types now parse a uniform width, none/hidden/solid style and supported sRGB color in any component order. Omitted shorthand components reset to medium/none/currentColor; the authoring reset border:0 remains distinct. currentColor stays symbolic, and none/hidden use zero width without discarding specified width. Six focused tests pass covering permutations, comments/function colors, duplicates, malformed/unclosed/nested functions and existing Chrome width observations. CSS-wide cascade, public admission, runtime geometry and border painting remain outstanding. Evidence: output/playwright/html-to-riv/border-shorthand-receipt.json.

2026-09-10 border internal cascade: Style now carries Border values and applies shorthand/longhands with CSS-wide inherit/initial/unset. Width resolves after the final font pass; inheritance copies computed pixels (including zero under none/hidden), while currentColor remains symbolic. The reset remains border:0 and borders do not inherit by default. Nine focused tests pass. Public validation explicitly rejects all four border properties, including initial/variable forms, pending rendering. Full compiler regression is running (border-cascade-full-tests.log). Invalid-variable classification, runtime border edges and painting remain pending.

2026-09-10 border variable invalidation: internal cascade now resets definitely invalid border substitutions (negative/nonlength widths, duplicate components and invalid colors) while retaining diagnostics for valid unimplemented CSS such as dashed style, side lists and math. Supported RGB/HSL variable values retain their color component. Ten focused tests pass. Prior full cascade suite357 tests/68 groups passed; custom-property regression is running after this validator extension. Public border admission remains gated. Logs: border-variable-tests.log and border-variable-regression.log in output/playwright/html-to-riv.

Border variable regression completed:51/51 custom-property tests pass. Receipt: output/playwright/html-to-riv/border-variable-receipt.json.

2026-09-10 runtime border-edge foundation: layout solver output now retains physical border edges alongside padding, detects edge changes as new layout, publishes both on ordinary/occurrence updates, and clears solved edges with layout resets. Opt-in clip-margin bounds now distinguish border, padding and content origins with signed offsets. A focused test covers asymmetric borders, padding and repeated sizes; runtime test session66021 is running. Ordinary content measurement, hidden/axis clipping, border ring painting and public transport/admission remain pending. Evidence: output/playwright/html-to-riv/border-runtime-edge-receipt.json.

Runtime border-edge tests completed:7/7 pass. Existing overflow resize and clip-margin lifecycle regression now running in session64249.

2026-09-10 border runtime policy: opt-in border geometry now subtracts used borders from inner dimensions with a zero floor; one-axis clips use padding-box intervals; default two-axis clips use inset rounded paths. Clip-margin bounds and draw paths share the same border/padding/content inset calculation. Box paint precedes its inner descendant clip. Policy cloning/clearing and collapsed dimensions have a focused test; session24081 is running. Earlier edge-retention overflow regression passed3 tests (2424 resize updates plus margin lifecycle assertions). Public host transport, compiler emission, measured content origins and border painting remain pending.

2026-09-10 runtime policy tests pass8/8. Added a54-scene diagnostic layout integration test using schema border-width injection into borderless compiled scenes; it compares every source box with independent Chrome references across original/clone and240/390/768/240 resize cycles (432 updates). Session97250 is running. This isolates runtime layout support and cannot qualify public compiler emission or pixels. Text origins need no separate padding adjustment for layout participants: their solved locations already come from the layout engine; richer text/image cases remain to verify.

Diagnostic border layout result:54/54 scenes and432/432 original/clone resize updates match Chrome source geometry. Session97250 exited0. Public border emission and rendered border pixels remain unqualified.

2026-09-10 border ring geometry: added an even-odd outer/inner contour builder using live border widths. Outer radius normalization precedes inset subtraction; zero widths yield an empty ring, collapsed inner boxes retain the outer fill, and nonfinite/negative widths reject. All5 css_clip_path tests pass, including2 ring-specific checks. Drawing, alpha/overlap validation and Chrome pixels remain pending; this is geometry evidence only. Receipt: output/playwright/html-to-riv/border-ring-geometry-receipt.json.

2026-09-10 border runtime drawing connected: optional border color paints the live rounded ring with even-odd fill, inherited render opacity and blend mode, after background and before descendant clipping. Color policy survives cloning; render paths/paint are allocated lazily. The diagnostic54-scene test draws all432 original/clone resize frames and confirms border draw presence while retaining exact Chrome geometry comparisons. Test passes. Rendered pixel/alpha/clipping validation, border-only proxy installation and checked public transport/emission remain pending. Evidence: output/playwright/html-to-riv/border-runtime-draw-receipt.json.

2026-09-10 border sequential pixel validation started: diagnostic test recorded432 original/clone resize frames successfully. The lifecycle runner now supports an explicitly labelled border-diagnostic recording, preserving original browser CSS and injected width/color separately from compiler request CSS. Native Metal replay/Chrome geometry and pixel comparisons plus repeated-state hash checks run in session17183. Initial89 frames pass numeric gates; no complete passing or visual-review claim yet. First1px/18px-radius content-box clip browser/native pair inspected with matching visible border and child clipping. Evidence: output/playwright/html-to-riv/border-initial-pixel-receipt.json.

2026-09-10 border initial native replay complete:432/432 diagnostic geometry/pixel comparisons pass, with162 distinct states and270 repeated original/clone states rendering byte-identically. Direct inspection of12 thick-border views (radii18/90, content-box/border-box, all widths) plus exact-image transfers gives audited64/432 visual coverage;368 remain. Square inner corners when thickness exceeds radius and normalized capsule contours match Chrome; sparse outer-edge raster differences remain visible under unchanged thresholds. Public emission, broader alpha/border-only compositions and full visual qualification remain pending. Receipt: output/playwright/html-to-riv/border-initial-pixel-receipt.json.

2026-09-10 border visible-overflow review: six thick rounded overlap views inspected across both sizing modes and three widths. Child paints over border/sibling as Chrome does; audited review coverage80/432,352 remaining. Added24 prospective paint cases covering zero/half alpha, transparent/opaque background, absent/present child and radii0/18/90. Chrome capture session2123 is running; runtime replay for this expansion remains pending.

Paint expansion Chrome capture completed:24 scenes/72 references on Chrome153.0.8010.12. Runtime admission, draw and pixel checks remain pending.


### Border transparency runtime evidence — 2026-09-10

The24-scene alpha/background/child matrix now passes192/192 Chrome/native geometry and pixel comparisons across original/clone resizing at240/390/768/240. Four full-resolution sheets were inspected directly; exact paired-image identity extends coverage to8/192 frames, leaving184 unreviewed. Inspection confirms half-alpha ring blending, empty transparent interiors, background beneath transparent borders, and rounded inset child clipping. Thresholds are unchanged. These are injected runtime experiments, not public compiler support; plain unclipped border-only proxy installation and public transport/parity remain outstanding. Receipt: `output/playwright/html-to-riv/border-paint-pixel-receipt.json`.


### Checked border occurrence installation — 2026-09-10

Added atomic `Artboard::set_css_borders_occurrence`, sharing late drawable-proxy installation with axis overflow. Plain nested unclipped border-only boxes now receive proxies; repeated installation avoids duplicate draws, invalid IDs leave policies unchanged, and clearing an original leaves its clone independent. All3 border runtime tests (including both Chrome geometry corpora through this installer) and8 positioned/axis/stacking runtime regressions pass. This does not establish pixel qualification for the new installer or public compiler support. The previous injected pixel recordings remain historical evidence. Receipt: `output/playwright/html-to-riv/border-installer-receipt.json`.


### Checked installer pixel coverage — 2026-09-10

The checked border installer passes192/192 transparency frames; exact HTML/CSS, injected values and both full PNG hashes match the prior injection recording for every frame (`border-installed-paint-native/installer-comparison.json`). This identity check does not claim completed visual review. A new18-scene plain border-only matrix has no background or overflow clip, covering widths1/8/24, radii0/18/90 and alpha128/255. All144 original/clone resize frames pass Chrome153 geometry and native pixel gates. Four original-resolution sheets were inspected; audited direct/exact-image coverage is10/144, with134 remaining. Thin/thick rings, normalized corners, empty interiors and sibling placement agree, with retained curve antialias differences under unchanged gates. Receipt: `output/playwright/html-to-riv/border-plain-pixel-receipt.json`. Public compiler emission, checked format/capability transport, Rust/WASM/JS parity, broader compositions and complete visual qualification remain open.


### Border requirement transport — 2026-09-10

Added runtime requirements version22, capability `layout-css-solid-borders-v1`, and strict `layout_borders` entries containing non-root object IDs and packed ARGB u32 colors. Widths remain live Rive layout-style data. Validation requires unique targets and matching capability/version, rejects malformed colors/fields and checks imported target types; borders can coexist with version21 clip-margin policies without relaxing legacy validation. Seven focused Rust border/clip-margin/axis contract tests and the TypeScript API check pass. The TypeScript union now represents version22 and optional earlier policies. This contract is not yet emitted by the compiler or installed from a public manifest; CSS border admission stays gated and native/WASM/public pixel qualification remains pending. Receipt: `output/playwright/html-to-riv/border-contract-receipt.json`.


### Public border emission — 2026-09-10

Compiler admission now emits uniform used physical border widths into Rive layout styles and version22 ARGB border requirements, including transparent solid borders. Border paint targets also request CSS pixel bounds. The validation probe checks the border capability and imported target types, then invokes the checked installer; `NUXIE_DISABLE_CSS_BORDERS=1` removes host support. Public compilation of all96 initial solid/transparency/plain scenes passes768 original/clone resize geometry updates. The full module suite passes366 tests across70 result groups; the obsolete gate-closed unit test was replaced with declaration acceptance coverage, with the original failed run retained. Public pixel qualification, native/WASM parity, host rejection coverage and richer compositions remain pending. Receipt: `output/playwright/html-to-riv/border-public-emission-receipt.json`.


### Version22 native/WASM and public pixels — 2026-09-10

Fresh native and WASM builds are frozen in `output/playwright/html-to-riv/border-v22-toolchain`. All13 JavaScript tests pass; the accepted-corpus parity test now includes all96 border fixtures and checks exact Rive bytes, source maps and runtime requirements. The version22 host contract test passes valid paint and rejects disabled capability, malformed color/version/fields and invalid/duplicate/root targets before writing draw streams. Public compiler/probe/renderer replay passes54/54 plain and162/162 solid-border Chrome geometry/pixel comparisons. Three plain public comparisons have audited direct visual review;51 plain and all162 solid public comparisons remain visually unreviewed. Existing diagnostic lifecycle evidence is retained separately and does not establish public lifecycle pixel qualification. Public alpha pixels, complete review, lifecycle pixels, richer composition and broad regression remain pending. Receipt: `output/playwright/html-to-riv/border-v22-parity-pixel-receipt.json`.


### Public border lifecycle and clip-origin compositions — 2026-09-10

Public transparency passes72/72 comparisons, with3 directly reviewed and69 remaining. The lifecycle recorder now stores actual public scene bytes and manifests with explicit `border-public` metadata, installs CSS pixel bounds, and uses monotonically increasing sample indices even without recording. All96 recorded artifacts and manifests match the frozen compiler exactly. The original/clone240/390/768/240 replay passes768/768 geometry/pixel checks;480 repeated frames across288 logical states are byte-identical. No public lifecycle visual coverage is claimed yet. Added18 combinations of rounded half-alpha borders, both box-sizing modes and signed clip margins(-8/0/12px) from border/padding/content origins; public replay passes54/54, with visual review and expanded parity/lifecycle pending. Text/image/axis compositions and broad regressions remain open. Receipt: `output/playwright/html-to-riv/border-v22-lifecycle-receipt.json`.


### Border text/image composition: visual defect preserved — 2026-09-10

Added24 text/image cases covering both box-sizing modes, visible/clip/hidden overflow and radii0/24. All72 aggregate geometry/pixel comparisons pass, but visual inspection detects squared native image-content corners where Chrome rounds the content inside border and padding. The focused `validation/check-border-image-corner.py` fails at all3 widths: pixel(40,36) is Chrome orange background(234,164,61) versus native red image(230,100,40). This is a real qualification failure despite aggregate passes; no tolerance changed. Text border-box clip review covers3 direct views and6 identical image-pair transfers(9/72); image corners are explicitly unqualified. Failure receipt: `output/playwright/html-to-riv/border-v22-content-native/image-corner-failure.json`. Next: minimize/diagnose replaced-content clipping, then fix and rerun both focused and aggregate checks.


### Image corner diagnosis — 2026-09-10

Reduced to a single padded image, with square, visible, no-border and no-padding controls. Rounded hidden/clip fail; square/visible/no-padding controls agree at sampled corners, and removing the border retains the failure. Chrome computed styles expose `overflow-clip-margin:content-box` on the image. Native no-border stream clips the outer124x106 rounded box, not the100x90 content region. Evidence supports missing replaced-image clip-origin semantics, rather than border paint or sampling. CSS Overflow4 also distinguishes the default replaced-element content-box origin: https://www.w3.org/TR/css-overflow-4/. This is an existing borderless image clipping gap exposed by the new matrix; it remains unqualified. Diagnosis: `output/playwright/html-to-riv/border-image-corner-minimal-native/diagnosis.json`. Next: preserve authored clip-margin/global-keyword semantics, implement image-specific default/hidden handling without changing container clips, and rerun focused corners plus the original corpus.


### Image content-corner correction — 2026-09-10

Compiler now seeds the image UA `overflow-clip-margin:content-box` before authored declarations and emits clip-margin policy for two-axis hidden images as well as clip. Explicit initial/unset remain padding-box zero; inheritance and variable values retain cascade semantics. A public compiler regression failed on the missing content-box requirement before the fix and passes afterward, alongside two existing margin contract tests. Frozen corrected toolchain passes the original72 comparisons and all3 formerly failing focused corners; all15 minimal comparisons and15 corner/control samples also pass. Corrected image corners were inspected at original resolution, with6/72 audited direct/identical-pair visual coverage. All13 JavaScript/parity tests pass with47 added content/margin/minimal scenes. Full module/broad renderer regression, full review and richer replaced-image axis/authored-margin lifecycle coverage remain pending. Receipt: `output/playwright/html-to-riv/image-clip-origin-fix-receipt.json`. Historical failing recordings remain intact.


### Image correction regression and border axis matrix — 2026-09-10

Full compiler suite after image-clip correction passes367 tests across71 result groups. New24-scene image/container matrix combines clip-visible, visible-clip, hidden and clip with content-box -4px, padding-box8px and border-box0px margins. All72 geometry/pixel comparisons pass;6 direct inspections plus9 exact-image transfers cover15/72, leaving57 unreviewed. Full native/WASM artifact corpus passes with24 new scenes. Three host checks initially failed to launch because the frozen compiler override lacked its separate probe override; targeted rerun with matching frozen probe passes3/3. Logs preserve both runs; no compiler mismatch was found. Expanded lifecycle, complete review and broad renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-axis-margin-receipt.json`.


### Border composition visual review — 2026-09-10

The border/axis/margin matrix now has complete audited72/72 visual coverage:21 direct original-resolution comparisons and51 exact browser/native image-pair transfers. Inspection covers border-box versus content-box clip origins, signed margins, horizontal/vertical strips, hidden containers and replaced images. Corrected text/image matrix review advanced to42/72:15 direct comparisons and27 exact-image transfers, including all36 text views across both box-sizing modes and square/rounded corners. Thirty image comparisons remain unreviewed. No thresholds changed and no aggregate-only pass was counted as inspection. Evidence: `output/playwright/html-to-riv/border-composition-visual-progress.json` and both replay visual-inspection receipts. Feature qualification still requires remaining visual/lifecycle and broad renderer regression coverage.


### Corrected content matrix visual completion — 2026-09-10

The corrected text/image matrix now has complete audited72/72 visual coverage:30 directly inspected original-resolution comparisons plus42 exact browser/native image-pair transfers. The final five image sheets verify both sizing modes, square/rounded corners and visible versus clipped image content at all three widths. Sparse image sampling/antialias differences remain within unchanged thresholds; the former content-corner defect stays fixed. Together with the completed axis/margin matrix, these two corpora have144/144 reviewed comparisons. Other border corpora, expanded same-scene lifecycle and broad renderer regressions still require qualification. Receipt: `output/playwright/html-to-riv/border-composition-visual-completion.json`.


### Border axis/image public lifecycle — 2026-09-10

Extended public lifecycle recording to image assets and emitted axis/clip-margin policies. All24 image/container axis-margin scenes pass192 original/clone240/390/768/240 geometry and native pixel frames, with120 repeated frames identical across72 logical states. Every recorded Rive artifact and manifest exactly matches the frozen public compiler. A dedicated repeatable audit validates source review completeness, source HTML/CSS/assets, identical Rive bytes/manifests, stream hashes and exact full browser/native PNG pairs before transferring reviewed coverage;192/192 lifecycle frames now have audited visual coverage from the completed static matrix. The replay helper embeds the actual quadrant asset and waits for decode. Other border review/lifecycle corpora and broad renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-axis-image-lifecycle-receipt.json`; audit: `validation/audit-border-lifecycle-review.py`.


### Deferred content-box border corpus admitted — 2026-09-10

The80 scenes deferred during box-sizing work now compile publicly. Fresh Chrome references compare against the corrected frozen compiler/host:240/240 native and240/240 vector comparisons pass. Exact paired browser/native PNG identity across profiles holds for240/240; no completed visual transfer is claimed. Zero-size and reversed-row controls were inspected at original resolution; native visual coverage is12/240, with228 remaining. All13 JavaScript/native-WASM tests pass with the80 scenes added to the accepted artifact parity corpus. The corpus retains its historical deferred filename for traceability; current status is implemented with qualification in progress. Live clone/resize coverage, complete review and broad full renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-deferred-receipt.json`.


### Deferred border lifecycle: clone paint failure — 2026-09-10

Extended recorder to80 deferred border sizing scenes. Initial geometry run exposed a recorder omission of emitted flex-factor policies; added their installation and CSS paint-order policy, and replaced self-approval of arbitrary capabilities with an explicit implemented whitelist. Seven recorder tests pass, and all640 original/clone geometry updates agree with Chrome. Recorded artifacts/manifests match the frozen compiler. Pixel replay passes632/640;8 failures affect only cloned zero-sized normal-row content-box/border-box cases. Visual inspection shows the overflowing brown child painting over the adjacent yellow sibling, unlike Chrome/original; geometry is unchanged. Both failed recording and failing pixel artifacts are preserved. Static review progressed to21/240; lifecycle qualification remains failed. Next: minimize clone overlap ordering and fix with a focused render regression. Failure receipt: `output/playwright/html-to-riv/border-deferred-lifecycle-failure.json`.


### CSS paint-order clone correction — 2026-09-10

A reduced borderless overlap test reproduced clone draw-order reversal. `clone_instance_definition` copied positioned/stacking policies but omitted `css_paint_order`; it now copies that policy too. An older test explicitly expected policy loss; updated it to assert inherited initial policy plus independent toggles afterward. Focused border/positioning tests pass16/16, and the full module suite passes370 tests across71 result groups. Corrected deferred lifecycle passes640/640 pixels, fixing all8 former clone failures; all640 full image pairs match static references. A formerly failing clone sheet was visually inspected, with4/640 direct/identical-pair coverage; full review remains pending. Old failure artifacts are retained. The frozen image-clip-origin probe predates this runtime correction and must be refreshed for subsequent broad runtime qualification. Receipt: `output/playwright/html-to-riv/clone-css-paint-order-fix-receipt.json`.


### Border full regression and sizing review — 2026-09-10

Frozen the rebuilt compiler/probe after the CSS paint-order clone correction in `border-clone-fixed-toolchain`. Full Chrome/native regression started with7180 checks, session80193, isolated `border-v22-full-native` review and `border-v22-full-artifacts` outputs; terminal result and full visual audit remain pending. Deferred static border sizing review now39/240 views (24 direct plus15 exact full-image transfers), audited after inspecting column border-box auto/basis-auto/basis-percent/basis-px at all three widths. No visible discrepancies in those sheets. Lifecycle640/640 pixels remain passing with incomplete visual coverage. Run handle/manifest receipt: `output/playwright/html-to-riv/border-v22-full-run.json`.


### Border column sizing visual review — 2026-09-10

Deferred border sizing visual review now93/240 views:60 direct and33 audited exact full-image transfers. Newly inspected forward columns cover fixed/maximum/percentage/zero dimensions and content-box flex-basis; reversed columns cover auto/basis-auto/basis-px. All three widths agree visually with Chrome, including zero-size child overflow and reversed bottom anchoring. `record-visual-review.py --audit` passes;147 views remain. Full7180-check regression session80193 was polled live; completion remains pending.


### Border reversed-column and row review — 2026-09-10

Deferred border sizing review now147/240 views:96 directly inspected and51 audited exact full-image transfers. Reversed-column border-box fixed/maximum/percentage/zero and content-box basis/fixed/maximum/percentage cases match Chrome, including bottom-edge child overflow. Initial horizontal auto/basis-auto/basis-percent cases also match at240/390/768 widths. Review audit passes;93 views remain. Full regression session80193 remains pending after live polling.


### Border sizing matrix visual completion — 2026-09-10

Deferred border sizing matrix now has complete240/240 static native visual coverage (162 direct, 78 exact paired-image transfers); review audit passes. All240 vector views transfer with exact compiler-input and full browser/native PNG identity. Dedicated public lifecycle audit verifies source HTML/CSS, identical Rive bytes/manifests, stream hashes and full PNG pairs before transferring all640 original/clone resize frames from the completed static review. No new direct inspection is claimed for these transfers. Historical clone failures remain preserved. Full7180-check regression session80193 is still live, and other border qualification work remains. Receipt: `output/playwright/html-to-riv/border-deferred-visual-completion.json`.


### Border signed clipping review — 2026-09-10

Public signed border clip-margin matrix visual coverage reaches33/54 views (27 direct, 6 exact full-image transfers). Inspected border-origin negative/zero/positive margins under both box-sizing modes and content-origin margins under border-box sizing. Border exposure, orange padding, teal child clipping, responsive right remainder and green sibling overlap agree with Chrome; sparse curve antialias differences are retained. Review audit passes. Full regression session80193 remains pending after live polling; remaining matrix reviews and border lifecycle compositions remain open. Evidence: `output/playwright/html-to-riv/border-v22-clip-margin-native/visual-inspection.json`.


### Border clip-margin review complete and lifecycle regression — 2026-09-10

Signed border clip-margin static matrix completes54/54 audited visual coverage (48 direct, 6 exact paired-image transfers). Final content/padding-origin cases match Chrome structural geometry and clipping with retained curved-edge antialias differences. Added permanent public_border_signed_clip_margin_lifecycle_matches_chrome using the18-scene pinned Chrome oracle; targeted test passes and records144 original/clone240/390/768/240 updates. Pixel replay launched in `border-signed-margin-lifecycle`; terminal result and lifecycle review transfer remain pending. Full regression session80193 remains pending. Static review: `output/playwright/html-to-riv/border-v22-clip-margin-native/visual-inspection.json`; targeted log: `output/playwright/html-to-riv/border-signed-margin-recording.log`.


### Border signed lifecycle complete and transparency review — 2026-09-10

Signed border clip-margin lifecycle completes144/144 geometry/pixel frames with90 identical repeats across54 states. Public artifact/source/full-PNG audit transfers complete visual coverage from the reviewed54 static views; no direct lifecycle inspection is implied. Receipt: `output/playwright/html-to-riv/border-signed-margin-completion.json`. Public border transparency review now33/72 views; transparent and half-alpha square rings inspected with all background/child combinations, matching Chrome colors, border geometry and clipping. Review audit passes. Remaining transparency/plain/solid reviews, text/image lifecycle and full regression qualification remain open.


### Border transparency visual completion — 2026-09-10

Public border transparency matrix completes72/72 visual coverage (66 direct,6 exact-image transfers), audit passes. Transparent/half-alpha borders at radii0/18/90 with background and child combinations match Chrome clipping, color blending, normalized corners and responsive paint. Sparse curve antialias differences remain recorded. Opaque border coverage is in the separate solid/plain corpora, still pending full review. Lifecycle paint frames remain part of the combined96-scene public recording and require audited review transfer once its other source corpora are complete. Full7180-check regression session80193 remains pending. Receipt: `output/playwright/html-to-riv/border-paint-visual-completion.json`.


### Plain border width and corner review — 2026-09-10

Plain public border visual coverage reaches30/54 views, audit passes. Inspected one-pixel opaque/half-alpha rings at radii0/18/90 and24px opaque borders. Border continuity, normalized contours and inner corner collapse when border exceeds radius match Chrome; curved-edge antialias intensity differences are retained. Full regression session80193 was polled live, with log progress past2765 checks; no full result claimed. Remaining plain/solid review and text/image lifecycle qualification stay open. Evidence: `output/playwright/html-to-riv/border-v22-plain-native/visual-inspection.json`.


### Plain border visual completion — 2026-09-10

Plain public border matrix completes54/54 directly inspected views, review audit passes. Widths1/8/24, radii0/18/90 and opaque/half-alpha cover continuous thin strokes, uniform translucent rings, normalized curves and square inner corners when border exceeds radius. Curved-edge antialias intensity differences remain documented; no tolerances changed. Combined solid-border source review remains outstanding before complete96-scene lifecycle review transfer. Text/image lifecycle and full renderer regression qualification remain open. Receipt: `output/playwright/html-to-riv/border-plain-visual-completion.json`.


### Solid border composition review — 2026-09-10

Solid public border composition review reaches40/162 views (18 direct,22 exact paired-image transfers), audit passes. Initial thin-border square and rounded clipped/visible controls match Chrome child overflow, sibling occlusion, content-box extents and responsive orange remainder. Curve antialias differences remain retained. Other composition views, combined lifecycle visual transfer, text/image lifecycle and full renderer regression remain open; session80193 polled live. Evidence: `output/playwright/html-to-riv/border-v22-solid-native/visual-inspection.json`.


### Thick border composition review — 2026-09-10

Solid public border composition review reaches70/162 views (36 direct,34 exact paired-image transfers); audit passes. Added large-radius thin content-box and thick square/rounded border cases in clipped/visible modes. Inner clipping, purple border coverage, child overflow and responsive orange remainder match Chrome; sparse curve antialias differences remain retained. Full regression80193 polled live; remaining solid views, combined lifecycle review and text/image lifecycle remain open. Evidence: `output/playwright/html-to-riv/border-v22-solid-native/visual-inspection.json`.

### Thick rounded border composition review — 2026-09-10

Solid border visual coverage is now 97/162 views, with 65 remaining; `border-v22-solid-native/visual-inspection.json` passes its image/sheet hash audit. Six additional original-resolution sheets cover 24px borders with radius18 (border-box visible; content-box clip/visible) and radius90 (border-box clip/visible; content-box clip), each at240/390/768. Outer/inner contour normalization, square inner corners when width exceeds radius, child clipping/visible overlap and sibling placement agree with Chrome. Sparse curve antialias differences remain under unchanged criteria. Exact-image transfers are recorded separately from direct inspection. Full7180 native regression remains live (session80193 confirmed by polling), with progress beyond4650; terminal completion and broad visual audit remain outstanding.

### Medium border composition review — 2026-09-10

Solid border visual coverage is now 136/162 views (26 remaining), with the review receipt passing its image/sheet hash audit. Nine additional original-resolution sheets cover 24px/radius90/content-box visible overflow, all four 8px/radius0 box-sizing/overflow combinations, and all four 8px/radius18 combinations; each includes240/390/768 widths. Border thickness, inner clips, rounded contours, orange remainder and child/sibling occlusion match Chrome structurally. Sparse curve antialias differences remain under unchanged criteria. Direct reviews and exact-image transfers remain separately recorded in `border-v22-solid-native/visual-inspection.json`. Full7180 regression was confirmed live by session80193 polling and has advanced beyond4950 checks; no terminal pass or broad visual qualification claimed.

### Solid border static visual review complete — 2026-09-10

All54 solid-border composition scenes /162 views now have audited visual coverage: 108 directly inspected and 54 exact-image transfers. Final sheets cover large-radius8px borders in both box-sizing modes with clip/visible overflow, plus remaining1px visible-overflow cases. Outer/inner contours, background extents, child clipping and sibling occlusion match Chrome structurally. Thin and large-radius contours retain documented antialias intensity differences under unchanged criteria. Receipt: `output/playwright/html-to-riv/border-solid-visual-completion.json`. This completes the static matrix only; combined lifecycle audit, text/image lifecycle coverage and the full regression audit remain outstanding. P01 remains in progress.

### Combined border lifecycle review and current replay — 2026-09-10

The public lifecycle audit now accepts multiple independently complete static references while rejecting ambiguous case identities. The existing96-scene combined border recording has768/768 frames audited against the solid/paint/plain references with exact HTML/CSS, RIV, requirements, stream hashes and both PNGs; receipt is `border-v22-public-lifecycle/public-visual-transfer.json`. Single-source signed-margin audit still passes144 frames. Negative checks reject changed HTML, stream/image hashes, missing frames and duplicate sources (`border-multisource-audit-negative-tests.json`). Current runtime recording was regenerated by the public combined geometry regression, which passes; fresh768-frame pixel replay is running as session88803 in `border-combined-clone-fixed-lifecycle`, using the frozen clone-fixed renderer. Historical success does not imply this fresh run has completed. Text/image lifecycle and broad regression qualification remain outstanding.

### Current combined lifecycle complete; text/image recording added — 2026-09-10

The current clone-fixed combined border run passes768/768 pixel comparisons and all768 frames pass the multi-source public artifact/visual audit. Receipt: `border-combined-clone-fixed-lifecycle/public-visual-transfer.json`. Added24-scene text/image original-and-clone regression using pinned Inter, CSS shaping precision/normal-wrap policies and native glyph recording under the native-glyph-controls feature. All192 geometry updates pass. Browser lifecycle replay now loads the same pinned font and waits for fonts; public audit checks the complete font/image asset map. The192-frame text/image pixel replay is running as session90133 in `border-content-clone-fixed-lifecycle`; no visual completion claimed yet. Text/image oracle preserves the original Chrome measurements with explicit border metadata. Full regression qualification remains outstanding.

### Text/image border lifecycle complete — 2026-09-10

All24 text/image scenes pass192 original-and-clone resize pixel comparisons and all192 frames pass the exact source, complete assets, RIV, requirements, stream and full-PNG visual audit. The native-glyph-controls border runtime test file passes9/9 tests. Receipt: `border-content-lifecycle-completion.json`. The support summary and P01 status now reflect completed focused static/lifecycle review; broad qualification still awaits the frozen7180-check full regression and visual audit.

### Version22 full regression passes — 2026-09-10

Frozen version22 full native regression exited0 with7180/7180 checks passing in31.1 minutes. Verified terminal handle80193, completed review status/check count and all four frozen toolchain file hashes. `border-v22-full-run.json` records terminal success and the review hash; qualificationComplete remains false. Exact compiler-input/full-PNG visual transfer audit is running as session96312 against the completed v21 full and regular-suite references; output `border-v22-full-visual-audit.log`. Separately, text/image audit negative checks reject modified font bytes, font weight and unexpected assets (`border-content-asset-audit-negative-tests.json`). Broad visual qualification is not inferred from numerical pass.

### P01 qualified; P02 browser baseline established — 2026-09-10

P01 is qualified for the documented version22 native-renderer scope: full7180 checks and7170/7170 exact compiler-input/full-PNG visual transfers pass, alongside completed focused static/lifecycle reviews and public parity. Receipt: `border-p01-receipt.json`; curved-edge antialias differences remain documented under unchanged criteria. P02 begins with24 individual-side scenes /72 Chrome153 views covering unequal widths, zero-width sides, distinct opaque/translucent colors, rounded joins and both box-sizing modes. All24 current compiler rejections are preserved in `border-sides-admission/admission.json`; individual sides are not yet supported. Runtime currently paints one ring color although solved border widths are per-side; independent colors need a checked transport and non-overlapping corner paint construction before qualification.

### P02 physical-value parser foundation — 2026-09-10

Added tested one-to-four-value expansion in CSS top/right/bottom/left order for widths, styles and colors. Functional colors remain single components; malformed lists, excess values, delimiters and unsupported widths reject. Shared single-color parsing now serves the existing uniform cascade. All12 border unit tests pass; initial missing-parser failure is preserved. Evidence: `border-sides-parser-progress.json`, red/green logs. This helper is not public side admission: cascade per-side state, versioned host transport, independent corner-color rendering, resize/clone pixels and full parity/qualification remain to implement. Existing P01 evidence refers to its frozen qualified toolchain.

### P02 physical border state — 2026-09-10

Replaced the uniform computed border field with four physical values in CSS top/right/bottom/left order. Final-font width resolution, CSS-wide component inheritance, uniform shorthand/reset assignment and schema edge emission now operate per edge. Version22 emission explicitly rejects nonuniform state until the per-side transport is implemented; public syntax admission remains unchanged. Initial library32 tests pass (`border-side-state-tests.log`). Added an unequal-edge inheritance/reset regression; the full module run is active as session38282 (`border-side-state-module-tests.log`) and requires terminal verification. Artifact equivalence to the frozen P01 compiler remains to check before claiming this refactor preserves qualified output.

### P02 state refactor verified — 2026-09-10

The complete module test run exited0: 374 tests across71 result groups pass. The additional unequal-edge inheritance/reset regression passes in a separate targeted run. Recompiled all242 focused P01 scenes with the current compiler; RIV, source-map and runtime-requirements bytes match the frozen reviewed artifacts exactly. Evidence: `border-side-state-completion.json` and `border-side-state-equivalence/receipt.json`. This proves existing focused output preservation through the physical-edge state refactor; it does not qualify P02 rendering or native/WASM parity for forthcoming side syntax. Next implement per-side cascade declarations and versioned runtime colors.

### P02 side cascade implementation — 2026-09-10

Added physical side shorthand/component application, CSS-wide side inheritance/reset and one-to-four-value uniform longhand expansion into per-edge state. Selected-edge overrides preserve other edges; uniform shorthand resets every edge. Substitution grammar dispatch recognizes side properties through their corresponding border component. Library35 tests pass, including selected-edge overrides, shorthand reset, inheritance and invalid negative-width variable reset (`border-side-cascade-tests.log`). Nonuniform computed borders still reject at version22 emission until runtime side-color transport exists. Additional substitution/cardinality tests, public diagnostics, parity and runtime/pixel coverage remain; this is not P02 qualification. Repeated uniform-value syntax may now resolve to the existing uniform transport, but expanded public syntax has not yet received the complete qualification gate.

### P02 invalid substituted component lists fixed — 2026-09-10

A new regression exposed incorrect errors for variables containing multiple values in a single physical side component. The top-level component-count check now runs before normalization as well as in substitution validity, so invalid width/style/color lists reset only that component. Function arguments remain one component; valid unsupported single calc(), dashed and oklch() values retain diagnostics. Preserved both failing stages: initial scalar-width error and normalization error for functional color plus extra value. Library36 tests now pass (`border-side-cardinality-green-r2.log`); receipt `border-side-cardinality-receipt.json`. Runtime transport and corner paint remain outstanding.

### P02 candidate side partition geometry — 2026-09-10

Added an experimental runtime helper constructing four side clip quadrilaterals for the shared rounded border ring. It uses solved unequal widths, validates finite/nonnegative inputs and collapses overfull inner dimensions proportionally to avoid crossed polygons. The geometry test checks positive orientation and total square-ring area for unequal/zero/overfull edges, plus invalid inputs; it passes. Existing css_clip_path tests also pass (`border-side-partition-clip-tests.log`). Inspected the Chrome390px translucent large-radius content-box reference: curved color transitions remain a pixel-comparison requirement. The helper is deliberately not wired into paint yet; area checks do not establish seam-free antialiasing or Chrome-equivalent rounded/overfull joins. Next wire diagnostic per-side paint and compare the full72-view reference matrix before public capability admission.

### P02 diagnostic per-side paint path — 2026-09-10

Added experimental per-side runtime colors in top/right/bottom/left order, cloned with the layout occurrence. The ordinary checked uniform installer clears the experimental override. Distinct colors draw the shared even-odd border ring through side clip paths with saved/restored renderer state; equal colors retain a single ring draw. This path has no public capability admission yet. Initial compile error from an incomplete edit is preserved in `border-side-diagnostic-render-build.log`; corrected runtime geometry build is running as session72616 (`border-side-diagnostic-render-build-r2.log`). Next extend the diagnostic recorder with side widths/colors and compare Chrome72 views. Neither seam-free rasterization nor side-paint lifecycle has been verified.

### P02 diagnostic recording pipeline — 2026-09-10

Corrected runtime side-paint build passed6 clipping geometry tests (`border-side-diagnostic-render-build-r2.log`). Extended the existing lifecycle recorder with explicit diagnostic compiler CSS, top/right/bottom/left widths and colors, per-edge schema injection and experimental paint installation. Prepared24-scene pinned Chrome oracle metadata from the untouched measurements. Diagnostic original/clone240/390/768/240 geometry recording is running as session6278 into `border-sides-diagnostic-recording`; log `border-sides-diagnostic-recording.log`. Replay provenance now preserves injected side widths/colors and remains runtime-experiment-only. No public P02 admission or pixel pass is claimed. On terminal geometry success, run clip-margin-lifecycle.mjs with this recording, the frozen border-clone-fixed renderer and a fresh border-sides-diagnostic-lifecycle output.

P02 diagnostic recording session6278 subsequently exited0: all192 geometry updates pass. The fresh192-frame Chrome/native pixel replay has started in `border-sides-diagnostic-lifecycle`; pixels and visual review remain outstanding.

### P02 first pixel failure: rounded inner-corner pockets — 2026-09-10

Initial side-paint replay finished with60/192 pixel failures (geometry had passed192/192). Direct inspection of the translucent radius90 content-box390px pair shows unpainted orange corner regions absent in Chrome: quadrilateral side clips stop at inner rectangular corners and omit portions of the rounded ring. Failure images and `border-sides-diagnostic-lifecycle/failure-review.json` are preserved. Candidate correction extends each partition via the inner center to cover the outer box; the shared ring removes the interior. Updated area invariant checks total outer-box coverage. New192-frame recording build is running as session29555 in `border-sides-pocket-fixed-recording`; log `border-sides-pocket-fixed-recording.log`. Geometry and pixel results for the correction remain unverified; no tolerance changes or public admission.

### P02 miter direction correction — 2026-09-10

The center-extension replay finished with12/192 pixel failures, down from60; the original and corrected failures remain preserved. Directly inspected the prior translucent radius90/content-box390 case: missing paint is fixed but corner color boundaries shift because clip edges turn toward the inner center. Chromium border painter implementation provides the relevant geometric approach: extend outer-to-inner miter rays to the chord between inner arc endpoints (reference: https://chromium.googlesource.com/chromium/blink/+/refs/heads/main/Source/core/paint/BoxBorderPainter.cpp). This is implementation guidance, not proof of pinned Chrome153 equivalence. Candidate runtime correction now normalizes radii, computes inner radii and preserves each miter ray; fresh geometry recording build session42071 targets `border-sides-miter-ray-recording`. Pixel replay and targeted seam review remain required. No public admission or tolerance changes.

### P02 miter-ray replay passes — 2026-09-10

Miter-ray recording geometry passes192 updates and pixel replay session73237 exits0:192/192 frames pass with unchanged criteria. Four original-resolution Chrome/native pairs inspected, including the previously defective translucent large-radius case, zero-top-width large radius, and opaque/translucent square joins. Missing paint and shifted joins are corrected in those inspected cases. Partial receipt: `border-sides-miter-ray-lifecycle/selected-image-review.json`; complete visual coverage is still pending. Added a focused runtime regression checking ray collinearity and the inner-arc chord endpoint; geometry suite currently building as session34717 (`border-side-miter-geometry-tests.log`). Public capability transport, broader edges and full P02 qualification remain outstanding.

### P02 structured visual review begins — 2026-09-10

All7 runtime clipping/miter tests pass. Created a72-view projection of original frames0/1/2 from the192-frame diagnostic replay, preserving original frame names and source replay hash; every projected row is verified equal to its source. Three full-resolution three-width sheets inspected: translucent unequal-width radius90 content-box, opaque zero-top-width radius90 border-box, and translucent square content-box. Visual receipt audited9/72 distinct views; exact full-image matches cover24/192 lifecycle frames. Evidence: `border-sides-miter-ray-static-review/visual-inspection.json` and `lifecycle-review-progress.json`. All remain runtime-experiment-only; remaining63 distinct views need review, and public capability integration remains open.

### P02 translucent zero-edge visual review — 2026-09-10

Six further full-resolution three-width sheets reviewed: zero-top-width translucent borders at0/18/90 radius in both box-sizing modes. Visual receipt now27/72 distinct views, with72/192 lifecycle frames matched by audited full-image identity;45 distinct views remain. Exposed top background, unequal side thickness, thin bottom, rounded color transitions, child clip and sibling placement match structurally. Sparse curve antialias differences retained under unchanged criteria. Projection row identity and source hash verified before updating `border-sides-miter-ray-static-review/lifecycle-review-progress.json`. This remains diagnostic evidence, not public compiler qualification.

### P02 opaque zero-edge visual review — 2026-09-10

Reviewed three further original-resolution three-width sheets: opaque zero-top-width square borders in both box-sizing modes and radius18 border-box. Brown/green/blue side colors, exposed top background, child clipping and sibling placement match structurally; small diagonal/curve antialias differences remain. Visual audit now36/72 distinct views and96/192 exact-image lifecycle frames;36 distinct views remain. Projection rows and source hash were revalidated for `border-sides-miter-ray-static-review/lifecycle-review-progress.json`. Public compiler capability integration remains outstanding.

### P02 unequal translucent side review — 2026-09-10

Six additional three-width sheets reviewed: opaque zero-top-width radius18/90 content-box and unequal four-side translucent square/radius18/radius90 combinations. Thin top and asymmetric side colors, flattened inner corners, child clips, background remainder and sibling position match structurally; sparse antialias differences retained under unchanged gates. Audit now54/72 distinct views and144/192 exact-image lifecycle frames;18 distinct views remain. Source projection identity and hashes revalidated. Evidence remains diagnostic-only; public capability transport and qualification are outstanding.


### P02 diagnostic visual audit complete — 2026-09-10

All72 distinct three-width Chrome/native/difference views are directly reviewed.
The final six opaque unequal-width sheets cover square,18px and constrained90px
radii in both box models. No missing corner pockets or shifted color boundaries
were observed; sparse curve/miter antialias differences remain under unchanged
gates. Source replay hashes and every projected row were revalidated, and all192
original/clone frames match reviewed full-image pairs. The receipt
`output/playwright/html-to-riv/border-sides-diagnostic-completion.json` binds the
oracle, recordings, replay, reviews and seven-test geometry log. This is runtime
experiment evidence only. Public per-side transport/emission, expanded edges,
parity and full regression qualification are the next work.


### P02 checked runtime installer — 2026-09-10

`Artboard::set_css_border_sides_occurrence` now atomically replaces border
policies using top/right/bottom/left ARGB colors. Uniform installation delegates
to the same validator and replacement path. Both lifecycle tests pass: invalid
roots and target IDs leave prior paint intact; plain-layout proxies survive
resize, clone, clear and repeated reinstallation; switching to uniform paint
clears side overrides without changing the clone. Receipt:
`output/playwright/html-to-riv/border-sides-checked-installer-receipt.json`.
Compiler capability/manifest emission and public pixel/parity qualification
remain outstanding; these tests establish runtime installation behavior only.


### P02 version23 compiler contract candidate — 2026-09-10

Public emission now represents nonuniform used widths/colors with
`layout-css-border-sides-v1` and `layout_border_sides` entries containing physical
top/right/bottom/left ARGB arrays. Uniform used state retains version22 and its
existing bytes. The host probe combines both target lists into one atomic
installation; manifest validation rejects overlap, root IDs, malformed arrays,
missing capabilities and wrong versions. Five border contract tests and
TypeScript checks pass. Public rendering, native/WASM parity and expanded edge
qualification remain pending.

The first full module run exposed a default-stack overflow at the resource-limit
regression. It reproduced in isolation. Moving per-side calculation out of the
recursive emission frame fixes that regression without increasing stack limits.
Logs: `border-sides-v23-stack-repro.log`, `border-sides-v23-stack-fix.log`.
The corrected full run writes `border-sides-v23-module-tests-r2.log`; completion
is not yet established. This remains a candidate, not P02 qualification.


### P02 public focused validation — 2026-09-10

Corrected full Rust suite passes382 tests across70 groups. The added public
individual-side recorder compiles the original24 authored scenes and installs
only their version23 manifest policies; all192 original/clone geometry updates
and192 Chrome/native pixel comparisons pass. The frozen toolchain reproduces
each recorded Rive artifact and requirements. The dedicated audit checks exact
authored HTML/CSS, full PNG pairs, streams and review hashes against the fully
reviewed diagnostic corpus:192/192 public frames have complete visual coverage.
This transfer does not treat diagnostic injection as public compilation.

Expanded native/WASM corpus parity passes all13 JavaScript tests, and TypeScript
passes. Receipt: `output/playwright/html-to-riv/border-sides-v23-focused-receipt.json`.
P02 remains active: broader edge/composition fixtures, malformed-host transport
checks and full native visual regression are outstanding.


### P02 edge corpus and host checks — 2026-09-10

All21 host tests pass, including version23 mixed uniform/side policies and17
malformed-manifest rejection controls. Five negative visual-audit controls reject
changed Rive bytes, authored CSS, stream hashes, image hashes and missing frames.
Added40 Chrome edge scenes: transparent sides, none/hidden, currentColor, two/three
value lists, shorthand ordering, variables/invalid substitution, fractional em
widths and borders exceeding authored dimensions, at square/large radius in both
box models. Public original/clone geometry and pixel replay pass320/320.

Visual review has begun:6/120 distinct views reviewed (overfull radius90 in both
box models). Used-border minimum sizing and colored wedge geometry agree with
Chrome; sparse curve/diagonal antialias differences remain.114 distinct views
remain unreviewed; expanded corpus parity and full regression are outstanding.
Receipt: `output/playwright/html-to-riv/border-sides-v23-edge-progress.json`.


### P02 edge parity and transparent/hidden review — 2026-09-10

The40 edge scenes are now included in the native/WASM corpus; all13 JavaScript
tests pass against the frozen version23 toolchain. Eight additional original-
resolution sheets (transparent top and none/hidden sides, square/large radius,
both box models) are directly reviewed. Space retention/removal, joins, child
clipping and sibling placement match structurally, with sparse antialias
differences under unchanged gates. Visual audit now30/120 distinct views; exact
full-image identity covers80/320 lifecycle frames.90 distinct views remain.
Projection/source hashes and rows were verified before writing
`border-sides-v23-edge-static-review/lifecycle-review-progress.json`.


### P02 equal-color seam found by visual review — 2026-09-10

Direct currentColor square-border review found a light diagonal join absent
from Chrome despite passing aggregate metrics. Failure receipt:
`border-sides-v23-edge-static-review/current-color-seam-review.json`. Started
7372-check full regression session98289 was interrupted (exit130); its frozen
pre-fix artifacts are preserved and cannot qualify the correction.

Runtime now combines all partitions with the same ARGB color into one NonZero
clip before painting the ring. This removes internal antialias seams. New
regression verifies paint grouping through clone, clear and uniform replacement.
Corrected edge recording and replay pass320/320 frames. Directly inspected the
corrected square currentColor390px native image; the light purple diagonal seam
is gone, and browser bytes match the previously viewed reference. Other changed
images still need review; prior edge review is not automatically transferred.
Receipt: `output/playwright/html-to-riv/border-sides-grouped-paint-receipt.json`.


### P02 corrected currentColor review and frozen full run — 2026-09-10

All12 currentColor views are directly inspected in the corrected replay: square
equal-color joins no longer show light seams; rounded edges, translucent right
border, child clips and sibling placement agree structurally. Partial-source
review transfer checks exact authored HTML/CSS, manifest, Rive bytes and both
full PNGs, plus projection/source hashes:18 earlier views remain identical.
Corrected coverage is30/120, with90 still requiring review; changed none/hidden
images were not transferred. Receipts are in
`border-sides-grouped-edge-static-review/{visual-inspection,prior-review-transfer}.json`.

Rebuilt/frozen `border-sides-grouped-toolchain`; all21 host-contract tests pass.
The corrected7372-check full native run is live as session65603; status receipt
`output/playwright/html-to-riv/border-sides-grouped-full-run.json`. Completion and
full visual audit remain pending.


### P02 corrected none/hidden and two-value style review — 2026-09-10

Eight full-resolution three-width sheets inspected: corrected none/hidden
physical edges and two-value none/solid styles, square/large radius in both box
models. Removed edge space, blue side crescents, red/green joins, child clips
and sibling placement match structurally; sparse curved and different-color
join antialias differences remain. Combined corrected coverage is54/120
(36 direct plus18 audited exact-input/artifact/full-image transfers).66 views
remain. Prior-transfer input, artifact and image hashes were revalidated.
Full regression session65603 was polled live and remains in progress.


### P02 corrected three-value list review — 2026-09-10

Twelve three-value width/color views directly inspected: thin red top, repeated
green sides and thick blue bottom match Chrome at all widths in both box models,
with square and large-radius joins. Child clips, background and sibling placement
agree structurally; sparse curve/different-color diagonal antialias differences
remain. Corrected coverage is66/120 (48 direct,18 transferred);54 remain.

`validation/audit-border-edge-review-progress.py` now reproduces the combined
partial audit, checking direct-review receipts, projection rows/source hashes,
public recording hashes, exact authored HTML/CSS/manifests/Rive bytes and full
image pairs. It does not transfer changed or unreviewed source rows.
Full regression remains underway; latest log observation passed check449.


### P02 corrected shorthand and variable review — 2026-09-10

Eight original-resolution sheets inspected for shorthand ordering and variable
width/color lists, at square/large radius in both box models. Later shorthand
reset, final bottom override, physical edge order, unequal widths, child clips
and sibling layout match Chrome structurally. Equal-red joins remain seamless;
sparse different-color/curve antialias differences persist under unchanged gates.
Combined corrected audit passes90/120 views (72 direct,18 exact-input/artifact/
image transfers), leaving30. Full native session65603 remains live.


### P02 corrected edge visual audit complete — 2026-09-10

All120 corrected edge views are visually accounted for:102 directly inspected
and18 transferred only after exact authored input, manifest, Rive artifact and
full-image identity checks. Final inspection covers invalid-variable medium
reset, fractional em widths and square overfull borders in both box models.
No equal-color seams remain in the reviewed cases; sparse different-color join
and curve antialias differences remain under unchanged criteria.

The frozen compiler reproduces all40 public recorded Rive artifacts and
requirements. All320 original/clone frame images match reviewed pairs and their
current hashes. Receipt: `output/playwright/html-to-riv/border-sides-grouped-edge-completion.json`.
Full regression session65603 remains live. Broader text/image/flex compositions,
corrected runtime regression completion and full visual audit remain open.


### P02 text/image and clipping compositions — 2026-09-10

All14 corrected border runtime tests pass. Added48 composition scenes derived
from the P01 text/image and axis/clip-margin cases, with unequal2/8/14/20 widths
and repeated translucent side colors. Pinned Chrome captures144 references.
Public compilation and checked installation pass384 original/clone geometry
updates. Native-glyph recording is complete; pixel replay session31879 and
expanded native/WASM parity session1072 are running. Visual review has not yet
begun for these compositions. Full regression session65603 remains active.
Receipt: `output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`.


### P02 composition replay passes; text/image review — 2026-09-10

Composition replay exits0 with384/384 geometry/pixel frames passing. Expanded
native/WASM parity passes13/13 tests. Eight original-resolution three-width
text/image sheets inspected for square/24px radius and both box models: matching
wrapping, intrinsic heights, image quadrants, translucent border blends, clipping
and sibling position. Equal-color corners remain seamless; sparse glyph/curve
and image filtering differences retained under unchanged gates.

Visual audit covers66/144 distinct views (24 direct,42 exact within-run
image transfers), with78 remaining. Receipt:
`output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`.
Full regression remains pending.

### Individual-side composition lifecycle audit — 2026-09-10

The focused48-scene composition corpus now has complete384-frame visual coverage across original/clone240/390/768/240 resizing. `validation/audit-border-composition-review.py` verifies projection provenance, complete frame sequences, stream and PNG hashes, exact reviewed image pairs, and fresh frozen-compiler reproduction of all48 RIV files and runtime requirements with Inter/quadrant assets. Both corrupted-RIV and corrupted-requirements negative controls reject at the intended artifact comparison. Static coverage is66 directly inspected views plus78 exact-image transfers. Geometry/pixels384/384 and native/WASM parity13/13 pass. Receipt: `output/playwright/html-to-riv/border-sides-grouped-composition-lifecycle/public-composition-audit.json`; negative controls: `output/playwright/html-to-riv/border-sides-composition-audit-negative-tests.json`. Full7372 regression and its broad visual audit remain pending; P02 remains unqualified.

### Full-gallery border lifecycle references

`python3 tools/html-to-riv/validation/border-gallery-reference.py output/playwright/html-to-riv` revalidates the completed corrected initial and edge lifecycle reviews, including source review transfers, recorded input/requirements identity and full image hashes. The source-only check currently verifies192 distinct border views.

After the full gallery finishes, add its directory as the second argument. The target audit requires a passed gallery, exact authored compiler request, RIV bytes, runtime requirements and matching complete Chrome/native PNGs for each border view. It records changed image pairs as remaining in `border-lifecycle-comparison.json`; it does not claim new visual inspection. The completed-gallery branch awaits the active full run and is not yet validated on that output. Combine its coverage with the ordinary prior-baseline audit before claiming full visual qualification.

### P02 native qualification complete — 2026-09-10

Full regression7372/7372 and visual coverage7362/7362 pass. The baseline comparison covers7170 unchanged pairs; the independently audited border lifecycle comparison covers all192 remaining pairs with exact public inputs, artifacts and both PNGs. Combined receipt preserves both component proofs. Focused matrices cover896 original/clone frames; module398, host21 and native/WASM13 pass. P02 is native-qualified for the documented physical solid/none/hidden border profile. Per-corner radii P03 is next; other renderer/style limitations and realistic composition expansion remain explicit. Receipt: `output/playwright/html-to-riv/border-p02-receipt.json`.

### P03 composition lifecycle audit complete — 2026-09-10

All108 distinct composition views now have audited visual coverage:54 directly inspected and54 exact full-image transfers. The final six sheets cover expanded container clips, axis image clips and inset image clipping at240/390/768. Image extents, quadrant placement, asymmetric contours and sibling positions agree structurally; sparse curve antialias and right-edge image sampling differences remain under unchanged gates. The public audit reproduces all36 RIV/requirements artifacts with complete assets and verifies288 original/clone resize frames, stream hashes and reviewed PNG pairs. Receipt: `output/playwright/html-to-riv/corner-radii-composition-lifecycle/public-composition-audit.json`. Full regression40992 remains live; broad qualification is pending.

### P03 full-gallery source evidence revalidated — 2026-09-10

The gallery transfer auditor now accepts combined comparisons only after checking their component hashes, original successful rows, unique/full scene coverage and retained source receipt hashes. Revalidated all7362 P02 source pairs against complete compiler inputs and full browser/native PNG bytes. Negative controls reject a changed component hash, changed union image hash and omitted row. Evidence: `output/playwright/html-to-riv/corner-radii-baseline-source-audit.json`. This prepares prior-scene transfer for the P03 full regression; it does not claim the still-running target passed. Full session40992 remains live, last logged3545/7444.

### P03 corner gallery references prepared — 2026-09-10

Added `validation/corner-gallery-reference.py`: source-only mode re-runs the public lifecycle audit, reproduces24 RIV/requirements artifacts and verifies192 reviewed frames before exposing72 initial-view references. Gallery mode reuses the exact public-input/artifact/full-PNG comparator. Source verification passes. A synthetic gallery control transfers one unchanged pair; changed RIV and requirements reject, while a changed native PNG remains explicitly unreviewed. Evidence: `output/playwright/html-to-riv/corner-radii-gallery-source-audit.log` and `corner-radii-gallery-audit-controls.json`. These are audit-tool checks, not full-target qualification. Full regression40992 remains confirmed live.

### P03 all-corner cascade tests and nested Chrome references — 2026-09-10

Public corner test suite passes5/5, including all four selected-corner initial/unset/missing/invalid-variable resets, em/rem/fractional resolution and matching-parent-corner inheritance. Log: `output/playwright/html-to-riv/corner-radii-all-corner-tests.log`. Added16 nested-flex scenes covering every isolated corner, fractional/oversized radii, inheritance and invalid-variable reset in both box-sizing modes. Pinned Chrome153 captures48 reference views successfully; runtime/pixel/parity/visual qualification is pending. Fixtures: `validation/corner-radii-nested-cases.json`; progress: `output/playwright/html-to-riv/corner-radii-nested-progress.json`. Updated stale implementation notes to distinguish completed focused evidence from remaining qualification.

### P03 nested public lifecycle geometry passes — 2026-09-10

Added the16-scene nested oracle to the permanent public runtime test and native/WASM parity corpus. Initial recording failed because missing borderColor metadata selected the harness historical default instead of the authored #76539a; corrected only the expected metadata and preserved the original log. Corrected public recording passes128 original/clone resize geometry comparisons. Pixel replay11084 and expanded parity16589 are running; full regression40992 remains confirmed live. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json` and `corner-radii-nested-recording-r2.log`. No nested pixel or visual qualification claimed yet.

### P03 nested pixels pass; isolated border-box corners reviewed — 2026-09-10

Nested geometry and native pixels pass128/128. Reviewed12/48 distinct views for all four isolated corners in border-box nested flex layouts; selected corner, inner clip, border contour, panel/aside sizing and sibling placement agree with Chrome. Sparse curve antialias differences remain. Expanded parity run passes10 tests including accepted-corpus native/WASM artifact equality; three host checks initially fail ENOENT because NUXIE_NATIVE_PROBE was omitted. Explicit frozen-probe rerun passes all three (no compiler/runtime changes). Original failure log retained alongside `corner-radii-nested-host-r2.log`. Visual audit remains36 views short; full regression40992 still live. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json`.

### P03 nested content-box and overlap review — 2026-09-10

Nested visual coverage reaches36/48 directly inspected views. All isolated content-box corners and fractional/oversized radii in both box models agree structurally with Chrome across240/390/768. Proportional overlap reduction follows changing nested flex width; inner clipping, background exposure, parent heights and sibling/aside placement match. Sparse contour antialias residuals retained under unchanged criteria. Twelve inheritance/variable-reset views remain, followed by the128-frame public artifact audit. Full regression40992 confirmed live, last logged5369/7444. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json`.

### P03 nested lifecycle qualification complete — 2026-09-10

All48 nested views directly inspected and hash-audited. Final inheritance and invalid-variable reset sheets match Chrome structurally in both box models; sparse curve antialias residuals retained. Public audit reproduces all16 compiled scenes/requirements and verifies all128 original/clone resize frames against reviewed PNG pairs and stream hashes. Together with initial192 and composition288, focused P03 coverage now totals608 frames across76 scenes. Native/WASM corpus equality and corrected host checks pass as documented. Full regression and broad target gallery audit remain pending; refreshed module suite launched. Receipt: `output/playwright/html-to-riv/corner-radii-nested-lifecycle/public-nested-audit.json`.

### P03 module suite complete; P04 references prepared — 2026-09-10

Refreshed module suite exits0: 396 tests across72 result groups pass (`corner-radii-final-module-tests.log`). P03 full visual regression remains pending. Prepared16 P04 elliptical/percentage scenes and48 pinned Chrome153 views; all16 frozen current-compiler rejections are preserved in `elliptical-radii-initial-admission.json`. Source inspection finds existing internal elliptical path construction but circular public geometry inputs; proposed live per-axis length/percentage occurrence contract and qualification sequence are documented in `validation/elliptical-radii-implementation-notes.md`. This is preparation only; no P04 syntax admitted.

### P04 elliptical border geometry foundation — 2026-09-10

Initial per-axis helper tests pass9/9. Added per-axis border ring and side partition entry points while retaining circular wrappers. New tests assert axis-specific inner contour offsets and miter-ray/chord intersections; replaced redundant wrapper equivalence checks with concrete expected geometry. Expanded geometry build53225 remains running (`elliptical-radii-border-geometry-tests.log`). No public P04 admission or pixel qualification yet. Frozen P03 full regression40992 remains confirmed live and reached7180/7444.

### P03 full regression passes; P04 responsive values foundation — 2026-09-10

Frozen P03 full regression40992 exits0:7444 checks pass,7434 rendered pairs. Baseline and new-corner visual audits have started; qualification remains pending those results. Receipt: `output/playwright/html-to-riv/corner-radii-full-run.json`. P04 elliptical border geometry tests pass10/10. Added checked unresolved pixel/percentage radius values with per-axis live dimension resolution and overflow rejection; tests16335 running. This value type is not yet installed on runtime occurrences or emitted by the compiler.

### P03 new gallery views audited; P04 value tests pass — 2026-09-10

All72 new corner gallery views pass exact public-input/RIV/requirements/full-image comparison to reviewed lifecycle evidence. Baseline comparison96393 remains confirmed running. Added reusable combined-gallery proof construction with completed-gallery identity, component row counts, conflict/missing-coverage rejection and retained hashes; positive union and missing-row controls pass. P04 responsive value tests pass2/2, covering independent width/height percentage resolution after resize/copy and invalid/overflow rejection. P04 remains unintegrated with runtime occurrences or compiler output.

### P03 circular corner radii native-qualified — 2026-09-10

Frozen full regression passes7444/7444; combined visual proof accounts for all7434 rendered pairs (7362 prior plus72 new), retaining both component comparisons. Fresh focused audits reproduce76 scenes and verify608 resize/clone frames; all228 distinct focused views have audited coverage. Default module suite396 passes; expanded parity10 plus corrected host3 checks pass, with the original omitted-probe failure preserved. P03 is native-qualified for the documented circular-length LTR profile against the frozen proportional toolchain. Curve antialias/image sampling, direction controls, vector and P04 limitations remain explicit. Receipt: `output/playwright/html-to-riv/corner-radii-p03-receipt.json`. Later P04 source changes are not covered by this qualification.

### P04 experimental occurrence integration — 2026-09-10

LayoutComponent now retains optional checked per-axis radius values, copies them on clone, and exposes experimental install/clear. Paint geometry resolves percentage axes from current layout width/height; background, border ring, side partitions and clip-margin paths consume common resolved pairs. Legacy circular callers retain equal-axis geometry. Added a resize/clone/clear regression asserting live path endpoints; test build26548 is running (`elliptical-radii-occurrence-tests.log`). This remains runtime experimentation: host contract, compiler admission, fractional browser discrimination and full pixel qualification are pending. P03 qualification continues to refer to its immutable frozen toolchain.

### P04 diagnostic browser lifecycle recorder — 2026-09-10

Occurrence/value tests pass3/3, including path endpoints after resize, clone and clear. Prepared16 diagnostic oracle cases with explicit pixel/percentage pairs and compiler CSS that omits unsupported radius declarations. Recorder installs experimental radius occurrence values after public base-scene compilation; evidence uses ellipse-diagnostic kind. Replay preserves injected pairs and actual compiler CSS and marks runtime-experiment-only, preventing confusion with public compiler qualification. Recording build16371 is running (`elliptical-radii-diagnostic-recording.log`). Public grammar/transport remain unsupported; geometry/pixel results pending.

### P04 first diagnostic pixels pass — 2026-09-10

Experimental ellipse recording and replay each pass128/128 original/clone frames. All16 cases cover percentages, slash lists, mixed pairs, zero axes and overlap in both box models. First6/48 distinct views inspected: border-box percentage ellipse and four-pair corners match Chrome structurally across240/390/768, retaining sparse contour antialias differences under unchanged gates. Evidence: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/replay.json` and `elliptical-radii-geometry-progress.json`. This is runtime-experiment-only; remaining visual coverage, public grammar/host transport/parity and expanded composition qualification remain pending.

### P04 content-box and overlapping percentage review — 2026-09-10

Diagnostic visual review reaches18/48 views. Content-box percentage and independent-pair ellipses, plus oversized percentage radii in both box models, match Chrome structurally at240/390/768. Border-box expansion, responsive overlap reduction, child clips and sibling positions agree; sparse contour antialias residuals remain under unchanged criteria. Runtime-experiment qualification only; public compiler/host work remains pending. Evidence: `output/playwright/html-to-riv/elliptical-radii-diagnostic-static-review/visual-inspection.json`.

### P04 mixed-unit and physical longhand review — 2026-09-10

Diagnostic visual coverage reaches30/48 directly inspected views. Mixed fixed/percentage axes and isolated top-left pairs match Chrome structurally in both box models across240/390/768. Responsive contours, inner clipping, exposed background and sibling placement agree; sparse contour antialias residuals retained under unchanged gates. Eighteen remaining views cover fixed ellipses, zero axes and unequal shorthand list lengths. Public compiler/host transport and expanded qualification remain pending. Evidence: `output/playwright/html-to-riv/elliptical-radii-geometry-progress.json`.

### P04 diagnostic visual lifecycle complete — 2026-09-10

All48 initial ellipse views directly inspected. Fixed ellipses, zero-axis square corners and independent shorthand list expansion agree structurally with Chrome in both box models; sparse contour antialias differences retained. The128-frame visual audit verifies source/projection identity, injected radius pairs, actual compiler CSS, complete original/clone resize sequences, stream hashes and reviewed PNG pairs. This is explicitly runtime-experiment-only and does not qualify public compilation/host transport. Receipt: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/diagnostic-visual-audit.json`. Public grammar/transport/parity and expanded fractional/composition evidence remain next.

### P04 specified-axis compiler parser — 2026-09-10

Added isolated corner-radii component parser preserving Pixels/Em/Rem/Percent values. Independent one-to-four slash lists expand in TL/TR/BR/BL order; physical longhands accept one or two axis values. Tests3/3 pass for distinct units, comments, zero, shorthand expansion and malformed/negative/nonfinite/unsupported rejection. CSS-wide cascade keywords remain handled outside this parser. Public style application and runtime transport are not yet integrated, so compiler admission remains unchanged. Evidence: `output/playwright/html-to-riv/elliptical-radii-parser-receipt.json`; source: `src/corner_radii.rs`.

### P04 axis pairs integrated with computed style — 2026-09-10

Computed style now stores four horizontal/vertical radius pairs and uses the new shorthand/longhand parser. Selected-corner inheritance copies both axes; final-font resolution converts em/rem while retaining percentage values. Existing circular values emit the established linked/unlinked scalar fields. Noncircular/percentage output rejects explicitly until checked transport exists. Library42 and public corner/custom-property56 tests pass. Evidence: `output/playwright/html-to-riv/elliptical-radii-style-receipt.json`. Equal-axis syntax may resolve to existing circular output; expanded public syntax qualification still awaits transport/parity/browser gates.

### P04 checked occurrence installer — 2026-09-10

Added Artboard::set_css_corner_radii_occurrence with all-target validation before mutation and replacement semantics that restore imported radii for omitted layouts. Regression passes for non-artboard roots, zero/missing/non-layout/duplicate IDs, no partial mutation, clone independence, clear and reinstall across240/390/768/240. Existing16-scene experimental ellipse Chrome geometry lifecycle also passes through this checked installer. Two focused tests pass; no new pixel replay or public compiler qualification claimed. Compiler transport, extreme percentage overflow behavior and expanded fractional/composition validation remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-installer-receipt.json`.

### P04 overflow-safe live corner resolution — 2026-09-10

Found a finite-value overflow path: percentage resolution could exceed f32 and hit the paint expect; large finite pixel pairs could overflow edge sums. Added f64 fallback resolution with one shared CSS overlap factor before narrowing, preserving ordinary representable inputs. Five runtime tests pass, including actual path endpoints after clone/resize and huge pixel/percentage controls; the existing16-scene checked-installer Chrome geometry lifecycle test also passes. Extreme inputs have analytic/runtime evidence only, with no new Chrome pixel claim. Public compiler transport and expanded visual qualification remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-overflow-receipt.json`.

### P04 public per-axis transport and initial pixels — 2026-09-10

Public compilation now emits version24 `layout-css-corner-radii-v1` with `layout_corner_radii` entries: unique layout object IDs and four TL/TR/BR/BL pairs, horizontal then vertical, each exactly `{pixels:number}` or `{percent:number}`. Finite nonnegative computed px/em/rem and percentage axes, slash-list expansion, physical longhands and existing cascade/reset rules are implemented. Circular pixel pairs retain legacy artifacts. Math and viewport units remain excluded. Checked native probe installation and TypeScript declarations are added; unsupported hosts reject before drawing. This is implemented, not yet fully qualified.

Focused border/corner public contracts14 tests pass; initial public recording and Chrome/native replay128/128 frames pass. Direct review9/48 views covers border-box fixed ellipses, percentage ellipses and four distinct corners. Full-resolution sheets are in `output/playwright/html-to-riv/elliptical-radii-public-static-review`. Native/WASM build/parity and remaining visual/public-artifact audit are pending.

Preserved failures: first recording compared JSON integer/float variants (fixed with typed comparison); r2 exposed30% becoming30.000002% (fixed by reading authored percentage points directly, with decimal/exponent/comment controls); module r1 exposed public border tests consuming diagnostic compilerCss (public tests now always compile authored CSS); r2/r3 exposed default-stack depth overflow (boxed retained computed styles fixes original resource-limit regression; added independent depth controls); r4 reached obsolete custom-property ellipse rejection (now checks public equivalence for newly accepted syntax). Initial TypeScript checks required updating the explicit version/capability assertions. Current full module/build sequence is running, so no full-suite or parity pass is claimed yet.

### P04 public transport tests complete — 2026-09-10

Default module409 tests across74 groups pass; native CLI/probe and WASM builds succeed. Expanded JavaScript/native-WASM suite13/13 passes including the initial16 ellipse corpus; targeted version23/24 host tests2/2 and TypeScript pass. Frozen compiler reproduces every recorded RIV/requirement from authored HTML/CSS (16/16). Public Chrome/native128-frame replay passes, with12/48 distinct views directly inspected and36 remaining. All associated processes are terminal. Receipt: `output/playwright/html-to-riv/elliptical-radii-public-transport-receipt.json`; immutable binaries: `elliptical-radii-public-toolchain`. P04 remains active pending full visual audit, fractional/composition expansion and full native regression.

### P04 initial public lifecycle audit complete — 2026-09-10

All48 initial public ellipse views are directly inspected and hash-audited. Content-box and border-box fixed/mixed/percentage pairs, independent shorthand expansion, physical longhands, zero-axis square corners and shared overlap reduction agree structurally with Chrome153 across240/390/768. Sparse curve antialias differences remain under unchanged gates. Fresh public audit reproduces16 compiled RIV/requirement pairs and verifies all128 original/clone resize frames against reviewed images and stream hashes. Receipt: `output/playwright/html-to-riv/elliptical-radii-public-lifecycle/public-ellipse-audit.json`. This completes the initial corpus only; fractional bases, zero-axis positive clip margins, nested/text/image/unequal-border compositions and full native regression remain open. P04 stays active.

### P04 fractional and clip-margin edge corpus — 2026-09-10

Added16 permanent public edge scenes and pinned Chrome153 references: fractional box dimensions/offsets, percentage and mixed axes, zero-axis square corners, overlap, and positive clip margins using border/padding/content origins in both box models. Public same-scene original/clone geometry128/128 and native pixel replay128/128 pass. Direct review9/48 views confirms fractional border-box percentage/mixed contours and square zero-axis expanded clips;39 views remain. No runtime change was needed for this initial edge run. Expanded corpus parity and full public artifact/visual audit remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-edge-progress.json`.

### P04 visual review exposes fractional clip-edge residual — 2026-09-10

Edge visual review reaches21/48 views. Fractional content-origin clip margins show thin continuous edge differences despite passing existing aggregate pixel gates. Preserved a six-scene control corpus contrasting fractional square corners with integer ellipse/square geometry in both box models. Frozen-toolchain replay14/18 passes; four fractional-square240/390 comparisons fail mismatch ratio, all geometry passes. Directly inspected nine border-box control views: integer square edges match and integer ellipses retain sparse curve residuals, while fractional square clips reproduce continuous edge mismatch. This isolates a shared clip/rounding issue, not ellipse-specific behavior. No tolerance changes or control qualification. Expanded native/WASM parity13/13 passes. Evidence: `output/playwright/html-to-riv/elliptical-radii-clip-control-replay/failure-inspection.json`; progress: `elliptical-radii-edge-progress.json`. Next: isolate fractional translation versus content inset and correct shared clip geometry. All processes in this increment are terminal.

### P04 fractional clip rounding isolated; first fix incomplete — 2026-09-10

Ten one-variable controls isolate failures to fractional padding (26/30 comparisons pass; translation/offset/width/height controls pass). Browser pixel samples show whole-pixel clip edges versus native partial coverage. Added failing runtime regression, then snapping final clip bounds makes the test and30/30 isolation comparisons pass; three padding-only views directly inspected show the continuous mismatch gone. Combined original controls still15/18, exposing double rounding from deriving clip edges from an already-snapped outer box. Preserved first-fix toolchain/results. Updated implementation now derives clip bounds from unrounded layout/insets and snaps once; added combined translation and affine opt-out checks. Session34940 is testing/building; no second-fix pass claimed. Receipt: `output/playwright/html-to-riv/elliptical-radii-clip-rounding-progress.json`.

### P04 clip rounding controls now pass — 2026-09-10

Revised clip geometry derives unrounded live layout/inset edges and rounds once in world pixel coordinates under the existing CSS pixel-bounds policy. Runtime regression passes including fractional translation+padding and affine opt-out. Frozen updated probe passes30/30 one-variable and18/18 combined Chrome comparisons; original and first-fix failures remain preserved. Directly inspected three combined fractional square border-box views: continuous edge mismatch and one-pixel shifts are gone. This is focused control evidence, not full runtime qualification. Updated public ellipse lifecycle, remaining visual/public audits and broad regression are next. Receipt: `output/playwright/html-to-riv/elliptical-radii-clip-rounding-progress.json`. All processes are terminal.

### P04 separate rounded and snapped clips resolve edge discrepancy — 2026-09-10

Rounding one combined path fixed square clips but changed rounded fractional-edge coverage. Pinned Chrome153 source confirms two clip nodes for ordinary rounded boxes: rounded border clip plus snapped overflow rectangle. Runtime now preserves the fractional rounded contour and intersects it with a separate snapped rectangle when needed. Source evidence: `output/playwright/html-to-riv/elliptical-clip-chromium-source/source.json`.

Added targeted fractional-edge sampling (`validation/clip-edge-samples.py`) which fails on the prior implementation despite passing area metrics; all8 updated lifecycle frames pass. Updated public edge replay128/128, isolation30/30, combined controls18/18 and default module410 tests pass. Directly inspected six content-origin ellipse views across both box models; continuous edge discrepancies are gone, sparse curve antialias differences retained.42 updated views and broader public/full regression qualification remain. Receipt: `output/playwright/html-to-riv/elliptical-radii-dual-clip-receipt.json`. All processes terminal.

### P04 refreshed initial lifecycle and fractional review — 2026-09-10

Current dual-clip runtime passes all128 original/clone frames in the refreshed initial public ellipse lifecycle (elliptical-radii-initial-dual-lifecycle); process80431 exits0. Its visual/public artifact audit is still pending. Updated edge review reaches18/48 hash-audited views: fractional percentage and mixed-unit corners in both box models agree structurally at240/390/768, with sparse curve antialias residuals retained.30 updated edge views remain, followed by composition expansion and full native regression. Summary row now reflects implemented v24 transport rather than historical preparation. Evidence: output/playwright/html-to-riv/elliptical-radii-dual-clip-receipt.json.

### P04 updated edge corpus audit complete — 2026-09-10

All48 updated edge views directly inspected and hash-audited. Zero-axis square corners, single square corner mixed with ellipses, border/padding/content-origin positive clip margins and oversized proportional overlap agree structurally with pinned Chrome153 across240/390/768 in both box models. Sparse curve antialias residuals remain; tolerances unchanged. Fresh public artifact audit reproduces16 RIV/requirement pairs from authored input and verifies all128 original/clone resize frames, source identities, stream hashes and reviewed images. Receipt: output/playwright/html-to-riv/elliptical-radii-edge-dual-lifecycle/public-ellipse-audit.json. Initial refreshed lifecycle visual audit, nested/text/image/unequal-border compositions and full native regression remain; P04 stays active.

### P04 refreshed initial corpus audit complete — 2026-09-10

All128 refreshed initial lifecycle image pairs are exactly identical to previously reviewed images, with unchanged authored HTML/CSS and runtime requirements. Revalidated the original48-view visual audit and fresh16-scene public artifact reproduction; matched refreshed RIV/requirement artifacts, checked all original/clone240/390/768/240 sequences, current stream hashes and full PNG hashes against the reviewed source. No new direct inspection is claimed. Reproducible verifier and receipt: output/playwright/html-to-riv/elliptical-radii-initial-dual-lifecycle/audit-refresh.py and public-ellipse-refresh-audit.json. Both initial and edge corpora are now audited under the corrected clip runtime. Nested/text/image/unequal-border composition expansion and full native regression remain; P04 is active.

### P04 composition expansion geometry passes — 2026-09-10

Added52 public ellipse scenes and156 pinned Chrome153 references: text/images with unequal translucent borders, visible/clip/hidden and axis clips, signed clip margins, nested flex, independent physical corners and inheritance. Public original/clone resize geometry416/416 passes. First run failed missing expected borderColor metadata in nested fixtures; corrected to authored #76539a, preserving original output. No compiler/runtime fix was needed for geometry. Expanded JS parity corpus registered but not yet run. Native pixel replay is running; no pixel or visual qualification claimed. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 composition renderer profiles separated — 2026-09-10

Default-feature recording completed as vector text:416 geometry frames pass,380 pixel frames pass and36 fail across six text scenes. Direct inspection of three visible border-box text views finds text raster residuals while wrapping/borders/layout agree structurally; all failures retained, no waiver. Native qualification requires native-glyph-controls; rebuilt recorder with that feature and all416 geometry frames pass. Native-glyph pixel replay and expanded native/WASM parity are running. Source, vector failure inspection and current paths are recorded in output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 native composition pixels pass — 2026-09-10

Native-glyph composition replay416/416 geometry/pixel frames passes with repeat-state stability. Prepared156 distinct full-resolution views; first three visible border-box text views directly inspected, with glyphs/wrapping/unequal translucent borders and elliptical corners matching structurally. Expanded accepted native/WASM corpus passes; initial suite10/13 with three ENOENT host setup failures from omitted frozen probe override. Explicit NUXIE_NATIVE_PROBE retries pass all three affected host tests (2+1); original failure log retained. Vector36 text pixel failures remain unwaived. All processes terminal. Three direct views plus six exact-image within-run transfers cover9/156; remaining147 views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 content text/image visual review — 2026-09-10

Composition visual coverage reaches42/156 (21 directly inspected plus21 audited exact-image transfers). Content-box text and both box-model image visible/clip cases agree structurally with Chrome across240/390/768; image clips follow elliptical inner contours, and negative content-origin clip margins contract coverage correctly. Sparse curved-border antialias and thin image-right-edge sampling residuals at768 remain explicitly recorded, including their presence in visible controls. No tolerance changes or pixel-perfect claim.114 views plus public artifact audit/full regression remain. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 axis and container clip-margin review — 2026-09-10

Composition coverage reaches78/156 views (39 direct plus audited exact-image transfers). Inspected horizontal-only/vertical-only overflow, negative content-origin and positive padding-origin clips, border-origin clips and static hidden behavior at240/390/768. Child bounds, exposed borders and sibling placement agree structurally with Chrome. Short shallow lower-left contour residual in border-origin clip and other curve antialias differences retained explicitly under unchanged gates; no pixel-perfect claim. Remaining78 views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 image matrix and nested upper corners reviewed — 2026-09-10

Composition coverage reaches120/156 (54 direct views plus audited exact-image transfers). Remaining image combinations are covered by exact within-run image identity after inspecting the one-axis image control; thin image-edge sampling residual remains documented. Nested isolated top-left/top-right ellipses in both box models agree structurally at240/390/768, including percentage flex sizing and independently rounded panel/aside. Sparse contour antialias residuals retained. Remaining36 nested views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 nested lower corners and fractional radii reviewed — 2026-09-10

Composition review reaches138/156 views (72 direct plus audited exact-image transfers). Lower-right/lower-left isolated ellipses and fractional independent axes in both box models agree structurally with Chrome across240/390/768: child clips, exposed backgrounds and nested flex positions match. Sparse contour antialias residuals retained under unchanged gates. Remaining18 overlap/inheritance/invalid-variable views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 composition visual and artifact audit complete — 2026-09-10

All156 composition views covered (90 directly inspected plus audited exact-image transfers). Final overlap, inherited-axis and invalid-variable reset cases agree structurally with Chrome in both box models across240/390/768; curve/image sampling residuals remain explicit. Fresh public artifact audit reproduces52 RIV/requirement pairs and verifies416 original/clone resize frames, authored inputs, stream hashes and reviewed image pairs. Native/WASM accepted corpus and corrected host checks pass; vector36 text failures remain unwaived. Receipt: output/playwright/html-to-riv/elliptical-radii-composition-native-lifecycle/public-ellipse-audit.json. P04 remains active pending full native regression and its visual audit.

### P04 full native regression launched — 2026-09-10

Registered84 audited ellipse scenes (initial16, edge16, composition52) in the regular browser regression, adding252 comparisons. Built current probe with native-glyph-controls; frozen prior dual-clip probe intentionally lacked that profile. Preserved prior toolchains and froze elliptical-radii-native-full-toolchain with new probe hash. Full native session7608 is confirmed running7696 checks with isolated output/results/gallery. No full-pass claim yet. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json. Focused native composition audit remains complete; full regression and visual audit are pending.

### P04 full-gallery reference audit prepared — 2026-09-10

Added ellipse-gallery-reference.py with fresh lifecycle/public-artifact audit for all84 scenes and252 reference views. Gallery comparison requires exact authored input including embedded Inter/quadrant assets, RIV, requirements and full browser/native PNGs; changed pairs remain unreviewed. Source-only verification passes252/252; completed-gallery comparison remains pending. Full regression7608 confirmed live, observed365/7696 checks. No full-pass claim. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json.

### P05 group opacity references prepared during P04 regression — 2026-09-10

Prepared12 prospective group-opacity scenes and36 pinned Chrome153 references: opacity0/0.5/1 with overlapping children, nested opacity, rounded clipping and translucent borders. Frozen compiler rejects all12; exact diagnostics preserved in output/playwright/html-to-riv/group-opacity-initial-admission/receipt.json. Renderer API inspection finds per-draw modulate_opacity but no group layer operation in the inspected Renderer trait; implementing child alpha alone cannot satisfy overlap semantics. No opacity syntax admitted or native qualification claimed. P04 full regression7608 remains confirmed live.

### P05 interior opacity discriminator established — 2026-09-10

Added group-opacity-samples.py for solid interior pixels in overlap/nested controls at opacity0/0.5/1 across240/390/768. All18 Chrome reference frames pass; sibling alpha remains unchanged. The analytic per-draw-alpha overlap control is rejected. Expected channels use integer quantization with one-channel-unit allowance; initial nearest-rounding expectation disagreed with Chrome half-alpha output and is preserved in group-opacity-initial-samples-rounding-failure.json. This is reference validation only, not native opacity support. Borders/clipping still require full visual validation. P04 full native session7608 remains running.

### P05 renderer boundary investigation — 2026-09-10

Documented existing texture-backed RenderCanvas as a candidate isolation mechanism, plus missing Renderer/recording/replay group contract and duplicate inherited-alpha risk. Planned checked group semantics, stacking-context isolation, clip/overflow bounds, nested resource ownership and lifecycle validation in validation/group-opacity-implementation-notes.md. No group opacity implementation/admission claimed. P04 frozen full regression remains active.

### P05 stacking-context discriminator — 2026-09-10

Added six prospective opacity stacking scenes and18 pinned Chrome references. At opacity0.5, child z-index -2/0/2 remains isolated below an outside sibling at z-index1. At opacity1, child z-index2 escapes the non-stacking parent and covers that sibling. All18 exact interior overlap samples confirm this distinction (output/playwright/html-to-riv/group-opacity-stacking-oracle/stacking-samples.json). Canvas begin_frame inspection shows shared render-context beginFrameExecutable; nested use must preserve parent work through an explicit scheduling/recording strategy, not assume independent contexts. Public opacity remains unsupported. P04 full regression remains running.

### P05 isolated opacity parser implemented — 2026-09-10

Added src/opacity.rs: single finite number/percentage, comments, exponent syntax, clamp before f32 narrowing and canonical positive zero. Three focused tests pass, covering large finite clamping and malformed/nonfinite/unit/math rejection. CSS-wide keywords/substitution remain cascade responsibilities. Parser is intentionally unconnected to public style admission until group rendering/recording/host support exists. Receipt: output/playwright/html-to-riv/group-opacity-parser-receipt.json. P04 frozen regression remains live, observed1866/7696 checks; new isolated parser does not change its immutable compiler.

### P05 real offscreen compositing test added — 2026-09-10

Added renderer test offscreen_canvas_composites_overlapping_shapes_with_one_opacity: draw overlapping red/green shapes on blue canvas, finish offscreen frame, composite image over white at0/0.5/1 and assert red-only/overlap/background interior pixels and opaque output alpha. Test build61153 remains confirmed running; no pass claimed. This verifies candidate sequential canvas mechanism only, not nested active-frame support or public CSS group opacity. P04 frozen regression7608 remains active, last observed2329/7696. Evidence: output/playwright/html-to-riv/group-opacity-offscreen-progress.json.

### P05 offscreen and nested alpha pixels pass — 2026-09-10

Corrected renderer test build to product feature renderer-metal after preserved internal-feature missing-adapter build failure. Real overlapping-shape canvas compositing passes at0/0.5/1. Added nested transparent canvas test: two half-opacity composites produce quarter-opacity red over white while untouched pixels remain white. Both opacity tests pass; filtered run5/5 including existing offscreen tests. This proves sequential canvas compositing only; parent-frame suspension/recording, runtime group boundaries, stacking policy and public transport remain pending. Evidence: output/playwright/html-to-riv/group-opacity-offscreen-progress.json. P04 frozen full regression remains running.

### P05 parent clip and state restoration pixels pass — 2026-09-10

Added real renderer regression for translated half-opacity canvas under a parent clip, followed by an opaque sibling after restore. Interior samples confirm clipped group coverage, excluded pixels and restored sibling transform/clip/alpha. All three opacity canvas tests pass; filtered offscreen run6/6. Candidate sequential composition now has overlap, nested transparency and parent-state evidence. Runtime scheduling/recording and public group admission remain pending. Receipt: output/playwright/html-to-riv/group-opacity-offscreen-progress.json. P04 full native regression remains active.

### P05 browser value parity and replay scheduling constraint — 2026-09-10

Pinned Chrome value probe passes21 controls for candidate opacity number/percentage parsing, exponent syntax, comments, finite large clamping and malformed token rejection. This supports the isolated parser contract; public opacity remains gated. Recorded replay_frame's already-open renderer constraint: nested textures require preparation before parent-frame creation or explicit suspension, with balanced-group/resource validation and retained texture ownership. Evidence: output/playwright/html-to-riv/group-opacity-values.json and validation/group-opacity-implementation-notes.md. P04 full native regression remains confirmed live, observed3762/7696.

### P04 gallery audit negative controls pass — 2026-09-10

Added executable verifier controls using freshly audited252 ellipse reference views. Synthetic complete report transfers252 exact pairs; changed embedded asset rejects, substituted image leaves one pair unreviewed, and missing check rejects incomplete gallery. These are verifier tests only, not full-regression or new visual qualification. Receipt: output/playwright/html-to-riv/ellipse-gallery-audit-controls.json. Full native session7608 remains active, observed4177/7696 checks.

### P05 portable group structure and prepaint rejection — 2026-09-10

Added typed/text beginOpacity/endOpacity commands and a child-first structural plan. Preflight checks finite normalized alpha, maximum64 nested groups, balanced boundaries and save/restore isolation; malformed groups cannot paint or allocate replay resources. Existing already-open-frame replay explicitly rejects valid groups until preparation exists. Stream suite11/11 passes, including4 new structural/negative tests. This is infrastructure only: group execution, renderer recording, runtime policy and public CSS opacity remain pending. Receipt: output/playwright/html-to-riv/group-opacity-stream-preflight-receipt.json. Full P04 session7608 confirmed live this turn; latest observed5668/7696 checks.

### P05 inherited group state planning passes — 2026-09-10

Group preparation now retains ordered inherited transform, clip and per-draw modulation state using persistent command chains. Group boundaries implicitly restore state; explicit save/restore and nested groups cannot leak state to following siblings. Two new tests cover nested state ordering and10000 sibling groups sharing10000 state nodes without copying their histories. Stream suite13/13 passes. This is preparation infrastructure, not texture execution or public opacity support. External clips must be applied at composition rather than baked and applied again. Receipt: output/playwright/html-to-riv/group-opacity-state-plan-receipt.json. P04 full native session7608 confirmed live this turn, latest observed6069/7696 checks.

### P05 sequential portable stream canvas execution passes — 2026-09-10

Implemented render_frame_to_canvas with shared loaded resources, child-first offscreen rendering and final owned output canvas. Inherited transforms place group contents; ancestor clips/modulation apply at composition, with inverse transforms returning textures to device coordinates. Frames finish even when execution reports an error. Real Metal nested overlap/translation/parent-clip/sibling pixel test passes; stream suite15/15 including allocation/transform/unsupported-backend controls passes. Candidate uses viewport textures,256MiB aggregate pixel budget and finite invertible transforms; caller must have no active shared-context frame. Tight bounds, singular transforms, runtime/recording/host/compiler integration and Chrome visual qualification remain. Public CSS opacity remains rejected. Receipt: output/playwright/html-to-riv/group-opacity-canvas-execution-receipt.json.

### P05 recording interface and repeated alpha controls pass — 2026-09-10

Renderer now exposes opt-in begin_opacity_group/end_opacity_group; unsupported backends decline without changing output. RecordingRenderer emits typed-stream syntax, checks finite normalized alpha/depth64 and unmatched ends. Stream16/16 tests pass. Expanded real Metal canvas test covers seven outer/inner alpha configurations including zero and one; returning to half/half produces identical full pixel buffers on the same factory. Runtime policy and compiler/host admission remain pending. Receipt: output/playwright/html-to-riv/group-opacity-recording-receipt.json.

### P04 full regression passes; ellipse references all transfer — 2026-09-10

Frozen full native session7608 terminates successfully:7696/7696 checks pass. Fresh focused-source/public-artifact audit transfers all252 ellipse gallery pairs by exact inputs/artifacts/full PNGs, with zero remaining ellipse pairs. Baseline audit13921 is still running; full visual qualification is not yet claimed. P05 render-api suite36/36 also passes after opt-in group recording methods. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json and group-opacity-recording-receipt.json.

### P04 native qualification complete — 2026-09-10

Full7696/7696 checks and7686/7686 visual pairs are audited:7434 exact baseline transfers plus252 exact focused ellipse transfers. Combined proof, completed checks and frozen binary hashes verified. P04 now native-qualified for the documented v24/LTR/native-glyph scope;84 scenes/672 original-clone frames, module410 and expanded native/WASM/host evidence retained. Vector36 text failures and curve/image sampling differences remain explicit. P05 remains active and unqualified. Receipt: output/playwright/html-to-riv/elliptical-radii-p04-receipt.json; contract: validation/elliptical-radii-review.md.

### P05 runtime stacking/group plan passes — 2026-09-10

Runtime paint planner now represents opacity and emits begin/end boundary events around complete isolated contexts. Alpha0 and0.5 isolate negative/positive positioned descendants; alpha1 retains ordinary stacking. Nested boundary ordering and deferred ancestor clips pass alongside existing stacking regressions:16/16 tests. Production runtime trees still default to1; occurrence policy, checked renderer admission and boundary execution remain pending. No public CSS opacity claim. Receipt: output/playwright/html-to-riv/group-opacity-runtime-plan-receipt.json.

### P05 runtime installation and recorded lifecycle pass — 2026-09-10

Artboard now atomically installs validated layout opacity targets, copies policy to clones, rebuilds stacking plans and emits renderer group boundaries. Renderer capacity preflight enables try_draw_internal_handle to return false before painting; legacy infallible drawing asserts on unsupported group rendering. Lifecycle integration test passes invalid-target/alpha atomicity, original/clone resize240/390/768/240, clear/alpha1 restoration and nested recording boundaries. Exhausted recording capacity rejects with an unchanged stream. Public compiler/host transport and runtime-emitted browser/native pixel qualification remain pending. Receipt: output/playwright/html-to-riv/group-opacity-runtime-install-receipt.json.

### P05 native replay CLI executes opacity groups — 2026-09-10

Native Metal replay detects group commands and renders their canvases before opening the output frame; flat streams retain the existing replay path. Both standard and atomic modes pass nested-alpha interior pixels, two-run PNG identity and explicit white-clear override. Translucent-clear controls confirm alpha is not applied twice; truncated group streams fail without producing a PNG. Four repeated opaque images plus two translucent images are exercised. This uses a synthetic stream, not runtime CSS/browser qualification. Receipt: output/playwright/html-to-riv/group-opacity-replay-cli-r2/receipt.json. Frozen P04 binaries remain unchanged.

### P05 runtime opacity Chrome/native corpus passes — 2026-09-10

Twelve initial opacity fixtures compile once with opacity stripped only in the diagnostic compiler request; checked runtime policy installs authored alpha.96 original/clone resize frames pass geometry and native pixel comparison against pinned Chrome153, with repeated native state identity. Targeted18 overlap/nested interior sample frames pass.12/36 distinct views directly inspected: half-alpha overlap/nested, elliptical clipping and translucent borders agree structurally; sparse contour antialias and uniform color quantization differences retained under unchanged gates. Remaining24 views, full diagnostic evidence audit, stacking/richer compositions and public compiler/host admission. Receipt: output/playwright/html-to-riv/group-opacity-initial-runtime-progress.json.

### P05 initial runtime visual and artifact audit complete — 2026-09-10

All36 initial views covered by30 direct reviews and6 exact-image transfers. Final zero/one controls preserve layout, child occlusion, nested alpha, elliptical clips and translucent borders; sparse contour/quantization differences remain explicit. Fresh diagnostic audit reproduces12 opacity-stripped RIV/requirement pairs, verifies authored CSS and injected alpha, group balance/value counts,96 original/clone frames, source/projection/stream hashes and reviewed full PNGs. Frozen replay renderer hash also verified. This qualifies runtime diagnostic evidence only; public CSS opacity remains rejected. Receipt: output/playwright/html-to-riv/group-opacity-initial-runtime-lifecycle/diagnostic-opacity-audit.json. Stacking/richer compositions and compiler/host transport remain.

### P05 opacity stacking runtime corpus audited — 2026-09-10

Six stacking fixtures pass48 original/clone resize geometry/pixel frames and exact overlap samples.18 distinct views covered by12 direct inspections and6 exact-image transfers. Half-opacity seals negative/zero/positive descendants under outside z1 sibling; full-opacity z2 escapes and covers it, z0 stays under and negative stays behind parent background. Fresh diagnostic audit reproduces6 stripped compiler artifacts and verifies injected alpha, streams/lifecycle and reviewed PNGs. Combined initial+stacking evidence now18 scenes/144 frames/54 reviewed views; public compiler/host transport and richer text/image/composition coverage remain pending. Receipt: output/playwright/html-to-riv/group-opacity-stacking-runtime-receipt.json.

### P05 glyph adapter and composition references prepared — 2026-09-10

GlyphRenderer now forwards opacity group capacity/boundaries and restores cached transform/modulation state. Added nested-state and unsupported-backend tests. Initial command omitted native-glyphs-experimental and selected zero tests; preserved. Corrected build36556 confirmed live, no pass claimed. Twelve text/image/mixed fixtures produce36 pinned Chrome references; removed unused image-only text rule with exact36-image recapture identity. Runtime composition recording/replay remains pending. Evidence: output/playwright/html-to-riv/group-opacity-glyph-adapter-progress.json and group-opacity-composition-reference-receipt.json.

### P05 text/image automated pass exposes visual discrepancy — 2026-09-10

Corrected glyph-adapter tests2/2 pass. Asset-aware runtime recorder and glyph wrapping produce96/96 composition geometry/pixel frames with repeat stability across12 scenes. Direct mixed-visible half-opacity inspection (three widths) catches nonuniform native image interiors despite aggregate passes: Chrome quadrant is constant, native varies by2–3 channel units. Opaque image-only control varies0–1; mixed opaque output (with nested text group) varies0–2. No composition visual qualification claimed and no tolerances changed. Frozen recordings/replay and diagnostic samples retained. Evidence: output/playwright/html-to-riv/group-opacity-composition-runtime-progress.json and group-opacity-composition-image-interior-diagnostic.json. Next investigate added offscreen/image-composition stages before public opacity admission.

### P05 intermediate dither accumulation fixed — 2026-09-10

Minimal flat rectangle reproduces growing interior variation:1 channel unit without groups,3–5 through1–3 groups. Nearest sampling does not improve it and was reverted. Same-binary dithering control removes growth;64 original image frames stay within1 unit. Added explicit RenderCanvas::begin_compositing_frame, implemented by native canvas with dithering disabled only for intermediate frames. Ordinary canvas and final frame semantics remain unchanged; temporary environment switch removed. Eight flat-color controls, CLI group/clear/rejection controls, render-api36 and stream16 tests pass. Fixed original composition96/96 replay passes and all native PNGs exactly match the positive diagnostic control. Three corrected mixed-visible widths directly reviewed (plus exact-image transfers); remaining composition review and fresh artifact audit pending. Receipt: output/playwright/html-to-riv/group-opacity-dither-diagnosis.json.

### P05 corrected composition visual and artifact audit complete — 2026-09-10

All36 corrected text/image/mixed Chrome/native views now have audited visual coverage (18 directly inspected and18 exact full-image transfers). Text wrapping, nested alpha, image quadrants, responsive borders and outside siblings agree visually; small quantization and sparse curved-edge antialias residuals remain documented without tolerance changes. Fresh asset-aware artifact audit reproduces12 opacity-stripped compiler artifacts and verifies injected alpha, recorded group balance, all96 original/clone resize frames and reviewed image hashes. Public opacity CSS remains unqualified. Next revalidate initial/stacking corpora with the corrected renderer, then complete compiler/host transport and parity. Evidence: output/playwright/html-to-riv/group-opacity-composition-compositing-frame-lifecycle/diagnostic-opacity-audit.json and group-opacity-composition-runtime-progress.json.

### P05 corrected initial and stacking replay passes — 2026-09-10

The frozen undithered-compositing renderer passes all96 initial and48 stacking frames against pinned Chrome, including original/clone repeat stability. Initial corrected review has6/36 directly inspected views (nested half-opacity and translucent border at three widths); stacking18 views await review. Only9 initial views are byte-identical to the old output; no wholesale review transfer claimed. Fresh artifact audits remain pending until visual coverage completes. Evidence: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json. Public opacity admission remains pending.

### P05 corrected runtime corpora fully audited — 2026-09-10

Corrected initial36 and stacking18 views now have complete audited coverage (initial30 direct/6 exact-image transfers; stacking12 direct/6 transfers). Fresh artifact audits verify all18 stripped compiler artifacts and144 original/clone frames against the recorded inputs and reviewed images. Alongside corrected composition12 scenes/96 frames/36 views, the current renderer has30 diagnostic scenes,240 lifecycle frames and90 reviewed views. Stacking discriminators preserve group isolation below opacity1 and allow child z-index participation at opacity1. Sparse curved-edge antialias and quantization differences remain explicit; public compiler admission, host transport, native/WASM parity and broader public regression are still pending. Receipt: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json.

### P05 computed opacity cascade integrated — 2026-09-10

Style now stores group opacity independently of paint colors, defaults to1 without inheritance, supports explicit inherit/initial/unset, and uses the existing normalized number/percentage parser. Added cascade/substitution tests. Initial focused run5/6 exposed math fallback incorrectly invalidating to unset; corrected compatibility classification retains unsupported math diagnostics. Full library48/48 passes. Public admission remains explicitly gated until isolated-compositing transport exists; one regression test covers8 authored values and passes. No new public opacity support claimed. Receipt: output/playwright/html-to-riv/group-opacity-computed-style-progress.json.

### P05 version25 requirement contract tested — 2026-09-10

Added LayoutGroupOpacityRequirement with unique non-root LayoutComponent IDs and finite normalized opacity in [0,1); opaque entries are omitted because opacity1 does not establish a stacking context. Version25 and layout-css-group-opacity-v1 must match nonempty entries. The capability explicitly requires isolated subtree compositing, undithered intermediate surfaces and checked renderer capacity. Structural validation rejects missing/extra fields, wrong types, duplicates, root targets, invalid alpha and mismatched versions/capabilities. Corner coexistence and old-version controls pass. Three opacity plus four corner requirement tests pass, and TypeScript version25 declarations/typecheck pass. Checked host installation, public emission and native/WASM parity remain pending; CSS admission stays gated. Receipt: output/playwright/html-to-riv/group-opacity-requirements-progress.json.

### P05 checked host installation verified — 2026-09-10

Probe now advertises and installs version25 group-opacity requirements, supports explicit disabled-capability controls and uses checked instance drawing. Added public try_draw/try_draw_handle entry points preserving artboard frame identity; insufficient group capacity refuses before paint. Existing atomic/clone/clear/resize integration test now exercises the checked instance API and passes. New host transport test passes nested alpha at four widths, missing capability and malformed manifest rejection before stream output. Initial full host run had two feature-configuration failures (probe lacked native-glyph-controls); preserved. Rebuilt with native-glyph-controls and full host regression passes. Public CSS emission remains gated; these are manually supplied manifest controls. Receipt: output/playwright/html-to-riv/group-opacity-host-progress.json.

### P05 public opacity emission candidate — 2026-09-10

Public CSS now emits version25 normalized group requirements for computed opacity below1, leaving Rive layout and paint bytes unchanged. Explicit inheritance, CSS-wide resets, percentages, clamping, important cascade and custom-property fallback are covered; opaque values omit requirements and preserve prior artifacts. Five requirement/public emission tests pass. General opacity math remains rejected. Full Rust regression initially stopped on a stale opacity-rejection expectation; that test now checks excluded calc syntax, with original failure retained. Full rerun and native/WASM parity are pending; this is an implemented candidate, not a native-qualified feature. Added 30-scene JS/native/WASM parity test using existing Chrome-reference corpora; execution awaits matching builds. Current scope remains static isolated compositing through the checked recording/replay host, not immediate backends without group support.

### P05 public emission module and parity pass — 2026-09-10

Full corrected Rust module suite426/426 passes. Matching native and WASM compilers build successfully; new30-scene public opacity parity test passes across initial, stacking and text/image composition Chrome-reference inputs, comparing complete Rive bytes, source maps and runtime requirements. Public Chrome/native pixels and compile-once clone/resize qualification remain pending; runtime-only diagnostic receipts are not promoted to public evidence. Receipt: output/playwright/html-to-riv/group-opacity-public-emission-progress.json.

### P05 public lifecycle pixels and artifact audit pass — 2026-09-10

Thirty authored opacity scenes now compile directly, install only emitted group requirements, clone once and resize both instances across240 frames. Geometry and native/Chrome pixel gates pass with repeated-state byte stability. All90 public views exactly match authored HTML/CSS and complete browser/native PNGs from the corrected, independently audited diagnostic reviews; specialized visual transfer records this without claiming new inspection or transferring compiler qualification. A separate fresh public artifact audit reproduces all30 Rive artifacts/requirements, checks source-map alpha targets, all240 stream/frame identities and group balance, and verifies reviewed images. Broader edges, audit negative controls and full regression remain before P05 qualification. Receipt: output/playwright/html-to-riv/group-opacity-public-lifecycle-progress.json.

### P05 audit negative controls and public precision correction — 2026-09-10

Eighteen negative controls reject changed HTML/CSS, hash claims, actual PNG/Rive bytes, qualification, missing/duplicate views, requirements, injected policy, frame identity and missing frames; two positive controls pass. Controls operate in temporary copies and leave originals read-only. Public boundary tests exposed generic CSS f32 token normalization rejecting finite1e100 opacity before clamping. Numeric opacity text is now preserved until computed-value invalidation and the f64 opacity parser; this also retains near100% precision. First fix prematurely validated multi-token var fallbacks; retained failure and corrected ordering. Library48 plus opacity requirement/public6 tests pass. Existing public visual receipts describe the preceding frozen outputs; refreshed parity, Chrome boundary evidence and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-audit-precision-progress.json.

### P05 Chrome opacity boundary semantics and parity pass — 2026-09-10

Pinned Chrome28 controls (14 values authored directly and through var fallback) agree with public requirement presence using an observable z-index/elementFromPoint discriminator. Chrome serializes computed opacity as1 for some values that still isolate stacking, so computed-style text alone is insufficient:99.999997% retains a group while99.999999% does not. Large finite values clamp correctly. Refreshed WASM and native builds pass opacity parity across30 composition scenes plus28 boundary inputs, comparing Rive bytes, source maps and requirements. Browser screenshots are captured but not claimed as new native visual qualification. Native boundary lifecycle pixels, broader combinations and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-precision-progress.json.

### P05 native boundary pixels visually audited — 2026-09-10

The28 direct/var boundary inputs now compile publicly and replay through the native renderer at240x320. All geometry/pixel gates and exact browser/native100,100 overlap-color samples pass. Four distinct full-resolution image pairs directly inspected: invisible, half-alpha, fully opaque (red over green), near-opaque isolated (green over red). Remaining24 views have verified exact full-PNG transfers; review audit complete. Small quantization differences remain explicit, tolerances unchanged. These static controls do not establish clone/resize or broader composition coverage. Receipt: output/playwright/html-to-riv/group-opacity-native-boundary-progress.json.

### P05 boundary clone/resize pixels reviewed; metadata correction — 2026-09-10

Twenty-eight boundary scenes now compile once and resize original/clone across224 frames. Geometry/pixels/repeat stability pass, with84 views reviewed (12 direct and72 full-image transfers). Fresh artifact audit exposed recorder serde_json::Value widening f32 alpha versus compiler shortest-roundtrip serialization. Recorder now serializes requirements through the public JSON serializer; corrected recording test passes and all224 stream files are byte-identical to the prior recording. Alpha comparison in the audit uses exact f32 representation for known f32 fields, with no tolerance. Corrected metadata/replay binding and final artifact audit remain pending, so no completed boundary qualification claimed. Receipt: output/playwright/html-to-riv/group-opacity-boundary-lifecycle-progress.json.

### P05 corrected boundary artifact audit complete — 2026-09-10

Corrected metadata recording replays224/224 frames successfully;84 views carry forward prior inspection only after authored source, complete PNG, stream and freshly generated sheet identity checks. Fresh public artifact audit now passes with exact f32 semantic alpha comparison and exact serialized requirement reproduction. Boundary controls are complete within this frozen native-renderer scope. Independently removed the per-group linear plan scan during canvas composition by retaining the group node alongside its indexed prepared canvas; stream tests pass, native verification pending. Receipts: output/playwright/html-to-riv/group-opacity-boundary-completion-progress.json and group-opacity-boundary-public-lifecycle-r2/public-opacity-audit.json. Broader composition/resource controls and full regression remain before P05 qualification.

### P05 indexed canvas lookup preserves all focused native frames — 2026-09-10

The indexed prepared-canvas lookup passes16 stream tests, native CLI alpha/clear/rejection controls and8 flat-color controls. All464 audited public opacity frames rerender with complete native PNG identity to their reviewed baselines; no new visual inspection claimed or needed for unchanged full images. Updated the current P05 backlog row and leading support contract to describe implemented version25 behavior, remaining qualification and explicit replay constraints. Broader opacity compositions, backend/resource limits and full visual regression remain. Receipt: output/playwright/html-to-riv/group-opacity-indexed-native-progress.json.

### P05 resource boundary and overflowing composition coverage — 2026-09-10

Added exact256MiB surface-budget boundary controls: root plus63 1024² groups reaches the allocation interface; an extra group or column rejects before paint, for nested and sibling groups. Stream17 tests pass. Added18 public scenes combining opacity0/.5/1, border/content-box sizing, visible/rounded/axis clipping and genuinely overflowing text/images. All144 original/clone geometry/pixel frames and repeat checks pass; expanded native/WASM opacity parity covers76 scenes. First three half-opacity rounded-clip views directly reviewed; remaining51 views and artifact audit pending. Recorder now installs content-box and axis policies for this corpus. Receipt: output/playwright/html-to-riv/group-opacity-overflow-composition-progress.json.

### P05 overflowing compositions audited and registered for regression — 2026-09-10

All18 overflowing text/image scenes now have complete54-view coverage and a fresh public artifact audit covering144 original/clone frames. Rounded and x-only clipping, both box-sizing modes, nested text opacity and outside-sibling stacking agree with pinned Chrome. Sparse edge antialias and image quantization differences remain within existing gates; tolerances unchanged. Across public focused corpora the total is76 scenes/608 frames/228 views. Registered all five opacity corpora in the regular browser regression, adding228 comparisons; the full gate has not yet passed. Evidence: output/playwright/html-to-riv/group-opacity-overflow-composition-lifecycle/public-opacity-audit.json and group-opacity-overflow-composition-progress.json.

### P05 full native regression launched — 2026-09-10

Frozen current compiler, native-glyph-controls probe, indexed native renderer and WASM in group-opacity-native-full-toolchain with SHA-256 manifest. Verified installed Chrome153.0.8010.12. Full native regression session82980 reports7924 checks (prior7696 plus228 opacity comparisons); module regression session80405 is separately running. Outputs and gallery are isolated from earlier receipts. No full-pass or P05 qualification claim yet. Evidence: output/playwright/html-to-riv/group-opacity-full-progress.json.

### P05 full module pass and gallery reference verification — 2026-09-10

Full native-glyph-controls Rust module regression passes442 tests across76 result blocks, exit0. Added opacity-gallery-reference.py, which freshly revalidates public artifacts/lifecycle/review evidence for all228 opacity views before comparing a completed full gallery by exact authored inputs, assets, Rive bytes, requirements and both full PNGs. Source verification passes228/228. Verifier controls pass exact pairs, reject changed embedded assets/incomplete galleries and leave changed native images unreviewed. Initial control failed because the synthetic fixture assumed separate requirements files; corrected to use the already-audited recording metadata, preserving the failure log. Full native session82980 remains live; full JS parity session28814 now running. No completed full visual regression claimed. Evidence: output/playwright/html-to-riv/group-opacity-full-progress.json.

### P05 decorated opacity compositions pass automated gates — 2026-09-10

Full JavaScript regression15/15 passes (before decoration expansion). Added12 public underline/strikethrough scenes crossing zero/half/opaque parent alpha, nested half-opacity text and visible/clipped overflow. Pinned Chrome36 references captured; helper now installs emitted underline/strikethrough policies. All96 original/clone geometry/native-pixel frames and repeat checks pass. Expanded focused public/native/WASM parity passes88 inputs. Three half-opacity visible underline views directly inspected;33 views and fresh artifact audit remain. These new fixtures are not part of the already-running7924-check frozen full regression. Evidence: output/playwright/html-to-riv/group-opacity-decoration-progress.json.

### P05 decorated opacity visual and artifact audit complete — 2026-09-10

All36 decorated-text views now have complete coverage (27 direct,9 exact full-image transfers for invisible controls). A fresh audit reproduces12 public Rive artifacts/requirements and verifies96 original/clone resize frames. Underline clipping and visible overflow, strikethrough positioning, nested text alpha and outside-sibling placement agree with Chrome; existing sparse antialias/quantization residuals retained without tolerance changes. Focused public totals now88 scenes/704 frames/264 views. The decoration corpus remains separate from the running7924-check full regression and will be registered afterward. Host transform/backend constraints and full regression review remain. Receipt: output/playwright/html-to-riv/group-opacity-decoration-progress.json.

### P05 host modulation reproducer exposes image-path double alpha — 2026-09-10

Added analytic native controls for ancestor translation/scale/rotation/shear and host modulation, with overlapping group shapes on both native modes. Initial stream writer emitted spaces unsupported by the parser; preserved and corrected. Native translation+host0.5+group0.5 then fails at interior(6,6):RGBA255,223,223,255 versus expected255,191,191,255. Image-as-path drawImage multiplies host modulation into image paint, then drawPath multiplies it again. Changed that branch to pass unmodulated image alpha; direct image branch unchanged. Renderer build33758 running, no fix-pass claim yet. The separate7924-check full regression uses the preceding frozen renderer and cannot qualify this correction. Receipt: output/playwright/html-to-riv/group-opacity-host-state-progress.json.

### P05 host modulation correction passes analytic controls — 2026-09-10

Renderer build exits0; all16 translation/scale/rotation/shear × host1/0.5 × two-native-mode controls pass both single-shape and overlap samples. Corrected image-path alpha is applied once. Frozen renderer in group-opacity-host-state-fixed-toolchain; existing public frame identity and broader image regression remain pending. Synthetic controls do not qualify CSS transforms. Receipt: output/playwright/html-to-riv/group-opacity-host-state-fixed/receipt.json.

### P05 corrected renderer preserves704 frames; full gate restarted after ENOSPC — 2026-09-10

All704 audited public opacity original/clone frames rerender with exact full native PNG identity after the host-modulation correction. Extended the identity tool to include overflow and decoration corpora. Original full session82980 terminated exit1 after1877 passes when disk exhaustion prevented artifact writes; failure evidence preserved, no full-pass claim. Removed only regenerable Cargo dependency/test executables (16.5GiB), retaining sources and validation artifacts. Froze corrected renderer plus existing compiler/probe/WASM in group-opacity-native-full-toolchain-r2. Registered12 decoration scenes; new full session15010 reports7960 checks. Expanded gallery source verification and controls pass264 exact references, changed assets rejected, changed images left unreviewed, incomplete galleries rejected. Full output review remains pending. Receipt: output/playwright/html-to-riv/group-opacity-full-r2-progress.json.

### P06 linear-gradient reference and rejection baseline — 2026-09-10

While corrected P05 full regression runs, prepared16 prospective linear-gradient scenes and48 pinned Chrome153.0.8010.12 references: cardinal/corner/angle directions, three stops, explicit/decreasing/coincident/out-of-range stops, alpha, and mixed pixel/percentage positions. Frozen public compiler rejects all16, preserving exact diagnostics. Runtime inspection finds explicit numeric gradient endpoints rather than layout-relative CSS geometry; responsive updates need a checked runtime seam, not compile-viewport coordinates. Documented implementation/validation order and remaining semantics in validation/linear-gradient-implementation-notes.md. No gradient syntax admitted or native pixels claimed. Receipt: output/playwright/html-to-riv/linear-gradient-initial-progress.json.

### P06 responsive gradient geometry and stop fixup implemented — 2026-09-10

Added runtime css_linear_gradient module resolving cardinal/angle/corner endpoints from current box dimensions, plus source-preserving omitted/decreasing/pixel/percentage stop fixup. Four standalone Rust tests pass, covering diagonal endpoints, corner midpoint constraints across aspect ratios, resize-dependent hard stops, out-of-range positions and invalid/degenerate inputs. Runtime cargo check passes. A separate harness compiles the runtime helper and compares its predicted red/blue ramp against120 pinned Chrome samples across eight directions and three widths; all pass the declared two-channel-unit quantization bound. This is geometry evidence only, not native rendering or public gradient admission. Parser, checked runtime paint update/clone seam, public transport/parity and pixel qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-geometry-progress.json.

### P06 specified gradient component parser tested — 2026-09-10

Added an unconnected gradient parser retaining direction, currentColor, px/em/rem/percentage/omitted positions and expanded double-position color stops. Supports existing named/hex/RGB/HSL colors, angles/cardinal/corner directions, comments and256 expanded stops. Three tests pass including all16 prospective Chrome inputs and malformed/unsupported/limit controls. Initial test caught nonfinite numeric token serialization producing a finite angle; validation now rejects before serialization and the original failure is retained. Independent pinned Chrome controls confirm single-color double-position gradients are valid, as are hints and explicit color spaces; the latter remain explicit parser exclusions pending implementation, not claimed invalid CSS. Public admission remains gated by missing runtime paint/host transport and qualification. Receipt: output/playwright/html-to-riv/linear-gradient-parser-progress.json.

### P06 computed gradient cascade integrated behind emission gate — 2026-09-10

Style retains a gradient separately from background color. Background shorthand resets both; background-color preserves the image; background-image none/initial/unset clears it. Relative stops resolve against the final font size (rem uses the profile16px root), so explicit inheritance copies computed lengths. CurrentColor stays symbolic for final color resolution. Gradient function syntax is preserved past the ordinary color-only serializer; variable fallbacks retain valid gradients and invalidate plain red/length/duplicate-none image values to unset. Public compilation explicitly rejects remaining gradients until responsive paint transport exists. Initial failures (test diagnostic-vector access, nested-function serialization) are preserved; corrected focused7/7 and complete library55/55 pass. P05 frozen full run remains independent and live. Receipt: output/playwright/html-to-riv/linear-gradient-cascade-progress.json.

### P06 checked gradient paint description and extended stop domain — 2026-09-10

Runtime inspection confirms ordinary Rive LinearGradient clamps stop positions to0..1. Added checked CssLinearGradient retaining authored direction/colors/positions with matching count and2..256 stop limits. Resolution first performs CSS stop fixup, then expands endpoints to contain out-of-range stops and normalizes them for the shader, preserving their colors inside the box. Seven standalone tests pass, including the prior four geometry controls, out-of-range color coordinates, clone/repeated resize and malformed/empty inputs. This does not yet install or draw CSS gradients. Layout drawing, positioning-area/border semantics, atomic host transport and native interpolation qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-paint-policy-progress.json.

### P06 experimental layout gradient drawing and atomic installation — 2026-09-10

LayoutComponent now retains optional checked gradient paint, clones authored values, and paints it after solid fills/before borders using current dimensions and rounded background geometry. The Artboard occurrence installer validates all non-root layout targets and duplicates before replacing the policy map; clearing one occurrence leaves its clone intact. Integration test passes eight original/clone resize draws with expected recorded endpoints, invalid-target atomicity, clear and clone independence. Early adapter type build failures and test-held factory borrow failure are preserved; corrected run passes. This is recorded-runtime evidence only: native interpolation, positioning-area/border/repeat behavior and public transport remain unqualified, and public gradient emission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-runtime-installation-progress.json.

### P06 native replay exposes missing gradient-only drawable proxy — 2026-09-10

Added16 explicitly injected gradient descriptions to the diagnostic oracle and recorded128 compile-once original/clone frames; geometry passes. First real native replay fails128/128 with blank gradients. Inspected native image and recording: no gradient commands were emitted because owners with no ordinary fill did not request a drawable proxy. The earlier white-background installation control masked this. Updated needs_drawable_proxy to include CSS gradients and removed that solid fill from the integration regression. Corrected full gradient-runtime recording session6233 building; no fix-pass claim yet. Expanded lifecycle harness labels gradient injection runtime-experiment-only. Public gradients remain gated. Receipt: output/playwright/html-to-riv/linear-gradient-initial-runtime-progress.json.

### P06 corrected proxy renders gradients;24 native residuals retained — 2026-09-10

Both gradient runtime integration tests pass after proxy admission correction. Corrected128-frame replay now renders gradients:104 frames pass existing native/Chrome gates and24 fail, confined to decreasing-stop, transparent-stop and mixed-alpha scenes (eight frames each). Geometry remains correct. First default native image inspected to confirm the blank-paint defect is removed; no complete browser/native visual review claimed. Residual interpolation/hard-stop diagnosis is next, with original and corrected outputs preserved. Receipt: output/playwright/html-to-riv/linear-gradient-initial-runtime-progress.json.

### P06 exterior stops fixed; alpha interpolation mismatch isolated — 2026-09-10

Direct browser/native inspection isolates decreasing-stop failure: native lost the first red color before coincident leading stops. CSS resolution now adds equivalent constant-color endpoint stops before native normalization, retaining hard transitions without approximation. Eight pure tests and two runtime recording tests pass. Native128-frame rerun improves to112 passing; only16 transparent/mixed-alpha frames fail. Eight preserved interior samples confirm Chrome premultiplied interpolation versus native straight-color interpolation (each matches its respective analytic formula within2 channel units). Shader source also explicitly premultiplies after interpolating unmultiplied colors. Next requires a checked CSS interpolation path while preserving ordinary Rive gradient behavior; public admission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-interpolation-progress.json.

### P06 explicit premultiplied gradient transport tested — 2026-09-10

Added opt-in Factory::make_premultiplied_linear_gradient returningNone for unsupported/invalid input, forwarding through persistent factory wrappers. Recording validates finite coordinates, matching stop counts and normalized monotonic positions, and emits makePremultipliedLinearGradient. Stream parser retains a distinct resource and replay refuses unsupported factories; ordinary gradient commands remain unchanged. API regression passes; stream19/19 includes round-trip through persistent recording, unsupported rejection, no output for malformed inputs and ordinary-mode controls. Initial new tests omitted the empty frame marker; preserved failure and corrected fixtures. Native mode implementation and CSS draw selection remain pending, so transparency residuals are not claimed fixed. Receipt: output/playwright/html-to-riv/linear-gradient-premultiplied-transport-progress.json.

## Frozen opacity full regression and visual audit

The corrected P05 frozen toolchain passes all 7,960 native checks against pinned
Chrome 153.0.8010.12. All 7,950 scene pairs have completed visual coverage:
7,686 exact compiler-input and full-image transfers from the reviewed P04
baseline, plus 264 exact transfers from the reviewed public opacity lifecycle
corpora. Both source audits completed, the union covers every gallery key, and
the combined proof was verified against its component hashes and completed
report. No new direct inspection or tolerance waiver is claimed.

Evidence: `output/playwright/html-to-riv/group-opacity-full-r2-validation-receipt.json`
and `group-opacity-full-native-r2/combined-visual-comparison.json` in the same
output root. The earlier disk-exhaustion run remains preserved. This evidence
applies to the frozen P05 toolchain, not subsequent experimental P06 changes.
P05 remains active for broader host/image modulation and backend qualification.

## Gradient coexistence and alpha visual controls

The frozen experimental P06 renderer passes16 synthetic images/160 analytic
samples exercising ordinary straight-alpha Rive ramps alongside premultiplied
CSS ramps. Both native paths, reversed insertion/draw orders, two-stop/simple
and three-stop/complex ramps, and host modulation1/.5 are covered. This tests
cache separation and preservation of existing straight-alpha behavior; it does
not qualify arbitrary host transforms or every renderer backend. Evidence:
`output/playwright/html-to-riv/linear-gradient-native-coexistence/receipt.json`.

All six distinct transparent/mixed-alpha browser/native/difference views at
240/390/768px have now been directly inspected and recorded. Remaining initial
gradient views still need inspection/artifact audits before public promotion.
The full128-frame automatic pass remains experimental runtime injection.

## Gradient diagnostic artifact audit

The new audit freshly recompiles all16 stripped diagnostic inputs with the
frozen baseline compiler and requires identical Rive bytes and runtime
requirements. It checks the independent authored gradient metadata, injected
pixel-bound owners, all128 original/clone frames at240/390/768/240px, source
geometry, stream identity and premultiplied commands, full image hashes, and
frozen renderer identity. Qualification remains runtime-experiment-only.
All16 scenes/128 frames pass. Twelve mutation controls reject missing or
duplicate frames, changed renderer/image/stream hashes, a false public claim,
changed paint or compiler CSS, missing pixel policy, wrong resize/geometry,
and missing clone identity. Source evidence is never mutated by the tests.

Evidence: `output/playwright/html-to-riv/linear-gradient-initial-lifecycle-r4/gradient-artifact-audit.json`
and `output/playwright/html-to-riv/linear-gradient-artifact-audit-controls.json`.
This closes the initial artifact audit, not the remaining42-view visual review
or public compiler promotion.

## Completed initial inspection exposes hard-stop residual

All48 distinct browser/native/difference views have been directly inspected.
The review receipt covers all128 lifecycle frames with exact full-image
transfers for repeated/original-clone views. Coverage is not qualification:
the768px hard-stop and decreasing-stop fixtures expose texture filtering
across a discontinuity. Chrome paints solid colors adjacent to the boundary,
but native produces mixed pixels. The hard-stop samples atx383/384 differ by
38 channel units; decreasing-stop x529 differs by161. Narrower initial widths
do not reproduce these sampled failures.

A new local boundary gate checks both fixtures across all16 lifecycle frames.
It fails4/16 (both768px scenes on original and clone), even though the existing
aggregate128-frame gate passes. No thresholds were widened and these failures
are not waived. See `linear-gradient-hard-stop-residual.json` and
`linear-gradient-hard-stop-gate-r4.json` under output/playwright/html-to-riv.
Fix discontinuity sampling before treating initial gradients as qualified.
Public CSS remains gated.

## Exact stop-table validation results

Generated function-name regression passes; all three minimal pipeline repros
(solid, ordinary gradient, CSS gradient) now render successfully. Temporary
debug logging is removed. With the frozen exact-table-r2 renderer, all128
initial Chrome/native frames pass and the targeted hard-stop gate passes16/16,
resolving the four r4 failures without threshold changes. Coexistence controls
pass16 images/160 samples on both native paths. The fresh artifact audit passes
16 scenes/128 frames. Both corrected768px boundary sheets were directly viewed:
the prior blended columns and highlighted difference stripes are gone.

Other changed images still need review. Table capacity/performance, wider
regressions, and public compilation/parity remain unqualified. Receipt:
`output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`.
The failed r5 pipeline run and r4 boundary reproducers remain preserved.

## Capacity and broader native regression

The frozen exact-table-r2 renderer passes14,336 analytic samples across both
native paths with adjacent tables containing2/3/258/510 stops and an ordinary
complex ramp in one frame.511 stops are rejected before rendering. Evidence:
`output/playwright/html-to-riv/linear-gradient-table-capacity/receipt.json`.

The first full renderer library run returned457 passed/17 failed/6 ignored.
Nine failures require missing external fixtures. Two ownership-test failures
pass when rerun individually with one test thread. The remaining failures
exposed additional stale names in standalone resource/atlas/preload helpers.
Those now use generated exports for generated libraries; the standalone
specialization compiler still uses its separate fixed fixture-source names.
After corrections, all170 native_metal module tests pass serially. This is
not a passing full suite; fixtures and broader regression remain. Earlier
failed runs are preserved. Latest source helper changes are not covered by
the already frozen exact-table-r2 binary receipt.
Receipt: `output/playwright/html-to-riv/linear-gradient-capacity-and-renderer-review.json`.

## Full renderer library regression completed

Restored missing fixtures using tools/fetch-test-assets.sh with checksum
verification and supplied the existing upstream checkout for parity tests.
With one test thread, the complete renderer-metal library suite passes474 tests
with6 intentionally ignored. The earlier17-failure run is preserved; its
fixture/ownership/name issues are now accounted for by the passing serial run.
The upstream commit and six parity fixture hashes are recorded in
`output/playwright/html-to-riv/linear-gradient-full-renderer-receipt.json`.

A fresh replay build is byte-identical to exact-table-toolchain-r2. Inspection
of module configuration confirms the most recent atlas/resource/preload helper
fixes are test-only; canonical replay pixel evidence remains applicable. This
does not finish the r6 changed-image review, performance qualification, broader
gradient combinations, or public compiler transport/parity.

### Exact-stop gradient visual audit complete — 2026-09-10

Corrected runtime experiment r6 passes128/128 Chrome/native original-clone frames and16/16 targeted hard-stop frames. Full-resolution review now covers every frame:39 direct views,79 within-run exact-image transfers and10 cross-run exact-input/full-image transfers; combined audit reports zero remaining. Five review-identity tests include eight semantic/RIV mutations and stale/wrong-path/failed audit rejection. Stop-table capacity controls pass14336 samples; full renderer suite passes474 tests with6 ignored. Public gradient CSS remains gated: transport/emission, native/WASM parity, performance and broader composition qualification remain. Evidence: `output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`, `linear-gradient-initial-lifecycle-r6/visual-coverage.json`, and `linear-gradient-review-identity-controls.json`.

### Gradient portable contract foundation — 2026-09-10

Added Rust version26 capability `layout-css-linear-gradient-v1` and occurrence payloads retaining corner/degree directions, unpremultiplied ARGB colors, omitted stops and signed pixel/percentage positions. Validation enforces2–256 stops, finite numeric values, unique non-root LayoutComponent targets and capability/version agreement. Four gradient contract tests and six opacity regression tests pass; the full library suite also passes. Version26 can coexist with opacity without weakening prior version checks. Public CSS emission remains gated while host installation, JS types and native/WASM parity are connected. Receipt: `output/playwright/html-to-riv/linear-gradient-contract-progress.json`.

### Gradient checked recording host and JS types — 2026-09-10

The probe validates version26 before drawing, converts retained gradient values into checked runtime paints, and installs occurrence targets atomically. Host tests pass four responsive widths,13 malformed-manifest controls and explicit missing-capability rejection; runtime tests pass original/clone resize and replacement/clear controls. JavaScript version26 declarations expose exclusive direction/position variants and pass typechecking. This qualifies the recording-host adapter only; public CSS emission, native/WASM parity, public native lifecycle pixels and unsupported-backend preflight remain. Receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

Gradient host regression completes24/24 after rebuilding the probe with required `native-glyph-controls`. Initial21/23 attempt is retained: both failures explicitly rejected the missing native-glyph build feature. Updated receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

### Overall progress page — 2026-09-11

Open `test-results/progress.html` for backlog totals, per-item evidence, current work and the last completed full regression. `python3 validation/progress-page.py --watch` rebuilds it when BACKLOG.md or validation/progress-state.json changes; the page reloads every60 seconds. Update progress-state.json at meaningful milestones and keep current failures distinct from frozen passing baselines. The watcher does not infer test status or declare qualification. Restart it if its process has ended.

### Public gradient candidate and stack correction — 2026-09-11

Public version26 emission is now connected. The previous gate is removed in the working tree, without claiming qualification. Six gradient transport/public tests and16-scene original-clone geometry test pass. Independent expectations exposed percentage fraction round-trip error30→30.000002; preserving authored percent token text fixes it. Negative angle expectations use equivalent canonical degrees. Large per-element emission frames caused default-stack failure on plain nested divs; finishing per-element emission before recursion fixes all46 contract tests and preserves65 accepted/66 rejected nested elements. Native/WASM/public pixel and broader composition checks remain. Receipts/logs: linear-gradient-stack-fix-receipt.json, linear-gradient-public-tests-r4.log in output/playwright/html-to-riv.

### Public gradient initial validation passes — 2026-09-11

Fresh native/WASM parity passes6 tests; public128 original-clone resize frames pass pinned Chrome geometry/native pixels. Five depth/element/hidden/source-target boundary tests pass. Fresh public artifact audit passes16scenes/128frames and23 negative controls reject mutations. All128 authored-source/full-image pairs match reviewed runtime diagnostics; specialized visual transfer audit remains pending. New composition corpus has54 Chrome reference images across18 accepted cases, including default image repetition beneath transparent borders. Broader semantics/performance/full regression remain; P06 is active, not qualified. Evidence: output/playwright/html-to-riv/linear-gradient-public-integration-progress.json.

### Gradient composition defect confirmed — 2026-09-11

Initial public visual transfer audit completes128/128 with zero new inspections, based on exact authored HTML/CSS and full browser/native PNG identity plus independent public provenance. Broader composition native r1 compares54 views:30 pass,24 fail, all geometry passes. Eight cases fail at each width, including transparent/translucent/corner/wide/rounded borders, content clipping and opacity overlap. Inspected horizontal transparent-border pair confirms Chrome repeats endpoint colors beneath borders while current native paint clamps. Correct fix needs2D tile coordinates wrapped before scalar gradient projection; simply repeating scalar t cannot represent diagonal/corner tiles. Evidence: linear-gradient-public-lifecycle-r1/visual-public-gradient-transfer.json and linear-gradient-composition-native-r1/receipt.json under output/playwright/html-to-riv. P06 remains active.

### Repetition implementation candidate — 2026-09-11

Public compiler regression before tile changes completes453 tests across79 suites. New checked tiled-premultiplied paint transport preserves existing untiled commands; API36 and stream22 tests pass. Candidate native gradient retains immutable tile metadata through modulation; runtime requests tiling for borders, shader auxiliary data preserves2D UV and wraps before projection. Native build/pixel verification pending; original24/54 composition failures remain evidence. Initial performance capture completes80 measured runs plus16warmups, recording end-to-end replay only (not GPU/frame timing). Evidence: linear-gradient-public-module-receipt.json, linear-gradient-tile-transport-tests.log and linear-gradient-performance-initial/receipt.json under output/playwright/html-to-riv.

### Tiled composition pixels corrected — 2026-09-11

Frozen tiled toolchain builds and solid smoke pass. Composition r3 now54/54 automatic comparisons pass, versus30/54 before; targeted border gate36/36 samples passes and detects the earlier3 fractional-border visual failures. Transport58 and coordinate3 tests pass. Corrected full-image review/artifact audit, tiled original-clone composition lifecycle, backend/modulation/performance and full regression remain. Initial128-frame replay is running. Evidence: output/playwright/html-to-riv/linear-gradient-tile-progress.json.

### Tiled lifecycle and renderer regression pass — 2026-09-11

All18 public composition scenes compile once and pass144 original/clone Chrome/native frames plus fresh lifecycle artifact audit. Every authored-source/full-image pair equals a reviewed static composition; final transfer audit underway. Corrected static audit passes54/54,27direct/27exact transfers. Full native renderer suite479pass0fail6ignored; host synthetic16images/1328analytic samples pass across bothMetalpaths/affine/modulation/sharedstops. Current compiler regression and broad integrated native/performance qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-tile-progress.json.

### Public gradient focused validation complete; broad integration running — 2026-09-11

Current compiler regression passes454 tests across80 suites, with no failures or ignored tests. All144 composition lifecycle frames now have audited exact-source/full-image review transfer from corrected static evidence; all4 stale/mutated evidence controls reject. Together with the initial128 public frames, focused public lifecycle coverage is complete. The normal browser regression now registers16 initial and18 accepted composition fixtures (102 additional comparisons); ten deliberately unsupported composition inputs remain rejection-only cases. Frozen tiled-toolchain full native regression is running against Chrome153.0.8010.12. P06 remains active pending broad regression, backend and performance qualification. Native Metal replay explicitly rejects MSAA mode, so shader compilation alone is not runtime MSAA evidence.

### P06 final integrated qualification — 2026-09-11

The final integrated toolchain completes 8,052 unchanged broad reviewed images,
23 JS tests, package/types, focused real-native lifecycle/composition/resource
checks and 144 retained-renderer timing configurations. Accepted scope and hash
bindings are in [linear-gradient-qualification.json](validation/linear-gradient-qualification.json).
Eight extreme-stop frames intentionally preserve a hard stop lost by Chrome;
these are explicitly recorded divergences, not passing Chrome pixel comparisons.
Performance includes completion wait/readback and excludes layout, paint
recreation, resize and presentation throughput. Prior failed experiments remain
retained and do not supersede the final scoped evidence.
