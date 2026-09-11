# Current color extension: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The full run passed 131 browser/native tests, 20 Rust tests, five JavaScript/WASM
integration tests, two gallery checks and TypeScript. Module Clippy passed with
warnings denied. Native/WASM scene bytes and maps agree across the corpus.

The three `currentcolor-*` fixtures in cases.json were visually inspected as
Chromium/Nuxie pairs at 240, 390 and 768 CSS px. Reference Chromium 153.0.8010.12,
DPR 1; native Rust Metal. The images agree in cascade-selected fill colors,
inherited text color, nested alpha compositing, default black and transparency.
The maximum global mean channel error was 0.246 out of 255 and the maximum
pixel mismatch fraction was 0.081%. No threshold was widened.

The public compile regression compares currentColor with equivalent explicit
colors, including both declaration orders, resetting an earlier color to the
inherited value, important backgrounds, transparent colors and descendants.
SUPPORT.md defines the compile-time semantics. This does not add runtime themes,
CSS variables, bindings or additional color spaces. Future compiler/runtime
changes must rerun the gate.
