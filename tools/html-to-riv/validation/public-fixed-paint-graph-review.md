# Preserved paint graph: private constant-folding checkpoint

The compact rectangle optimization changed eight native frames despite equivalent geometry. Native replay isolates a rectangle-extent/clip interaction; see `public-fixed-folding-diagnosis.md`. The replacement removes calculated helpers while retaining every actual paint path, clipping object and draw-order record. It does not alter the runtime or use observed geometry as compiler input.

`fixed_paint_graph.rs` accepts only the typed, source-bound, fully rounded Ordered Derived route with fixed geometry. It keeps the full base/sizing prefix and all source IDs. All seven paint record types remain in their original relative order. Four reference fields are remapped. Shape root placements are proven bounded integer constants, and all six Shape and Rectangle matrix components must recompose bit-exactly in native operation order, including signed zero. An independent output reader checks fields, references and prefix identity. Unknown routes, fields, references or matrix hazards reject; this is not an arbitrary Rive optimizer.

Independent source review found no concrete gap in this bounded route. ClippingShape traverses retained parent/source subtrees; the removed helpers carry no inherited draw rules or other effects in the closed Derived grammar. Existing source/span/scalar/paint validators establish the removed-helper closure. This review is separate from native evidence.

## Evidence

- Frozen constructor `output/public-fixed-paint-graph-constructor-r1`: five source recipes construct twice with all seven artifacts identical. Each original has 7,239 records; each replacement has 264 (98 retained suffix records). Native object counts exclude Backboard and are 7,238 and 263.
- Fresh original and replacement captures: each passes 40 geometry and 40 Chrome pixel gates plus 80 alternate-clear controls. The same original and cloned file traverses four viewports. The private overlap fixture includes actual translucent sibling overlap.
- Corrected exact verifier: 972,400 scalar/matrix comparisons, complete mapped object sets, preserved prefix wire fields, and all 40 old/new decoded native images equal. Old/new Chrome references are also identical; this is not a claim that native and Chrome pixels are identical.
- Six full-size sheets directly reviewed: 15 representative pairs and 25 exact repeat transfers. See `public-fixed-paint-graph-visual-root.md`.
- Full compiler suite: 503 tests pass. Source guard passes; current constructor-bound source files still match their frozen hashes. No new public CLI/WASM/JS qualification is claimed because this pass is private and not dispatched by the public compiler.
- Verifier negative controls reject missing/empty/incomplete mappings, prefix collisions, invalid wire bools and failed capture gates. The first graph verifier run failed on core property189 missing from an older validation reader; failure is preserved. The new local decoder uses pinned schema types and leaves the historical reader unchanged. An earlier frozen-copy unit invocation lacked image fixtures; the subsequent full suite ran from the module workspace and passed.

## Matched lifecycle measurements

Using the unchanged release probe, five original/replacement pairs ran sequentially after builds and captures completed: two warmups and nine measured fresh imports per file, then original/clone resizing. Median import ranges across the five fixtures are 7.734–8.189ms original and 0.138–0.141ms replacement. Clone ranges are 7.051–7.512ms and 0.110–0.112ms; resize/update ranges are 1.092–1.203ms and 0.040–0.042ms. Exact distributions, hardware and hashes are in `output/public-fixed-paint-graph-lifecycle-r1/receipt.json`. CPU draw recording is not GPU rendering, and process peak RSS is not a leak test. These results cover three-owner scenes only.

## Remaining work

Public integration, broader source/domain and resource-boundary campaigns, and CLI/WASM/JS parity for the integrated path remain. Construction still builds the expensive original graph before folding; compile-time limits are not relaxed. Fixed scenes do not prove responsive reflow. Retain the prior compact candidate's failure evidence, but keep it outside public dispatch. Continue exact centered layout quantization and all99 backlog items after this bounded optimization. Public support counts remain13 qualified/25 partial/4 investigating/57 pending. No unfinished replacement is pushed.
