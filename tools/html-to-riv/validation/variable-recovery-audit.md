# Variable recovery: bounded implementation path

The current custom-property resolver already distinguishes a guaranteed-invalid
substitution (`Ok(None)`), a successfully expanded token stream (`Ok(Some(...))`),
and syntax/resource failure (`Err(...)`). The public compiler previously rejected
guaranteed-invalid substitution and syntax/resource errors, while passing successful
tokens to property admission. Only the first class can be recovered without a new
ordinary-property grammar classifier.

The proposed change replaces `Ok(None)` with an explicit `unset` value at the
original declaration's cascade position. It then runs the same property and
target admission as ordinary authored values. It does not drop a declaration,
return to an earlier cascade winner, or restore the module's authoring reset.
For inherited `color` and `font-size`, unset inherits. For width/background/order
it uses CSS initial values. Flex direction becomes row even though the authoring
reset is column; min-width becomes auto even though the reset is zero.

This matches the [CSS Values draft's invalid substitution rule](https://drafts.csswg.org/css-values-5/#invalid-at-computed-value-time).
The same specification's shorthand section explains why pending substitutions
remain at the shorthand's original cascade priority. Existing sequential
shorthand lowering can preserve that priority when the recovered value is unset.
The compiler continues to reject every unsupported declaration, including losers.

## Preserved before-change evidence

`output/variable-recovery-audit-r1/probe.mjs` ran 30 small sources through frozen
compiler `a9a2601b6def75b674a769ca443ac51ad04eb1aa19fd65fe0fcd8a622107536a`
and Chrome 153.0.8010.12. All 30 compiler requests rejected and emitted neither
RIV nor source map. Twenty-four Chrome sources exactly matched independently
written explicit-unset controls for all recorded computed properties and DOM
boxes. Twenty-one unset controls already compile; three remain unsupported.
The receipt binds sources, compiler, reset, browser observations and diagnostics.

Every example below uses the explicit module reset and admitted empty boxes.

| Authored declaration sequence | Chrome result | Implementation scope |
|---|---|---|
| `width:31px; width:var(--missing)` inside a 200px parent | width 200px, from auto | Recover |
| `color:red; color:var(--missing)` under navy parent | navy | Recover |
| `background:coral; background:var(--missing)` | transparent | Recover |
| `--x:var(--x); background:var(--x)` | transparent | Recover |
| `--x:var(--missing); width:var(--x)` | auto width | Recover |
| `font-size:11px; font-size:var(--missing); width:2em` under 24px parent | 24px font context, 48px width | Recover |
| `order:-4; order:var(--missing)` | order 0 | Recover in ordering prepass and full style |
| `flex-direction:column; flex-direction:var(--missing)` | row | Recover |
| `padding:8px; padding:var(--missing); padding-right:4px` | zero except right 4px | Recover, preserve shorthand order |
| `background:var(--missing)!important; background:red` | transparent | Recover, preserve importance |
| `width:var(--missing,)` or valid empty primary | auto width | Still diagnostic; successful empty tokens require property-grammar classification |
| `--size:20px; background:var(--size)` | transparent | Still diagnostic; successful wrong-typed tokens |
| `--n:10; width:var(--n)px` | auto width | Still diagnostic; token-boundary invalidity |
| `--size:calc(20px + 10px); width:var(--size)` | 30px | Still diagnostic; valid CSS outside current admission |
| `--size:1000001px; width:var(--size)` | 1000001px | Still diagnostic; compiler resource boundary |
| `display:var(--missing)` | initial inline, then flex-item blockification | Still diagnostic; unset display is outside current public profile |
| `flex-shrink:var(--missing)` or `flex:var(--missing)` | shrink 1 | Still diagnostic; nonlegacy flex |

The 17 recoverable characterization cases cover missing/direct cyclic/transitive
values, inherited and noninherited properties, order, direction, alignment,
spacing, gaps, padding, margins, min/max bounds, losing declarations and importance.
The four successfully substituted but typed-invalid cases remain a separate,
explicit S09/S10 follow-up. These are compiler gaps, not runtime limitations.

## Code locations and constraints

- `compiler.rs::resolved_declarations`: preserve every declaration, resolve custom
  environments once, replace only failed ordinary substitution, then revalidate.
- `compiler.rs::ordering_key`: use exactly the same substitution behavior before
  determining sibling order. Do not compute full Styles just to obtain order.
- A compiler-owned shared helper can call `variables::substitute_with_provenance`.
  Preserve `Err`; preserve `Some` including empty values; map only `None` to unset.
  Synthesized unset has no authored numeric token provenance. CSS-wide computation
  already derives inherited/default provenance without needing such tokens.
- `variables.rs` needs no algorithm change. Its `None` remains useful for invalid
  custom properties, fallback selection and cycle participants.
- `css.rs` needs no cascade/parser change. Global validation of unmatched literals,
  unsupported names, malformed variables and overridden semantics remains intact.
- Do not catch `unsupported-target-semantics` generally: it also represents valid
  unsupported values, input bounds, computed font overflow and context restrictions.

## Validation required for admission

Use public Rust exact-output controls for missing and cyclic values versus explicit
unset, including selector specificity, inline style, importance, repeated declarations,
shorthand/longhand interleaving, inherited font/color/currentColor, order ties and
DOM-based selectors. Include same-output determinism at multiple viewports.

Keep empty-value, wrong-type, token-boundary, unsupported CSS, display/flex defaults,
literal invalid losers, malformed variables, context and variable-expansion failures
as strict diagnostics. Require Rust/CLI/WASM parity and no-output failure behavior.

Render observable color/layout compositions through the immutable native importer
and renderer, discard source metadata before load, compare with pinned Chrome and
resize the same original and clone. Include nested boxes for direction/gap tests
so those values alter visible pixels. Inspect distinct pairs; retain all failures
and ordinary pixel tolerances. Neither audit screenshots nor explicit-unset controls
alone qualify the changed compiler's native output.
