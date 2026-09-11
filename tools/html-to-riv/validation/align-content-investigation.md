# L04 align-content preparation

Status: compiler/runtime/host implemented; corrected native297/297 passes and
all297 image pairs are reviewed. Corrected vector geometry297/297 passes;
7 text pixel failures remain explicit. Fresh full native regression is running.
The feature remains unqualified until that gate completes.

[CSS Flexbox section8.4](https://drafts.csswg.org/css-flexbox-1/#align-content-property)
defines line distribution separately from item alignment. The six Flexbox1 values
are flex-start, flex-end, center, stretch, space-between and space-around. The
property is not inherited and its initial value is stretch. It has no effect on
nowrap containers. A wrapping container remains eligible even when its items fit
on one line. The authoring reset currently selects flex-start explicitly; that
reset must stay distinct from the CSS-wide initial keyword.

Local seam: layout/layout_component_style.rs currently derives both align-items
and align-content from the same LayoutAlignmentType. Setting one compiler
alignment field would change both, violating independence. The layout style
applier already maps all six values to Taffy; a typed per-layout override applied
after base style translation can preserve raw Rive behavior. Its host payload,
capability, target/schema validation and clone/clear invalidation need the same
explicit checks as L03. Further distributed keywords belong to the following L05
work; do not imply their support from this reference.

`node tools/html-to-riv/validation/align-content-oracle.mjs` records72 scenes and
216 viewport measurements using Chromium153.0.8010.12. Four directions, six
values, wrap/nowrap/single-item modes and240/390/768 widths exercise independent
align-items:center, gaps and mixed cross sizes. Output:
output/playwright/html-to-riv/align-content-oracle.json. Log:
/tmp/align-content-oracle.log. These are independent browser expectations, not
runtime qualification or browser-baked compiler output.

Compiler red control: output/playwright/html-to-riv/align-content-initial/input.json
selects row/wrap/center. The public CLI exits1 with unsupported-property at
css:1:106, retained in diagnostics.txt. Next: public cascade/manifest and native
geometry regression against this oracle, then implementation and full visual
validation. Add negative free-space, percentage/min/max sizing, nested text,
explicit inherit versus default noninheritance, CSS-wide/var() controls and real
compositions before qualification. Grid and editor integration remain excluded.


## Runtime seam implementation

Added CssAlignContent with six Flexbox1 values and a private per-occurrence
override on LayoutComponent. The handle-based setter releases its owner borrow
before style synchronization and dirty propagation. Clone preserves the override;
None restores the legacy Rive policy. The final style sweep applies align-content
after container translation, without altering align-items. Default instances
retain None. No compiler grammar/format/host support is claimed yet.

Public CLI red control already rejects align-content. The first runtime test
build exposed the missing API (/tmp/align-content-runtime-red.log), then the
first implementation run encountered the compiler's existing wrapped/center
guard (/tmp/align-content-runtime-first.log). For isolated runtime testing,
align_content_runtime.rs compiles nowrap and restores the authored wrap bit
through the imported style's actual serialized field before owner sync. It
strips unsupported align-content and installs the proposed runtime override.
This temporary test setup must be removed when the compiler transport lands.

Runtime tests2/2 pass (/tmp/align-content-runtime-restored.log):216 independent
Chromium box comparisons across4 directions,6 values,3 modes,3 widths; plus
post-layout policy replacement, per-instance isolation, clone, clearing to legacy
default and resize. Oracle fixture is tests/assets/align-content-boxes.json,
compact copy of the independently generated reference with generator metadata.
No browser rectangles enter compiled Rive or production runtime state.

Next: explicit accepted syntax/reset-versus-initial semantics, compiler/requirements
transport and capability, strict target/schema validation, native/WASM parity,
remove the runtime test bypass and existing wrap alignment guard only when the
independent line policy is installed, then broader browser/native/vector visuals.
Also cover cross-axis overflow and CSS-wide/custom-property substitution before
claiming qualification.

Combined alignment runtime regression passes6/6
(/tmp/align-content-runtime-regression.log): new216 line-alignment comparisons
and prior432 item-alignment comparisons, with both lifecycle suites. All
processes in this increment are terminal. Runtime source is newer than the
publisher/probe binaries; rebuild them after compiler/host transport is added.


## Compiler and host transport

Accepted: flex-start, center, flex-end, stretch, space-between and space-around.
The authoring reset defaults to flex-start; CSS-wide initial/unset and invalid
var substitution use stretch. Explicit inherit copies the parent's computed
value; omission does not inherit. Modern normal/start/end, baseline forms,
space-evenly and safe/unsafe modifiers remain explicit unsupported syntax.
Block-text inner flex properties remain ignored.

Version7 requires nonempty unique layout_align_content targets and capability
layout-css-align-content-v1. It may carry previous payloads; version1..6 reject
line-alignment payloads. Host validates capabilities, schema and all actual layout
targets before installing policies. Missing support, old version, duplicate,
style-object and absent targets are tested and rejected before producing a stream.

Every wrapped container emits the profile's independent line policy, including
flex-start, so the old wrap/align-items/justify-content guard is removed. Nonwrapped
nondefault values are also transported; nowrap itself remains unaffected.
Align-self and paint descendants use max(version,old-version), preventing a
version7 downgrade. Native tests now compile the original CSS and validate/install
the published target; the temporary wrap-bit deserialization helper is removed.
The lifecycle test intentionally ignores the manifest to test clearing to raw
Rive behavior; it does not substitute for compiler/host validation.

Public suite215/215 passes (/tmp/align-content-module-first.log), followed by
3/3 focused contract tests after adding a combined-payload/reset regression
(/tmp/align-content-contract-final.log). Host6/6 passes (/tmp/align-content-host.log).
Native publisher/probe and WASM builds pass (/tmp/align-content-publisher-build.log,
/tmp/align-content-wasm.log). Native/WASM parity9/9 passes for the initial216-case
alignment matrix (/tmp/align-content-parity.log). TS checks pass after adding
the new capability literal to the explicit union test (/tmp/align-content-types-final.log).
The initial schema test caught and fixed a missing serde name on the capability.

Native initial216/216 passes (/tmp/align-content-first.log). Sheets generated in
align-content-first/sheets; none have been directly inspected yet. Expanded the
independent oracle with negative cross-axis free space:96 scenes/288 viewports.
Both runtime tests pass including all288 boxes (/tmp/align-content-overflow-runtime.log).
Original216 oracle artifact is preserved; expanded output and log are
align-content-oracle-expanded.json and /tmp/align-content-oracle-expanded.log.
Added overflow fixtures to the rendered corpus. Native overflow72 cases running
in /tmp/align-content-overflow.log (session82009). Recheck its live handle before
starting another Playwright pixel run. Final expanded parity, composed layouts,
visual inspection, vector checks and full native regression remain required.

Native overflow72/72 now passes (/tmp/align-content-overflow.log); all288
focused native cases pass in total. Both pixel runs are terminal. No direct
visual inspection has occurred yet. The expanded corpus parity is next.

Expanded parity is running in /tmp/align-content-expanded-parity.log
(session93785). Check completion before claiming the expanded corpus matches
WASM. No other build or pixel run is live at this checkpoint.


## Visual review caught a fixture specificity mistake

Expanded parity9/9 passed (/tmp/align-content-expanded-parity.log). Three real
compositions were added: centered plan tiles with order/mixed item heights,
column-reverse summary with independent space-between line distribution, and
row-reverse percentage/max-width cards with space-around. Native9/9 passes
(/tmp/align-content-compositions.log), and all9 were directly visually reviewed.

The first box contact sheet revealed that the intended mixed cross sizes were
actually equal in both browser and native. The selector #root>div overrides #b
and #c because it has higher specificity. Inspecting the independent oracle
confirmed all child heights20, despite attempted35/50 overrides. The earlier288
passes are valid equal-size cases, but do not prove mixed-size alignment.
Preserved original oracle-expanded, native-first/overflow and vector artifacts.

Corrected generator and rendered fixture selectors to #root>#b and #root>#c.
The generator now asserts actual cross sizes20/35/50 for every multi-item case
and throws if the source stops exercising them. Corrected oracle output:
align-content-oracle-mixed.json, log /tmp/align-content-oracle-mixed.log.
The288 geometry comparisons and lifecycle tests still pass2/2
(/tmp/align-content-mixed-runtime.log). This fixes test coverage, not runtime code.

The initial vector run ended290/297 with7 composition text pixel failures
(/tmp/align-content-vector.log); it used the old equal-size matrix and is not
final qualification. The full native run that loaded those obsolete fixtures
was deliberately interrupted with SIGINT to its verified Playwright PID33157,
then session44234 returned exit130. Its partial artifacts/log remain, and it
must not be reported as a passing full run. This was a source coverage correction,
not a timeout or observation failure.

Corrected native297-case run is active in /tmp/align-content-mixed.log
(session18972).
Corrected parity is active in /tmp/align-content-mixed-parity.log (session56282).
After these settle, inspect corrected box matrices, rerun vector, then fresh
full native regression. Only9 composition images are fully visually qualified
so far. The inspected old box sheet must not qualify the corrected source.

Added box-contact-review.py to generate full-frame half-scale box contact sheets.
It groups only identical browser/native PNG hash pairs and retains every source
case and hashes in a manifest; sheet generation never marks a review complete.
This is for box geometry visuals only, not text raster inspection.


Corrected mixed native297/297 passes (/tmp/align-content-mixed.log). All297
pairs now visually accounted for. The288 box cases comprise138 distinct PNG
pairs, all inspected in23 full-frame half-scale contact sheets; exact byte hashes
cover the150 duplicate pairs, with every source retained in the contact manifest.
The9 compositions have identical source/browser/native hashes to their direct
reviews. Receipts: align-content-mixed/visual-inspection.json and
align-content-mixed-box-review/{manifest,visual-inspection}.json. No outstanding
native image review. Corrected parity9/9 passes (/tmp/align-content-mixed-parity.log).

Corrected vector297-case run active in /tmp/align-content-mixed-vector.log
(session7325). Fresh full native regression must start after this pixel run
finishes; the earlier align-content-full remains interrupted and unqualified.


Corrected vector290/297 pixels pass; all297 geometry comparisons pass with
maximum absolute error0.00937652587890625px. Seven composition text raster
failures remain. All297 pairs reviewed:288 exact source/browser/native matches
to corrected native box reviews and9 directly inspected text compositions.
Evidence: /tmp/align-content-mixed-vector.log and
align-content-mixed-vector/{baseline-comparison,visual-inspection}.json.

Fresh full native run started after corrected focused runs and reviews settled:
/tmp/align-content-corrected-full.log, session6656. Do not confuse it with the
interrupted align-content-full. On completion compare current images against
align-baseline-full, align-composition-native and align-content-mixed, inspect
any changed pairs, then finalize native-profile qualification if all gates hold.
No production code or fixtures changed after this fresh full run started.

Full public compiler suite rerun completed successfully:216/216 tests, zero
failed or ignored, command `CARGO_INCREMENTAL=0 cargo test -p nuxie-html-to-riv
--features native-glyph-controls`, log /tmp/align-content-module-final.log.
Session12456 returned exit0. This includes the added contract regression in
one complete run. Full native visual regression session6656 remains active.

Final L04 native qualification: full corrected regression2029/2029 passed in
11.5minutes, session6656 returned exit0. All2019 image pairs match exact source
HTML/CSS and browser/native PNG hashes in reviewed baselines align-baseline-full,
align-composition-native and align-content-mixed. No image inspection remains.
Receipts: align-content-corrected-full/{baseline-comparison,visual-inspection}.json;
log /tmp/align-content-corrected-full.log.

L05 runtime source edits began near the end of this run, but compiler, probe,
renderer, WASM, fixtures and reset hashes were captured before editing and
verified identical after full run completion. Evidence:
align-content-corrected-full-input-hashes.json. L05 runtime investigation tests
are separate and do not extend this L04 qualification to new policies.

L04 is native-profile qualified. Corrected vector geometry297/297 passes, but
seven composition text pixel failures remain open; vector profile not qualified.
