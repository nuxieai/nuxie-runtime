# Private constant-folding foundations

This checkpoint adds three compiler-only modules, registered privately. It does not enable public paint folding, change runtime behavior, or qualify additional backlog items. The runtime source guard passes at the immutable baseline; native probe and renderer hashes match the previously audited tools.

- `fixed_scalar` evaluates closed literal Node/TranslationConstraint files with source-byte identity, exact native f32 matrix/constraint operations and whole-file validation. A source audit found unvisited objects escaping closure checks; global ownership/kind checks and visiting every Node repaired that gap. Layout, TransformConstraint and DistanceConstraint remain unresolved.
- `fixed_geometry` derives initial fixed wrapping layout from the bound original base/source records. It follows native packing, reversed physical traversal and coupled alignment, retains geometry owners and exact source provenance, and rejects unproved contexts. It does not evaluate the subsequent sizing/position constraints. Source matching uses explicit provenance equality; Debug formatting was rejected because distinct opaque identities may print alike.
- `fixed_paint` emits and independently binds four ordinary records per visible directly described rectangle, preserving the base prefix and descriptor identities. This is a paint primitive, not proof that a rectangle corresponds to source CSS or the final constrained scene.

## Evidence

`public-fixed-foundations-unit-receipt.json` binds the final source and full compiler test run: 493 Rust tests pass. No new product WASM/JavaScript qualification is claimed for these private foundations.

Scalar: `output/public-fixed-scalar-r1/native-receipt.json` records 60 scenes, 480 original/clone resize frames and 140,096 exact f32 coordinate comparisons with no mismatches, including signed zero. Construction is deterministic and original/clone/repeated-size coordinates match. The source audit and subsequent native confirmation are in `public-fixed-scalar-review.md`.

Initial geometry: `output/public-fixed-geometry-native-r1/receipt.json` records 30 source-authored base scenes and 240 frames. The original observer used Python integer JSON parsing, which loses negative zero. The preserved captures were independently rechecked without new native execution in `output/public-fixed-geometry-native-r2/receipt.json`: 12,928 exact size/world comparisons pass, all array lengths and owner sets match, and 785 prior artifact hashes verify. No negative-zero scalars occurred in this corpus. This does not establish final constrained geometry or Chrome fidelity.

Paint primitive: `output/playwright/public-fixed-paint-primitive-r1` records six directly authored descriptor scenes, 48 Chrome/native pixel passes, 96 alternate-clear comparisons and 30 exact repeat transfers. Root directly reviewed all six unscaled sheets (18 distinct frame pairs); see `public-fixed-paint-visual-root.md`. Faint filled-region differences remain. The strict offline native stream reader confirms all 48 ordered rectangle/color streams and rejects five mutated controls. Its initial artboard clip-rule assumption was corrected for the same captured simple rectangular path; no image or scene was changed. This is ordinary primitive rendering evidence, not public CSS compilation or whole-scene fold equivalence.

The copied-crate validation bridges are explicitly recorded patches and are absent from the product API. Their constructors emit ordinary bytes. Native observations are validation outputs only; no observer measurements enter a recipe or compiler calculation. Historical receipts/captures are retained.

## Next implementation

Connect certified initial geometry to a restricted evaluator of the actual sizing/position/paint graph. Extend support for the exact TransformConstraint and DistanceConstraint operations needed by that graph, including layout bounds, anchor behavior and machine operation order. Final world transforms must follow constrained parent transforms; initial world seeds cannot be substituted for final corners.

Then run the three-leaf fractional/reversed paint-only folding experiment from `public-fixed-folding-plan.md`, retaining original geometry records and source IDs. Compare old/new final geometry, strict native paint commands, Chrome/native pixels and original/clone resize behavior. Benchmark only after equivalence is established. Keep the existing path for unproved cases. Exact centered LayoutUnit geometry and the wider responsive/nested/typography backlog remain outstanding; all 99 items remain the goal.
