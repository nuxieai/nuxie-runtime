# L02 order

Status: qualified for the documented native rendering profile. Three focused
vector text-card pixel failures remain explicit renderer limitations. Historical
increment notes below preserve failed experiments; the final qualification receipt
at the end records the current outcome.

Accepted syntax is a signed CSS integer, including explicit plus signs and leading
zeros, plus the existing inherit/initial/unset and var() handling. Initial is0;
order is not implicitly inherited. The shared CSS parser clamps finite oversized
integer tokens to signed32 endpoints, matching the pinned Chromium oracle.
Decimals, exponent syntax, units and lists are rejected. Integer-producing math
functions remain outside the profile with a diagnostic, including via var().
Invalid substituted integer syntax resets to unset; valid unsupported functions
are not silently reset. General non-finite-number admission remains unchanged.
Oracle: validation/order-oracle.mjs and /tmp/order-oracle.log. It checks both
integer endpoints, out-of-range values, decimals, exponent form and signed syntax.
[CSS Display 3](https://drafts.csswg.org/css-display-3/#order-property).

The compiler resolves each sibling's style against its original DOM and computed
parent, then stably sorts complete sibling subtrees by order for emission. Imported
child layout follows this visual sequence; CSS paint order remains unqualified. The source
map is returned in numeric authored-path order. Authored IDs, paths, selectors,
text identities and their correct object references are preserved. Generated
object indices address a particular artifact; they are not stable IDs across CSS
edits (paint/font/style records already change those indices). No runtime policy,
new binary format primitive, DOM rewriting or browser-baked positions are added.

Current profile containers are flex, with text-only block boxes containing no
child layout items. This does not admit general block flow, grid, absolute
positioning, interaction/navigation behavior, scripts or animations.

The first public test failed on unsupported-property before implementation.
It now imports the emitted scene, checks visual order at240/390/768, ties,
negative order, source IDs/paths and :nth-child matching. A second regression
covers cascade, explicit inheritance, non-inheritance, resets, substitution and
integer admission. Earlier failed fixture setup (missing viewport) and the
superseded out-of-range rejection expectation are retained in /tmp/order-module*
logs; Chromium evidence corrected the latter expectation.

Six permanent browser fixtures cover row/column, reverse wrapping, nested
paint overlap with/without clipping, and responsive text cards. These have been rendered, with the failures below. All browser geometry/pixel/visual
qualification must finish before L02 qualifies.

Current validation: public suite201/201 passed, native and WASM builds pass.
The initial browser invocation found no tests because the fixture append script
constructed its additions without extending the saved list; fixed, with the
empty-run log retained. Actual six-fixture native run passes12/18: row, column,
reverse wrapping and cards pass. Overlap and clipped-overlap fail at all widths,
with geometry matching exactly. Inspected order-overlap240: Chromium paints red
last, runtime stream paints red, green, blue, making blue incorrectly topmost.
This disproves the claim that sorted subtree emission alone guarantees CSS paint
order. The implementation is not qualified.

Ranked paint hypotheses: reversed runtime traversal; unpainted-container grouping;
order-specific interaction. Three controls (no-order, painted parents, and both)
fail9/9, so this is not exclusive to the order property or unpainted parents.
Controls remain in known-gaps.json; logs/artifacts order-controls and
/tmp/order-controls.log. LayoutComponent::needs_drawable_proxy depends on clip,
paints or ForceDrawableProxy; adding paint did not fix the controls. Next inspect
DrawRules/DrawTarget format semantics and imported paint traversal using a
minimal direct-sibling overlap control before changing raw runtime behavior.
Source/image mapping and .riv references must stay correct. Parity passed9/9 in /tmp/order-parity.log. Original failure artifacts order-native-2 remain.

Native/WASM corpus parity now passes9/9 including the six actual order fixtures
(/tmp/order-parity.log). All processes from this increment are terminal.

DrawRules investigation (2026-09-09): three bounded experiments were rejected.
1. Targets parented to artboard:9/18 native, losing text-card pixels as well as
   the six overlap cases (order-drawrules-after).
2. Targets correctly parented to their DrawRules:9/18 (order-drawrules-parent).
3. Additionally copying flattened_draw_rules to layout proxies:12/18
   (order-proxy-rules), restoring cards but still failing overlap/clipped overlap.
The latter two overlap240 native images were inspected directly; both retain
blue0–100/green100–140/red140–180, versus Chromium blue0–40/green40–80/red80–180.
The recording stream still paints red, green, blue. None qualifies paint order.
Other experiment images have not been fully visually reviewed.

Runtime evidence: Artboard collects authored drawables in import order and injects
layout proxies after each layout subtree. sort_draw_order traverses the final
linked list backwards to paint. DrawRules flattening occurs before proxy injection;
DrawableProxy starts with default rules. A DrawTarget addresses the authored layout
start, not its generated end proxy, so merely moving earlier siblings after that
record does not reverse complete nested paint groups. The local C++ source at
/Users/levi/dev/oss/rive-runtime/src/artboard.cpp has the same proxy injection and
backwards list model; this is not evidence for blindly reversing raw Rive paint.
The simple direct-box overlap control would require currently excluded negative
margins, positioning or transforms; the existing nested overflow control exercises
only admitted syntax.

All experimental compiler records and the proxy-inheritance edit were removed.
The attempted writer is preserved in order-drawrules-experiment.rs.txt; corresponding
.riv files, maps, streams, browser/native PNGs and logs remain in each experiment's
output. Next investigate an explicit compiler-scene paint policy that orders whole
layout subtrees without changing layout child order or raw Rive behavior; include
clipping, nested text, translucent backgrounds and resize in its regression suite.
This is an internal implementation dependency, not an externally blocked goal.

Restoration checks: build passed; public suite201/201 passed; native/WASM
JavaScript corpus9/9 passed; focused native12/18 (the original six overlap failures).
Logs: /tmp/order-restored-build.log, /tmp/order-restored-module.log,
/tmp/order-restored-parity.log, /tmp/order-restored.log. All processes terminal.
All18 restored browser/native/diff image triplets were directly inspected via six
three-width sheets in output/playwright/html-to-riv/order-restored/sheets. Row,
column, reverse wrapping and text cards match geometry/reading order at all widths;
text has the already bounded raster differences. Both overlap fixtures visibly
retain reversed sibling coverage at every width. No tolerance was changed.

Whole-subtree experiment (2026-09-09): Artboard now has an explicit, default-off
set_css_paint_order switch. It derives a paint-only sequence from the original
imported drawable list, recursively reversing sibling groups while keeping each
authored layout and its generated background/clip proxy paired. Consecutive
non-layout content records retain their internal order. The stored drawable list
and imported layout children are unchanged; disabling the switch restores raw
Rive paint order. Hosts must apply it to each instance. The probe currently enables
it only under NUXIE_EXPERIMENTAL_CSS_PAINT_ORDER=1; no published capability yet.

Initial experiment order-subtree failed18/18 with a RefCell borrow panic while
ancestry traversal reached the already mutably borrowed artboard. An explicit
root boundary fixed that; original panic logs/artifacts remain. Corrected
order-subtree-root passes18/18 native. Its image review has11 exact source+browser+
native identity transfers from order-restored and seven changed cases inspected
directly (both overlap fixtures at all widths and text-cards240). All18 reviewed.
The source/image comparison receipt is baseline-comparison.json in that gallery.

Additional native controls pass9/9 (order-subtree-controls), including no order
and painted-parent variants; all nine images inspected directly. Four new
composition fixtures cover translucent overlapping paints, child clips, nested
clips and overlapping text. order-subtree-composition passed12/12, but inspection
found that its nested-clip fixture had failed to insert inner HTML because the
replacement assumed unquoted IDs. The actual corrected fixture now includes
app/bpp/cpp; order-subtree-nested-corrected passes3/3, directly visually inspected.
Do not count the superseded three cases as evidence of nested content. All other
nine composition images were inspected directly. Combined current native coverage
is39/39 across original, controls and corrected compositions. No tolerances changed.

Public suite202/202 passes (/tmp/order-subtree-module.log), including a new
import/draw regression which checks default raw paint order, enabled CSS order,
idempotent reapplication, disabling/restoration, and unchanged source-mapped layout
positions through240/390/768 resizing. This policy is still experimental: publish
a strict capability/version contract, update JS/native host validation and instance
application, add policy rejection tests, run native/WASM parity and the broad native
visual corpus before qualifying L02. In particular, a focused passing experiment
is not evidence that ordinary published scenes already enable the behavior.

The unchanged publish contract and current fixture corpus retain native/WASM
parity9/9 (/tmp/order-subtree-parity.log). This checks bytes/maps/requirements,
not runtime activation of the experimental switch. All processes from this
increment are terminal.

Published capability increment (2026-09-09): compiler now emits
layout-css-paint-order-v1 whenever authored source paths contain two siblings,
including top-level siblings and all-default order. Single-element and single-child
chains do not require it. The capability is artboard-wide and valid in existing
requirements versions1–5, like other global capabilities; it has no per-object
payload and does not alter the schema version. Old hosts reject the unknown or
unsupported capability rather than silently painting incorrectly. The TypeScript
capability union and README host instructions now include it.

Probe support is manifest-controlled before first draw; the experimental env flag
is removed. NUXIE_DISABLE_CSS_PAINT_ORDER simulates an unsupported host and rejects
before a stream is written. Host regression also removes the capability and sets
the obsolete experimental flag, proving raw order remains off unless declared.
Native/WASM builds pass; public suite203/203 passes; host tests4/4 pass; native/WASM
corpus9/9 passes; TypeScript passes. First module run failed a text-policy test
host's explicit capability list, and initial type checks failed its expected union;
both now enumerate the new capability. Logs use /tmp/order-capability-* prefixes.

Shipping-path focused native30/30 passes (no experimental flag), with all30 source,
browser and native images identical to reviewed experiment baselines. Gallery
order-capability-focused/baseline-comparison.json is the transfer receipt.
The broad native1612-case run is ACTIVE, session16401, log
/tmp/order-capability-full.log and gallery order-capability-full. Do not restart
it without polling that handle or proving termination. L02 remains in progress
until the broad run and its visual review finish.

Additional instance isolation regression passes in the focused order suite5/5
(/tmp/order-capability-instance-2.log): cloning an enabled instance starts with
raw order; explicitly enabling the clone changes only that clone, and disabling
the original does not change it. First test harness held a RecordingFactory
borrow through draw and panicked; releasing it before draw fixed the harness
(/tmp/order-capability-instance.log retained). The earlier203-test full suite
preceded this additional test; the new five-test file passes independently.

Broad native run completed1609/1612, terminal session16401. Three failures are
reverse-row-overflow-paint at240/390/768. Of1602 image pairs,1599 are exact
source/browser/native matches to normal-wrap-corrected-full and
order-capability-focused. Direct inspection of240 confirms reversed coverage;
390/768 original failure images still await direct review. The original evidence
is preserved in order-capability-full and /tmp/order-capability-full.log.

Four new oracle fixtures cover row-reverse/column-reverse with ties and mixed
order, all12 failed before the direction-aware correction (order-reverse-paint-red).
Directly inspected both ordered240 browser images: Chromium reverses its flex
paint sequence in both axes. This is observed Chromium behavior; do not infer a
universal cross-browser conformance claim from these results. The runtime policy
now retains sibling list order inside reverse-direction parents (Rive traverses
backwards), and reverses it for normal parents; nested parents are evaluated
independently. A public regression covers all four directions with mixed order.
Corrected focused and public runs are active; broad requalification remains.

All1602 original broad pairs are now visually accounted for:1599 identity
transfers and the three reverse-overflow failures directly inspected. Corrected
focused native45/45 passes;33 exact image transfers and12 new reverse-direction
images directly reviewed. Public suite205/205 passes, including all-direction
paint order. Logs /tmp/order-reverse-paint-fixed.log and
/tmp/order-reverse-paint-module.log.

Focused vector42/45 passes, with all45 geometry comparisons within tolerance
(max error 0.022491455078125px). Three text-card pixel cases fail; direct review finds
matching order, wraps and boxes with glyph raster differences. At240 global mean
error1.05079 exceeds1 and title interior error7.01764 exceeds6. All45 images
reviewed:39 identity transfers from native and six text images directly inspected.
No tolerances widened; this does not qualify vector text-card pixels.

Broad corrected native run is ACTIVE: session12827,1624 tests, log
/tmp/order-direction-full.log, output/gallery order-direction-full. Previous
session16401 is terminal failed. Poll12827; do not restart on an observation
timeout. L02 remains in progress until broad corrected validation/review finishes.

Current native/WASM corpus parity remains9/9, including new reverse-direction
fixtures (/tmp/order-direction-parity.log). All focused/build/test processes
are terminal; only broad native session12827 remains active.

Final L02 native qualification (2026-09-09): corrected broad native1624/1624
passed, session12827 terminal exit0. Machine receipt reports expected1624,
checks1624, zero reporter errors and no non-passing checks. All1614 scene pairs
are exact source/browser/native image matches to fully reviewed
normal-wrap-corrected-full and order-reverse-paint-fixed baselines. Comparison and
visual-inspection.json in order-direction-full have no remaining cases.
The10 non-image controls also pass. /tmp/order-direction-full.log and
/tmp/order-direction-full-compare.log retain full command evidence.

Qualified scope: signed integer order/cascade/substitution, stable ties with
authored source IDs/paths/selectors, responsive layout and direction-aware sibling
paint groups with nested clips/transparency/text. Published artboard-wide capability
is required for sibling layouts, checked before draw, installed explicitly per
instance, and absent by default for raw Rive. Native/WASM parity9, host4, types and
public205 tests pass. No CSS grid, block flow, positioning, script, editor, animation,
interaction or binding admission. Unsupported integer math retains diagnostics.

Vector scope remains42/45 focused pixels,45/45 geometry and all45 images reviewed.
The three text-card raster failures are not waived and do not imply a failure of
qualified native paint/layout semantics. Broader existing vector-text issues stay
open. L02 is complete for its native profile; the overall compiler goal and next
backlog items are not complete. All processes from L02 are now terminal.
