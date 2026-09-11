# Experimental mixed wrapping: axes, unequal lines, and independent item alignment

The ordinary-file line-group composition passes this bounded matrix on the immutable runtime: 16 scenes, 128/128 geometry checks, 128/128 pixel checks, and 256/256 clear-color checks. All 40 distinct Chrome/native image pairs were directly inspected on ten sheets; the other 88 frames have verified, whole-image RGBA identity to a directly reviewed pair, separately for each renderer. No visible divergence was found. This is experimental evidence, not public compiler admission. The reused driver's `passed-public-baseline` label does not describe this adapter's scope.

## Tested scope

Four directions (row, row-reverse, column, column-reverse), both wrapping modes, and two item alignment profiles produce 16 scenes. Three items have main sizes 60/60/40 and unequal positive cross sizes 30/50/70. Each item contains a 90×90 overflowing leaf with translucent coral, gold, or teal paint. A navy parent has main extent 50%, cross extent 140, and explicit `align-content:flex-start`. Square viewport sizes 200→320→120→200 exercise two→one→three→two lines on both the original and cloned artboard. Each scene is compiled once before those resizes.

The first alignment profile uses `align-self:flex-start` on all three items. The second uses start/center/end respectively, implemented through perpendicular, transparent item wrappers whose cross extent follows the line. Native inner alignment is assigned independently; authored item and descendant geometry remains in the compared map. Parent background, translucent overlapping descendants, wrap reversal, main reversal, and return resize are covered. Arbitrary descendant trees and painted item backgrounds are not established by this fixture.

## Common line anchors

Comparing actual item top/left positions is invalid for unequal sizes under reversed wrapping, or for independently aligned items. The adapter instead appends one Node plus TransformConstraint per item. In the direct profile it captures the item's common line edge. In the independent profile it captures the stretched wrapper's common line edge. The chosen edge is top/left for ordinary wrapping and bottom/right for reversed wrapping; columns compare X, rows compare Y.

The earlier ordinary DistanceConstraint/clipping primitive then derives line-leader and same-line membership masks from these anchors. Three potential line groups each contain three foreground paint copies. Group order reverses with wrap direction; member order reverses with main direction. Ordinary DrawRules order those foreground copies. Each copy preserves its original SolidColor record, including alpha. Native layout determines membership as the same imported file resizes; no scripts, host branches, bindings, raster assets, or recompilation select a layout.

The augmenter asserts and reports 138 appended records per scene: the prior 132-record composition plus six anchor records. Parser counts reproduced 27→165 for direct scenes and 33→171 for wrapper scenes. The three wrappers add another six records before augmentation. This construction has quadratic growth; these three-item results do not establish acceptable general resource bounds.

## Evidence and reproduction

All files are under `tools/html-to-riv/output/mixed-wrap-axes-r1/`. `prepare.py` preserves fixture generation and actual Rust build invocation; `src/main.rs` and `src/wire.rs` are the emitted experiment sources. `build-command.json` and `build-source-bindings.json` bind the augmenter and linked Rust libraries. The standalone compiler binary is frozen from `public-spacing-checkpoint-r1`; the original immutable probe and renderer are used. No public compiler or runtime source changed for this experiment.

`render-command.json` records the driver invocation. Chrome 153.0.8010.12 was compared with the immutable Rust Metal renderer in RasterOrdering mode. `render/receipt.json` preserves all frame results and tool/source hashes. `freeze.py` independently checks those bindings, reruns the complete adapter for every scene, verifies identical RIV bytes and equal parsed maps, and validates the visual coverage partition. `reproductions.json` records all 16 commands, output hashes and record counts. `visual-direct.json` identifies the 40 reviewed pairs; `visual-transfer.json` proves the remaining 88. All sheets `visual-0.png` through `visual-36.png` in increments of four were inspected.

`experiment-manifest.json` binds the full artifact set and frozen tools. The adjacent tracked receipt binds that manifest and key evidence. Reproduction from generated source uses the recorded build command from this worktree, then the recorded adapter commands; regeneration additionally uses the prior alpha experiment sources identified in the manifest.

## Remaining limits

These results extend the previous opaque and alpha row experiments to both axes, unequal positive line extents, and independent positional item wrappers. They target the pinned Chrome fragment painting behavior already documented in the foreground direction audit, not a claim about every browser or all normative CSS painting cases.

The gate primitive's preserved failures for zero and 0.0005-pixel line extents still apply: its DistanceConstraint early return below 0.001 and coincident anchors need explicit guards. The 65536 normalization distance and 32768 mask extent require scene-coordinate bounds. No arbitrary transforms, negative coordinates, borders, clipping ancestry, group opacity, deeply nested wrapping, ordering permutations, large item counts, gaps, percentage cross sizing, or content distribution beyond explicit flex-start are qualified here. Translucent results demonstrate correct visible contributions for these frames, not a universal mathematical exclusivity proof. Public admission must define guards and broader composition/resource tests.
