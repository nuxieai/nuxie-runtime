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

## Immediate next work

1. Add a focused boundary/overlap campaign using the current actual constructor: negative/positive saturation endpoints and spanning boxes, clipping at the artboard boundary, zero/thin extents and near-half values, translucent overlaps across line transitions and both reversed axes. Keep the 48-case regression result; new cases must exercise additional premises, not duplicate it. The domain certificate deliberately rejects uncertain thin thresholds instead of admitting an unproved result.
2. Measure practical native import, resize/update, draw, clone and release cost for 1/3/8 owners and the accepted aggregate budget boundary. Report encoded bytes, repeated timings and build/hardware identity. Thousands of helper records per owner require practical evidence before public admission; exact cost arithmetic alone is insufficient. Use ordinary native observations; no runtime feature changes or CSS-aware host execution.
3. Resolve remaining sufficient-certificate gaps with actual file capabilities. Raw world-corner subtraction can lose a thin extent at large origins; switching sourceSpaceLocal alone cannot recover it because native TransformConstraint forms world corners first. Native f32 rounding is not a proof of Chrome LayoutUnit equivalence near half boundaries. Preserve counterexamples and record precise unresolved contexts. Do not narrow the overall goal or declare arbitrary compositions impossible.
4. Integrate wrapping through the public parser/planner with authored decimal provenance, diagnostics, resource limits and Rust/CLI/WASM/JavaScript parity. Public CSS needs actual interface tests, not private recipes. Normal/stretch distribution, intrinsic sizing and broader wrapping semantics remain separate open backlog work.
5. Continue remaining backlog in priority/dependency order: typography, broader flex, image combinations, borders/painting, positioning, responsive expressions, assets and quality. Make concrete behavior progress; avoid endless evidence infrastructure or repeated unchanged experiments.

The old validation/rounding-wrapping-integration-plan.md preserves design details, but its extraction/integration/48-case instructions are now completed. This brief is authoritative for next action.

## Validation and finish boundary

Use pinned Chrome153.0.8010.12 and immutable Rust Metal RasterOrdering (CLI clockwise-atomic). Validate ordinary bytes-only import with diagnostic metadata discarded, deterministic public output, real native geometry/pixels, same-file original/clone resizing, realistic compositions, lifecycle and resource boundaries. Directly inspect results, preserve failures and never widen thresholds merely to pass. New browser artifacts go under compiler output/playwright.

Existing tools: output/wrapped-snapped-gate-r1/node-probe and output/immutable-baseline-toolchain-r2/renderer-replay plus its effective build manifest. Run `python3 tools/html-to-riv/validation/check-target-runtime.py`; source identity alone does not establish effective build identity.

Parallel subagents for concrete independent work are authorized by the saved goal; keep ownership disjoint. Maintain SUPPORT.md, VALIDATION.md, per-item evidence and progress webpage. Commit reviewed compiler-only checkpoints locally. Keep the unfinished replacement isolated; do not push it. Upstream the completed reviewed replacement under the user's existing instruction.

Completion requires every scoped item qualified or documented with an evidenced immutable-runtime limitation or external dependency, with no independent implementation work remaining. An unresolved proof is not impossibility. Keep the goal active until that condition holds.

## Restart prompt

> Resume the existing 99-item compiler goal in /Users/levi/.codex/worktrees/html-css-immutable, branch levi/html-css-immutable-runtime. Read tools/html-to-riv/GOAL-RESTART.md first. Owned rounded wrapping now passes its 48-case/384-frame campaign; do not repeat completed integration. Next validate additional boundary/overlap cases and practical resource/lifecycle cost, resolve remaining certificate gaps, then integrate public wrapping and continue the backlog. Keep the runtime immutable, pinned Chrome/native visual validation and all failures preserved. No editor integration or pushing unfinished work.
