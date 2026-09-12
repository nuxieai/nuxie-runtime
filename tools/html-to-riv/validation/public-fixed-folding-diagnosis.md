# Private paint folding: retained raster mismatch

The compact rectangle candidate is not qualified as an exact paint-preserving optimization. It preserves the certified geometry prefix and passes the existing Chrome pixel gates, but 8 of 40 original/folded native frames differ in decoded RGBA at one or two pixels by one color level. Public support is unchanged.

## Reproduction and probes

`output/playwright/public-fixed-folding-debug-r1` preserves deterministic original/folded native replay and output-preserving command minimization. The minimal 19×31 case differs at (0,30): original (143,178,216,255), folded (143,179,216,255). The original draws a 32768×32768 translucent rectangle; the folded version draws a 20×50 rectangle under an artboard clip. Paint color and blend mode match.

`public-fixed-folding-render-probes.py` writes the fresh `output/playwright/public-fixed-folding-debug-r2` receipt. Nine variants were rendered twice each using the hash-verified unchanged renderer. All repeats agree exactly. Increasing only width or only height to 32768 restores the original image. Increasing both also restores it. Removing only the folded artboard clip restores it. Adding the same clip to the original giant rectangle leaves its image unchanged. Changing only the draw-path ID or paint ID leaves the folded mismatch unchanged.

The evidence identifies an interaction between rectangle extent and active clipping. It does not identify a particular internal raster arithmetic operation or establish a general rule for other scenes. Resource IDs and nondeterminism do not explain this minimized case. No renderer change, color correction or threshold relaxation is justified by this experiment.

## Compiler-only next candidate

Preserve original paint paths, clip sources and draw order, and replace source-proven constant calculation helpers with literal Shape placement. Keep all original base/sizing records and source IDs. Require native-order bit-exact Shape and Rectangle matrix recomposition, including signed zero, plus field/reference binding. This retains geometry that the compact intersection path discarded. It remains a candidate until actual ordinary-file import, original/clone resize, exact native image comparison, Chrome gates and visual review pass.

The general experiment verifier now exits 1 on mismatches. The retained failure was replayed offline under `output/playwright/public-fixed-folding-exit-gate-r1`: 957,680 checks, 40 frames, eight failed frames, exit code 1. Its inputs link to immutable saved construction/capture artifacts; no rendering or geometry was regenerated. Subsequent verifier changes add decoded RGBA comparison and mapped paint-object world checks for the new candidate; these require their own execution evidence.
