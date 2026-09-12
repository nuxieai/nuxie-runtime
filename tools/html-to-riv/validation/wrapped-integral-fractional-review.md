# Integral paint at fractional native viewports

Completed a native-only comparison of four existing ordinary rounded files and their integral-paint counterparts, for1,3,8 and34 owners. Their base.riv files are byte-identical; both variants were already compiled before this check. No compiler, runtime, renderer, dependency or output file was changed to adapt to a viewport.

The frozen node-probe source accepts positive finite f32 viewport sizes up to16384, calls ordinary `set_size` and update on the same original and clone, then records geometry and draw commands. It uses ceil(width/height) only for the integer recording canvas; the Artboard layout and clip retain the fractional requested f32 values. Consequently this is not a Chrome fractional-window or pixel comparison.

Eight steps use both coordinates: immediately adjacent f32 neighbors below/above60, exact60 with59.5,59.5 with60.5,60.5 with59.5, immediately adjacent neighbors below/above160, their reversal, and a return to the first viewport. Every sequence is applied to the original and clone without reimport/recompilation. The retained receipt records viewport float bits, exact subprocess commands and source/binary identities.

All64 old/new frame pairs (128 native frames) agree on Artboard, parent, slot and visible geometry. Every visible owner retains20×20 dimensions and integral world corners. The reported2944 visible-edge checks count matched corner observations per pair; both variants were individually checked, giving5888 native integer-edge checks. Original/clone and repeated viewport geometry/draw observations agree exactly.

The stream reader validates identity linear transforms, stack restoration, the exact fractional Artboard clip, all expected active/inactive line masks, paint color/order and every replica. Old rounded replicas must have their four box masks plus expected line masks and a root32768-square draw. New integral replicas must use the expected local20×20 draw and the observed owner's integral translation, with only line masks. Unknown commands/masks or an unexpected replica fail. After clipping against the same fractional Artboard, each draw's effective rectangle (or empty result) and color matches across variants. This is a source-command/intersection comparison, not direct GPU-state or rendered-pixel evidence.

No geometry or effective-paint differences were found. Source and original records remain bound; this experiment supports the optimization at the sampled fractional states, while its general integral-geometry certificate and independent sizing/position/field proofs remain required. No arbitrary fractional CSS layout or public wrapping admission follows.

Receipt: [receipt.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/wrapped-integral-fractional-r1/receipt.json), SHA-256 `a9ff73b1ef08163b506b0b8937e35375ca90ecd896e0f48be920e3a8ff90d6c7`. Runner SHA-256 `50c072bfa2250e4aee956b3d3f14dc46271884c6784842e3b1e03596af97b266`.

Observed distinct slot-line counts by original-instance step:

- owners-1: [1, 1, 1, 1, 1, 1, 1, 1].
- owners-3: [2, 1, 1, 2, 1, 1, 1, 2].
- owners-8: [4, 3, 3, 4, 3, 2, 1, 4].
- owners-34: [17, 12, 12, 17, 12, 5, 5, 17].
