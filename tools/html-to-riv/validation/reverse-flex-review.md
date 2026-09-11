# L01 reverse flex directions

CSS Flexbox: https://drafts.csswg.org/css-flexbox-1/#flex-direction-property .
row-reverse and column-reverse reverse the main-axis flow, not source or painting
order. Existing Rive flexDirectionValue values1/3 map to ColumnReverse/RowReverse
in layout_style_applier.rs and into Taffy's reverse directions. No new runtime
capability or format extension is required.

The compiler tracks axis and reversal independently. Axis-sensitive sizing,
flex basis and alignment keep using the axis; emission includes the reverse bit.
Explicit inherit copies both; unset/initial reset to row. Text-only display:block
ignores inner flex direction, including the reverse bit. Authored IDs, source-map
order and object IDs are preserved. RTL/writing modes, order, wrap-reverse and
other previously excluded flex features remain outside this increment.

Two public regressions were red on unsupported-value, now pass. The first imports
the actual .riv and resizes it across three width/height pairs, verifying item
positions and source-map equality with forward flow. The second checks var(),
inherit and unset. Full module197/197 passes. Twenty permanent browser fixtures
cover main/cross alignment, percentages/margins/gaps, wrapping, text and overlapping
child paint with/without clipping. Native/WASM and both renderer-profile visual
qualification pending. Do not qualify L01 yet.

Evidence: /tmp/reverse-flex-{red,module,build}.log.

## Current validation and residual

Module197, native/WASM builds, TypeScript and final JS9/9 corpus parity pass.
Both renderer profiles pass58/60 comparisons. Four original start/end cross-axis
fixtures had zero-size children; visual review caught that inadequate coverage.
They were corrected to visible dimensions and rerun12/12 in each profile. Final
source corpus parity was rerun after those fixture changes. Current effective
coverage remains58/60 per profile, with all20 native sheets inspected (four from
reverse-flex-align). Vector54/60 source/image pairs match initial native images;
the six vector text pairs were inspected separately. Corrected vector alignment
12/12 pairs are identical to the newly inspected native alignment images.

The two failures are reverse-row-text240 and reverse-column-text240. The native
renderer splits unbreakable words, e.g. Second -> Secon / d, whereas Chrome lets
them overflow. Forward-direction controls reproduce the same failures. A single
48px-wide block containing Second (Inter16px, line-height24px) fails geometry at
all viewport widths: native height48, browser24. This is independent of reverse
flex. Original/forward/minimal failures remain in their output directories;
control inputs are in known-gaps.json, original reverse scenes stay in cases.json.

A width60 control passes3/3; changing only white-space to pre-line in the minimal
48px control passes3/3 and its sheet was inspected. Ranked hypotheses were default
emergency splitting, missing compiler wrap policy, or wrong width. Source evidence
supports the first two: text.rs break_lines_for_layout only selects css_pre_wrap
when a policy is installed; normal text uses line_breaker.rs, whose overlong-word
branch backs up glyph-by-glyph. Compiler lib.rs installs CSS wrapping for pre-wrap
and pre-line only. The correct next step is an opt-in CSS normal-wrap policy with
public/runtime requirements and regression coverage, preserving raw Rive behavior.
Do not paper over this by changing authored white-space or widening thresholds.

Tight red command: NUXIE_HTML_KNOWN_GAPS=1 NUXIE_NATIVE_GLYPHS=1
NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/reverse-flex-minimal
npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs
--project=native --grep reverse-control-word-minimal
--output tools/html-to-riv/test-results-reverse-flex-minimal
(Join as one shell command.) Result3/3 fail word.height:48 vs24.

Artifacts/logs/inspection receipts: output/playwright/html-to-riv/reverse-flex*,
including -controls, -minimal, -pre-line, -vector, -align and -align-vector.
All processes are terminal. L01 is partial, not fully qualified; normal wrapping
is an internal actionable dependency. Broad forward regression still needed when
that runtime/compiler policy is implemented. No runtime sources changed in L01.

## Final corrected native qualification

Full native regression passes1,582/1,582 (1,572 scene comparisons plus10 controls),
/tmp/normal-wrap-corrected-full.log. All1,572 source/browser/native image pairs
are identical to reviewed baselines: layout-contract-full, function-type-custom
and normal-wrap-corrected. Transfer evidence and completed visual-inspection.json
are in normal-wrap-corrected-full. Both original normal-wrap regressions are
resolved without tolerance changes. All processes for this increment are terminal.

L01 reverse flex directions is qualified for its documented profile:60/60 reverse
comparisons pass in both glyph renderer modes, source identity tests pass,
native/WASM parity passes, and the full native regression is green and reviewed.
The normal-wrap policy has native qualification; its nine focused vector pixel
failures and the broader corrected vector lane's39 pixel failures remain open
renderer limitations. Vector geometry471/471 passes and all471 pairs are reviewed.
Do not interpret L01 qualification as a complete vector-text qualification or as
completion of the compiler backlog. Next independent item L02 remains unsupported;
its source/runtime investigation is in order-investigation.md.
