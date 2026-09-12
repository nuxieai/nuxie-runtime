# Public compiler numerical-provenance controls

Four actual public-compiler files confirm the scalar provenance counterexamples through the unchanged native importer and original/clone resizing. All32 final width observations exactly match the expected native binary32 values. No modified renderer, metadata setter or browser-baked layout participates.

Direct width100.71428680419922px, the same value through a custom property, and the same value as a missing-variable fallback all produce width100.71399688720703. This is0.00028991699218875 away from the original exact decimal. Those inputs must retain the original value separately from the normalized native spelling.

One hundred nested elements use font-size:1.1em and width:1em. A final child sets font-size:10px and width:inherit. Its width remains220490.171875: inheritance copies the computed width rather than recomputing it against10px. The exact decimal reference16×1.1^100 differs by0.3744378436770541. The result is finite and below the compiler's1000000 size cap. This confirms that a finite-size guard cannot substitute for accumulated numeric-error accounting.

Each emitted file is imported/cloned once and resized400×100,200×80,100×40,400×100. Widths are read from native geometry. The exact decimal reference is a mathematical reference, not a Chrome observation; no browser or pixel qualification is added. The existing public font-relative profile remains subject to its documented bounded evidence. The upcoming provenance carrier must represent this error faithfully rather than labeling a previously rounded computed value exact.

Reproduction: output/provenance-native-controls-r1/run.py uses the frozen public compiler from flex-descriptor-public-regression-r1. It requires a fresh output directory; the current files, commands/script, maps, geometry and frozen binary are bound in provenance-native-controls-receipt.json. No compiler/runtime source changes were needed for these controls.
