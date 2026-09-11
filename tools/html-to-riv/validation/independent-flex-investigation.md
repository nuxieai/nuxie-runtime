# L08 independent grow/shrink — investigation

Compiler support is pending; no qualification claim.

`Style::resolve_flex` currently rejects unequal factors. `Style::write` selects
Fill only for positive grow, stores grow as the Rive fractional weight, and
substitutes basis into the main dimension when grow is zero. Runtime
`LayoutComponent::apply_sizing_item_style` maps Fill's single fraction to both
grow and shrink; fixed/hug sizing forces both factors to zero and basis to auto.
Removing the rejection alone would therefore silently change authored behavior.

Proposed interface: one optional occurrence policy carrying independent finite,
nonnegative grow/shrink factors. Apply it after ordinary Rive sizing resolves
the item style. Preserve default None and copy the policy on scene cloning;
the setter must release object borrows before style synchronization and dirty
layout when installed, changed or cleared. The existing explicit basis fields
must reach layout even for shrink-only items. Keep CSS width/height distinct
from basis so min/max and automatic basis retain their intended meaning.

Transport: versioned runtime requirements with a new capability and per-object
payload. Reject unsupported hosts, duplicate/non-layout targets, invalid
numbers and inconsistent version/capability combinations before drawing.
Native publisher, WASM publisher, JS types and checked native host must agree.
Use the existing alignment occurrence-policy pattern instead of changing Rive's
single-weight binary interpretation for all ordinary files.

`validation/independent-flex-oracle.mjs` captures Chromium geometry across four
directions, six unequal grow/shrink pairs, and four basis/constraint modes at
240/390/768. Fixed basis is70px so unconstrained columns have positive free
space; constrained columns and explicit100px auto basis exercise negative free
space. Percentage basis uses a definite parent. Sibling b has independently
incremented factors. Capture output is
`output/playwright/html-to-riv/independent-flex-oracle.json`.

Required tests: parser/cascade, shorthand and longhand precedence, inherit and
initial/unset, public manifest validation, native/WASM byte/map/requirements
parity, original/clone resize geometry, lifecycle install/change/clear, checked
host rejection, realistic growing/shrinking compositions, native/vector pixels,
actual visual review and full regression. Keep L09 sub-unit partial-fill,
L10 content-derived auto basis and L11 indefinite percentage basis reproducers
explicit until their own implementation is demonstrated. L08 should preserve
the existing supported equal-factor behavior and avoid weakening its checks.

During this investigation the L07 full native run15587 is active. No runtime or
compiler executable is rebuilt for L08; preserve its recorded input hashes.

Capture completed successfully:96 scenes/288 viewports, Chromium153.0.8010.12 (`/tmp/independent-flex-oracle.log`, session93159 exit0). Four original public compiler rejection inputs and diagnostics retained in `independent-flex-initial/`, all exit1 with unsupported-flex-factors. No executable changes.

Runtime policy source added: `CssFlexFactors::new` validates finite/nonnegative factors, optional per-occurrence storage defaults None and clones, setter synchronizes after releasing owner borrow and dirties layout. Item style applies independent factors and encoded basis after ordinary Rive sizing, excluding grid. Added lifecycle test for grow-only and shrink-only siblings across390/240/180/390, clone and clear, plus invalid-number tests. These tests are not executed yet; compile check running82264 (`/tmp/independent-flex-check.log`). Compiler parsing/manifest/host transport remain pending. Active L07 binaries preserved.

Requirements source added: version9, layout-css-flex-factors-v1 capability, per-object finite nonnegative grow/shrink payload. Validation rejects duplicate IDs, missing capability, wrong versions, invalid numbers and non-layout targets. Version9 allows older distribution payloads without requiring them. Public contract tests cover positive roundtrip and rejection cases. Two compile errors in the new lifecycle test (Option handle access, then redundant unwrap on with_artboard result) were corrected; no production compile failure identified. Current all-tests compile check28598 (`/tmp/independent-flex-contract-check2.log`) pending. Compiler emission, checked host and JS version/types are not integrated yet; no version9 scene is emitted by the compiler.

Compiler/host source integrated: unequal factors emit version9 occurrence payload; authored main dimensions stay distinct from encoded basis; shrink-only basis reaches runtime. Content-derived auto basis remains an explicit error, and existing equal-factor lowering stays intact. Older descendant distribution now uses max(version,8) to prevent downgrading version9. Checked host advertises the new capability, supports disabling it for rejection tests, and installs validated factors before layout. JS declarations add version9 and exclude the payload from older variants. Source compile check including all tests/probe passes (31022 exit0, `/tmp/independent-flex-emission-check.log`). Updated two earlier rejection expectations to reflect accepted factors versus still-unsupported content basis. No executable rebuild while L07 full regression is active; public/native/WASM execution remains pending.

Public oracle test added: compiles each96 original CSS scenes, installs emitted requirements, compares288 reference viewports on original and clone. Public compiler tests cover unequal shorthand/longhands, variables, inherit/initial/unset, important, and version9 surviving nested older alignment policies. JS type expectations now include version9/new capability and narrow the required factor payload; typecheck passes (78097 exit0, `/tmp/independent-flex-types3.log`). All-tests/probe compile check97432 (`/tmp/independent-flex-oracle-check.log`) pending; tests are not yet executed.

All-tests/probe source compile check97432 passed (`/tmp/independent-flex-oracle-check.log`). Added 99 prospective visual fixtures (96 factor/basis boxes and three text compositions), kept separate from active L07 cases. Added host resize and fail-before-stream tests for missing capability, old version, negative/duplicate payloads and invalid targets. Browser composition capture completed (3 scenes/9 viewports; `/tmp/independent-flex-composition-oracle.log`); all nine screenshots directly inspected, with receipt in `independent-flex-composition-oracle/visual-inspection.json`. Narrow reverse-row text overflow is intentional and preserved. Extended lifecycle test to change factors without resizing, assert unchanged setters return false, and check clone independence. These new tests still await execution after L07 full regression and capture retry; no native/pixel qualification claimed.

Lifecycle source check80422 passed (`/tmp/independent-flex-lifecycle-check.log`), including same-viewport mutation and clone independence. Requirements tests now exercise NaN, both infinities and negative values on each factor, plus rejection of unknown payload fields. All five recorded L07 input hashes still match while full regression15587 runs. SUPPORT and VALIDATION distinguish this source candidate from the qualified equal-factor baseline.

L07 full regression and isolated infrastructure retry are terminal and native-qualified; original failure and verified input hashes retained. Merged99 L08 scenes into the visual corpus after that gate. Full public test suite running18463 (`/tmp/independent-flex-module.log`), first executed L08 validation. No version9 native/WASM publisher rebuild or pixels yet.

First executed public suite18463 exited101 (`/tmp/independent-flex-module.log`): existing custom-properties test incorrectly expected unsupported auto basis although its shared fixture supplies height20px in a column. Updated that negative case to explicitly set height:auto; valid explicit-size reset case remains. Updated diagnostic version ranges to include9. Public suite retry9678 running (`/tmp/independent-flex-module2.log`). Original failure log retained.

Public retry9678 exited101 on a new lifecycle assertion: occurrence setter returns successful target acceptance, not whether the value changed, matching existing alignment setters. Corrected the repeated-set assertion to expect success; preserved mutation, resize, clone isolation and clear geometry assertions. Original failure in `/tmp/independent-flex-module2.log`; no production source/binary change needed. Independent-factor288 Chromium references and manifest/cascade tests passed before this failure. Native build and WASM build pass, typecheck51029 passes, host44519 passes10/10. Focused native297 run51095 and parity9452 active.

L08 executed milestone: public232/232 passes (59786, `/tmp/independent-flex-module3.log`), including288 Chromium geometry references on original/clone and same-viewport mutation/clear lifecycle. Native and WASM builds pass, parity9/9 (9452), host10/10 and typecheck pass. Focused native297/297 passes (51095, `/tmp/independent-flex-native.log`). All nine composition comparisons and72 unique box pairs directly inspected across12 sheets;216 exact box duplicates account for all288 box cases. Native visual receipt remaining[]. Tolerances unchanged. Vector297 run72737 active (`/tmp/independent-flex-vector.log`); full regression pending, so L08 remains unqualified overall.

L08 vector run72737 exited1:296/297 pixels pass; narrow panels body interior RGB error8.6835 exceeds unchanged6 threshold. Geometry independently audited297/297, max0.028076171875px; all source IDs and hidden states agree. All nine composition pairs directly reviewed (glyph raster differences), and288 box pairs match fully reviewed native sources and both PNG hashes exactly. Vector visual receipt remaining[]; pixel failure retained. Full native3382 regression20764 is running (`/tmp/independent-flex-full.log`). All five input hashes verified and recorded; do not rebuild or modify the case corpus until terminal.

Full20764 INVALIDATED and stopped exit130. cargo rustc --test partial_flex also rebuilt the publisher despite its help describing only the specified test target. Post-build hash guard caught changed target/debug/html-to-riv; probe/replay/WASM/cases unchanged. The contract test runner aborted before execution on that guard; dependent shell command nevertheless compiled oracle target, no further full-run qualification is valid. Original compiler executable unavailable among target/debug/deps hashes. Preserve full log/artifacts with run-invalidated.json. Restart full regression against a copied executable snapshot after coherent L09 builds and focused tests; its superset can qualify L08 and L09 together. No clean-full claim.

Joint L08/L09 full native94020 passes4114/4114 (38.7m). All4104 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed auto-margin, independent-flex and partial-flex baselines; no images remain unreviewed. Frozen toolchain hashes revalidated. Receipt: partial-flex-full/visual-inspection.json and baseline-comparison.json. L08 and L09 are now native-qualified; their documented vector text raster failures remain. The invalidated earlier L08 run is retained and not used as qualification.
