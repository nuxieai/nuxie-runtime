# Snapped unsigned gate native experiment

This is an isolated ordinary-RIV primitive experiment, not public compiler qualification. It overrides the raw driver's public scope/status labels. Runtime and renderer use immutable baseline 6c7ac16617835b5f581784ff08a9e779bb52faf3. The separate node-probe is the existing read-only probe extended to observe all WorldTransformComponents; non-layout nodes report zero dimensions as an observation placeholder. It is linked to baseline dependencies, with no runtime changes.

The explicit experimental epsilon is 1/64 (0.015625); it is not inferred from compiler input or a public admission rule. Constant ordinary Nodes represent two scalar anchors. Their observed difference feeds abs(d)=max(d,-d), shifted=abs(d)-epsilon, dead=max(0,shifted), DistanceConstraint exact distance D=65536 from origin, and snapped=min(D,2*normalized). Leader is D-snapped. All operations use existing Nodes and constraints. The badge's ordinary ForegroundLayoutDrawable is clipped by a [0,32768]^2 rectangle parented to snapped. Zero means visible; D means absent. Browser reference directly chooses the expected background independently of the graph.

Eleven scenes run four viewport steps (240x160,390x200,768x120,240x160) on an imported occurrence and its ordinary clone: 88 frames. Geometry passes 88/88; pixel gates pass 80/88; clear checks pass 176/176. Ten scenes produce exact expected 0/D and complementary D/0 in all 80 frames. The retained transition failure produces neither a binary gate nor the correct absent badge in eight frames.

| Input gap | Anchor translation | Observed dead | Snapped | Result |
|---|---:|---:|---:|---|
| 0 | 0 | 0 | 0 | exact |
| 0.0078125 | 0 | 0 | 0 | exact dead zone |
| 0.015625 | 0 | 0 | 0 | exact epsilon |
| 0.016125 | 0 | ~0.00049999915 | ~0.0009999983 | preserved failure |
| 0.016625 | 0 | ~0.0010000002 | 65536 | exact |
| 0.016725 | 0 | ~0.0011 | 65536 | exact |
| 41 / -41 | 0 | 40.984375 | 65536 | exact unsigned |
| 0.001953125 | 16384 | 0 | 0 | exact residual removal |
| 41 | 16384 | 40.984375 | 65536 | exact |
| 41.015625 | 0 | 41 | 65536 | normalizer rounding repaired |

For the final case, native normalized is the f32 value 65535.99609375 (probe JSON shortest decimal 65535.996); multiplying by two and clamping returns exactly 65536 and leader exactly zero. This directly exercises the earlier arithmetic rounding example. The translated residual case injects two ordinary constant anchor Nodes; it does not claim a real CSS layout produces that residual or establish a layout-derived error bound. Resizing and cloning preserve signals, but these constant-anchor cases do not exercise a signal changing across a wrap boundary.

The transition failure is necessary evidence: the DistanceConstraint skips magnitudes below 0.001, and snapping cannot repair that region. A public proof must guarantee every post-dead-zone separated magnitude is above the native threshold with arithmetic margin, while all same-line differences are within epsilon. The nominal threshold sample passes but does not prove the boundary universally. Zero-height distinct lines remain indistinguishable from same-line anchors. Finite normalization/intermediate bounds and mask coverage remain required.

Each scene adds exactly 32 records: 28 signal/setup records plus ForegroundLayoutDrawable, Shape, Rectangle, and ClippingShape. This transparent probe retains the dead and normalized stages separately for observation; no minimization claim. The 28 include three Nodes for origin/two anchors, four records for anchor subtraction, sixteen for abs/dead-zone/normalize/snap including epsilon, and five for constant D plus leader subtraction. Given an existing projected difference, the retained-stage unsigned gate needs 16 records plus shared origin; epsilon is shareable. No timing/performance qualification follows from these counts.

All eleven RIV byte streams and parsed maps reproduce exactly through the adapter. Commands, source, compiler/probe/renderer identities, dependency hashes, raw per-frame observations, render receipts and repeated output hashes are bound in experiment-manifest.json. The original ten-scene source snapshot is render/source-cases.json; pairs.json was subsequently extended with the final rounding case, whose separate run is render-rounding.

Directly inspected visual-0.png, visual-4.png and visual-8.png: all eleven full frame-0 Chrome/native pairs. The transition badge is visibly present only in native, as expected for the preserved failure. No visible divergence in the ten passing cases. For the other 77 pairs, visual-transfers.json proves independently for Chrome and native that each frame's complete 100x80 origin crop is byte-exact RGBA equal to its directly reviewed frame-0 crop, and every pixel outside that crop is opaque white. Thus coverage is eleven directly viewed pairs plus 77 crop/white-extension transfers (154 independent image checks), covering all 88 pairs without claiming all were directly viewed.

## Tracked receipt and observer identity

[wrapped-snapped-gate-receipt.json](wrapped-snapped-gate-receipt.json) binds the actual augmenter source/binary, adapter, cases, two raw runs, read-only observer source/binary, renderer, frozen compiler, Rust dependencies, native archives, exact reproduction and complete visual coverage. Its embedded unified diff verifies that the observer differs from the retained immutable probe by exactly two substitutions: making layout size optional and reporting those optional dimensions. Import, clone, update, resize, draw, transforms and stream recording follow the original probe unchanged. The original baseline source identity, dependency commands and feature list are separately hash-bound. No runtime or renderer source is modified.

Retained local evidence is under `tools/html-to-riv/output/wrapped-snapped-gate-r1/`; tracked receipt stores its content identities because the output tree is ignored. Reproduction means eleven exact RIV byte streams and parsed source maps from the retained binary/adapter. It is not a claim of independently rebuilt executable identity.
