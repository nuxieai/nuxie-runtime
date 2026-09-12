# Mixed preferred coefficients with combined opposite-axis bounds

This private ordinary-file composition matches Chrome geometry and image presence for all 64 frames across eight files. Pixel gates pass 54 frames and fail ten. The failures remain in the receipts; this is not pixel-perfect qualification or public compiler admission.

The source matrix crosses width/height preference and row/column parents with two automatic-axis bound pairs: `min:100px; max:35%` and `min:80%; max:100px`. Every preferred axis starts at `50%` with `min:75%; max:60%`, so the source coefficient resolves to 75%. Parents have 20px padding, and the embedded opaque image is 96×64. Both conflicting and active bounds are exercised across the existing viewport sequence and clone replay.

The unchanged `image-preferred-mixed-candidate.py --cases` generator uses the frozen 354-Rust-test preferred-constraints build as a seed. All eight original requests remain rejected by that seed compiler. Source composition emits ordinary owner bounds and resolves compatible source coefficients without consuming browser geometry. Native captures use the immutable baseline probe and renderer; the build and capture receipts bind those artifacts. Public integration must independently establish exact scene/request/map identity or capture its own results.

All eight original-resolution gallery sheets were directly inspected, covering 24 representative Chrome/native pairs. The helper verified 40 additional visual transfers, preserving each frame's own numerical evidence. Complete colored source patterns and both image corner markers remain visible; padded placement and row/column tail placement agree visually. Subtle fractional raster edges remain in failed comparisons.

The ten failed rows are six tail-local-RGB-only failures, two mismatch-ratio-plus-tail failures, and two mismatch-ratio-only failures. The mismatch-ratio failures are height-preferred point-min/percent-max opposite bounds in both row and column at frames 1 and 5. Therefore these are not accurately summarized as ten tail-only failures. No threshold was relaxed and no failing row was removed.

Authoritative compact bindings and exact failed rows are in `image-preferred-mixed-combined-receipt.json`. The completed direct review is `output/image-preferred-mixed-combined-r1/visual-r1/review-receipt.json`; its SHA-256 is `c34a57d026e22a06beae66d89f2138e4867135fbb8433d2e2d29566e39d6e160`.

Reproduction from the module directory (use a fresh output directory):

```sh
python3 validation/image-preferred-mixed-candidate.py output/image-preferred-mixed-combined-r1 --cases validation/image-preferred-mixed-combined-cases.json
node validation/public-image-native.mjs output/image-preferred-mixed-combined-r1 output/immutable-baseline-toolchain-r2/baseline-probe output/immutable-baseline-toolchain-r2/renderer-replay
python3 validation/public-image-transfer.py output/image-preferred-mixed-combined-r1 output/image-preferred-mixed-combined-r1/native-receipt.json
python3 validation/public-image-visual.py --root output/image-preferred-mixed-combined-r1 --label visual-r1 --private-candidate
```

Gallery generation does not complete direct review. Inspect every original sheet, record truthful notes, and use the helper's `--complete` operation. This evidence covers these eight bounded cases only; other intrinsic sizes, padding rules, layouts, and declarations require separate evidence.
