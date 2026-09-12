# Large-coordinate content-box diagnosis

**Two independent effects are now observed:** the compiler changes some numeric
tokens before emitting ordinary Rive fields, and Chrome's measured local and
projected widths differ for this percentage child. The preserved original case
still fails. No production, runtime, renderer, dependency or shared support
document was edited for this diagnosis.

All public compiles use frozen r2 CLI
`e1b16fd2a068b96afe5f878ca0423877538760178ac78041f5c88b5918c2e9fc`,
not the mutable build being used for variable recovery. The read-only baseline
probe is unchanged. Chrome remains `153.0.8010.12`. The
[diagnostic receipt](content-box-rounding-receipt.json) binds the artifacts and
four r2 source snapshots independently of later production edits.

## Reproduction and controls

The focused command is:

```sh
node tools/html-to-riv/validation/content-box-rounding-measure.mjs \
  tools/html-to-riv/validation/content-box-rounding-repro-cases.json \
  tools/html-to-riv/output/content-box-rounding-r1
```

It was run successfully as a red-capable diagnosis and exited 1: both the exact
original source and a reduced source without paint or explicit heights report
native child width 1,000,000 versus Chrome rectangle width 999,999.75. A fresh
output directory is required for another run. Each geometry-only run imports
and clones an ordinary file and observes both at 390×200. It makes no pixel or
multi-viewport qualification claim. The original eight-frame native/Chrome run
and its failures remain bound by the prior public content-box receipt.

The ranked predictions were: an outer-size conversion can discard content
precision; percentage resolution may introduce additional local rounding; and
placement at a large x coordinate may change the reported rectangle. Eight
controls in `content-box-rounding-controls.json` isolate padding, child sizing
and position. All share authored parent content width `999999.875px`.

| Child sizing; left/right padding | Native parent / child width | Chrome parent content width (ResizeObserver) | Chrome child local width (ResizeObserver) | Chrome child rectangle width |
| --- | --- | ---: | ---: | ---: |
| 100%; 1m / 1m | 3m / 1m | 999999.875 | 999999.8125 | 999999.75 |
| 999999.875px; 1m / 1m | 3m / 1m | 999999.875 | 999999.875 | 999999.875 |
| auto; 1m / 1m | 3m / 1m | 999999.875 | 999999.875 | 999999.875 |
| 100%; 0 / 0 | 1m / 1m | 999999.875 | 999999.8125 | 999999.8125 |
| 999999.875px; 0 / 0 | 1m / 1m | 999999.875 | 999999.875 | 999999.875 |
| 100%; 0 / 1m | 2m / 1m | 999999.875 | 999999.8125 | 999999.8125 |
| 100%; 1m / 0 | 2m / 1m | 999999.875 | 999999.8125 | 999999.75 |
| 100%; .5m / .5m | 2m / 1m | 999999.875 | 999999.8125 | 999999.75 |

Here `1m` means 1,000,000px. Native values in this table are exact decoded
binary32 observations. Chrome parent rectangles are 3m in the first three
controls, 999999.875 with no padding, and 1999999.875 in the last three. Both
original and clone have the same differences in every control. These are
retained failing geometry observations, not qualified cases.

Chrome's CSS Typed OM avoids the computed-style string's loss of precision:
`computedStyleMap().get('width').value` for the parent is **999999.875**, while
`getComputedStyle(...).width` serializes `1e+06px`. For the child Typed OM
returns the computed percentage coefficient 100 and unit `percent`, not its
used width. ResizeObserver reports child content and border sizes of
999999.8125. Moving one million pixels of padding from right to left leaves
that local width unchanged but changes the rectangle from999999.8125 to999999.75.

These facts isolate an observable local percentage-sizing difference and an
additional position-dependent rectangle difference. They do not by themselves
identify Blink's precise floating-point instruction sequence, internal layout
unit conversion or projection implementation. The native width errors also
occur for fixed dimensions with zero padding, which rules out a diagnosis
limited to content-box addition or percentage arithmetic.

## Compiler normalization defect

The standalone `content-box-rounding-token.rs` probe uses the already-built
cssparser0.37 dependency. Both available local dependency artifacts produce the
same results; their hashes and build commands are retained in
`output/content-box-rounding-r3`:

| Input | Parsed token binary32 value, shown as f64 | `Token::to_css_string()` |
| --- | ---: | --- |
| 999999.875px | 999999.875 | 1000000px |
| 999999.9375px | 999999.9375 | 1000000px |
| 62499.9921875rem | 62499.9921875 | 62500rem |
| .03125px | .03125 | .03125px |

cssparser's `serializer.rs::write_numeric` calls `dtoa_short::write`; the pinned
dtoa-short0.3.5 implementation formats f32 with **six significant digits**.
This is useful serialization behavior but does not preserve every authored
binary32 numeric value for a compiler that reparses the resulting text.

The r2 production paths are precise:

1. Literal ordinary values reach `compiler.rs::resolved_declarations`, then
   `css::ordinary_value` / `value_text_inner`. Its fallback serializes numeric
   tokens using `token.to_css_string()`. `size()` then parses that changed text.
2. Variable values first pass `variables.rs::parse_inner`. It already preserves
   raw `Token::Number` spelling for integer grammar, but Dimension and Percentage
   take its serialization fallback. Selected values later pass the ordinary
   normalizer as well. Original-token provenance survives separately; it does
   not replace these normalized native values.

The read-only ordinary-file decoder in `content-box-rounding-wire.py` verifies
the emitted LayoutComponent width property7 directly. Literal width, custom
property substitution, missing-variable fallback and escaped `p\78` unit forms
all produce the **same Rive bytes and map**, with field7 equal to **1,000,000**
at byte offset92 for the one-node control. This rules out native layout or probe
serialization as the origin of that particular change. The decoder accounts
for the file's Backboard record preceding the artboard object index.

The original padded source still adds two million pixels to the content width.
Even after preserving999999.875, ordinary binary32 outer sizing gives3,000,000;
subtracting the stored padding does not recover the original content width.
Consequently **a serializer repair alone cannot be presumed to fix the original
content-box negative control**. It does address a separate, demonstrated
compiler-only error in direct sizes. The subsequent percentage and world-box
effects must remain visible in qualification.

## Bounded repair plan, not implemented here

Introduce one shared numeric-token serializer that retains the original numeric
prefix and emits only the suffix from the decoded token: nothing for Number,
`%` for Percentage, and a canonically serialized decoded unit for Dimension.
Use it in both `value_text_inner` and `variables::parse_inner`. The existing
numeric-prefix scanner in `computed_provenance::original_number` already handles
sign, fraction and an exponent with required digits without relying on the
length of an escaped unit. Reuse that logic rather than deriving decimal intent
from a rounded token value. Keep nonfinite-token checks and token boundaries.

Tests must exercise literal/inline, variable/alias/inherited/fallback, escaped
unit, exponent and sign forms. Preserve Number lexemes that matter to integer
grammar, including rejection of1.0/1e2 where integers are required. Preserve
token separators so adjacent substituted tokens cannot merge. Point,
percentage, font-relative, padding, min/max and color numeric consumers all
need their changed-output impact reviewed. Provenance must still bind the
actual parsed native number and original ideal value independently.

Use expected binary32 values and exact emitted fields for the lexical regression
test, then run CLI/WASM parity and classify old-output changes explicitly.
Do not assume the historical unchanged-output matrix should remain bit-identical
for values that were previously rounded by serialization. Re-run the preserved
native/Chrome controls after that separate compiler checkpoint; no gate or source
limit should be reduced merely to avoid the boundary.

A plausible later ordinary composition would separate the fixed content owner
from its intrinsic outer box, with left/right spacer objects carrying padding.
That could retain an independent content dimension while the outer contribution
rounds separately. The current serializer also rounds the inner owner's source
dimension, including a straightforward rem rewrite, so no such wrapper was
rendered or qualified here. Per the parent's updated scope, wrapper exploration
is deferred until the compiler's numeric normalization is repaired and isolated.
No browser-derived shipping dimensions, field patches, extra runtime APIs or
universal impossibility conclusion were introduced.

The small setup failures are preserved: the token-probe launcher initially
assumed one cssparser artifact before explicitly testing both; an initial wire
inspection used artboard object indices without the leading Backboard offset.
The successful probes validate both corrections. No failed renderer result was
discarded, and no native or visual candidate result is claimed in this diagnosis.
