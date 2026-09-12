# Original paint at fractional native viewports

Compared the existing integral-replica resource files against the new original-paint resource-r2 files for1,3,8 and34 owners. The original base.riv bytes match exactly for each pair. Both inputs were compiled beforehand; this check only imports ordinary files through the pinned baseline node-probe.

Each original and clone receives eight continuous f32 viewport steps: nextdown/nextup60 on opposite axes,60×59.5, nextup/nextdown60,59.5×60.5,60.5×59.5, nextdown/nextup160, their reversal, and a return to the initial fractional size. The probe passes exact f32 dimensions to set_size; only its recording canvas uses ceiling to integer pixels. No browser viewport, renderer replay, screenshot or timing benchmark was used.

All64 frame pairs (128 native frames) preserve the measured Artboard, parent, slot and visible geometry. Visible20×20 rectangles have integer corners and disjoint interiors. There are2944 matched corner comparisons, checked on both variants (5888 individual native integer-edge checks). Each original/clone sequence and repeated viewport matches.

The prior integral stream is checked against its exact per-line replica sequence, masks, color and foreground draw bounds. The new ordinary original-paint stream is checked independently: exactly one Artboard clip, one white file-owned background, and each visible source owner painted exactly once with its authored color and matching local/world rectangle. Missing, duplicate, overlapping or unexpected paints, unknown commands, nonidentity linear transforms, incorrect fractional clip sizes and unbalanced state reject. Every corresponding owner's clipped effective paint interval or empty result agrees across the two modes. Disjoint interiors make traversal order irrelevant for this observation; the comparison does not require identical command order or helper counts.

The source copies the established original-stream reader with one explicit adaptation: frameSize must equal the ceil of the fractional layout dimensions, while all Artboard clips and geometry retain the exact f32 values. The source observer and strict AABB/extraction dependency are hash-bound. No captured command stream is rewritten to accommodate the reader.

Result: zero layout differences and zero effective-paint differences. This is native command/geometry evidence for the sampled resize states, not direct GPU pixels or a fractional Chrome comparison. The full-domain Nonoverlap certificate and source/position/field binding remain required independently.

Receipt: [receipt.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/wrapped-original-fractional-r1/receipt.json), SHA-256 `e2d2e3d136178d32f39b7a14ce2aae6effa173dae25498d2585af28aad3fc6d3`. Runner SHA-256 `b7ffe64388b1d580118914c3127fa4f41208f885ca892662ae40d8ac33dcf000`.

- owners-1: 16 pairs, cloned and repeated geometry/effective paint exact; 64 paired corner checks.
- owners-3: 16 pairs, cloned and repeated geometry/effective paint exact; 192 paired corner checks.
- owners-8: 16 pairs, cloned and repeated geometry/effective paint exact; 512 paired corner checks.
- owners-34: 16 pairs, cloned and repeated geometry/effective paint exact; 2176 paired corner checks.
