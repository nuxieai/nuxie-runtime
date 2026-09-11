# Main-axis spacing investigation — L05

Status: experimental, not public compiler admission. Runtime and renderer remain immutable.

The ordinary `layoutAlignmentType` exposes main-axis space-between (values9–11) but not space-around/evenly. Internal YGJustify enum support does not make those values reachable through the ordinary file format. `LayoutParticipant::apply_base_style` maps main-axis Fill to a zero flex basis and equal grow/shrink weight from the participant's fractionalWidth/fractionalHeight. This suggests an ordinary-file composition with zero-cross-size flexible spacers.

## Composition candidate

For N visible items, insert N+1 transparent spacers into the layout sequence. Space-evenly uses weight1 throughout. Space-around uses weight0.5 at the two edges and weight1 internally. The sum of edge and interior shares is N, so each interior gap is twice an edge gap. With a single item, the two edge shares sum to1 and consume all positive free space. Retain original authored layout parents and dimensions; spacer objects do not enter public source maps or selector matching.

The [Box Alignment distribution rules](https://www.w3.org/TR/css-align-3/#distribution-values) specify safe-center fallback for around/evenly. Do not assume that zeroed spacer widths plus the compiler's existing direction compensation produces the correct overflow fallback. Test every direction, negative free space, one item and responsive transitions. Original/clone resize must update spacing without recompilation.

The experimental adapter in `output/spacing-spacers-r1` transforms exact fixtures by adding zero-size spacer elements, compiles through the frozen public compiler, then sets the ordinary spacer main-axis Fill mode and fractional weight. It removes spacer source-map entries before the independent geometry comparison. The first attempt incorrectly set fractionalWidth on LayoutComponentStyle; the schema rejected that property. The preserved second candidate sets weight on the LayoutComponent participant instead, with the Fill scale on its style. This is an adapter construction correction, not a runtime mutation.

## Required public integration checks

- Derive spacer count and placement from order-modified authored children; never use fixture IDs/constants. Preserve source-map DOM identity and existing selector/cascade semantics.
- Keep helpers out of semantic child counts. Existing baseline measurement objects have zero main extent and must not receive extra distribution shares.
- Baseline summaries for nested columns currently assume packed children. A nontrivial justify-content changes preceding positions and baseline origins; diagnose unresolved combinations or derive the correct expression before admission.
- Account for existing alignment wrappers, intrinsic/min/max dimensions, percentage bases, empty containers and resource bounds. Do not transfer fixture qualification to all combinations.
- Check safe overflow in normal and reverse flows. Keep all failed receipts and tolerances unchanged.

A40-scene experiment covers around/evenly × all four directions × positive, overflow, single-item, intrinsic and responsive contexts. Terminal results are recorded separately before any implementation decision.

## Completed experimental evidence

The first native candidate passed284/320 geometry and pixel frames; all36 failures were reverse-flow safe-overflow positioning. The corrected composition sets native physical-start main alignment for every direction (row alignment2, column alignment6), while flexible spacers consume positive free space. This passes all320 geometry/pixel frames and640 clear controls across the same40 scenes. All64 distinct viewport pairs were inspected, with256 exact decoded crop/white-extension transfers; all40 files reproduce exactly. Column240px content beyond the tested viewport remains outside direct pixel coverage and requires taller viewports during public qualification.

See flex-spacing-experiment-receipt.json. The original failed candidate remains preserved. Public syntax is still rejected: the next step is compiler-derived spacer placement and context diagnostics, followed by Rust/CLI/WASM parity and a broader public native corpus.
