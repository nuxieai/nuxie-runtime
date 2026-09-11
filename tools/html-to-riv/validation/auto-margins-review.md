# Auto-margin candidate: retained initial failures

The initial public compiler candidate failed validation: 48 cases and 384 frames produced 304 geometry passes, 304 pixel passes, and 768 successful clear checks. The authoritative driver session exited 1. All 48 files reproduced byte-identically through the frozen CLI with equal parsed maps. This receipt preserves a failed candidate and does not qualify auto-margin support.

All 86 distinct Chrome/native pairs were directly inspected on 22 sheets (`visual-0.png` through `visual-84.png`, increments of four). The remaining 298 frames have independently verified full decoded-RGBA identity to a reviewed pair, separately for each renderer. No frame is automated-only. Visible differences correspond to the recorded failures; passing pairs showed no additional visible divergence.

## Failure matrix

Twenty cases fail four frames each (80 total). The 28 remaining cases pass all eight frames. Both row and column axes exhibit the same groups:

- Reversed-main direction, main start/end/both auto margins, both fixed and intrinsic variants: all six cases per axis fail when positive main-axis space is available. At the initial viewport the native layout shifts items beyond the parent's trailing edge; Chrome retains the expected arrangement. The original/clone return sequence repeats this failure.
- Cross-start and cross-both auto margins with fixed cross extent, both normal and reversed main direction: four cases per axis fail cross-axis overflow placement. A representative row cross-start scene has native b/leaf y=-10 versus Chrome y=0; cross-both gives -5 versus 0. Column cases show the corresponding X displacement.

These are observable failures of the candidate, not evidence that immutable-runtime composition is impossible. Follow-up implementation or admission decisions must preserve this original evidence and validate a fresh output set.

## Scope and provenance

Fixtures vary four flex directions, six main/cross start/end/both auto-margin choices, and fixed/intrinsic variants. A three-item parent contains translucent colored fills, a white descendant, and an ordinary footer. Four viewports exercise positive free space, overflow, and return resizing on both original and cloned instances. There is no wrapping or grid qualification in this corpus.

Artifacts are under `tools/html-to-riv/output/public-auto-margins-r1/`. `html-to-riv` is the actual frozen public CLI used by the driver and all reproductions. There was no contemporaneous compiler source/build snapshot for this binary; the evidence binds its bytes and observed behavior, not an asserted relationship to later edited source. Driver, fixture, reset and pixel-check snapshots and hashes are recorded in `render/receipt.json`. Chrome is pinned to 153.0.8010.12 and compared with the immutable Rust Metal RasterOrdering renderer.

`collect-evidence.py` preserves full CLI reproduction and visual partitioning. `matrix.json` includes every failing frame, geometry differences and pixel metrics. `reproductions.json` records all 48 exact outputs. `visual-direct.json`, `visual-transfer.json` and `visual-transfer-verified.json` account for all 384 frames exactly once. `freeze-evidence.py` verifies driver/tool bindings and the transfer proof; `evidence-commands.json` records the collection/validation commands and failed exit. `experiment-manifest.json` binds this artifact set. No native rerun, public source edit or runtime edit was performed for this evidence review.
