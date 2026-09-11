# Text alignment extension: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The run passed 143 browser/native checks, 21 Rust tests, five JavaScript/WASM
integration tests, two gallery checks and TypeScript. Module Clippy passed with
warnings denied. Native and WASM outputs agree across the accepted corpus.

The four `text-align-*` fixtures were visually inspected as Chromium/Nuxie pairs
at 240, 390 and 768 CSS px. Chromium 153.0.8010.12 at DPR 1; native Rust Metal.
Centered and right-aligned line positions, wrapping, padding, left overrides,
flexible text box resizing, inheritance and important declarations agree within
the existing pixel thresholds. No threshold changed. The same compiled scene
was imported and resized, rather than compiling separate fixed-width layouts.

The public compile test verifies explicit/inherited equivalence and rejects
justification, logical alignment and other deferred values in unmatched rules.
The qualified text profile remains plain leaf text with the checked-in Inter
font and left-to-right content. See SUPPORT.md for limits. Future compiler or
runtime changes must rerun the gate.
