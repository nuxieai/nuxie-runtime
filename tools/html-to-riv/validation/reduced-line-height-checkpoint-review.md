# Independent review of the reduced-line-height checkpoint

No blocking implementation or measurement flaw was found in the bounded
conclusion: these seven ordinary files demonstrate that signed synthetic
padding can reduce the native text owner's height, including to zero, on
the frozen runtime. They do not establish public typography support or
uniform paint equivalence. The retained metric and pixel failures agree
with the artifacts.

## Verification performed

This review read the example, native driver, text observer, pixel helpers,
ink supplement, visual coverage script, source fixtures, and completed
receipts. It performed no build, compiler invocation, browser capture,
native render, runtime mutation, or change to existing evidence.

- **498 recorded SHA-256 comparisons across 362 distinct files matched**,
  covering source/tool bindings and all 56 frame Rive, geometry, stream,
  text-observation, Chrome, native, and literal-control identities. The
  actual probe input Rive equals each generated scene.
- All seven selected runtime/layout/lockfile sources independently match
  `git show 6c7ac16617835b5f581784ff08a9e779bb52faf3:<path>`.
- The text observer binary is identical to the previously qualified
  `unitless-line-height-r1/text-probe`; its source matches that earlier
  receipt. The immutable layout probe and renderer retain hashes
  `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`
  and `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`.
- All seven saved requests and browser-source records match the authored
  fixtures. Each Rive contains the complete unchanged Roboto font exactly
  once. The generator uses source coefficients and font metrics only.
- Recomputing both comparison profiles and ink bounds from existing PNGs
  reproduces **22/56 original passes and 10/56 supplemental passes**.
  All 56 Chrome images equal their literal controls in complete RGBA.
- All **35 same-source repeat/clone transfers** reproduce complete RGBA
  and exact native/browser metric identity. All **42 image placements**
  in the six review sheets equal their complete, unscaled source images.
  The completed 21-pair direct review belongs to the original receipt;
  this independent review additionally inspected `frame-0-group-1.png`.

## Baseline and box claims

The measured totals decompose as follows; a zero-height box pass must not be
presented as a measured baseline pass:

| Check | Passed / compared | Skipped |
| --- | --- | --- |
| Owner rectangle | 40 / 56 | 0 |
| Native versus browser baselines | 24 / 32 | 24 zero-height frames |
| Native versus browser line count | 32 / 32 | 24 zero-height frames |

Within the five signed candidates, all 40 owner rectangles pass; the 24
nonzero frames also pass baseline and line-count comparisons. The remaining
16 signed zero-height frames have box and paint evidence, with baseline
and line-count comparisons explicitly skipped. The native overlapping
baseline at world Y=31 is an observation, not an independently measured
zero-height browser baseline.

The saved text-observer viewport/instance/step agrees with every native
probe frame. Its text observations match the corresponding receipt rows.
All world/internal linear transforms are identity in this corpus, so the
driver's sum of world Y, internal Y, and ordered-line Y correctly computes
the native baseline here. That addition must not be generalized to rotated
or scaled transforms. Nonzero browser baselines use Range top plus Canvas
font-bound ascent in this homogeneous font profile; the result is not a
general mixed-font or vertical-text oracle.

## Zero-height paint checks

All 24 zero-height DOM regions are empty, and the original helper reports
zero local RGB error for them. The review explicitly acknowledges that
limitation. The supplement reconstructs a nonempty union of native and
reference ink bounds in every frame; the smallest region contains 4,352
pixels. Its region is observer-only and never influences emitted files.

The exact supplemental scope is **local RGB and one-pixel-inset RGB means
over the union**, together with the unchanged whole-frame mismatch ratio,
whole-frame mean-channel error, and ink-presence check. The mismatch ratio
is not normalized to the ink region. Pixelmatch still excludes antialiasing
for this existing text profile; both RGB means include all their sampled
pixels. A supplemental pass therefore is not exact paint identity or a
per-glyph shape proof.

This distinction matters in an actual retained control. For
`signed-zero-32-responsive`, frame 0, the original profile passes with zero
Pixelmatch mismatches and whole-frame mean error about 0.576. The union
region's mean RGB error is about 6.772 and its inset mean is about 7.001;
the inset exceeds the unchanged limit of 6, so the supplement fails. The
stored result captures a difference that the original empty local region
could not detect. The new review does not promote that original pass.

The current evidence needs no rerender to support its stated private
capability conclusion. Future summaries should preserve the separate box,
baseline, and paint denominators, and the precise supplemental metric scope.

## Reviewed identities

Paths below are relative to `tools/html-to-riv`; hashes bind the versions
examined by this independent review.

| Artifact | SHA-256 |
| --- | --- |
| `validation/reduced-line-height-review.md` | `fbbb17639a82641e292dfd09ed48202f5f7c2d04002602cdd39fb7f4396e333a` |
| `validation/reduced-line-height-receipt.json` | `b35274dfdb8575368bb9407fc2ab152d63371abff7591bf82d6469cd31f17340` |
| `output/reduced-line-height-r1/receipt.json` | `cfff9664ab235d77a95f45e7d1f33965bdd11f4bbe446a0edf642ab23893abc3` |
| `output/reduced-line-height-r1/native-receipt.json` | `a09a0d5000c0a598599778377ff63b5a2b4870d38502c093c65aafab2ec0c6de` |
| `output/reduced-line-height-r1/ink-region-receipt.json` | `e7a8b1f1728eb5c79cdc339eed5bae8d1075c9eb38a447e34d27615a6ef3298d` |
| `output/reduced-line-height-r1/visual/coverage.json` | `f505273cdbc2928a6a83dc7db013712e20fbba7c5a5f6b65d6988331b30c3b0b` |
| `output/reduced-line-height-r1/runtime-source-bindings.json` | `cff03100a82fd89c9367cde8a83ce44c539ab92c908438fa6b881e8bf7a3fd3a` |
