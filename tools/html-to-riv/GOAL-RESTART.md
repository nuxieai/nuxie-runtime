# Reviewed compiler goal and restart checkpoint

Reviewed and resumed on 2026-09-12. The saved goal remains active; its instruction to restore the public compiler interface is obsolete because that work is complete. The resumed content-owner checkpoint is documented in [the public checkpoint review](validation/public-content-owner-checkpoint-review.md). This brief preserves the restart snapshot below; use that newer checkpoint and the current git state when continuing.

## Goal to resume

Finish the standalone HTML/CSS-to-Rive compiler in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`, following [TARGET.md](TARGET.md) and all 99 items in [BACKLOG.md](BACKLOG.md).

Treat runtime, renderer, schema, shared dependencies and build configuration at baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` as immutable, including effective dependency resolution. Emit self-contained ordinary `.riv` files. Implement semantics through existing capabilities or compositions of existing objects stored in the file. No required host CSS logic, sidecars, post-import setters, custom renderer, raster fallback, browser-baked geometry or recompilation on resize.

For each feature, record its admitted contexts, intentional exclusions, implementation and evidence. Distinguish qualified native support, qualified file compositions, unproven candidates, evidenced limitations and external dependencies. Investigate plausible compositions before declaring a feature impossible. Unsupported computed contexts must produce actionable diagnostics. Propose potential runtime enhancements separately; do not implement them as part of this compiler.

Use pinned Chrome as the browser reference and the immutable native importer/renderer as the target. Validate the public Rust, CLI, WASM and JavaScript interfaces, deterministic output, same-file original/clone resizing, geometry, pixels and direct visual inspection. Include combinations, malformed input, lifecycle and resource boundaries. Preserve failing controls and existing tolerances. Transfer prior evidence only when exact source, emitted file, reset, asset, viewport and relevant tool identities justify it.

First finish the existing public content-owner checkpoint described below. Then continue the backlog in priority order, choosing independent work when an earlier item remains unresolved. Each investigation should test a concrete candidate or distinguish a concrete limitation. Preserve its result and next discriminating experiment; do not repeat unchanged experiments or validation without a new reason. Keep progress moving across the backlog without relabelling unresolved work as impossible.

Maintain SUPPORT.md, VALIDATION.md, per-item evidence and the progress webpage. Use parallel agents for independent bounded work. Keep CSS Grid, editor integration, scripting, interactions, bindings and animation excluded. Keep unfinished replacement work on the isolated branch; upstream the completed, reviewed compiler replacement under the user's existing publication instruction.

The goal is complete only when every scoped item is qualified in its explicitly documented scope or has an evidenced immutable-runtime limitation or external dependency, and no independent implementation work remains. Partial support and unexplored combinations are not complete merely because the implemented subset passes.

## State at the restart review

The original coupled change was reverted by PR #629. The immutable source guard passed again during this review, including current uncommitted changes. No files outside the compiler module differ from the target under that guard; effective build identity is separately recorded by the frozen validation tools.

The backlog has **13 qualified, 14 partial, 3 investigating and 69 pending items**. These are coverage states, not a percentage estimate of remaining effort. Broad typography, image assets, painting, positioning, responsive expressions and compiler quality work remain.

Latest committed checkpoint: `b161337818` (`Validate ordinary content-owner composition against the immutable runtime`). Subsequent public integration, tests and validation reviews are present but **uncommitted**. Preserve them. SUPPORT.md, VALIDATION.md, BACKLOG.md and progress-state.json still describe the preceding private experiment in their content-owner status; they need reconciliation before the next feature.

The public implementation uses an ordinary unpainted inner layout object for eligible content-box dimensions with fixed padding. The outer object retains padding, paint and the authored source identity. The inner object owns the authored content dimensions and children. Numeric bounds cover both actual objects; proof consumers explicitly reject topology they do not yet model. Percentage mixtures and fractional painting remain open. This does not complete L12.

## Completed evidence for the uncommitted integration

| Check | Result | Record |
| --- | --- | --- |
| Public build and transports | 270 Rust tests, 46 Node tests, strict TypeScript and native/WASM builds pass; frozen source unchanged during build | `output/public-content-owner-build-r1/summary.json` |
| Historical output regression | 794 requests compile; 719 exact files/maps, 75 intentional composition changes; authored IDs/paths preserved | [Regression review](validation/public-content-owner-regression-review.md) |
| Changed-record audit | 92 added ordinary layout/style pairs across 75 files; no new schema vocabulary or asset types; paint ownership, colors and order preserved | Same regression review and `output/public-content-owner-regression-r1/semantic-review.json` |
| Native changed-file coverage | All 75 references covered; 600/600 geometry and 594/600 pixel comparisons pass; 1,200 clear checks pass | [Native review](validation/public-content-owner-native-review.md) |
| Known visual failures | Six previously recorded fractional-edge failures remain; some other passing frames have visible edge differences inside the existing gates | Same native review |
| Depth boundary | 128-level control adds eight passing native/Chrome frames | Same native review |
| Element boundary | 8,192 authored elements import, clone and resize with finite native geometry; no Chrome/pixel claim for this control | Same native review |
| Source review | No actionable defect found in the seven production files; descriptor limitations remain explicit | [Source review](validation/public-content-owner-source-review.md) |

Native coverage combines 34 exact-source/file/map evidence transfers and 41 fresh changed-source captures. Visual review is complete for this bounded corpus. Aggregate: `output/public-content-owner-native-r1/receipt.json`. Passing tolerances do not mean exact browser pixels or complete feature support.

Frozen CLI SHA-256: `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5`.

Frozen WASM SHA-256: `c475d543a5f9a4ffcf5f629ef05db5b970b102e2b7682be44419f4b200ed977e`.

Chrome: `153.0.8010.12`. Native renderer: Rust Metal, RasterOrdering. The historical CLI mode token `clockwise-atomic` does not change that recorded effective mode.

## Restart actions and subsequent progress

1. Read the current diff and the three public content-owner reviews. Recheck source/artifact bindings against the frozen build; preserve completed evidence rather than blindly regenerating it. If production changes, rebuild and validate affected evidence.
2. Finish the parent checkpoint review, including selected direct inspection of fresh images, all 75 changed-reference coverage and the six retained failures. Keep the resource-only claims separate from pixel qualification.
3. Reconcile SUPPORT.md, VALIDATION.md, L12 and progress-state.json with the public implementation and its actual boundaries; regenerate the progress page. The 99-item counts do not automatically advance.
4. Run the immutable guard and diff checks, then commit the completed compiler-only checkpoint locally. Existing full tests need repeating only if changes or an unresolved concern justify it.
5. Resume independent backlog implementation. Keep broader content-box mixtures and fractional painting explicitly unresolved while progressing other features. Do not reopen the already completed public-interface restoration.

The integration agents finished their current reviews and stopped during this goal review. No new feature implementation or publication was started.

The resumed run completed the parent source/pixel review and reconciled SUPPORT.md, VALIDATION.md, L12 and the progress page. It rechecks the bound artifacts before the checkpoint commit. A separate unitless-line-height experiment is now testing an ordinary layout/text composition; public text admission has not changed. Read current git state and the latest progress page rather than repeating completed restart actions.
