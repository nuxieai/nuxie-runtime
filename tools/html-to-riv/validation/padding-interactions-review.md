# Padding interactions on ordinary immutable files

The36-scene expansion passes288/288 Chrome geometry frames and280/288 pixel frames, with576 clear checks. Eight local RGB failures remain in padded row/reverse parents using space-evenly. This supports narrowing some blanket padding guards but is bounded experimental evidence, not public integration qualification.

The adapter uses frozen public-auto-margins-r3/html-to-riv to emit existing ordering, alignment wrappers, around/evenly helpers and main auto-margin helpers. It then edits only authored LayoutComponentStyle physical padding value/unit fields. Source IDs, parent relationships, paint records and helper topology stay unchanged. Each input is compiled once, imported/cloned once and resized400x200→200x120→100x80→400x200 in both occurrences; runtime/renderer remain the immutable target.

The main24scene corpus has six profiles in all four directions: pixel-padded parent with main automatic margin on B; percentage-padded parent with space-around; pixel-padded parent with space-evenly; unpadded centered A containing a padded D; pixel-padded parent with order2/−1/0; and mixed-unit computed inheritance/cascade. A separate four-case percentage-D run retains empty descendants as controls. A final eight-case pixel/percentage-D run adds a50%×50% grandchild E inside D, making its content insets and percentage containing block observable. Total wrapper family is16scenes (eight empty-D controls plus eight painted-grandchild cases), not a claim that empty padding alone tests content positioning.

| Family | Scenes | Geometry | Pixels |
|---|---:|---:|---:|
| Padded parent + main auto margin | 4 | 32/32 | 32/32 |
| Percent-padded parent + around | 4 | 32/32 | 32/32 |
| Pixel-padded parent + evenly | 4 | 32/32 | 24/32 |
| Padded parent + order | 4 | 32/32 | 32/32 |
| Mixed computed inheritance/cascade | 4 | 32/32 | 32/32 |
| Unpadded aligned ancestor + padded descendant, including grandchild controls | 16 | 128/128 | 128/128 |

The wrapper candidate leaves A and its synthetic wrapper unpadded. Padding belongs to D, whose containing block is still the authored definite60×40 A. D is50%×50%; E is50%×50% of D's content box. Both px and percentage padding inset E and change its used dimensions correctly across all directions/resizes. This supports allowing a padded descendant under an unpadded wrapper when that authored containing-block relationship is preserved. It does not support percentage padding on the wrapped A itself, or arbitrary intrinsic wrapper bases.

The mixed case computes P's1em top padding at20px font size; A inherits that computed20px despite its10px font size. Percentage right padding remains a percentage descriptor resolving against A's own containing width; longhands override left/bottom. These descriptors are fixture-authored inputs to the adapter. Chrome executes the actual CSS cascade/var/inherit declarations independently, but this is not a test of the new padding parser or computed pipeline; public integration must reproduce these files independently.

The eight failures are row and row-reverse `parent-px-evenly`, frames0/3/4/7, reported as D local RGB error. All geometry passes; the full comparison metrics/images remain failed. No threshold adjustment or renderer-impossibility claim is made. Around and main auto helpers pass in the tested F0/no-growing-sibling profile; these results do not settle interaction with authored growth or a directly padded alignment wrapper.

All36 RIV byte streams and parsed source maps reproduce exactly. Visual deduplication independently hashes complete decoded Chrome and native RGBA including dimensions. All91 distinct pairs were directly inspected in23 full-frame sheets, visual-0.png through visual-88.png step4. No visible layout divergence; the eight fractional descendant color failures remain acknowledged. The other197 pairs are independently exact fullRGBA equal to directly inspected references, including cross-fixture empty-D controls where the render is unchanged. This gives complete288pair visual coverage (394 independent transferred image checks); it does not claim direct inspection of all288 frames.

Source/tool/fixture/receipt identities, build command, dependencies, reproductions, explicit per-case failure matrix and visual bindings are recorded in padding-interactions-receipt.json. No compiler/runtime source changes or commit were made for the expansion. The separate isolated padding-module test receipt cleanup was completed under padding-module-tests-r2 and does not reuse the collided directory.

Tracked bindings: [padding-interactions-receipt.json](padding-interactions-receipt.json). Local evidence: `tools/html-to-riv/output/padding-interactions-r1/`.
