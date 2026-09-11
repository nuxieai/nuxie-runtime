# Direction and wrapping boundaries of static foreground paint chains

All **128 geometry comparisons pass**, but only **48/128 pixel comparisons pass** across 16 scenes; all 256 clear controls pass. Original/cloned scenes share the same files through three main-axis sizes. Four particular candidates pass all 8 frames each. This is experimental composition evidence, not public compiler admission.

## Matrix

Both row and column axes show the same result:

| CSS main direction | Wrap mode | Forward paint chain | Reverse paint chain |
|---|---|---|---|
| ordinary | wrap | 8/8 | 0/8 |
| ordinary | wrap-reverse | 2/8 (one line) | 2/8 (three lines) |
| reverse | wrap | 2/8 (three lines) | 2/8 (one line) |
| reverse | wrap-reverse | 0/8 | 8/8 |

At the two-line viewport, neither tested static chain matches either mixed-reversal profile. Thus the simple axis-based direction mapping cannot choose one of these two chains for all resize states. No claim is made that all possible ordinary-file arrangements are ruled out.

## Construction and isolation

The three items measure 60, 60 and 40 px on the main axis, inside a parent measuring 50% on that axis and 100 px on the cross axis. Viewports of 200×200, 320×320, 120×120 and 200×200 produce available main sizes of 100, 160, 60 and 100 px, resulting in two, one, three and two lines. Each item measures 25 px on the cross axis; its colored leaf measures 90 px on the main axis and 45 px on the cross axis, exposing overflow across both boundaries. The navy parent background remains an ordinary layout fill. Exact CSS includes `align-content:flex-start` and `align-self:flex-start`.

Fixture adapter compiles transformed row-reverse/column-reverse to preserve DOM collection, then sets the corresponding native direction (0/1/2/3) and wrap mode (1/2). It moves only the three leaf fills to ordinary authored ForegroundLayoutDrawables and chains them with DrawRules/DrawTarget placement 1. Forward chain references A→B→C; reverse chain reverses those references. Layout geometry, authored parents and source-map IDs remain intact. No host geometry updates, raster fallback, runtime changes or recompilation on resize.

## Evidence and review

Tracked binding: [foreground-directions-receipt.json](foreground-directions-receipt.json). All experiment source hashes and driver tool hashes were reverified during consolidation, the counts and matrix were recomputed from the raw receipt, stored reproductions were compared again, and the 20 decoded RGBA transfers were rechecked. The binary is bound by its hash and preserved build command; this consolidation did not rerun native rendering or claim new public API qualification.

Artifact paths below are relative to `tools/html-to-riv/output/flex-foreground-directions-r1/`.

- `render/receipt.json` retains all 128 frames, comparisons, immutable probe streams and clear controls. Raw driver public labels are overridden by `experiment-manifest.json`.
- `matrix.json` records the precise passing frame indices by direction/wrap/chain without relying on ambiguous concatenated fixture names.
- All 16 frame 0 Chrome/native pairs were visually inspected, showing both successful profiles and failures. The four candidates passing every frame also have frame 1 and 2 pairs directly inspected, total 24 direct pairs on six sheets (`visual-0.png` through `visual-20.png`).
- `visual-transfer.json` proves the remaining 20 frames of those four fully passing candidates equal reviewed Chrome/native counterparts independently by decoded RGBA. Remaining partial/failing frames have automated comparisons only.
- `reproduction/` re-executes the complete adapter for all 16 cases; exact RIV and parsed map equality verified. Manifest binds sources, frozen compiler, augmenter and receipt hashes; `build-command.json` preserves build arguments.

## Implication for next work

Foreground drawables provide useful layout/paint separation for ordinary+wrap and reversed+wrap-reverse in this profile. Mixed reversal changes required paint order when line membership changes, consistent with Chrome's visual-line ordering; neither static DOM order nor its global reverse handles it. A broader compiler design must investigate conditional/order compositions, line-aware ordinary capabilities, or an explicitly diagnosed restriction for affected overlap. Do not silently claim general wrapping just because geometry passes. Nested painted parents, subtree background ordering, clipping/opacity, order ties, independently aligned items and baseline sharing still require separate evidence.
