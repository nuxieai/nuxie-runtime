# Explicit inheritance: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The accepted-profile run passed 176 browser/native checks, 24 Rust tests, five
JavaScript/WASM tests, two gallery checks and TypeScript. Module Clippy passed
with warnings denied. Native/WASM bytes and maps agree across the corpus.

All six new `inherit-*` fixtures were visually inspected as Chromium/Nuxie
pairs at 240, 390 and 768 CSS px. Chromium 153.0.8010.12, DPR 1; Rust Metal.
Spacing and shorthand precedence, inherited percentage dimensions, typography,
currentColor backgrounds, flex shorthand and host dimensions agree within the
existing thresholds. Each scene was compiled once and resized after import.
No threshold changed.

Public compile-contract checks compare explicit inheritance with equivalent
values across supported box, spacing, paint and flex properties. Browser
fixtures exercise text properties, inline/important precedence and retained
currentColor semantics. Unsupported properties and mixed global-keyword values
are rejected even when their selector does not match.

A03 is qualified for every currently supported property. Percentages and
currentColor are copied as computed values rather than parent used pixels or
resolved paint. Parent-dependent flex validity is checked in the child's own
context. The root semantic style now reflects reset.css's 100% host dimensions.

This increment does not qualify the unrelated typography compositions tracked
in known-gaps.md, or enable CSS Grid, initial/unset, or unsupported properties.
