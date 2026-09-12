# Compiler goal: current restart brief

Updated on 2026-09-12 after the public authored-image checkpoint. The saved goal remains active and unlimited. Its public-interface restoration instruction is already satisfied. Work only in `/Users/levi/.codex/worktrees/html-css-immutable` on `levi/html-css-immutable-runtime`; the older `7c27` worktree is not the implementation checkout.

## Goal to resume

Finish the standalone HTML/CSS-to-Rive compiler in `/Users/levi/.codex/worktrees/html-css-immutable`, branch `levi/html-css-immutable-runtime`, following [TARGET.md](TARGET.md) and all 99 items in [BACKLOG.md](BACKLOG.md).

Treat runtime, renderer, schema, shared dependencies and build configuration at baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` as immutable, including effective dependency resolution. Emit self-contained ordinary `.riv` files. Implement semantics through existing capabilities or compositions of existing objects stored in the file. No required host CSS logic, sidecars, post-import setters, custom renderer, raster fallback, browser-baked geometry or recompilation on resize.

For each feature, record admitted contexts, intentional exclusions, implementation and evidence. Distinguish qualified native support, qualified file compositions, unproven candidates, evidenced limitations and external dependencies. Investigate plausible compositions before declaring a feature impossible. Unsupported computed contexts must produce actionable diagnostics. Propose potential runtime enhancements separately; do not implement them as part of this compiler.

Use pinned Chrome as the browser reference and the immutable native importer/renderer as the target. Validate the public Rust, CLI, WASM and JavaScript interfaces, deterministic output, same-file original/clone resizing, geometry, pixels and direct visual inspection. Include combinations, malformed input, lifecycle and resource boundaries. Preserve failing controls and existing tolerances. Transfer prior evidence only when exact source, emitted file, reset, asset, viewport and relevant tool identities justify it. Evaluate glyph paint locally as well as across the frame; a larger blank viewport or an empty text box must not conceal a text-paint failure.

Continue from the completed checkpoints below. Work through the backlog in priority order while advancing independent work when an earlier item remains unresolved. Each investigation must test a concrete candidate or distinguish a concrete limitation. Preserve its result and next discriminating experiment. If no new testable hypothesis is available, leave that item unresolved and move to independent implementation work. Do not repeat unchanged experiments, broaden validation without a reason, or treat artifact volume as feature completion.

Make usable public compiler support the delivery unit. Before implementing a feature, state its intended CSS contexts, ordinary-file encoding, unsupported contexts and a finite acceptance matrix covering the material boundaries and combinations. A successful private experiment is a prerequisite, not a delivered feature: carry a viable candidate through the public APIs, diagnostics, regression checks and support documentation. Keep quality campaigns tied to concrete risks and changed code. Broaden the matrix when a failure or newly admitted context justifies it; passing one finite matrix does not prove arbitrary browser equivalence.

Maintain SUPPORT.md, VALIDATION.md, per-item evidence and the progress webpage. Use parallel agents for independent bounded work. Keep CSS Grid, editor integration, scripting, interactions, bindings and animation excluded. Keep unfinished replacement work on the isolated branch; upstream the completed, reviewed compiler replacement under the user's existing publication instruction.

The goal is complete only when every scoped item is qualified in its explicitly documented scope or has an evidenced immutable-runtime limitation or external dependency, and no independent implementation work remains. Partial support and unexplored combinations are not complete merely because the implemented subset passes. A failed composition does not establish that every ordinary-file composition is impossible.

## Verified current state

The coupled compiler/runtime PR was reverted by PR #629; the recorded restored-main commit is `9738049372ffd45639de39c2217c1f548963340a`. Runtime, renderer, schema and shared sources still match baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`. The source guard passes with the image changes present. Effective build identity is separately bound by frozen receipts. No replacement push or merge has been performed at this checkpoint; do not infer the live remote head from these local records.

The public Rust/CLI/WASM/JavaScript compiler exists. Its implemented profile includes static boxes, solid fills, selectors/cascade, custom properties, dimensions, bounded layout compositions and now explicit static image assets. Admission is broader than completed visual qualification; [SUPPORT.md](SUPPORT.md) records the actual boundaries.

All 99 rows remain: **13 qualified, 23 partial, four investigating, 59 pending**. The progress page combines partial/investigating into 27 items. These are coverage states, not a percentage of effort completed. L14 and I01–I05/I09 now have public partial-support evidence; broader codec/layout/asset contexts remain open.

## Completed checkpoint to preserve

Read [public-image-checkpoint-review.md](validation/public-image-checkpoint-review.md) and [public-image-receipt.json](validation/public-image-receipt.json). The image implementation, fixture generators, public APIs, diagnostics, grammar, validation drivers, visual review and support documentation are complete for this bounded checkpoint. Check git status/log for its commit; do not treat these files as disposable private groundwork.

- Final frozen build: `output/public-image-build-r2/frozen`. **301 Rust tests, 56 Node tests, strict TypeScript and native/WASM builds pass** with 153 verified source/artifact bindings. CLI SHA-256 `8f49ac9a7272f0770cc3ae67eb29755830219ff5af0716741b2f77735aa218df`; WASM `2e636686877bdebf9e8c70352ae28a98f3ecba4cb00f224386267f0363c899bf`.
- All **794** historical requests reproduce exact Rive files/maps; `output/public-transport-malformed-image-regression-r1/receipt.json`.
- Final public image corpus: `output/public-image-layout-r3`, **27 successful scenes and 13 diagnostic controls**. All 40 requests and 27 files/maps match the measured r2 candidate. All 40 also match through raw ABI/public JS; `output/public-image-parity-r1/receipt.json`.
- Combined native evidence: `output/public-image-layout-r2/combined-receipt.json`, **216 geometry / 198 pixel / 216 presence passes from 216 frames**. Four stretch fixes account for 32 fresh frames; 184 unchanged frames transfer through exact request/RIV/map/reset/native-tool and output bindings. Each original and clone uses 240×240 → 390×320 → 768×560 → 240×240 without recompilation.
- Full visual coverage: 45 inspected pairs on 12 unscaled sheets, four before/after triples and 171 checked RGBA/opaque-white-extension transfers. [Gallery](output/public-image-layout-r2/visual/gallery.html). All 18 pixel failures remain: eight proportional-height column-tail frames, four fractional-linear, four fractional-nearest and two fractional contain-alignment frames. Never widen gates, omit local paint differences or call these failures qualified.
- Alpha/overlap host-paint checks pass all 32 cyan/transparent-clear comparisons; `output/public-image-clear-r1/receipt.json`.
- Asset campaign: **86 cases / 252 interface observations pass**, including exact encoded limits before deduplication, dimensions, malformed data, duplicate raw JSON keys, output retention and recovery. Six raw-JSON cases have no JS object equivalent. `output/public-image-assets-r1/{receipt,verification}.json` checks 2,373 case artifacts, 62 encoded files and 153 frozen bindings. It does not establish peak process memory or worst-case CPU.
- Strict compiler-owned JPEG admission: final independent 108 valid / 1196 malformed controls pass. `output/public-image-jpeg-review-r2/receipt.json`. The missing-entropy Chrome/native disagreement remains preserved separately. No decoder/runtime source was modified.
- Image variable recovery: four new public Rust tests, 71 pinned-Chrome observations, independent literal/unset controls and retained valid-unsupported diagnostics; [review](validation/image-variable-recovery-review.md).

The actual browser target is **Chrome 153.0.8010.12**, not Firefox. Native tools remain `output/immutable-baseline-toolchain-r2/baseline-probe` and `renderer-replay`; effective mode is Rust Metal RasterOrdering, selected by the existing `clockwise-atomic` CLI token.

Keep the first public image capture as a harness failure: reused image URLs contaminated Chrome references. Its corrected `render-reference-r2-receipt.json` namespaces URLs per run/case and preserves authored src attributes. The original corrected driver was recovered against its capture-time hash in `output/public-image-layout-r1/recovered-reference-driver/`. New drivers snapshot their source. Do not reuse the contaminated reference or render unchanged native files just because a compiler parser changed.

## Earlier completed work, not restart tasks

Q06's source/depth/element campaign is committed as `75eed68624`: 40 cases / 120 CLI/raw-ABI/JS observations pass. Q06 stays partial. Named-object/structured-error repair is committed as `bcc4a89a20`; the image checkpoint intentionally supersedes its old asset-rejection wording while preserving all prior valid outputs.

Content-owner integration (`389b94c8a8`) and source-backed line-height arithmetic (`a27c4993f0`) remain preserved. Typography has 344 candidate metric passes plus source-precision controls, but unresolved glyph paint; public fonts/text are unadmitted. Do not repeat the completed line-height geometry investigation without a new discriminating hypothesis. Private image groundwork is complete and has been carried through the public APIs; do not restart its 31-case campaign.

## Next work

1. Read this brief, TARGET.md, SUPPORT.md, the current backlog and git state. Recheck the immutable source guard. Preserve the completed image checkpoint and its exact frozen outputs.
2. Advance an unfinished image context with a finite acceptance matrix. A concrete next step is common JPEG 4:2:2/4:2:0 baseline/progressive input: preserve current diagnostics, test ordinary-file decoding/pixels on even and odd dimensions, then extend compiler-owned validation if the unchanged consumer supports it. Investigate MCU/component/restart behavior explicitly; the current strict entropy screen intentionally assumes 4:4:4. Do not alter vendor decoders or runtime configuration.
3. Expand intrinsic image geometry to additional aspect ratios and meaningful constraint/alignment combinations. Keep the known fractional-paint failures separate from geometry. Public own padding/minmax/auto-margin/baseline/wrapper contexts still diagnose; proposed ordinary-file compositions need concrete experiments before admission. CSS aspect-ratio declarations are a separate backlog item from internal intrinsic-image ratio owners.
4. Continue other backlog work in priority/dependency order when an item has no new testable hypothesis. Fonts/text, broader flex/wrapping, borders/painting, positioning, responsive expressions, orientation/color management/SVG and compiler quality all remain in the 99-item scope. Unresolved is not impossible.
5. Finish each useful public feature checkpoint through API/parity/regression/native/visual evidence and explicit support documentation. Reuse unchanged evidence only with justified identities. Commit reviewed compiler-only work. Keep unfinished replacement work isolated; upstream the finished reviewed replacement under the user's existing instruction.

Restart instruction: **Resume the existing standalone HTML/CSS-to-Rive compiler goal from this checkpoint. Preserve the immutable-runtime contract and all 99 backlog items.**
