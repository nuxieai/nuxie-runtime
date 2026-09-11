# Immutable runtime audit

**Current decision:** PR #629 is merged and the baseline is restored. Static rasterization, fixed-viewport output, host wrappers and recompilation on resize mentioned below are tradeoff candidates outside the approved responsive-file contract; none is adopted. File-level ordinary-Rive compositions are the only shim strategy currently authorized. See ../TARGET.md.

Audit boundary: `6c7ac16617835b5f581784ff08a9e779bb52faf3` (required immutable pre-PR runtime) through `20248ee6835a7bb071dbf83a364606f5e58aeef9` (completed PR snapshot). This audit reads those Git objects, not the later uncommitted P07 prototype. It covers every changed path under `crates/nuxie-runtime` and `vendor`: 62 paths, including new vendored upstream source, tests, fixtures and licenses. Renderer, render API, render stream, compiler and host changes require the companion integration audit; they are not silently classified as runtime-free here.

**The previous qualification results do not transfer to an unchanged runtime.** Many CSS features were implemented by occurrence policies outside the RIV file, and several shared Taffy/text fixes changed behavior below those policies. Removing the policies while keeping their emitted files is not a validated implementation. Backlog IDs below identify the historical feature work; their old statuses are not claims about the immutable-runtime architecture.

No Cargo, GPU, browser or new behavior tests ran for this audit. Alternatives are source-supported candidates, not tested replacements. This file is the only audit output written.

## Mutation-to-feature map

| Group | Historical IDs | Actual runtime/vendor changes | Consequence on immutable runtime |
| --- | --- | --- | --- |
| R1: layout solve and policy | A09, A11; L01–L18, particularly L03–L15 | `layout_component.rs` adds CSS pixel bounds, alignment/justification overrides, separate grow/shrink, content-box sizing, intrinsic sizing, ratio pair/precision, percentage spacing, positioned modes, baseline measurement and clone retention. `layout_style_applier.rs` adds CSS-only SpaceEvenly alignment. | Existing Rive layout properties remain usable, but CSS override values and changed solve algorithms cannot be requested by the PR's sidecar setters. Per-feature browser equivalence must be re-established. |
| R2: drawable ordering and composition | L02, L17–L20; P01–P06 | `artboard.rs` adds CSS occurrence installers, gradient/border-only drawable proxies, whole-subtree order, positioned and z-index plans, retained group opacity, ancestor clips, checked drawing and clone state. `artboard/css_paint_plan.rs` plans CSS groups/stacking contexts/opacity boundaries. | Object order alone is not evidence of equivalent stacking contexts, nested clip lifetime or isolated group opacity. The old sidecar callbacks do not exist at the required base. |
| R3: backgrounds, borders and corners | A09; P01–P06; L20 | `layout_component.rs` creates extra clip/overflow/border/side/gradient paths and paints; uses padding-box gradient coordinates, rounded border-box fill, border-side grouping and group render opacity. `css_corner_radii.rs` resolves per-axis percentages and overlap; `css_clip_path.rs` constructs elliptical clip/outset/border geometry; `css_linear_gradient.rs` retains stop units, fixes positions, computes exact CSS geometry and adds huge finite-stop fallback. | Existing paths/paints can be candidate output objects, but no retained CSS percentages or redraw-time gradient/border calculation remains without a separate compiler/wrapper solve. Ordinary Rive gradients are not proven to implement the PR's exact premultiplied stop shader. |
| R4: font shaping and lifecycle | A07–A09, A12–A18; T06/T08 are residual qualification dependencies | `font_asset.rs` stores occurrence shaping/spacing/tab/space-break policies through font replacement. `font_hb.rs` adds CSS shaping scales/metrics, cluster letter spacing, optional-ligature behavior, preserved spaces and line-relative tabs; also changes legacy kern suppression when GPOS exists but lacks kern features. `text_engine.rs` adds font hooks, source glyph counts and CSS run construction. | A compiler cannot infer matching CSS glyph positions from font size/letter spacing fields alone. The kern fix is a shared behavior change, not solely an opt-in CSS flag. An immutable runtime restores the older behavior too. |
| R5: text layout, decoration and cloning | A12–A23; L03, L10/L11; T08/T09 boundaries | `text.rs` adds CSS normal/pre-wrap/pre-line breaking, nowrap alignment, ellipsis, intrinsic width and baseline policies, decoration state, source-preserving line handling and explicit clone copying. `css_pre_wrap.rs`, `css_ellipsis.rs`, `css_decoration.rs`, `css_skip_ink.rs` implement those algorithms. `text_style_paint.rs` integrates source-aware decoration painting and snapping. `core_registry.rs` changes Text cloning to `clone_core()` so policy state survives. | Native Text fields alone do not reproduce these new algorithms. Decoration shapes or precomputed lines are candidate compiler output for a fixed viewport; live reflow, font replacement, source offsets, skip-ink and ellipsis still need a stated strategy and tests. |
| V1: shared flex corrections | A11; L05–L11 | Taffy flexbox cross-size limit clamping before basis measurement; reversed-axis overflow alignment; auto-margin remainder/cross-overflow rules; partial-factor gap accounting; content basis/min-max isolation; padding/border floors; unresolved percentage-basis behavior; intrinsic column and shrink contribution corrections; ContentSize/InherentSize cache isolation. | Several changes are unconditional shared-engine corrections. Merely removing CSS-specific flags cannot retain their browser results. A separate compiler-owned layout implementation is a possible architecture, not yet a demonstrated replacement. |
| V2: explicit CSS layout extensions | L03, L13/L13a and dependent L14/L15 compositions | Taffy optional host baseline callback, CSS ratio 26.6 input quantization, gap quantization, intrinsic sizing policy, exact integer aspect-ratio pairs and cache/style propagation. | These additions are unavailable in unchanged engine types/APIs. Scalar Rive aspect ratio and ordinary baseline propagation are not substitutes for the exact pair/measurement behavior. |
| V3: compiler-only Unicode casing vendor | A19; Q01/Q04 source mapping | Adds vendored ICU4X casemap 2.3.0 plus scalar-boundary offset tracing for lowercase/uppercase/titlecase; tables/contextual rules stay upstream. `NUXIE_PATCH.md` explicitly says the compiler is its only user. | This is not runtime mutation semantically, although the entire new vendor directory is in the PR diff. It can potentially move to a compiler-local dependency under the user's boundary; removal from global vendor alone does not prohibit compiler-owned casing. Requires build/dependency verification. |
| R6: dependencies and validation assets | A17/A18/A23, A20/A21, L03; Q04/Q10 | Runtime adds unicode-linebreak and unicode-segmentation; runtime tests add Inter/OpenSans bytes/licenses, text baseline and decoration checks. Vendor adds independent Chrome geometry fixtures for flex corrections. | Algorithms/dependencies must move with any compiler-owned implementation. Tests/assets are evidence inputs, not a production runtime extension; retain or relocate them for immutable-runtime requalification rather than treating them as proof of compatibility. |

### Hunk-level seams that must not be lost in broad grouping

- R1 has new setters `set_css_align_self_occurrence`, `set_css_align_content_occurrence`, `set_css_justify_content_occurrence`, `set_css_flex_factors_occurrence`, `set_css_content_box_occurrence`, `set_css_ratio_content_box_occurrence`, `set_css_ratio_pair_occurrence`, `set_css_percentage_spacing_occurrence`, `set_css_positioned_occurrence`, `set_css_relative_position_occurrence`, plus internal intrinsic/pixel/baseline policies. These span live measurement, placement and clone behavior, not just serialization.
- R2 includes both per-element `order` and separate positioned/stacking plans. `try_draw_handle` preflights opacity depth and then scopes ancestor clips; the newly added `try_draw` contract cannot be assumed available in the base.
- R3 includes `set_css_overflow_axes_occurrence` and `set_css_overflow_clip_margins_occurrence`; generic rectangular clipping is only a candidate subset of L20. P01/P02 also alter border geometry, not merely border color.
- R4's legacy-kern change tests for an actual GPOS `kern` feature before suppressing fallback. Restore-to-base deliberately removes that correction even for fonts without CSS policies.
- R5's generated-registry change affects only Text's cloning dispatch in this diff; it calls the new explicit clone function. A replacement file wrapper must recreate its own external metadata on clone rather than depending on this override.
- V1's cache changes and V2's style-field storage are necessary parts of the old behavior. Keep a separate compiler layout dependency separate from the immutable runtime's Taffy instance and ABI; changing the workspace patch still changes the runtime.

## Existing object/file alternatives: evidence and limits

The following objects/properties exist at the **base commit**. Their existence is evidence of a possible output target, not CSS parity.

| Candidate | Base source seam | Possible compiler/file-wrapper direction | Intentionally unresolved until tested |
| --- | --- | --- | --- |
| Ordinary layout nodes/styles | `generated/layout/layout_component_style_base.rs` has gaps, physical border/margin/padding/inset values; sizing/node styles and `layout/layout_style_applier.rs` map ordinary Rive layout | Emit the old wire properties for a narrowed layout subset; or run a separate compiler-owned CSS solve and emit positioned objects for one viewport | A11 and L03–L15 engine corrections; independent flex factors; indefinite bases; ratios/percent rounding; responsive behavior after `set_size` |
| Rectangle / custom shape path | `generated/shapes/rectangle_base.rs` exposes separate TL/TR/BL/BR circular radii; generated shape/path objects pre-exist | Emit filled border-side paths and elliptical Bézier geometry, rather than asking LayoutComponent to paint CSS | Automatic recomputation of percentage radii, border overlap, fractional AA, clip-margin semantics, repeated resize and shape-to-layout anchoring |
| LinearGradient / GradientStop | `generated/shapes/paint/linear_gradient_base.rs` has start/end and opacity; `shapes/paint/linear_gradient.rs` is the ordinary paint owner | Emit supported ordinary gradients for a measured narrow subset; compiler-generated color geometry or raster assets are alternative output forms if permitted | Premultiplied alpha, exact coincident stops, stop capacity, tiling below transparent borders, exterior/extreme stops and all backend pixels. Do not copy the P06 qualification claim. |
| ClippingShape | `generated/shapes/clipping_shape_base.rs` exposes source_id/fill_rule/is_visible | Build explicit clipping shapes around emitted subtrees | Axis-only CSS clipping, clip-margin reference boxes, rounded clips, negative z-order escape/re-entry and group boundaries |
| Text and TextStyle | `generated/text/text_base.rs` includes align/sizing/overflow/wrap/width/height/origin and baseline controls; ordinary text style carries font/size/paint | Emit existing text for a deliberately narrowed typography profile; for fixed-size output, compiler-owned shaping/line layout could instead emit separate lines or outline paths | CSS measurement/cluster spacing, contextual shaping, tabs, wrapping, ellipsis, skip-ink, fallback/fonts, editable/source-preserving text. Paths sacrifice text semantics unless an explicit wrapper retains them. |
| Precomputed drawable order / hierarchy | Existing artboard drawable list, parent relationships, Shape/Node objects | Reorder emitted objects for a fixed resolved stacking plan; isolate static groups as embedded images if raster fallback is an accepted output | Subtree clips, stacking-context nesting, non-isolated per-child opacity versus isolated group compositing, dynamic updates/animation |
| Compiler-owned scene wrapper | No new runtime API required by definition; wrapper owns input HTML/CSS, source IDs and emitted RIV | Recompile/reimport on viewport or content changes using only old import/draw APIs, while exposing compiler-level source identity | This is a different runtime/publish contract: cost, assets, state/animation preservation, clone independence, old artboard lifetime and atomic replacement all need tests. It does not mean one emitted RIV becomes intrinsically CSS-responsive. |

No evidence in this audit establishes a pure file encoding for all former CSS sidecar policies. A wrapper may orchestrate existing APIs, but must not patch runtime internals, replace its shared vendor implementation, require new renderer opcodes or advertise the removed capability setters.

## Immutable boundary and requalification order

1. Pin runtime, renderer, render API/stream and runtime-facing vendor dependencies to the requested base. Audit source/dependency identity separately from compiler output. Do not reintroduce Taffy modifications via workspace dependency overrides.
2. Classify each accepted syntax as old-runtime-native, compiler-resolved static output, wrapper-recompiled responsive output, or unsupported. Keep the distinction machine-readable; an old requirements version 26 assertion is not an implementation.
3. Start with a bounded file-only scene using existing shapes, layout and text. Compile at a fixed viewport and import/draw on the untouched runtime. Compare actual Chrome geometry and full PNGs before restoring any feature status.
4. Add separate original/clone and resize tests appropriate to the chosen contract. For wrapper recompilation, test actual replacement, preserved source IDs/assets and resource lifetime; for a single RIV, test its genuine old-runtime response.
5. Use historical Chrome cases as independent inputs, but regenerate native artifacts/toolchain hashes and re-review changed images. Historic native receipts bind the modified runtime and cannot certify the reverted one.

## Exhaustive changed-path inventory

Every path below comes from `git diff --name-only BASE TIP -- crates/nuxie-runtime vendor`. Group membership links each mutation to the feature map above. `A` means newly added at TIP; `M` means changed from BASE. New ICU paths include upstream package material as well as the narrowly documented custom observer; their presence does not mean every line is a new CSS algorithm.

| Change | Path | Groups |
| --- | --- | --- |
| M | `crates/nuxie-runtime/Cargo.toml` | R6 (A17/A18/A23) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs` | R2/R3 (L02/L17–L20/P01–P06) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/artboard/css_paint_plan.rs` | R2/R3 (L02/L17–L20/P01–P06) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/assets/font_asset.rs` | R4 (A07–A18) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/css_clip_path.rs` | R3 (L20/P01–P04) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/css_corner_radii.rs` | R3 (L20/P01–P04) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/css_linear_gradient.rs` | R3 (P06/Q05) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/generated/core_registry.rs` | R5 (A12–A23 clone state) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs` | R1/V2 (L04/L05) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | R1/R2/R3 (A09/A11/L01–L20/P01–P06) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/mod.rs` | R3 registration (P01–P06) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/text/css_decoration.rs` | R5 (A12–A23/L03/L10/L11) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/text/css_ellipsis.rs` | R5 (A12–A23/L03/L10/L11) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/text/css_pre_wrap.rs` | R5 (A12–A23/L03/L10/L11) |
| A | `crates/nuxie-runtime/src/mechanical_port/source/text/css_skip_ink.rs` | R5 (A12–A23/L03/L10/L11) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/text/font_hb.rs` | R4/R5 (A07–A23/L03/L10) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/text/mod.rs` | R5 (A12–A23/L03/L10/L11) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/text/text.rs` | R5 (A12–A23/L03/L10/L11) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/text/text_engine.rs` | R4/R5 (A07–A23/L03/L10) |
| M | `crates/nuxie-runtime/src/mechanical_port/source/text/text_style_paint.rs` | R5 (A12–A23/L03/L10/L11) |
| A | `crates/nuxie-runtime/tests/assets/fonts/Inter-LICENSE.txt` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/assets/fonts/Inter-Regular.ttf` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/assets/fonts/OpenSans-LICENSE.txt` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/assets/fonts/OpenSans-Regular.ttf` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/assets/fonts/README.md` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/css_decoration.rs` | R6 (A20/A21/L03) |
| A | `crates/nuxie-runtime/tests/css_text_baseline.rs` | R6 (A20/A21/L03) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/Cargo.lock` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/Cargo.toml` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/Cargo.toml.orig` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/LICENSE` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/NUXIE_PATCH.md` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/README.md` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/benches/casemap.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/benches/data/Iliad.txt` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/casemapper.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/closer.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/greek_to_me/data.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/greek_to_me/mod.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/internals.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/lib.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/provider/data.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/provider/exception_helpers.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/provider/exceptions.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/provider/mod.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/provider/unfold.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/set.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/src/titlecase.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/tests/conversions.rs` | V3 (A19/Q01/Q04) |
| A | `vendor/icu_casemap-2.3.0-nuxie-offsets/tests/gen_greek_to_me.rs` | V3 (A19/Q01/Q04) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/NUXIE_PATCH.md` | V1/V2 (A11/L03/L05–L13a) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | V1/V2 (A11/L03/L05–L13a) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/src/style/flex.rs` | V1/V2 (A11/L03/L05–L13a) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/src/style/mod.rs` | V1/V2 (A11/L03/L05–L13a) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/src/tree/cache.rs` | V1/V2 (A11/L03/L05–L13a) |
| M | `vendor/taffy-0.12.1-rive-yoga-order/src/tree/taffy_tree.rs` | V1/V2 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/README-nuxie-oracles.md` | V1/V2/R6 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/content-auto-isolated-boxes.json` | V1/V2/R6 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/indefinite-basis-matrix.json` | V1/V2/R6 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/indefinite-shrink-edges.json` | V1/V2/R6 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/partial-flex-boxes.json` | V1/V2/R6 (A11/L03/L05–L13a) |
| A | `vendor/taffy-0.12.1-rive-yoga-order/tests/assets/wrap-reverse-boxes.json` | V1/V2/R6 (A11/L03/L05–L13a) |

## Reproducible source references

All paths are repository-relative and should be inspected at the stated commit, because the original worktree has later P07 edits. Examples:

```sh
git diff 6c7ac16617835b5f581784ff08a9e779bb52faf3 20248ee6835a7bb071dbf83a364606f5e58aeef9 -- crates/nuxie-runtime vendor
git show 20248ee6835a7bb071dbf83a364606f5e58aeef9:vendor/taffy-0.12.1-rive-yoga-order/NUXIE_PATCH.md
git show 20248ee6835a7bb071dbf83a364606f5e58aeef9:vendor/icu_casemap-2.3.0-nuxie-offsets/NUXIE_PATCH.md
git show 6c7ac16617835b5f581784ff08a9e779bb52faf3:crates/nuxie-runtime/src/mechanical_port/source/generated/shapes/paint/linear_gradient_base.rs
```

The feature IDs and historical validation pointers were read from `tools/html-to-riv/BACKLOG.md`. Specific useful historical failure evidence includes `validation/percentage-limits-review.md`, `validation/wrap-reverse-investigation.md`, `validation/auto-margin-investigation.md`, `validation/partial-flex-investigation.md`, `validation/align-self-investigation.md`, and `validation/known-gaps.md`. These explain why the old mutations existed; they do not prove any proposed replacement.

P07's later `css_radial_gradient.rs` work is outside this committed range and remains excluded from this audit's inventory. Its implementation was stopped when the architecture changed.
