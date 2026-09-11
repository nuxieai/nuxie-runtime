# Combined slot sizing with optimized nested paint groups

The cheaper paint-group composition preserves the completed combined-slot corpus exactly. **48 scenes / 384 geometry and pixel frames / 768 clear controls pass**, and all 48 RIV/map outputs reproduce. Every Chrome and native frame is decoded-RGBA identical to the prior combined-slot run. This is the selected experimental construction to carry into guarded public implementation; no further alternative is needed for this tested profile.

## Changes and preserved behavior

`output/wrapped-combined-optimized-r1` copies the prior `size-augment` binary and slot source byte for byte. That stage still computes cross-line extents and alignment through ordinary slot objects and constraints. The authored B/A/C order, slot anchors, line alignment factors, and parent-before-descendant order within each painted item remain unchanged.

Only the paint stage changes: candidate members preceding a proposed line leader are omitted; adjacent leader/membership difference signals are shared; and each signal source anchor uses one shared copy node. Two painted objects per item (authored parent and its descendant) share membership and leader masks while retaining their own ForegroundLayoutDrawable/Fill/color and clipping records. Paints remain ordinary file objects selected at runtime by geometry, with no browser coordinates or host CSS setters.

The per-fixture record count is now **43 raw → 175 after sizing → 285 final**. Sizing adds the same 132 records. Paint adds **110 instead of 195**, saving 85 records (about 44% of paint overhead and 23% of the whole file's object count). Deterministic reproduction checks all stages through the complete adapter; the raw and sized phases remain frozen.

For N≥1 with two paints per item, let M=N(N−1)/2 and T=N(N+1)/2. The optimized paint phase contains 1 origin, 2N anchors, 2(N−1) shared source-copy records, 5M remaining difference-signal records, 2M membership-mask records, 4(N−1) leader inversion/mask records, 4M clipping records, 6T paint records and 2(2T−1) draw-chain records. Their sum is **(21N²+15N−14)/2**; at N=3 it is 110. This algebra is not a claim that combined sizing for arbitrary N was tested: this prototype deliberately keeps the frozen three-slot sizing implementation and exact 48-scene matrix. Empty containers need no group. Arbitrary paint-tree growth requires budgeting actual emitted records rather than assuming two paints per item.

## Evidence and visual coverage

The pinned driver compares Chrome 153.0.8010.12 with immutable Rust Metal RasterOrdering on the same original and cloned RIV through four viewport steps. All 384 geometry/pixel checks and 768 cyan/transparent clear controls pass without changed tolerances. `reproductions.json` records 48 identical RIV byte outputs, equal parsed maps, and exact raw/sized/final counts.

`prior-pixel-identity.json` checks every corresponding Chrome and native image against `output/wrapped-combined-slots-r1` (384 pairs / 768 images), requiring equal sizes and identical full decoded RGBA bytes. The prior corpus has 96 directly reviewed pairs and 288 independently verified exact transfers, as bound by `validation/wrapped-combined-slots-receipt.json`. `visual-review-transfer.json` records that evidence chain. No additional direct image review is claimed here, and there are no unmatched optimized frames.

The experiment manifest binds sources, binaries, adapters, fixtures, native receipt, provenance, comparison proofs and reproductions. The copied sizing binary's hash and identity are separately recorded in `sizing-identity.json`. The generic driver's public status string applies mechanically to adapters; this experiment's manifest and tracked receipt explicitly classify the results as experimental, with no public syntax admission.

## Boundaries for integration

The prior combined corpus covers all four main-axis directions, normal/reverse wrap, start/center/end line placement, percentage and bounded cross dimensions, authored order and nested alpha paint for three items. Its existing limitations remain: tiny/zero line-separation gates, finite mask/coordinate bounds, arbitrary paint-tree/geometry nesting, unproven baseline/stretch/distribution combinations, and whole-scene resource growth. This optimization does not relax those guards or erase earlier fractional failures in other corpora. Use this validated cheaper paint phase with the existing sizing phase when implementing the corresponding guarded public profile; qualify public interfaces separately.

No runtime, renderer, schema, shared dependency or public compiler source was changed for this experiment. Nothing was committed by this evidence task.
