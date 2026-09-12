# Private snapped wrapped-sizing checkpoint

The compiler-private sizing discriminator now uses the same ordinary-node arithmetic as snapped paint: absolute anchor difference, epsilon subtraction, zero floor, distance normalization, doubling and upper clamp. Forward/backward line-maximum carries consume the finalized unsigned gate. `align_items` takes an explicit validated epsilon and returns observation IDs without emitted names or extra observation records. This is not public CSS wrapping admission or a runtime change.

The experiment uses epsilon 1/64. It does not derive that constant from source error bounds. Exact preflight cost is zero for no items and 61N−33 for nonempty sizing plans. For each three-item fixture the sizing graph adds 150 records; the unchanged paint graph adds 149. Independent encoded-record counting verifies 43 raw records, 193 after sizing and 342 final records: 299 added in total. Invalid epsilon inputs diagnose before mutation, including empty plans; construction tests cover variable counts and axis/reversal combinations.

## Native and visual evidence

The 48 preserved authored inputs cover all four main directions, both wrap directions, positional line placement, percentage/bounded-percentage cross sizes, source order and nested alpha paint. Every final RIV/map and sizing trace reproduces deterministically. Actual original/clone imports run through the existing four-step viewport sequence without recompilation. Chrome 153.0.8010.12 is the reference; immutable Rust Metal RasterOrdering (`clockwise-atomic`) supplies actual native pixels. The read-only observer's source substitutions and immutable dependencies remain bound.

All 384 geometry/pixel checks and 768 alternate-clear checks pass. All 768 adjacent sizing gates are exactly 0 or 65536 as independently expected from line membership. Forward carries, backward carries, line maxima and offsets each supply 1,152 checked values. Carry/offset comparisons retain the existing 0.1 geometry tolerance; the gate check itself is exact. Every scene changes gate state during resize; clones and repeated sizes reproduce the same observed values.

All 768 complete Chrome/native PNG files are byte-identical to the prior snapped-paint campaign, and named geometry/browser measurements match. The collector reconnects every pair through the preserved 96 directly inspected pairs and 288 exact transfers. This checkpoint claims no new direct inspection and does not replace failed images or alter tolerances. The collector’s 5,400 bindings cover source snapshots, tools, files, measurements and the complete visual chain.

## Public isolation and remaining work

The full public build passes 361 Rust tests, 56 Node tests, strict TypeScript, native/WASM builds and the immutable-runtime guard; 288 live/frozen source bindings match. CLI and WASM hashes remain identical to the prior mixed-image checkpoint. All 282 public image files/maps and 794 historical outputs remain exact, confirming that this private change does not open or alter the public route.

The earlier threshold experiment still has eight failures among 88 frames: a positive post-dead-zone distance below 0.001 can remain nonbinary because the native normalizer returns early. This work does not repair or generalize away that failure. Zero/tiny line separation, finite mask coverage, larger-item native behavior, quadratic paint cost and arbitrary transforms remain unresolved. Public admission still requires the source-derived bounds and final-record checks described in `wrapped-sizing-certificate-next.md`; historical percentage-only fixtures do not automatically satisfy a positive line-size lower bound over the whole viewport domain.

Run `python3 validation/wrapped-sizing-snapped-checkpoint.py` from the module directory to recheck the collector inventory, observations, actual record counts, full build logs and public regressions without rendering or building. It writes the consolidated verification under `output/wrapped-sizing-snapped-checkpoint-r1`, outside the collector's input tree.
