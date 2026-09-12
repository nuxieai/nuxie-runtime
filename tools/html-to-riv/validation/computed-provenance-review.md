# Computed scalar provenance checkpoint

Selected original numeric tokens now follow computed fonts, dimensions and bounds alongside their unchanged native values. Inherited computed values retain their provenance; em uses the final element font, font-relative font sizes use the parent, and rem uses the fixed root. Percentage dimensions retain coefficients rather than capturing the compilation viewport. Missing or invalid metadata remains explicitly unresolved.

Original variable token streams share immutable Arc storage through aliases and inheritance, using copy-on-write when concatenated. Existing logical metadata limits remain in force. The nested-variable control observed process peak resident memory of 118,669,312 bytes before and 7,356,416 after, with identical Rive bytes. This is a single-run observation of the combined change, not an isolated benchmark or general performance guarantee.

Validation: 177 Rust tests and 35 Node tests pass. The WASM build completes. All 482 retained public outputs, 79 positive controls and 48 private flex outputs retain exact bytes and source maps. The checkpoint source hashes match all 20 compiled source files; the separately staged, unregistered flex_numeric.rs is excluded. The immutable-source guard passes. No new pixel qualification is claimed by byte regression.

Flex factor/basis metadata, descriptor export, parent/world error domains and analyzer integration remain to be implemented. This checkpoint does not expand CSS admission. Artifact and source bindings are in computed-provenance-receipt.json; executable harnesses and retained results are under output/computed-provenance-checkpoint-r1.
