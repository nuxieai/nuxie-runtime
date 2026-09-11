# Normal line-height: partial validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
Expanded main gate: **230 pass, 3 fail** out of 233. All 29 Rust tests, five
JavaScript/WASM tests, two gallery checks and TypeScript pass. Module Clippy
passes with warnings denied. The new public compile test failed before the
mapping was implemented and passes now. Native/WASM bytes and maps agree for
all accepted inputs in the expanded corpus.

The compiler retains normal through inheritance and resolves its line spacing
from individually rounded ascent, descent and line gap at each element's font
size. Font shorthand resets omitted line-height to normal; line-height:initial
also uses normal. It never substitutes the profile's default 24px line-height.
A Text bottom-trim flag measures through the final baseline. Font-derived
trailing space restores the normal line box even when rounded metrics are
slightly smaller than fractional natural metrics. Line counting, shaping and
wrapping remain runtime operations. No runtime source was changed.

All nine new geometry comparisons pass. Browser/native pairs were visually
inspected at 240, 390 and 768px; wrapping, line heights, inherited font sizes,
backgrounds and cascade agree. Glyph rasterization differences remain. Chromium
153.0.8010.12, DPR 1; native Rust Metal. No tolerances were widened.

Failures remain in the default suite:

* font-shorthand-normal-resets at 390px: text a interior RGB error 6.28139.
* normal-line-height-cascade at 390px: text a error 6.21169.
* normal-line-height-cascade at 240px: text b/c errors 7.61345/7.06623.

The interior limit is 6. The two 390px scenes pass when only the diagnostic
browser control uses -webkit-font-smoothing:antialiased (a errors 5.30866 and
5.00473). This provides evidence of a smoothing contribution, not a complete
explanation of every residual difference. The reference reset and acceptance
fixtures stay unchanged. The diagnostic controls are reproducible via
`node tools/html-to-riv/validation/text-diagnosis.mjs`; its results are reports,
not acceptance-test passes.

A07 remains partial. Next: investigate a compatible text-rasterization path
alongside the preserved Q09 specimens. Normal/default syntax and responsive
geometry are implemented; full visual qualification is unfinished.
