# Ordinary flex fractions and point basis: native experiment

The ordinary immutable LayoutComponent mapping matches Chrome geometry in all192 frames of this24-scene experiment. Pixels pass184/192; eight thin fractional alpha-paint cases remain failed and are preserved. All384 clear checks pass. This is bounded experimental evidence, not public CSS qualification.

The adapter compiles the same document without its flex factor/basis declarations, retaining automatic preferred main size, explicit minima, cross size and nested paint. It then changes only each authored flexible participant's main scale to Fill(1), main fractionalWidth/fractionalHeight to authored grow, flexBasis to the selected value and flexBasisUnitsValue to Point(1). The native shrink factor consequently equals grow. All record IDs, parent relationships, source-map entries and paint records remain unchanged. No wrapper or flexible spacing helper participates. No runtime/public compiler source is changed.

The six profiles run in row, row-reverse, column and column-reverse. Parent main size is50% of the viewport, resolving200→100→50→200 on the original, then the same sequence on its ordinary clone. Cross dimensions are fixed; both flexible items include a fixed10px nested alpha-painted descendant.

| Profile | Grow A/B | Authored shrink A/B | Point basis A/B | Bounds / fixed sibling |
|---|---|---|---|---|
| equal-basic | 1/1 | 1/1 | 60/40 | min0; fixed40 |
| equal-partial | .25/.25 | .25/.25 | 60/40 | min0; fixed40 |
| equal-clamped | 1/1 | 1/1 | 60/40 | min20/30,max70/90; fixed40 |
| zero-unequal | 2/1 | 0/7 | 0/0 | min0; fixed40 |
| zero-partial | .25/.25 | 9/0 | 0/0 | min0,maxA20; fixed40 |
| zero-minimum | .2/0 | 7/3 | 0/0 | minA10/minB0; fixed100 |

All equal-factor point-basis cases pass every geometry/pixel frame, including actual shrink and partial-factor overflow. Zero-basis unequal-factor cases also match geometry despite native shrink being tied to grow, within this definite-container/nonshrinking-sibling profile. At parent200, zero-partial resolves A20/B40/fixed40, retaining unused space. At parent50, zero-minimum resolves A10/B0/fixed100 and overflows, matching Chrome.

## Preserved failures

Zero-partial fails local RGB gates at parent50 in both original/clone (frames2/6), across all four directions: eight failures total. A and B are each exactly2.5px in Chrome and native; positions and nested descendant geometry also match exactly. Row local RGB error is approximately6.57 for A and6.61 for B, while the global mismatchedPixels count is0 under the separate pixel threshold. The failure remains meaningful because the local gates intentionally catch small-region errors. Do not classify these frames as pixel passes or infer general renderer impossibility. The raw receipt, images, differences and metrics remain retained.

## Visual and reproduction coverage

Directly inspected all72 original frames0/1/2 Chrome/native pairs in18 full-frame contact sheets `visual-0.png` through `visual-68.png`, step4. Layout progression, min/max freezes, unused free space, reverse directions, overflow and descendant paint visibly follow the same geometry. The failed2.5px cases retain a subtle thin-edge/color discrepancy; their numerical failures are not waived by the overall visual similarity. No other visible divergence was found in the passing pairs.

The other120 pairs transfer by exact decoded-fullRGBA equality independently for Chrome/native to those directly inspected frames: frames3/4/7→0,5→1,6→2. `visual-transfers.json` binds240 independent image checks, giving complete192-pair coverage without claiming all were directly viewed. All24 final RIV byte streams and parsed source maps reproduce exactly through the retained adapter/binary. Source, build command, dependencies, compiler, immutable probe and renderer identities are hash-bound in the receipt.

## Pinned Chrome shorthand characterization

A separate CSSStyleDeclaration probe uses pinned Chrome153.0.8010.12; it is syntax/serialization evidence only. `1 auto 2` is rejected. `10px 2 .25` and `2 .25 10px` both serialize to `2 0.25 10px`. Omitted basis in `2` or `2 1` serializes as0%; explicit third zero in `2 1 0` becomes0px, while `2 1 0%` retains0%. Comments separate tokens in `2/**/1/**/10px`. `auto` becomes1 1 auto; `none` becomes0 0 auto. `initial` preserves CSS-wide longhand tokens and computes0 1 auto. Source and complete acceptance/longhand/computed results are preserved in `probe-shorthand.mjs` and `shorthand-chrome.json`.

## Remaining qualification boundaries

The input profile uses automatic preferred main sizes, explicit zero/fixed minima, point bases, definite parent main size and fixed nonshrinking siblings. It does not qualify mixed positive-basis shrinking siblings with independent factors, intrinsic ancestors, auto/percentage basis, main automatic margins/around-evenly helper interaction, baseline summaries, cross alignment wrappers, ordered siblings, inherited parser integration or automatic minimum semantics. Those remain separate work from this direct runtime mapping. No broad support or performance claim follows from these24 fixtures.

Tracked evidence bindings: [flex-factors-native-receipt.json](flex-factors-native-receipt.json). Retained local artifacts: `tools/html-to-riv/output/flex-factors-native-r1/`.
