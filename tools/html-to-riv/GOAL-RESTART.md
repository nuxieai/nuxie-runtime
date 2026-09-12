# Compiler goal: current restart brief

Reviewed 2026-09-12. This brief supersedes historical next-step instructions, including the saved goal's instruction to restore the public interface: that restoration is complete. The saved goal remains active and unlimited; the user controls restarting execution.

## Workspace and contract

Work only in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`. Use explicit command working directories and `login:false`; ambient `7c27` is obsolete. Confirm HEAD and status before editing. The signed-rounding module and paint validation checkpoint are complete; inspect git log for the local checkpoint commit. Preserve this work and any newer edits.

Build the standalone HTML/CSS-to-Rive compiler across all 99 BACKLOG.md items, following TARGET.md. Runtime, renderer, schema, shared dependencies and root build configuration stay immutable at `6c7ac16617835b5f581784ff08a9e779bb52faf3`, tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`. Coupled PR628 was reverted by PR629. The source guard passed at this review; effective build identity is a separate check.

Emit ordinary self-contained .riv files using native capabilities or compositions of existing file objects. No runtime modifications or dependency overrides, required policy sidecars, host CSS/setters, custom rendering adapters, browser-baked layouts, implicit raster fallback or recompilation on resize. Preserve original raster asset bytes. Exclude CSS Grid, editor integration, scripting, interactions, bindings and animation.

For each feature distinguish native support, proposed compositions, qualified implementations, evidenced limitations and unresolved questions. Investigate plausible compositions before declaring impossibility. Reject unsupported semantics clearly; record proposed runtime enhancements separately without implementing them.

## Overall progress

13 qualified / 23 partial / 4 investigating / 59 pending = 99 items. These are scoped evidence states, not a percentage of implementation effort. Public Rust/CLI/WASM/JavaScript restoration is complete. Public wrapping and text/font rendering remain unadmitted. Do not transfer qualifications from the reverted modified-runtime implementation.

## Latest completed checkpoint: signed rounding and standalone paint

The standalone fractional-paint isolation and positive-predicate probes are complete. Do not repeat them. Their evidence is in validation/fractional-paint-review.md, fractional-paint-chrome-rule.md and positive-clamp-review.md.

The new private `src/paint_rounding.rs` implements bounded signed rounding through ordinary Node/TranslationConstraint records, with 564 records per scalar and ties toward positive infinity. Its selected live coordinate must remain finite in [-16384,16384], in identity artboard space with no feedback dependency. These are caller obligations, not a general public domain certificate.

- Scalar evidence: output/rounding-scalar-r1 and r2; 106 cases, 1696 original/clone frames, 410432 exact numerical f32 checks. Signed zero is compared numerically, not by sign bit. All 3922 reproduction artifacts match byte exactly. The original JSON-decimal observer error is preserved and explained in validation/rounding-scalar-review.md.
- Private live paint prototype: output/playwright/rounding-paint-r1/receipt.json. All 28 cases / 224 geometry and pixel frames pass unchanged gates; 448 alternate-clear checks pass. Same original and cloned files resize without recompilation. Cases cover both axes, fractional and responsive edges, thin boxes, negative positions and accumulated offsets. Passing gates do not mean exact RGBA identity.
- Paint construction uses four live rounded edges and ordinary clipping masks around a solid rectangle. Layout geometry remains unsnapped. Each one-owner recipe emits 2321 records, so practical resource cost is a material unresolved issue.
- Stream observations: path-receipt.json checks all 224 frames and 1120 clips against independently calculated expected boundaries. Browser measurements are validation-only, never constructor input.
- Direct visual notes are saved separately in visual/inspection-root.json and inspection-agent.json: together 24 sheets / 84 distinct pairs. Coverage also records 140 exact repeat transfers. The consolidated visual/review-receipt.json now records completion; coverage.json retains its earlier pending label as historical input.
- before-after-receipt.json preserves 160 historical comparisons, including 24 former failures whose corresponding new paint cases now pass. Do not overwrite or relabel historical failures.
- Frozen full product build: output/rounding-product-build-r1/summary.json, 406 Rust / 56 Node tests plus native/WASM/TypeScript checks. Public CLI/WASM are unchanged from the preceding build; prior-public-identity.json records evidence transfer through executable identity, not a fresh historical render campaign.

Read validation/rounding-paint-composition-review.md for the closed recipe assumptions. Arbitrary JSON domains, large-coordinate cancellation near the thin threshold, native f32 versus Chrome LayoutUnit quantization, multiple overlapping owners, aggregate resource limits and public admission remain unresolved. This experiment is not public CSS support.

## Immediate next work

1. The checkpoint is consolidated in validation/rounding-paint-review.md and rounding-paint-receipt.json. Its verifier checks 17092 artifact bindings and explicitly records 32 numerical-zero sign differences. Documentation/progress are updated. Do not repeat unchanged builds/captures simply for bookkeeping. Read validation/rounding-wrapping-integration-plan.md and implement the owned integration next.
2. Integrate the successful paint composition through compiler-owned construction, domain checks, independent emitted-field validation and aggregate object budgets. Update construction, paint bindings, mask bounds and costs together. Do not mutate readonly Derived records after their proof was built.
3. Apply it to the actual wrapped Derived candidate and rerun the relevant 48-case campaign. Its existing output/playwright/wrapped-derived-r2 evidence is 384 geometry passes / 336 pixel passes / 48 retained failures. The new standalone paint result has not yet fixed or requalified that candidate. Check overlap, alpha and paint order as well as edges and resizing.
4. Public wrapping admission additionally needs parser/planner integration, authored decimal provenance, diagnostics, resource boundaries and Rust/CLI/WASM/JavaScript parity. Private recipes are not a substitute for exercising the public compiler.
5. Continue the remaining backlog in priority/dependency order: typography, broader flex, image combinations, borders/painting, positioning, responsive expressions, assets and quality. Make concrete behavior progress; avoid endless unchanged captures or evidence machinery. A failed proposal or unresolved proof is not proof of impossibility.

## Validation and completion

Use pinned Chrome 153.0.8010.12 and immutable Rust Metal RasterOrdering (CLI clockwise-atomic). Compare public compiler output via ordinary bytes-only import with diagnostic metadata discarded. Check geometry and real native pixels, deterministic output, original/clone same-file resizing, realistic compositions, lifecycle and resource boundaries. Directly inspect visual results, preserve failures and never widen thresholds merely to pass. New browser artifacts belong under compiler output/playwright.

Useful existing tools: output/wrapped-snapped-gate-r1/node-probe, output/immutable-baseline-toolchain-r2/renderer-replay and the latter directory's build manifest. Run `python3 tools/html-to-riv/validation/check-target-runtime.py`; source identity alone does not establish effective build identity.

Parallel subagents for concrete independent work are authorized by the saved goal. Keep ownership disjoint. Maintain SUPPORT.md, VALIDATION.md, per-item evidence and the progress webpage. Commit reviewed compiler-only checkpoints locally. Keep the unfinished replacement isolated; do not push it. Upstream the completed, reviewed replacement under the user's existing instruction.

Completion requires every scoped item qualified or documented with an evidenced immutable-runtime limitation or external dependency, with no independent implementation work remaining. Keep the goal active until that condition holds.

## Restart prompt

> Resume the existing 99-item HTML/CSS-to-Rive compiler goal in /Users/levi/.codex/worktrees/html-css-immutable, branch levi/html-css-immutable-runtime. Read tools/html-to-riv/GOAL-RESTART.md first; it supersedes stale resume instructions in the saved objective. The signed-rounding/paint checkpoint is complete. Follow validation/rounding-wrapping-integration-plan.md to integrate its successful file-only paint composition into the actual wrapped candidate with domain, field and resource validation. Keep the runtime immutable, use pinned Chrome/native visual validation, preserve failures, and continue the backlog. Do not restart completed probes, add editor integration or push the unfinished replacement.

Earlier restart instructions are preserved under validation/history and are historical, not current work orders.
