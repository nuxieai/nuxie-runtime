# Text baseline correction: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
Final transform-based implementation: **224 checks pass**, 28 Rust tests pass,
five JavaScript/WASM tests, two gallery checks and TypeScript pass. Native/WASM
bytes and maps agree across the expanded corpus. Module Clippy passes with
warnings denied; formatting and diff whitespace checks pass.

The former font-shorthand-resets failures at 240/390px pass unchanged. Two new
fixtures cover a tightly bounded single H and fractional font size/line-height.
Those pairs and the corrected shorthand fixture were visually inspected at
240, 390 and 768px. Wrapping and baseline placement agree within unchanged
thresholds. Chromium 153.0.8010.12, DPR 1; native Rust Metal. Imported scenes are
resized by the runtime; no browser geometry is baked into the compiler output.

Diagnosis isolated identical shorthand/longhand browser pixels and native
bytes, then a single-glyph failure. The compiler now translates glyphs from
Rive's fractional natural baseline to the pinned browser's rounded font metrics
and floored half-leading. It retains the original line-box spacing and native
measurement. A padding-only prototype was replaced because it could reject
valid tight leading: 22px text with 27px line-height. That case now compiles and
its diagnostic pixel comparison passes (interior RGB error 4.992).

The separate known-gap run has seven failures and two passes across nine
comparisons. Membership/typography specimens remain intact; a minimized Hg
case adds three observed rasterization failures. A diagnostic-only browser
smoothing control strongly reduces that error, without changing the reference
profile. See known-gaps.md and text-diagnosis.mjs. No runtime/renderer defaults,
font outlines or acceptance thresholds were changed.

A07's explicit-line-height subset now passes the main gate. A07 remains partial
until normal line-height and omitted shorthand resets are implemented and
qualified. Q09's rendering work remains open; these are not external blockers.
