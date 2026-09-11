# Combined wrapped alignment and nested paint ordering

The perpendicular item wrappers and foreground paint chains can work together for the tested ordinary-direction/wrap and reversed-direction/wrap-reverse profiles. Moving only leaf paint is insufficient: nested translucent item backgrounds also need explicit ordering. Mixed reversal still lacks a single passing static chain across all resize states. These are fixture-specific ordinary-file experiments, not public compiler admission.

## Construction

The 72 scenes combine four main directions, two wrap modes, three positional line alignments, and three paint strategies. The parent has 50% main size and a 180px cross size. Items A/B/C have main sizes 60/60/40px and cross sizes 30/50/70px, with flex-start/center/flex-end self alignment. Their order values are 2/-1/2, so the stable order-modified sequence is B/A/C. Each item has its own translucent fill and an overflowing translucent child measuring 90px on the parent main axis and 45px on its cross axis. A following green footer exposes extent and outside painting.

The wrapper transformation copies each authored fixed main size to a perpendicular, cross-stretched line participant and transfers the authored order to that participant. The original item receives stretch inside the wrapper. Existing alignment fields independently position the item and the collection of lines. These fixture constants are authored values; no browser layout values enter file generation. General percentage/auto/bounded sizing requires the separate wrapped-sizing audit.

The three paint strategies all use existing ForegroundLayoutDrawable, DrawRules and DrawTarget records:

- Leaf-forward: move only the three child fills and chain them in order-modified B/A/C order. Item backgrounds remain native layout paints.
- Subtree-forward: move both the item fill and its child's fill, preserving ancestor-before-descendant order inside each B/A/C group.
- Subtree-reverse: reverse the groups to C/A/B while retaining ancestor-before-descendant order within each group. Globally reversing every paint would incorrectly reverse that internal order.

The navy container background and green footer retain their ordinary layout fills. The source map omits wrapper identities and is read only; all visible layout geometry remains owned by the authored objects. The augmenter first round-trips the original bytes exactly, then changes only existing ordinary properties and appends existing object types.

The three wrappers add six layout/style records. Each foreground chain over k paints adds k foreground records and 2(k-1) rule/target records: seven for the leaf strategy or sixteen for the six-paint subtree strategies. This is a bounded three-item experiment, not a resource guarantee for arbitrary trees.

## Results

Every row aggregates both axes and all three positional line alignments, 48 frames per strategy:

| Main direction / wrap | Leaf-forward | Subtree-forward | Subtree-reverse |
| --- | --- | --- | --- |
| Ordinary / wrap | 4/48 pixels | 48/48 pixels | 0/48 pixels |
| Ordinary / wrap-reverse | 6/48 pixels | 28/48 pixels | 12/48 pixels |
| Reversed / wrap | 4/48 pixels | 12/48 pixels | 36/48 pixels |
| Reversed / wrap-reverse | 42/48 pixels | 28/48 pixels | 48/48 pixels |

All 576 geometry checks and 1152 clear controls pass. Pixel checks pass 268/576 frames; all 308 failures remain in the raw receipt. Fifteen cases pass every frame: twelve subtree cases across the two passing direction/wrap profiles and three incidental leaf cases. The leaf cases do not establish general nested-paint support.

The subtree strategies pass the same-file original/clone resize sequence 400×400 → 200×200 → 120×120 → 400×400 in those bounded profiles. Line membership changes from one to two to three lines. They combine actual order sorting, independent item/line alignment, overflow and nested alpha painting. This closes a composition gap between the earlier separate geometry and paint experiments, but does not establish clipping, group opacity, deeper descendants or general wrapping.

A representative leaf-forward failure, row/wrap/flex-start at frame 0, has zero global mismatch ratio but fails the local RGB gate for item C. The automated workflow therefore retains regional paint checks; geometry and global pixel coverage alone would miss this case.

## Reproduction and visual evidence

Artifacts are under `tools/html-to-riv/output/wrapped-paint-alignment-r1/`. The preserved build command records compiler version, exact Rust sources, schema/compiler dependency hashes and augmenter hash. All 72 exact HTML/CSS requests reproduce the original RIV bytes and parsed source maps through the complete fixture adapter. Source snapshots, pinned Chrome, immutable probe and renderer identities are retained by the raw receipt.

The experimental manifest and tracked receipt override the raw driver's public scope label. Visual coverage is recorded separately in the manifest and `visual-review.md`; partial/failing cases' unreviewed resize frames remain explicitly automated-only. No public Rust/CLI/WASM/JavaScript feature is added by this adapter.

## Next decision

Preserve the passing subtree compositions as candidates while investigating geometry-dependent ordering for mixed reversal. Resolve cross-percentage containing blocks and auto-cross stretch before generalizing wrapper sizing. Clipping, deep subtree paint order, nested wrapping, baseline sharing, main-axis distribution and resource ceilings require independent evidence. A failing static chain does not establish an immutable-runtime impossibility.
