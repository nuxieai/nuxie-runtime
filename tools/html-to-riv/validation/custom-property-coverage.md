# Custom-property computed-value coverage audit

S09/S10 remain in progress. Passing fixtures prove their stated cases, not all
CSS property grammar. This inventory records the remaining work after the
flex/overflow enum implementation; custom-properties-progress.md contains receipts.

| Property family | Current invalid-substitution handling | Remaining qualification |
| --- | --- | --- |
| width/height, min/max, flex-basis | Empty, malformed scalar/list tokens, negative values, invalid keywords | Units/functions beyond current classifier; min-size initial values outside profile remain diagnostic |
| padding/margin/gap and longhands | Empty, scalar/list arity, sign and keyword checks | Functions and unclassified syntax; preserve valid-but-excluded negative margins |
| color/background-color/decoration-color | Empty, wrong scalar type, hex, keywords, scalar RGB/HSL | Other function forms and composition; keep system/relative/wide-gamut colors as exclusions |
| flex-grow/flex-shrink | Negative/non-number/multi-token factors reset; fractional factors validated; compiler159/159 and combined Chrome/native93/93 pass | Function forms remain excluded; unequal resolved factors remain a format diagnostic |
| flex-direction/flex-wrap/overflow/text-overflow | Enum/token/arity checks | Additional function forms; retain reverse/scroll/two-value/string exclusions |
| flex shorthand | Missing/empty and malformed flat grammar reset; later longhands and important ordering validated | Further specificity/inheritance combinations and function forms; valid excluded basis/weights retain diagnostics |
| font shorthand/family/size/weight/line-height | Size/weight/line-height flat invalidation and inherited unitless line-height verified with real text; missing/empty inheritance | Family malformed-list inheritance verified; generic keyword positions now directly checked in Chrome; leading CSS-wide substitutions now inherit per Chrome control; shorthand/longhand precedence, importance and final em basis validated; supported-prefix malformed shorthand grammar now inherits; unknown prefixes and combined system-font keywords now invalidate; recognized excluded-prefix and function combinations remain |
| align-items/justify-content/text-align | Flat keyword/scalar grammar resets invalid values; safe/unsafe and baseline pairs retain diagnostics; inherited right-aligned wrapping validated | Function/string/vendor forms remain outside rendering profile; broader cascade combinations |
| white-space/text-transform | Text-transform scalar/keyword/group invalidation and inherited casing validated | White-space flat keyword groups and inherited preservation validated; valid unsupported compound modes plus functions/vendor forms remain diagnostic |
| text-decoration shorthand/line/style/thickness/skip-ink, underline offset/position | Missing/empty values; color classifier; style and skip-ink enum invalidation with runtime-record checks | Line invalidation and ancestor propagation validated; metric scalar invalidation and inherited offset validated; position keyword-group invalidation validated within auto-only rendering; shorthand var preflight fixed and flat grammar/reset/later-longhand/importance validated; scalar RGB/HSL shorthand functions validated; all eight decoration properties covered for missing/empty and CSS-wide fallback resets, plus custom-wide inheritance; remaining unclassified functions and broader composition |
| border-radius | Flat scalar/list/slash grammar resets invalid values; square versus rounded controls validated | Valid percentages, elliptical/per-corner radii and unimplemented functions/units retain diagnostics |
| background shorthand | Missing/empty reset, pure scalar RGB/HSL invalidation; simple colors/none/layer errors and scalar RGB/HSL combinations reset; later color override validated | Full distinction between malformed syntax and valid excluded position/image/layer forms |
| display/box-sizing | Valid substitutions still checked by profile | Invalid computed initial values are outside the profile; must remain explicit diagnostics rather than guessed layouts |

Cross-cutting checks still needed before qualification:

- Verify each accepted shorthand's pending substitution and longhand cascade
  behavior with conflicting specificity/importance and later declarations.
- Audit unsupported initial values as explicit profile boundaries, not silent
  fallbacks to the compiler reset stylesheet.
- Complete public tests for CSS-wide values produced inside custom values versus
  ordinary-property fallbacks, and complex short-circuit cycle graphs.
- Keep native/WASM artifact parity, same-scene resizing and real native pixels
  as gates; inspect new results and record identity for unchanged prior sheets.
- Retain dynamic variable names as an evidenced Chrome153 exclusion until the
  pinned reference supports them. Do not infer support from editor-draft syntax.

The 512-property nested-chain stack-overflow reproducer is fixed by an explicit
continuation stack and retained in public tests and the corpus. Expansion limits
and property-count failure/recovery are tested; these do not replace the grammar
and cascade checks above. No tolerance increase or browser-baked layout is part
of this work.

Realistic light/dark composition gate added: compiler184/184, publish parity pass;
Chrome/native220/222 with two new240px underline fractional-translation failures.
Both sheets inspected. Semantic equivalence to explicit-value CSS passes including
runtime records. Preserve these fixtures while fixing runtime snapping; do not
qualify the compositions based only on their passing390/768 renders.

The two native240px composition failures are now fixed by runtime paint-space
underline snapping. All222 custom-property comparisons pass within the full native
run1,470/1,471 (unchanged A09 only). Vector composition qualification remains open.
See underline-fractional-translation-review.md; original reproducers retained.
