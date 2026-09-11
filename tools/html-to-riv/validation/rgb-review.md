# RGB function extension: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The full run passed 122 browser/native tests (114 scene comparisons and eight
pixel sensitivity checks), 19 Rust tests, five JavaScript/WASM integration tests,
two gallery checks and the TypeScript check. Module Clippy passed with warnings
denied. The accepted corpus produced identical native/WASM scene bytes and maps.

The following Chromium/Nuxie screenshot pairs were visually inspected at 240,
390 and 768 CSS px, using the same gallery artifacts and contact sheets at one
image pixel per CSS pixel. Chromium 153.0.8010.12, DPR 1; native Rust Metal.

* `rgb-legacy-and-modern`
* `rgb-alpha-compositing`
* `rgb-clamping-and-rounding`
* `rgb-text-inheritance-and-cascade`

Fills, layered alpha and text colors agree within the existing tolerances.
Text line breaks and layout also agree. The largest global mean channel error
among the new cases was 0.577 out of 255; the largest mismatch fraction was
0.386%. No threshold changed. Scene colors have 8-bit precision; this does not
claim exact browser floating-point compositing or identical edge/text rasterization.

Public compile tests compare known equivalent hex and RGB colors and exercise
malformed/deferred forms in both inline styles and unmatched rules. Regressions
cover important suffixes after functions, numeric overflow before token
serialization, missing closing parentheses and nested functions.

See SUPPORT.md for the explicit accepted subset. This receipt records this
increment; future compiler/runtime changes must rerun the gate.
