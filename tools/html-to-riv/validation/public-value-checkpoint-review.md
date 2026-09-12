# Primitive variable recovery on the immutable runtime

The public compiler now recovers proven invalid nonempty variable values as
`unset` at their original cascade priority. Unicode and escaped token boundaries
remain significant, including inside the literal color and flex parsers. This
checkpoint changes the compiler module only. The runtime, renderer, schema,
shared dependencies and effective baseline renderer remain unchanged.

## Behavior and implementation

`src/value_grammar.rs` separates receiving-value grammar from Rive admission for
the 33 admitted ordinary property names. It checks primitive types, scalar and
shorthand arity, known keywords, numeric lexemes and nonnegative requirements.
Invalid results recover in the shared ordinary substitution helper used by both
style computation and sibling ordering. Successful fallback selection is not
repeated after receiving-value invalidity; earlier cascade declarations are not
restored. Empty custom values remain valid data and suppress fallbacks.

Literal declarations retain strict source validation. A final grammar check
prevents target numeric parsers from accepting invalid authored values after
binary32 rounding. Grammar coefficients use the authored binary64 value: a
negative `-1e-50px` width remains invalid even when Rive's float would be zero.
Pinned Chrome's separate `1e-500` underflow behavior has explicit controls; this
does not claim exact real-number arithmetic or a general numerical proof.

The shared whitespace helper recognizes TAB, LF, FF, CR and SPACE. Escaped flex
identifiers preserve token identity, so one identifier decoding to `0 0 auto`
cannot become three components. Unicode custom-property names remain distinct.
NBSP/vertical-tab-only HTML text reaches the existing text diagnostic, with a
nonempty source label at the document root.

The [CSS tokenization rules](https://drafts.csswg.org/css-syntax-3/#tokenization)
define significant token boundaries. The
[receiving-value substitution rules](https://drafts.csswg.org/css-values-5/#invalid-at-computed-value-time)
require inherited/initial behavior for invalid computed values. These rules do
not make every target-parser rejection into a recoverable CSS error.

## Evidence

| Check | Result |
| --- | --- |
| Public Rust suite | 262 passing tests |
| CLI/WASM/JavaScript suite | 42 passing Node tests; strict TypeScript passes |
| Receiving-value corpus | 610 cases at two viewports; 1,876 CLI/WASM request/control/rollback comparisons |
| Preserved prior outputs | All 731 Rive files and source maps byte-identical |
| Independent grammar audit | 594 Chrome observations; no supported Chrome input classified invalid |
| Native/Chrome scenes | 35 files, 280 passing geometry and pixel comparisons |
| Clear independence | 560 passing cyan/transparent checks |
| Visual review | 37 direct pairs on 10 inspected sheets; 243 exact complete-RGBA transfers |

All native files use the frozen compiler from
`output/public-value-build-r1/frozen/`. The unchanged baseline importer loads the
ordinary bytes, diagnostic metadata is dispensable, and originals/clones resize
without recompilation. Rendering uses pinned Chrome 153.0.8010.12 and native
Rust Metal RasterOrdering with the existing geometry/pixel gates.

The [public test receipt](public-value-token-review.md) preserves the red runs,
fixture corrections and 25 explicitly migrated historical rejection controls.
The [native receipt](public-value-native-review.md) binds emitted files, source
and unset controls, pixels and complete visual coverage. The
[prior-output receipt](public-value-regression-review.md) preserves all 694
corrected previous references plus 37 distinct numeric controls, with one
explicit duplicate. The aggregate `public-value-checkpoint-receipt.json` binds
these results and the unchanged private bridge build.

Reproduce build/transport checks with `python3
tools/html-to-riv/validation/public-value-build.py FRESH_OUTPUT`, then use that
directory's frozen CLI with the native and prior-output commands recorded in
the linked receipts. The build script checks source identity before and after
building, snapshots the inputs, and never runs transport tests against mutable
Cargo artifacts.

## Remaining work

S09/S10 remain partial. Opaque functions, multi-component background grammar,
unclassified units/vendor extensions and unresolved display combinations still
diagnose. Current draft/browser differences such as overflow alignment with
`normal` remain unclassified. `revert-rule`, background clipping values and other
valid unsupported CSS are not erased. Full substitution still checks syntax and
resource limits before any recovery.

This checkpoint does not repair the retained fractional-paint or large padded
content-box failures. Their ordinary files remain byte-identical in regression;
that fact is not a new visual qualification. The next independent investigation
is the content-owner/padding composition, while broader grammar and the full
99-item backlog remain open. No feature is declared impossible merely because
these candidate encodings or a limited parser cannot implement it.
