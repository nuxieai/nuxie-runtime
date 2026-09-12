# Compiler goal: reviewed restart brief

Reviewed on 2026-09-12. This replaces the older restart snapshot. The saved goal is still active and has no token budget. Its instruction to restore the public compiler interface is obsolete: that work is complete. Resume from the checkpoints below.

## Goal to resume

Finish the standalone HTML/CSS-to-Rive compiler in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`, following [TARGET.md](TARGET.md) and all 99 items in [BACKLOG.md](BACKLOG.md).

Treat runtime, renderer, schema, shared dependencies and build configuration at baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` as immutable, including effective dependency resolution. Emit self-contained ordinary `.riv` files. Implement semantics through existing capabilities or compositions of existing objects stored in the file. No required host CSS logic, sidecars, post-import setters, custom renderer, raster fallback, browser-baked geometry or recompilation on resize.

For each feature, record admitted contexts, intentional exclusions, implementation and evidence. Distinguish qualified native support, qualified file compositions, unproven candidates, evidenced limitations and external dependencies. Investigate plausible compositions before declaring a feature impossible. Unsupported computed contexts must produce actionable diagnostics. Propose potential runtime enhancements separately; do not implement them as part of this compiler.

Use pinned Chrome as the browser reference and the immutable native importer/renderer as the target. Validate the public Rust, CLI, WASM and JavaScript interfaces, deterministic output, same-file original/clone resizing, geometry, pixels and direct visual inspection. Include combinations, malformed input, lifecycle and resource boundaries. Preserve failing controls and existing tolerances. Transfer prior evidence only when exact source, emitted file, reset, asset, viewport and relevant tool identities justify it. Evaluate glyph paint locally as well as across the frame; a larger blank viewport or an empty text box must not conceal a text-paint failure.

Continue from the completed checkpoints below. Work through the backlog in priority order while advancing independent work when an earlier item remains unresolved. Each investigation must test a concrete candidate or distinguish a concrete limitation. Preserve its result and next discriminating experiment. If no new testable hypothesis is available, leave that item unresolved and move to independent implementation work. Do not repeat unchanged experiments, broaden validation without a reason, or treat artifact volume as feature completion.

Maintain SUPPORT.md, VALIDATION.md, per-item evidence and the progress webpage. Use parallel agents for independent bounded work. Keep CSS Grid, editor integration, scripting, interactions, bindings and animation excluded. Keep unfinished replacement work on the isolated branch; upstream the completed, reviewed compiler replacement under the user's existing publication instruction.

The goal is complete only when every scoped item is qualified in its explicitly documented scope or has an evidenced immutable-runtime limitation or external dependency, and no independent implementation work remains. Partial support and unexplored combinations are not complete merely because the implemented subset passes. A failed composition does not establish that every ordinary-file composition is impossible.

## Verified current state

The coupled implementation was reverted by PR #629. The immutable source guard passed again during this review, including current uncommitted additions. Runtime, renderer and shared dependency sources remain unchanged. This is a source check; effective build identity is established separately by the frozen validation receipts. This review did not fetch or publish remote changes.

All **99** backlog rows were counted directly: **13 qualified, 15 partial, four investigating and 67 pending**. These are coverage states, not an estimate that the project is 13% finished. The progress webpage combines partial and investigating into 19 incomplete items with evidence.

The public Rust/CLI/WASM/JavaScript compiler exists. Its current profile covers static boxes, solid fills, selectors/cascade, custom properties, dimensions and bounded layout compositions. See [SUPPORT.md](SUPPORT.md) for the actual admission rules. Public text/font assets, images, borders, wrapping, broader flex factors, positioning and richer painting remain unadmitted or incomplete. The compiler is not a fully featured or qualified release yet.

| Latest committed checkpoint | Evidence and remaining boundary |
| --- | --- |
| `389b94c8a8` — ordinary content owners | 270 Rust tests, 46 Node tests, strict TypeScript and native/WASM builds pass. Of 794 historical requests, 719 files/maps remain exact and 75 have reviewed ordinary composition changes. Changed references pass 600 geometry and 594 pixel comparisons; six known fractional-paint failures remain. L12 is partial. [Review](validation/public-content-owner-checkpoint-review.md). |
| `fcedc9e8ff` — line-height investigation | Eight private cases, 64 original/clone frames: 16 metric passes, eight pixel passes, four passing both. Responsive/inherited-factor wrappers provide useful geometry evidence; baseline and paint failures are preserved. A08 is investigating; no public text admission. [Review](validation/unitless-line-height-review.md). |
| `ba538dcd1d` — malformed-source and recovery campaign | 1,000 cases, 929 distinct requests: 174 accepted, 826 structured rejections. CLI/WASM parity, determinism, ownership, recovery and deliberate worker replacement pass. This is finite source mutation, not complete fuzzing or transport/resource qualification. Q05 is partial. [Review](validation/public-malformed-mutations-review.md). |

The latest implementation checkpoint at review time is `ba538dcd1d`. Existing artifacts and failed controls must be preserved. Documentation commits may follow it without changing implementation.

Frozen public CLI SHA-256: `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5`.

Frozen public WASM SHA-256: `c475d543a5f9a4ffcf5f629ef05db5b970b102e2b7682be44419f4b200ed977e`.

Browser reference: Chrome `153.0.8010.12`. Native renderer: Rust Metal, RasterOrdering. The historical CLI mode token `clockwise-atomic` does not change that recorded effective mode. Firefox is not a qualification target.

## Preserved work awaiting integration review

These are new local files and ignored output artifacts, not an expansion of public support. Inspect current `git status` before editing or committing them.

1. **A08 source arithmetic:** [line-height-quantization-source-review.md](validation/line-height-quantization-source-review.md) pins Chromium source to `971a7443b0c9b0a9b2860529b33331b76077ec62`. It distinguishes integer font metrics, leading rounding and used-height quantization. Seventeen source snapshots and 33 artifact bindings are retained under `output/line-height-quantization-source-r1`. General platform font metrics and paint equivalence remain unqualified.
2. **Independent Chrome controls:** `validation/line-height-quantization-browser.mjs` and `line-height-quantization-cases.json` record 32 cases under `output/line-height-quantization-browser-r1`. Fonts are explicitly loaded; all 32 zero-size baseline-marker controls preserve box geometry and complete pixels. The simpler integer-leading prediction matches 28 cases exactly; four fractional-height cases require the source-backed used-height conversion. No native candidate implementing that conversion has been validated yet.
3. **Reduced and zero line height:** [reduced-line-height-review.md](validation/reduced-line-height-review.md), `examples/reduced-line-height.rs` and `validation/reduced-line-height-*` preserve seven cases and 56 original/clone frames. Five signed-padding candidates pass 40/40 metric checks; direct controls fail 16/16. Zero-height checks establish owner height, not an independently measured browser baseline. Original pixel gates pass 22/56 frames, but supplemental measured-ink-region gates pass only 10/56: all 24 reduced-height candidate frames fail that local check. Twenty-one distinct full pairs were inspected, with 35 exact transfers. Artifacts and bindings are under `output/reduced-line-height-r1` and `validation/reduced-line-height-receipt.json`. Verify this evidence before promotion; it does not qualify public text.

## First work after restart

1. Read this brief, TARGET.md, SUPPORT.md, the current diff and the new A08 reviews. Recheck the immutable source guard. Do not restore the already completed public interface or repeat the completed content-owner checkpoint.
2. Finish reviewing and checkpointing the uncommitted A08 evidence. Test a new ordinary-file candidate using Chromium's source-backed arithmetic, preserving the old failed modes. Keep source number precision, float operation order, 1/64px used-height conversion and signed leading division distinct. Font-table metrics are a bounded fixture assumption, not a general font contract.
3. Compare that candidate against pinned Chrome through original/clone resizing, explicit font readiness, geometry, nonempty local glyph regions and complete visual review. Separate repaired metrics from unresolved paint. Integrate public typography only when its asset, layout and paint admission requirements are met; otherwise retain actionable rejection and the next concrete hypothesis.
4. In parallel, advance independent compiler quality work: broader Q05 malformed-input/transport cases and Q06 resource boundaries are concrete starting points. Broader asset handling, painting, positioning, responsive expressions and the other backlog items remain in scope. One unresolved paint issue must not monopolize all progress.
5. Reconcile feature evidence and progress docs, run checks justified by actual changes, and commit completed compiler-only checkpoints. Continue until the goal's completion condition is actually met; publish the finished reviewed replacement under the existing authorization.

The review itself introduces no feature implementation and no publication. Resume the existing goal using this file as the current checkpoint, preserving the unfinished work listed above.
