# Compiler goal: current restart brief

Reviewed 2026-09-12. This brief supersedes historical next-step instructions and the saved objective's stale public-interface restoration step. The goal remains active and unlimited.

## Workspace and immutable contract

Work only in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`. Use explicit command working directories and `login:false`; ambient `7c27` is obsolete. Confirm HEAD/status before edits and preserve newer work. Inspect git log for the latest compiler-only checkpoint.

Build the standalone HTML/CSS-to-Rive compiler across all 99 BACKLOG.md items, following TARGET.md. Runtime, renderer, schema, shared dependencies and root build configuration stay immutable at `6c7ac16617835b5f581784ff08a9e779bb52faf3`, tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`. Coupled PR628 was reverted by PR629. Source guard and effective build identity are separate checks.

Emit ordinary self-contained .riv files using native capabilities or compositions of existing file objects. No runtime changes/dependency overrides, required policy sidecars, host CSS/setters, custom rendering adapters, browser-baked layouts, implicit raster fallback or recompilation on resize. Preserve original raster bytes. Exclude CSS Grid, editor integration, scripting, interactions, bindings and animation.

Distinguish native support, proposed compositions, qualified implementations, evidenced limitations and unresolved questions. Investigate plausible file compositions before declaring impossibility. Reject unsupported semantics clearly; record proposed runtime enhancements separately without implementing them.

## Overall state

13 qualified / 23 partial / 4 investigating / 59 pending = 99. These are evidence states, not a percentage of implementation effort. Public Rust/CLI/WASM/JavaScript restoration is complete. Public wrapping and text/font rendering remain unadmitted. Do not transfer qualifications from the reverted modified-runtime implementation.

## Current completed implementation: owned rounded wrapping paint

Read validation/wrapped-rounded-review.md, wrapped-rounded-receipt.json and wrapped-rounded-domain-review.md. The standalone signed-rounding/paint checkpoint is committed as 8e2ad1850c; the current integration follows it. The original 48-case wrapped campaign has now been rerun against the actual owned Derived candidate with rounded masks.

- New private paint_box emits 2310 shared records per visible geometry: live corners, four saturated scalar inputs, four signed-rounding graphs, dynamic raw thin predicates and four masks. Each replica is an ordinary root Shape/Rectangle plus cloned fill/color and box/line clips. Shared masks preserve physical paint ordering without changing layout geometry.
- paint_box_binding reads actual fields/defaults and all scalar operands independently; wrapping binding validates cached ownership, masks, colors, order and the complete trace. Candidate owns readonly records/trace; no post-proof mutation.
- paint_box_domains checks finite raw corners and local differences, proves the native thin predicate using a correlated subtraction-error bound, and checks saturation/viewport coverage. Sizing and paint spans prevent feedback. The three-owner test requires exactly 7237 total records: 7236 rejects. No public budget was silently increased.
- Frozen constructor output/wrapped-rounded-constructor-r1 accepts all 48 authored recipes twice with exact artifacts. Candidate SHA 69ba5be02217cc432133ce2fa6b8bc07f23bab55f0f0a152fe5dd773b94740d7. Validation bridge emits the actual Derived records unchanged; this is not the public parser path.
- Fresh output/playwright/wrapped-rounded-r1: 384 geometry / 384 pixel passes, 768 alternate-clear passes. All 48 formerly failing wrapped frames now pass unchanged gates. Requests, browser images and DOM boxes match the old campaign; historical failures remain preserved. Passing gates do not imply identical RGBA pixels.
- rounded-observation/receipt.json: 52992 native scalar/mask checks, 384 strict stream checks and 2304 Chrome snapped-edge comparisons pass; five invalid stream controls reject. Observer reads actual clips/draw order and infers intersections, not internal GPU state.
- Visual review: 36 full-size sheets / 144 distinct pairs directly inspected, 240 exact repeat transfers. Root notes cover00–17 and agent notes18–35. No missing paint, wrong overlap or edge bands observed; faint filled-region differences remain. Consolidated receipt binds the saved notes.
- Frozen product build output/wrapped-rounded-product-build-r1 passes 419 Rust / 56 Node tests plus native/WASM/TypeScript/source guard. CLI/WASM remain byte-identical to the prior build; historical282 image/794 transport evidence transfers through verified executable identity. No historical render rerun is claimed.

Source inputs are frozen in the build/constructor directories. Source observer setup failure r1 is preserved; corrected r2 observed the same captures without changing them. Use validation/wrapped-rounded-evidence.py for the consolidated recheck. Do not rerun completed unchanged captures or proofs merely for bookkeeping.

## Latest boundary and cost checkpoint

Read validation/wrapped-boundary-review.md, wrapped-boundary-quantization-review.md and wrapped-lifecycle-review.md. Product source remains exactly the owned rounded-paint implementation at200c9573d2; no compiler/runtime modifications were needed for this new investigation.

- All32 new recipes construct deterministically through the same frozen candidate. output/playwright/wrapped-boundary-r1 captures256geometry passes,228pixel passes and28retained failures,512clear controls. These additional failures do not invalidate the earlier48-case result; they expose untested semantics.
- Failure cases: row/column thin-next-above eachfail frames0–7 (16total); row/column accumulated-decimal eachfail0,1,3,4,5,7 (12total). The first emits an extra native1px line; the second shifts a painted edge1px. Saturation, spanning and the tested alpha overlaps match visible coverage.
- Observer checks20608native scalar/mask values and all256streams with no native semantic failure. Of92raw Chrome edge differences,64are intentional saturation-only differences and28change clippedpaint. No saturation-intersection failure. All96distinct pairs on26sheets were directly reviewed;160exacttransfers. Verifier checks9224bindings,3776clips,992draws, and exact failure-frame correspondence.
- Pinned Chrome source and captures confirm fixed Length→LayoutUnit truncation towardzero to1/64 BEFORE flex accumulation. nextf32(1/16) becomes1/16; separately specified64.249 and.251 become64.234375 and.25, summing64.484375 rather than native64.5. Keep original predictions, source snapshots and failed captures intact.
- Resource corpus output/wrapped-resource-r1:1/3/8/34owners accepted twiceexact;35rejects exactly Derived: ResourceBudget with no artifacts. Total records16N²+2376N−33;34uses99247records/1658530bytes,35needs102727records. This100000-record private limit is not a practical public budget.
- Native CPU lifecycle timings use identical files/probe source with2warmups+9trials percase on AppleM5Max128GiB. Both dev and standardlockedrelease are measured. Release medians(ms), owners1/3/8/34: import1.507/7.809/54.855/1385.640; initialsettle1.528/8.472/58.016/1396.172; clone1.162/7.119/53.417/1370.399; resize-update0.374/1.143/4.164/29.234; CPUdrawrecord0.015/0.067/0.431/7.842. PeakprocessRSS18.78/34.75/81.78/405.22MiB. Timed draw is CPU RecordingFactory work, not GPUrender; dropping handles is not leakproof. These costs require optimization beforepublicsupport.
- output/wrapped-lifecycle-toolchain-release-r1 preserves a standaloneprobe linkfailure (LLVMbitcode/linkerversion). r2 uses the existing release profile's fatLTO/codegen-units1 through rustc; no runtime/profile/root configuration was changed. Finaldev/release88trials704frame samples are bound by333artifactchecks. See lifecycle-review for exact identities and distribution limits.

## Latest implementation: fixed layout normalization

The private fixed-layout normalization is now implemented and tested. `fixed_layout.rs` retains authored provenance, computed bits, raw1/64 units and exact emitted floats. Domains independently bind all six dimensions for parent/slot/visible owners; Candidate retains the owned source descriptors. Source conversion applies before layout and leaves percentage coefficients unchanged.

- Frozen corrected constructor: `output/wrapped-normalized-constructor-r2`. All80recipes accept twice deterministically; all400artifacts exactly match r1 used for capture. The validation bridge now checks the pinned double-to-float decimal conversion order; adversarial double-rounding controls preserve the old failed evidence.
- Boundary capture `output/playwright/wrapped-normalized-boundary-r1`:256geometry/256pixel/512clear passes. All28oldfailures repaired. Observer20608scalarchecks/256streams pass;64rawsaturation-only differences,0clipped differences.96representative pairs directly reviewed on26sheets;160exact repeats.
- Regression `output/playwright/wrapped-normalized-regression-r1`:384geometry/384pixel/768clear passes. All384native images, browser images, scene bytes and geometry exactly match the previous reviewed wrapped-rounded-r1 campaign. Observer52992scalarchecks/384streams/2304Chrome edges pass. Prior direct visual review transfers through exact identity.
- Product build `output/wrapped-normalized-product-build-r1`:431Rust/56Node tests, native/WASM/TypeScript and source guard pass. Executable hashes differ, so the newCLI and rawWASM were each run on all1076prior accepted requests (282image+794transport): allscene bytes match; CLI map bytes and WASM map structures match. No rerender is claimed. See wrapped-normalized-public-regression-review.md.
- Read `validation/wrapped-normalized-review.md`, consolidated evidence, source review and offline proof checker. No public wrapping admission, graph optimization or general percentage/flex arithmetic fidelity is claimed.

## Latest implementation: integral paint optimization

Read `validation/wrapped-integral-review.md` and its evidence. Private `wrapping_integral` selects ordinary ForegroundLayoutDrawable replicas from normalized source/domain constraints, with mixed rounded fallback and independent field binding.442Rust/56Node tests pass; public binaries exactly match the normalized checkpoint. All80recipes construct twice.688native/Chrome geometry+pixel frames and1376clear controls pass across boundary/regression/resource campaigns; direct visual reviews and exact transfers are recorded. Fractional native original/clone checks cover64old/new frame pairs with matching layout/effectivepaint.

Actual1/3/8/34resource counts are44/279/1339/17732;83ownersfit98239records,84reject100557. The prior plan's17731table entry was offbyone. Matched release34-owner import falls1385.640→71.163ms, resize-update29.234→4.506ms. CPUdraw is not GPUperformance. All captures/builds/probes/timing runs completed; do not repeat unchanged runs. No public wrapping admission or runtime changes.

## Latest implementation: original paint under nonoverlap

Read `validation/wrapped-original-review.md`, its evidence, transparent-paint followup and lifecycle review. The new whole-scene certificate owns exact base bytes; every visible owner must be integral and fit its slot. Original colors stay intact, the paint suffix is empty, and all sizing/finite/source proofs remain. Mixed and overlapping cases keep previous paths.

450 Rust / 56 Node tests pass; public binaries are unchanged. All 80 prior recipes retain exact scene/base/map/trace bytes, transferring 640 earlier frames without rerendering. Nine dedicated fixtures pass 72 fresh geometry/pixel and 144 clear checks; 27 representative pairs directly reviewed, 45 repeats transferred. The initial eight transparent-owner stream failures were observer errors: baseline zero-alpha paints are culled. Corrected observation checks every nonzero-alpha draw, retains geometry/nonoverlap checks for transparent owners, and passes all streams. Both observer versions and unchanged images are preserved.

1/3/8/34 owners now use 38/166/486/2150 records. 84 owners succeeds; theoretical budget cases 1562/1563 instead reject `Arithmetic(Separation)`, so no maximum-capacity claim. The initial resource-harness failure is preserved. Fractional old/new observation passes 64 frame pairs. Matched release 34-owner import is 1.460 ms and resize/update 0.485 ms, versus 71.163/4.506 ms in the integral checkpoint. All native captures/builds/probes/timings completed. Public wrapping is still unadmitted.

## Immediate next work

1. Integrate wrapping through the public HTML/CSS parser and planner using owned authored provenance, normalization, source-bound certificates and independent record binding. Start admitting contexts actually proven by the pipeline; diagnose unresolved contexts explicitly. Validate deterministic Rust/CLI/WASM/JavaScript outputs, metadata-free ordinary import, native pixels, same-file original/clone resizing and practical budgets. Private recipe support is not public language support. Keep editor integration excluded.
2. Continue unresolved wrapping semantics: responsive percentage results must quantize after the live basis calculation, not by normalizing coefficients. Later flex arithmetic, normal/stretch distribution, intrinsic sizing and broader positioning still need their own proofs or evidenced limitations. Fixed-input and integral-paint success do not establish arbitrary browser fidelity. The high-count `Arithmetic(Separation)` rejection is a proof limitation to investigate, not a runtime impossibility.
3. Continue all 99 backlog items in priority/dependency order, including typography, broader flex, images, borders/painting, positioning, responsive expressions and assets. Preserve failures and direct visual review. Make concrete language behavior progress; do not repeat unchanged captures, builds or timings merely for bookkeeping.

Historical design/restart instructions are preserved under validation/history and checkpoint reviews. Current boundary/cost campaigns are complete; do not repeat them before implementing a changed behavior.

## Validation and finish boundary

Use pinned Chrome153.0.8010.12 and immutable Rust Metal RasterOrdering (CLI clockwise-atomic). Validate ordinary bytes-only import with diagnostic metadata discarded, deterministic public output, real native geometry/pixels, same-file original/clone resizing, realistic compositions, lifecycle and resource boundaries. Directly inspect results, preserve failures and never widen thresholds merely to pass. New browser artifacts go under compiler output/playwright.

Existing tools: output/wrapped-snapped-gate-r1/node-probe and output/immutable-baseline-toolchain-r2/renderer-replay plus its effective build manifest. Run `python3 tools/html-to-riv/validation/check-target-runtime.py`; source identity alone does not establish effective build identity.

Parallel subagents for concrete independent work are authorized by the saved goal; keep ownership disjoint. Maintain SUPPORT.md, VALIDATION.md, per-item evidence and progress webpage. Commit reviewed compiler-only checkpoints locally. Keep the unfinished replacement isolated; do not push it. Upstream the completed reviewed replacement under the user's existing instruction.

Completion requires every scoped item qualified or documented with an evidenced immutable-runtime limitation or external dependency, with no independent implementation work remaining. An unresolved proof is not impossibility. Keep the goal active until that condition holds.

## Restart prompt

> Resume the existing 99-item compiler goal in /Users/levi/.codex/worktrees/html-css-immutable, branch levi/html-css-immutable-runtime. Read tools/html-to-riv/GOAL-RESTART.md first. Fixed normalization, ordered integral paint and whole-scene nonoverlap/original paint are implemented and validated privately. Next integrate proven wrapping contexts through the public compiler with clear diagnostics and full native/Chrome validation, then continue the entire backlog. Preserve high-count proof rejections and historical failures. Keep the runtime immutable; no editor integration or pushing unfinished work.
