# Font shorthand: partial implementation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
Initial expanded gate: **216 passed, 2 failed** out of 218. Rust: 28 tests pass;
five JavaScript/WASM tests, two gallery checks and TypeScript pass. Module Clippy
passes with warnings denied. Native/WASM bytes and source maps agree across the
expanded corpus. The public shorthand regression first failed on unsupported
property, then passed after implementation.

All three new font-shorthand fixture pairs were visually inspected at 240,
390 and 768px. Wrapping, backgrounds and element geometry agree. Text contour
and placement differences remain in the mixed-size reset fixture. Chromium
153.0.8010.12, DPR 1, native Rust Metal. No thresholds were widened.

`font-shorthand-resets` fails at 240 and 390px, passes at 768px. At 240px the
mean channel error is 1.20565 (limit 1) and the 20px text interior RGB error is
6.96490 (limit 6). At 390px that interior error is 8.48806. Geometry passes at
all widths. The two other shorthand fixtures pass at all widths. The failing
fixture remains unchanged in validation/cases.json and the default suite exits
nonzero. This is not a qualified feature increment.

An explicit-longhand control for the failing scene produces byte-identical
Rive output. This narrows the next investigation toward shared text lowering
and rendering rather than shorthand serialization; it does not itself prove
browser equivalence. Compare browser longhand pixels and glyph baseline/outline
positions next, alongside the existing Q09 typography specimens.

The parser supports the explicit-line-height upright/static-font subset in
SUPPORT.md, including weight resets, inheritance and important precedence.
Omitted and normal line-height are explicitly rejected. A07 stays partial until
its pixel failures and font-dependent normal line-height/reset behavior are
resolved and pass the full gate. No editor or runtime implementation changed.

Reset semantics reference: [CSS Fonts](https://www.w3.org/TR/css-fonts-4/#font-prop).

## Subsequent baseline correction

The two failures above now pass unchanged after the text baseline correction.
The expanded main suite passes 224 checks; see text-baseline-review.md. A07
remains partial because omitted/normal line-height is still unsupported. The
historical failures above document the reproducer, not the current gate status.
