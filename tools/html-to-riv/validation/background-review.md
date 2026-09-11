# Solid background shorthand: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The accepted-profile run passed 200 browser/native checks, 26 Rust tests, five
JavaScript/WASM tests, two gallery checks and TypeScript. Module Clippy passed
with warnings denied. Native/WASM bytes and maps agree across the corpus.

The three `background-shorthand-*` fixture pairs were visually inspected at
240, 390 and 768px. Chromium 153.0.8010.12, DPR 1; native Rust Metal. Solid colors,
alpha, precedence over/under background-color, none/initial/unset clearing and
inherited currentColor behavior agree within unchanged thresholds. Scenes are
compiled once and resized after import. No tolerance was widened.

Compiler-contract tests cover equivalent explicit longhands, important flags,
resets, inheritance and rejection of unsupported background layers, including
unmatched declarations. A06 is qualified for the solid-color contract in
SUPPORT.md. This does not enable background images, gradients, multiple layers
or positioning. Future additions must extend shorthand reset semantics with
the corresponding background state. Existing Q09 typography gaps remain open.
