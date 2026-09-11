# Initial and unset: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The run passed 191 accepted-profile browser/native checks, 25 Rust tests, five
JavaScript/WASM integration tests, two gallery checks and TypeScript. Module
Clippy passed with warnings denied. Native/WASM bytes and maps agree.

Visually inspected all five new initial/unset fixture pairs at 240, 390 and
768 CSS px. Chromium 153.0.8010.12, DPR 1; Rust Metal. CSS layout defaults, removed
size limits, transparent paint, initial text settings, unset inheritance and
flex cascade results agree within the unchanged thresholds. The imported scene
was resized without recompiling at each width.

Public compiler tests compare representable CSS initial values with explicit
equivalents, test shorthand/longhand resets, and reject unsupported initial
behavior even in unmatched rules. The contract deliberately distinguishes CSS
initial values from reset.css: row direction and shrink 1 are not the profile's
column direction and shrink 0. Unset follows each property's inheritance flag.

A04/A05 are qualified for the explicit SUPPORT.md contract. Inline formatting,
content-box and automatic minimum sizing, normal line-height, default fonts and
system initial text color remain outside that contract and produce diagnostics.
These are intentional profile rejections, not invented approximations. The
unrelated Q09 typography rendering gaps remain open.
