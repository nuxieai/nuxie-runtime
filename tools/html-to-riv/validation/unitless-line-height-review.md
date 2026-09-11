# Unitless line-height: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The accepted-profile run passed 209 browser/native checks, 27 Rust tests, five
JavaScript/WASM tests, two gallery checks and TypeScript. Module Clippy passed
with warnings denied. Native/WASM bytes and maps agree across the corpus.
The public contract test was rerun successfully after adding minimum/maximum
used-height rejection assertions.

The three unitless line-height fixtures were visually inspected at 240, 390
and 768px: inheritance, unitless versus fixed pixel height, and cascade. The
Chromium/native pairs agree on wrapping, line spacing, backgrounds and placement
within unchanged thresholds. Scenes are compiled once and resized after import.
No tolerance was widened.

The compiler retains an inherited multiplier and resolves it against each
descendant's font size. Explicit inherit and unset preserve this behavior;
pixel heights remain fixed through inheritance. Public tests compare equivalent
pixel declarations and reject malformed, nonfinite, nonpositive and excessive
multipliers, heights below the font's natural metrics and heights above the
documented maximum.

A08 is qualified for the contract in SUPPORT.md. Normal line-height, percentage
line-height and shorter-than-natural line boxes remain unsupported. The existing
Q09 typography pixel gaps remain open; this increment does not qualify those
compositions. Font shorthand is the next increment.
