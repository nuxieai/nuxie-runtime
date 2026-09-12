# Ordinary-value grammar before target admission

S09/S10 still need recovery when variable substitution succeeds but its result
does not match the receiving property's grammar. The compiler currently rejects
these values. The completed failed-substitution change handles a different case:
the resolver returns no token value at all.

The new audit uses frozen compiler `2e63e64dabb3a18790e8c42812bf4afc2803b238bded24a2475bcba4ad8af2f9`
and Chrome 153.0.8010.12. It changes no production code and provides no native
pixel qualification. Run it from the repository root with:

```sh
node tools/html-to-riv/validation/variable-value-grammar-audit.mjs FRESH_OUTPUT
```

## Corrected background example

Earlier `variable-recovery-audit.md` and `public-variable-recovery-review.md`
called `--x:20px; background:var(--x)` a wrongly typed value. That classification
was incorrect: the background shorthand accepts a position. Chrome reports
`CSS.supports('background','20px') === true` and computes background-position
as `20px 50%`. The current compiler rejects it because that shorthand component
has no admitted implementation. It must remain distinct from typed invalidity.

By comparison, `background-color:20px` is invalid. These are different grammars.
The [background shorthand definition](https://drafts.csswg.org/css-backgrounds-3/#background)
allows a background position and resets unspecified components. The
[substitution rules](https://drafts.csswg.org/css-values-5/#invalid-at-computed-value-time)
require parsing the substituted result against the receiving property's grammar;
a failed parse uses inherited/initial behavior instead of restoring an earlier
cascade winner.

The old audit measured background color but omitted background position, so
its selected computed fields could agree with an unset control despite the
different shorthand meaning. Its receipts and sources remain preserved as
historical evidence. This new audit observes both the full computed background
shorthand and background-position. The existing None-to-unset implementation
and its positive native qualification are unaffected by this correction.

## Browser and compiler evidence

The final r3 run has 124 cases:

| Case group | Cases | Chrome result | Current compiler |
|---|---:|---|---|
| Empty fallback, empty primary, whitespace/comment primary across 32 ordinary properties | 96 | Literal empty value is invalid; variable use matches explicit unset | All reject |
| Property-specific wrong types, including dimensions, colors, order, flex, padding and alignment | 16 | Literal value is invalid; variable use matches explicit unset | All reject |
| Valid controls, including zero, integer order, calc, intrinsic sizing, negative margin, font keyword, color space, background position and source limits | 12 | All literal values are valid | 3 compile; 9 retain target/resource diagnostics |

All 112 invalid-substitution cases match explicit-unset controls for every
recorded computed field and all five DOM rectangles. Of these, 102 explicit
unset controls already compile. Ten require currently unadmitted display or
shrink defaults. Therefore grammar recovery can unlock actual compiler inputs
without changing the runtime, while existing target checks still have work to do.

The r1 run stopped on the incorrect background classification; its driver,
fixture list and assertion log are retained. The r2 run preserved that correction
but stopped on a duplicate fixture name. R3 has an up-front uniqueness check and
stores each completed observation separately. Neither failed run contributes to
the final counts. `variable-value-grammar-receipt.json` binds all three runs.

## Implementation boundary

Introduce a compiler-owned grammar classification separate from supported-value
lowering. Its result should distinguish **valid**, **invalid**, and **unclassified**
token sequences for a named ordinary property. Valid does not mean implemented.
Unclassified continues through existing admission and diagnostics; it must not
be treated as invalid simply because a limited parser rejects it.

Apply this classification only after successful variable expansion and before
existing property/target admission, in the shared ordering and style path.
An invalid result becomes unset at the same declaration priority. Preserve the
existing handling of resolver errors and missing values. Literal invalid
declarations retain the module's explicit strict source-validation policy.

Empty token streams are an independently decidable first branch for the admitted
ordinary properties. Count CSS tokens after skipping whitespace/comments; do not
confuse the empty stream with an empty string token, an empty function, or an
unused empty fallback. Keep custom-property values outside this rule, since an
empty custom property is valid data and suppresses a var fallback.

For nonempty streams, use the full receiving grammar: scalar numeric types,
units, keywords, arity, CSS-wide keywords, and shorthand constituents. A numeric
or runtime resource limit is separate from syntactic validity. Preserve original
numeric spelling where integer grammar depends on it. Functions or keywords
whose validity has not been established remain unclassified; a permissive catch
of `unsupported-target-semantics` would incorrectly erase valid CSS such as
background positioning, calc, negative margins and large finite widths.

Before public admission, add Rust/CLI/WASM controls for cascade priority,
inheritance, shorthand/longhand ordering, literal strictness, successful empty
custom values, valid unsupported syntax and resource failures. Render the changed
compiler's ordinary files against Chrome, resize originals and clones, and inspect
distinct native/reference pairs. This audit identifies the next implementation
and corrects its oracle; computed browser equality alone does not qualify it.
