# L11 native qualification

The standalone compiler accepts finite nonnegative percentage flex bases under indefinite main sizes, including the implicit percentage-main path of auto basis. The existing factor limits and CSS grammar remain unchanged. The accepted semantics and exclusions are defined in [the contract](indefinite-basis-contract.md).

Qualification is against `output/playwright/html-to-riv/indefinite-v12-toolchain`, whose four artifact hashes were reverified. Requirements version12 and `layout-css-indefinite-basis-v1` prevent older hosts from rendering these scenes without the corrected runtime. Authored main dimensions remain distinct from basis, including equal/zero factors. Percentages resolve at runtime; no viewport layout is baked into output.

| Requirement | Evidence |
| --- | --- |
| Public acceptance, cascade and requirements | 233 tests across45 targets; `/tmp/indefinite-v12-full-public-v2.log`; `tests/percentage_basis.rs` |
| Checked host rejects missing/inconsistent capability before stream | 13 tests; `/tmp/indefinite-admission-host-tests.log` |
| Native/WASM RIV, source-map and requirements parity | 1847 scenes in9 tests; `/tmp/indefinite-auto-main-parity.log` |
| Original and cloned instances repeatedly resized | 120 scenes ×2 instances ×4 widths =960 comparisons; `tests/indefinite_basis_runtime.rs`; `/tmp/indefinite-v12-lifecycle.log` |
| Full final native regression | 5551 passes, zero skipped/flaky/unexpected/errors; `indefinite-v12-full/terminal-summary.json` |
| Full native visual accounting | All5541 scene pairs have exact source and complete-image identity to reviewed baselines; `indefinite-v12-full/visual-inspection.json` |
| Additional implicit percentage-main path | 36 native and36 vector comparisons pass; all reviewed in their visual receipts |
| Vector diagnostic profile | Original75, factors360, expanded216, threshold108 and implicit-main36 pass and are visually accounted for. Text stress288 geometry passes,200 pixel passes and88 pixel failures; all288 reviewed |

Paths naming result directories above are relative to `output/playwright/html-to-riv`. `indefinite-v12-full/qualification-audit.json` records the final audit. Native and vector receipts retain their actual toolchain manifests; focused pre-admission visual baselines are transferred only after exact source/image comparison to the final native run.

## Residual limitations

The vector text renderer is not qualified by this item. Its88 stress-image pixel failures remain unwaived, with visible glyph edge/weight differences. Matching layout does not constitute matching pixels. No tolerance was widened. This native qualification does not close other compiler backlog items or expand the documented exclusions.

The original failure corpora and intermediate failing receipts remain available in `indefinite-basis-investigation.md`. Content-box sizing, Grid, percentage padding/margins, negative margins, positioning, new overflow behavior, editor integration, scripting, interactions, bindings and animation are outside L11.
