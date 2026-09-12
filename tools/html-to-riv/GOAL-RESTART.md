# Compiler goal: current restart brief

Reviewed 2026-09-12. The saved goal remains active and unlimited. Work only in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`. Confirm git HEAD/status before editing. This brief supersedes historical next-step instructions.

## Latest completed paint isolation (supersedes standalone experiment instructions below)

Read validation/fractional-paint-review.md and fractional-paint-chrome-rule.md. The standalone20-case experiment is finished:160geometry/136pixel passes,24retained failures,320clear checks,60directly inspected pairs and100exact transfers. All80LayoutComponent/Shape native route pairs are identical. An exact local band reproduces the prior wrapped edge discrepancy in both engines. Ordinary CSS background snapping explains the tested edge behavior; no direct paint-snap wire field was found, but arbitrary compositions are not proven impossible.

**Next complete the native gain/clamp positive-predicate probe, then test a live paint-only edge composition if the predicate works.** The probe uses only existing TranslationConstraints: clamp input to[0,1], then three stages multiply by2^64 and clamp to[0,1]. Test minimum subnormal, negative/zero, positive and large values, both axes, dependency updates, original/clone and a live value crossingzero. Inspect current positive-clamp artifacts/process handles before resuming; do not infer a process is running from files. Keep all99items, runtime immutability and failures. Do not repeat the completed standalone or wrapping captures without a changed hypothesis.

## Contract and overall state

Build the standalone HTML/CSS-to-Rive compiler across all99items in BACKLOG.md, following TARGET.md. Runtime, renderer, schema, shared dependencies and root build configuration stay immutable at `6c7ac16617835b5f581784ff08a9e779bb52faf3`, tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`. Coupled PR628 was reverted by629. Source guard passes; effective build identity is checked separately.

Emit ordinary self-contained .riv bytes through native capabilities or file-level compositions. No host CSS/setters, rendering adapters, policy sidecars, browser-baked layouts, implicit raster fallback or recompilation on resize. Preserve original raster bytes. Exclude Grid, editor integration, scripting, interactions, bindings and animation.

Coverage is13qualified/23partial/4investigating/59pending, totaling99. These are scoped evidence states, not implementation percentages. Public Rust/CLI/WASM/JavaScript restoration is done. Public wrapping and text/font rendering remain unadmitted. SUPPORT.md defines accepted contexts; BACKLOG.md retains partial results and failures. Never transfer qualifications from the reverted modified-runtime implementation.

## Latest completed checkpoint

Read validation/wrapped-derived-review.md, wrapped-derived-receipt.json, wrapped-derived-constructor-review.md and wrapped-derived-edge-audit.md. Prior committed paint/mask binding was a8b7216ec4; use git log for the current checkpoint commit.

The real compose_with_bounds candidate now has a fresh48-case native/Chrome campaign, not merely the earlier experimental1/64graph. It uses source-derived epsilon approximately0.073181–0.074615 over [0,16384]², anchored visible alignment, independent native sizing slots, bound scalar/paint records and clipped viewport masks. Rows/columns, both main directions, both wrap modes, three line alignments, unequal/responsive visible sizes and overflow remain in the matrix.

- All48recipes accepted and deterministic. The adapter invokes the frozen actual constructor and rechecks file/map/trace/proof equality. It is validation-only direct-record construction, not public HTML/CSS compilation; exact-constant provenance applies to the authored integer/dyadic recipe values.
-384/384geometry frames pass;336/384pixel frames pass. **All48fractional-edge pixel failures remain failures.** They occur at the third viewport in responsive-visible cases, including original and clone. No thresholds were widened.
-768/768alternate-clear checks pass.240repeat/original-clone image pairs are exact. Same files are resized through one/two/three lines and return to initial dimensions.
-144distinct full Chrome/native pairs directly inspected on36unscaled sheets;240exact repeat transfers. No gross placement, ordering, clip leakage or missing paint was observed; fine edge/color differences remain even in some passing frames.
-Current stream checks cover384frames/2688clips/1152empty-clipped draws plus four rejecting controls. This observes command inputs and source-inferred intersections, not internal GPU state.
-Final full frozen build: output/wrapped-derived-background-build-r2,404Rust/56Node, TypeScript/native/WASM/source guard,300source bindings. CLI/WASM unchanged from the prior paint/mask checkpoint;282image/794historical output regressions transfer through exact executable identity. The final duplicate-background negative test is stronger than the constructor snapshot; its explicit test-only diff is preserved and non-test source equality verified.
-Verifier: validation/wrapped-derived-evidence.py; output/wrapped-derived-verification-r1.json checks8873bindings. Captures: output/playwright/wrapped-derived-r2. Visual gallery: output/playwright/wrapped-derived-r2/visual/gallery.html.

The only product change in this checkpoint permits one optional ordinary Artboard Fill/SolidColor in the private base, preserved without replication. Constructor R1 and capture R1 harness failures are preserved; R2 succeeded. Do not reinterpret those setup failures as runtime failures.

## Immediate next work

**Run the standalone fractional-paint experiment in wrapped-derived-edge-audit.md.** The current campaign and its complete visual review are finished; do not repeat them or completed scalar/paint/mask proofs.

1. Independently author ordinary LayoutComponent Fill and Shape/Rectangle Fill controls at fractional coordinates, with integer controls, quarter/half/three-quarter offsets and a responsive percentage edge. No wrapping helpers, paint replicas or masks beyond the ordinary artboard clip. Do not obtain product geometry from browser measurements.
2. Compare unchanged Chrome/native gates and exact sampled pixels, with bytes-only import and same-file original/clone resizing. The concrete question is whether ordinary fractional paint reproduces the wrapping-edge discrepancy. Preserve all failures and distinguish alpha/interior differences from edge coverage.
3. Investigate a plausible ordinary file composition or record the precise unresolved limitation. No direct serialized pixel-snap control was found in the inspected layout/rectangle/paint fields, but this does not prove arbitrary compositions impossible. Do not silently round authored layouts or reject all fractional geometry to get a passing subset.
4. Public wrapping admission still requires public parser/planner integration, decimal provenance, diagnostics/resource bounds and Rust/CLI/WASM/JavaScript parity. Private construction and finite successful matrices are not public support or complete wrapping semantics.
5. Continue independent backlog work in priority/dependency order when a premise has a concrete unresolved hypothesis. Typography/glyph paint, broader flex, image combinations, borders/painting, positioning, responsive expressions, assets and quality remain open. Avoid repeated unchanged captures or evidence machinery without behavior progress.

Use Chrome153.0.8010.12 and immutable Rust Metal RasterOrdering (CLI clockwise-atomic). Existing tools: output/wrapped-snapped-gate-r1/node-probe and output/immutable-baseline-toolchain-r2/renderer-replay. Keep new browser artifacts under compiler output/playwright. Preserve current source/artifacts and explicit identities when transferring evidence.

## Finish and publish boundary

Use parallel subagents for concrete independent work as authorized by the saved goal; avoid overlapping edits. Update SUPPORT.md, VALIDATION.md, item evidence and the progress webpage after meaningful changes. Commit reviewed compiler-only checkpoints. Keep the unfinished replacement isolated; upstream only the completed, reviewed replacement under the user's existing instruction.

Completion requires every scoped item qualified or an evidenced immutable-runtime limitation/external dependency, with no independent implementation work remaining. An unresolved proof is not impossibility. Keep the goal active until those conditions are actually satisfied.

## Restart prompt

> Resume the existing compiler goal in /Users/levi/.codex/worktrees/html-css-immutable using GOAL-RESTART.md. Wrapped and standalone fractional-paint campaigns are complete with48and24pixel failures retained. Next finish the native gain/clamp positive-predicate probe and, if viable, test a live file-level paint-edge rounding composition. Keep the full99-item scope and immutable runtime; no data bindings, animation, host policy or recompilation on resize. Do not repeat completed captures; keep the unfinished replacement isolated.

Earlier chronological notes are preserved in validation/history/GOAL-RESTART-before-derived-review-2026-09-12.md and checkpoint reviews. Their next steps are historical.
