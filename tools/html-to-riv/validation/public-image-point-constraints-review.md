# Public point image constraints: partial support

The standalone compiler resolves point image preferred sizes and min/max bounds from authored CSS, intrinsic image dimensions and point padding. It emits the resulting content and outer sizes using ordinary layout owners and embeds the original image asset. No browser measurements enter compilation. For these admitted source-only dimensions, the same result remains valid when the parent or artboard resizes. No runtime, renderer, schema, shared dependency or host behavior is changed.

## Admitted and unresolved contexts

The new plan admits intrinsic dimensions, one automatic axis or two definite axes; point minima and optional point maxima; both box-sizing modes; point padding; and the existing ordinary nonflexible image contexts. An automatic cross axis requires start alignment. Default/stretch alignment remains admitted when the cross size is authored definite, including a ratio-derived automatic main axis. Existing image asset, fit, clip, parent and numeric restrictions still apply.

When both axes are automatic, the plan preserves the intrinsic ratio until opposing bounds require independent clamping. Minimum wins over an inconsistent maximum. When one axis is definite, that axis is clamped before deriving and clamping the automatic axis; its size is not changed by the automatic axis's bounds. Two definite axes clamp independently. Border-box dimensions and bounds subtract padding with a zero floor. The outer owner restores those insets; a padded image retains an ordinary content owner with the source-derived content size. Numeric provenance follows the actual floating-point operations, including subtraction residuals and final outer sums; a finite final clamp does not excuse an overflowing intermediate computation.

Constrained percentage preferred sizes or bounds, nonzero percentage padding, explicit automatic minima, genuine automatic cross-axis stretch, flexible sizing, automatic margins, baseline alignment and alignment wrappers remain diagnosed. The earlier unconstrained percentage-padding profile is preserved. This point implementation does not prove the remaining responsive constraint families impossible. The direct native-property and ratio-owner experiments in `image-constraints-investigation.md` remain historical evidence; their public requests were rejected when captured, before this public implementation.

## Evidence

The frozen build is `output/public-image-point-constraints-build-r1/frozen`: 335 Rust tests, 56 Node tests, strict TypeScript, native/WASM builds and immutable-source guard passed. Native CLI SHA-256 is `3e41fb052dd4315f2ba7e1ede94cc1b9bc4237241c10ceb70ee7290d20f7f4e1`; WASM is `c61206d5d0008ade5f802db8041f440bc37bbbd3ff5fa697c425ab24306dff37`. The verifier checks live and frozen source bindings; it does not rebuild or rerender unchanged outputs.

The primary 49 public requests produce 44 files and five expected diagnostics. Two asymmetric padding-floor controls add two files. All 102 raw ABI/JavaScript comparisons agree with CLI, including diagnostics and emitted bytes/maps. All 126 prior image files/maps and 794 historical outputs remain exact. The verifier rehashes both prior and resulting regression artifacts rather than trusting aggregate pass counts alone.

The 46 ordinary files have 368 actual native/Chrome frames across the same original and cloned scenes at 240×240 → 390×320 → 768×560 → 240×240, without recompilation. Chrome is pinned to 153.0.8010.12; the immutable native probe and renderer identities are bound in the receipts. All 368 geometry checks pass. There are 352 pixel passes and 352 presence passes. These two sets of nonpasses are different:

- Sixteen pixel failures remain on the following sibling in the padded border-box and content-box max-width column cases. They are fractional edge differences. Their image-presence gates pass. No tolerance was widened.
- Sixteen intrinsic `max-width:0` row/column controls have zero-area outer boxes and cannot supply a fully contained visible pixel for the blank-content presence expectation. Their presence result remains `unsupported-control`, not a pass or an observed paint defect. Their geometry and pixel gates pass.

The asymmetric floor controls use `padding:8px 40px; max-width:60px; min-height:80px`, with no preferred dimensions. The expected content is 0×64 and outer box 80×80 in both directions. All 16 new frames pass geometry, pixels and blank-content presence; the image has no paint because its content width is zero, while the nonzero outer padding region can be sampled.

## Visual review and transfer

The completed private point matrix review directly inspected 126 Chrome/native pairs on 43 original-resolution sheets, with 210 exact visual transfers covering its 336 frames. The final primary public files, requests, maps and measured rows match those 336 private frames exactly. The two additional default-alignment public cases have 16 independently captured native frames. Their actual Chrome/native PNG pairs are hash-identical to the corresponding already reviewed start-alignment donor frames. Only visual inspection transfers for those 16: requests and computed `alignSelf` differ, and their own geometry/pixel/presence gates remain authoritative.

The explicit transfer proof is `output/public-image-point-constraints-r2/visual-r1/visual-transfer-review.json`. It claims zero newly inspected primary sheets and records every donor and target frame. A generated primary gallery by itself is not a completed direct inspection. The floor review independently inspected all six representative pairs on three original-resolution sheets and binds ten exact visual transfers, covering its 16 frames. Its completion receipt is `output/public-image-point-constraints-floor-r1/visual-r1/review-receipt.json`.

Run `python3 validation/public-image-point-constraints-evidence.py` to check the frozen/live inputs, public outputs, actual PNG/geometry/stream identities, completed reviews, visual transfers, API comparisons and regression files. The result is `output/public-image-point-constraints-checkpoint-r1/verification.json`. Failure controls are asserted by case and count and preserved verbatim in that result. Historical output equality does not constitute new native visual qualification.

Independent source review found no concrete defect within the admitted point profile, including the public integration and numeric provenance paths. That finite source audit is separate from native evidence and does not qualify untested contexts.

This advances partial image support, not a fully qualified backlog item. Responsive min/max, automatic minima, broader flex and stretch compositions, and the retained fractional pixel differences remain open. The next constraint investigation should select a concrete unresolved family and ordinary-file composition from `image-constraints-plan.md`, preserving this completed point evidence.
