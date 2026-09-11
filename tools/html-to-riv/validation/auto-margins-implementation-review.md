# Auto margins: ordinary-file implementation with retained pixel limits

The final implementation corpus and boundary corpus contain 86 scenes and 688 frames: all 688 geometry checks and 1376 clear checks pass; 680 pixel checks pass and eight fail. The main 64-scene r3 run exited 1 (512 geometry, 504 pixel, 1024 clear passes). The separate 22-scene boundary run exited 0 (176 geometry/pixel, 352 clear passes). Auto margins remain partially validated, not fully qualified.

The eight retained failures are `auto-margin-row-multiple-main` and `auto-margin-column-multiple-main`, frames 0/3/4/7: the white descendant fails its local RGB check at fractional positions while geometry passes. Thresholds were not relaxed. Earlier r1 native-margin failures and r2 static-alignment overflow failures remain preserved in separate evidence.

## Implementation and admitted limits

The frozen compiler's `margins.rs` separates authored physical margins from alignment-helper margins. It parses auto and zero-length physical longhands, one-to-four-value shorthand, and supported CSS-wide values through the ordinary cascade. Nonzero lengths and percentage margins remain rejected.

`compiler.rs` emits flexible, zero-cross-size ordinary layout participants for main-axis auto margins. They consume positive free space before native justification and collapse when space is exhausted, preserving the parent's original alignment during overflow. Cross-axis auto margins use the existing perpendicular safe-alignment wrapper instead of native cross auto margins that distributed negative free space in r1. Cross margins override the item's positional self-alignment as expected. These changes reside in the emitted file and compiler; the runtime and renderer remain immutable.

Explicit composition guards reject main-axis auto margins with space-around/space-evenly and effective baseline sharing with main-axis auto margins. Percentage heights/bounds inside an auto-height parent retain the existing rejection. Grid and wrapping are not admitted by this work. Arbitrary nonzero/percentage margins, margin collapse, and unrestricted baseline/distribution combinations are not qualified.

## Tested combinations

The main corpus covers four flex directions, main/cross start/end/both auto margins, fixed and intrinsic child variants, multiple main auto margins, all-auto shorthand, percentage cross sizing and cascade combinations. Three colored items, a white descendant, translucent paints and a following footer make both placement and paint differences visible. The viewport sequence changes positive free space to overflow and returns, using the same imported file on original and clone instances.

The boundary corpus adds intrinsic parent extents, min/max-constrained intrinsic parents, cross auto margins combined with space-around/evenly, and row baseline sibling interaction. All 176 boundary frames pass. These finite examples do not establish every composition allowed by the broader language.

## Reproduction and complete visual coverage

The main artifacts are `output/public-auto-margins-r3/`; boundary artifacts are `output/public-auto-margins-boundaries-r1/`. All 86 complete CLI outputs were reproduced byte-identically with equal parsed maps, using the frozen r3 `html-to-riv`. A byte-identical local copy of that binary was placed in the boundary output directory for collection; the original boundary driver receipt records the r3 binary it actually used.

Every frame has visual coverage. All 98 distinct r3 pairs match reviewed r2 pairs exactly in decoded RGBA for both Chrome and native; 414 additional r3 frames repeat those pairs. The boundary corpus contains 50 newly inspected pairs, eight distinct pairs transferred from reviewed r2, and 118 within-run repeats. All 13 new boundary sheets (`new-visual-0.png` through `new-visual-48.png`, increments of four) were directly inspected with no visible divergence. Thus the 688 frames comprise 50 new direct reviews, 106 distinct prior-review transfers, and 532 within-run transfers. No frames are automated-only. The prior r2 evidence itself resolves every transfer to direct r1/r2 reviews; a transfer never means only similar appearance or matching failure status.

Each directory preserves `reproductions.json`, `matrix.json`, `counts.json`, `visual-direct.json`, `visual-transfer.json`, `prior-review-transfers.json`, and independently verified `visual-transfer-verified.json`. `prior-review-transfer.py` proves cross-run matches and identifies new pairs. The r3 `freeze-final.py` checks source snapshots, actual frozen binaries, driver/tool hashes and the complete visual partition. `evidence-commands.json` records the entry points and authoritative exits. Per-directory `experiment-manifest.json` files bind the artifact sets and prior reviewed evidence.

The r3 `source/` and `source-hashes.json` preserve the implementation snapshot and were hash-verified; the current source may gain later tests or documentation. Chrome 153.0.8010.12 is compared with immutable Rust Metal RasterOrdering. Driver, reset, pixel gate and fixture snapshots are preserved. No native reruns, runtime edits, public source edits or commit were performed during this evidence task. Broader compiler test/regression results are tracked separately by the parent task and are not inferred from these visual receipts.

## Public transport, regression and expansion checks

The final source passes111Rust tests and31Node tests, native/WASM builds, TypeScript checking and the immutable source guard. Exact byte/source-map regression preserves482previous unique public outputs, including the complete112-scene spacing corpus. Existing failures in those historical corpora retain their earlier status; byte identity is not a new pixel pass. Source/build identities and logs are bound by `output/public-auto-margins-r3/build-provenance.json`. The public checkpoint freezes the22passing boundary scenes and112existing spacing scenes; it deliberately does not relabel the failing64-scene margin corpus as a passing receipt.

At most two main-axis margin spacers add four records per authored item, and at most one perpendicular cross wrapper adds two. Automatic margins therefore add at most6Nrecords under the existing8,192authored-element limit (49,152records), excluding unrelated existing baseline/spacing compositions. The resource regression compiles8,192auto-margin elements and rejects8,193before publishing output. This is a construction bound, not a device performance guarantee.

The fixed-spacer control in `auto-margin-fractional-control-review.md` replaces the three flexible main-axis spacers with their observed23.333334px target extents. All eight target frames retain exact prior geometry, full native RGBA and independently captured Chrome RGBA; all eight pixel failures persist. This excludes flexible distribution as the differentiating cause at those frames only, without asserting renderer impossibility or ruling out other ordinary-file compositions. Other resize states intentionally differ and are not qualification evidence.
